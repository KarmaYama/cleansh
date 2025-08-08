//! Core trait and data structures for pluggable sanitization engines.
//!
//! This module defines the `SanitizationEngine` trait, which provides a standard
//! interface for different sanitization implementations, such as regex-based or
//! entropy-based engines. This design promotes a modular and scalable architecture
//! by decoupling the core application logic from the specific sanitization method.

use crate::config::{RedactionConfig, RedactionSummaryItem};
use crate::redaction_match::{RedactionMatch, log_captured_match_debug, log_redaction_action_debug, redact_sensitive};
use crate::sanitizer::{compile_rules, CompiledRules};
use crate::validators;
use anyhow::{Result, Context};
use std::collections::HashMap;
use log::debug;
use strip_ansi_escapes::strip;

/// A trait that defines the core functionality for a sanitization engine.
///
/// Any struct that implements this trait can be used by the main application
/// to perform sanitization tasks. This allows for a flexible and extensible
/// design where new sanitization methods can be added easily.
pub trait SanitizationEngine {
    /// Returns a list of all redaction matches, aggregated by rule name.
    ///
    /// This is the only truly engine-specific method. It performs
    /// the matching logic without creating a new, sanitized string,
    /// and instead returns a map of all detected matches for each rule.
    ///
    /// # Arguments
    ///
    /// * `content` - The input string slice to be analyzed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` where keys are rule names and
    /// values are vectors of `RedactionMatch` objects, or an `anyhow::Error`.
    fn find_all_matches(&self, content: &str) -> Result<HashMap<String, Vec<RedactionMatch>>>;

    /// Retrieves the underlying redaction configuration for the engine.
    ///
    /// This method allows the application to inspect the rules and configuration
    /// being used by the engine. It's useful for debugging, logging, and
    /// providing detailed feedback to the user.
    ///
    /// # Returns
    ///
    /// A reference to the `RedactionConfig` instance associated with this engine.
    fn get_rules(&self) -> &RedactionConfig;

    /// Provides a default implementation for the full sanitization process.
    ///
    /// This method leverages the engine-specific `find_all_matches` to
    /// get the raw matches, then uses a generic `SanitizationContext`
    /// to handle the boilerplate logic of resolving overlaps and building
    /// the final string.
    fn sanitize(&self, content: &str) -> Result<(String, Vec<RedactionSummaryItem>)> {
        let all_matches = self.find_all_matches(content)?;
        let context = SanitizationContext::new(content, all_matches)?;
        Ok(context.run())
    }
}

/// A generic struct to handle the shared logic of sanitization.
///
/// This separates the concerns of *finding* matches from the concerns of
/// *processing* and *applying* them. This is where the boilerplate lives.
struct SanitizationContext<'a> {
    original_input: &'a str,
    pending_replacements: Vec<PendingReplacement>,
}

// A temporary struct to hold a match and its replacement info before final processing
struct PendingReplacement {
    start: usize,
    end: usize,
    replacement: String,
    rule_name: String,
    original_string: String,
}

impl<'a> SanitizationContext<'a> {
    /// Constructs a new `SanitizationContext` by processing all raw matches.
    pub fn new(original_input: &'a str, all_matches: HashMap<String, Vec<RedactionMatch>>) -> Result<Self> {
        let mut pending_replacements: Vec<PendingReplacement> = Vec::new();
        for (rule_name, matches) in all_matches {
            for m in matches {
                pending_replacements.push(PendingReplacement {
                    start: m.start,
                    end: m.end,
                    replacement: m.sanitized_string,
                    rule_name: rule_name.clone(),
                    original_string: m.original_string,
                });
            }
        }

        // Sort and resolve overlapping matches
        pending_replacements.sort_by(|a, b| {
            a.start.cmp(&b.start).then_with(|| b.original_string.len().cmp(&a.original_string.len()))
        });
        
        let mut resolved_replacements: Vec<PendingReplacement> = Vec::new();
        let mut last_end = 0;
        for pending_match in pending_replacements {
            if pending_match.start >= last_end {
                last_end = pending_match.end;
                resolved_replacements.push(pending_match);
            } else {
                debug!("Skipping overlapping match for rule '{}' at position {}-{} because it overlaps with a previous match.",
                    pending_match.rule_name, pending_match.start, pending_match.end);
            }
        }
        
        Ok(Self {
            original_input,
            pending_replacements: resolved_replacements,
        })
    }

    /// Performs the content reconstruction and returns the final result.
    pub fn run(self) -> (String, Vec<RedactionSummaryItem>) {
        let mut all_redaction_matches: Vec<RedactionMatch> = Vec::new();
        let mut current_pos = 0;
        let mut final_string = String::new();
        
        for replacement in &self.pending_replacements {
            final_string.push_str(&self.original_input[current_pos..replacement.start]);
            final_string.push_str(&replacement.replacement);
            current_pos = replacement.end;
            
            log_redaction_action_debug(
                "cleansh_core::engine",
                &replacement.original_string,
                &replacement.replacement,
                &replacement.rule_name,
            );
            
            all_redaction_matches.push(RedactionMatch {
                rule_name: replacement.rule_name.clone(),
                original_string: replacement.original_string.clone(),
                sanitized_string: replacement.replacement.clone(),
                start: replacement.start,
                end: replacement.end,
            });
        }
        
        final_string.push_str(&self.original_input[current_pos..]);

        let summary = self.aggregate_matches_into_summary(all_redaction_matches.clone());
        (final_string, summary)
    }

    /// A private helper function to convert a flat list of `RedactionMatch`es
    /// into a summary of `RedactionSummaryItem`s.
    fn aggregate_matches_into_summary(&self, matches: Vec<RedactionMatch>) -> Vec<RedactionSummaryItem> {
        let mut summary_map: HashMap<String, RedactionSummaryItem> = HashMap::new();

        for m in matches {
            let item = summary_map.entry(m.rule_name.clone()).or_insert_with(|| RedactionSummaryItem {
                rule_name: m.rule_name.clone(),
                occurrences: 0,
                original_texts: Vec::new(),
                sanitized_texts: Vec::new(),
            });
            item.occurrences += 1;
            if !item.original_texts.contains(&m.original_string) {
                item.original_texts.push(m.original_string.clone());
            }
            if !item.sanitized_texts.contains(&m.sanitized_string) {
                item.sanitized_texts.push(m.sanitized_string.clone());
            }
        }
        summary_map.into_values().collect()
    }
}

/// A sanitization engine that uses regular expressions for pattern matching.
///
/// This struct now only handles its specific task: finding matches using regex.
#[derive(Debug)]
pub struct RegexEngine {
    compiled_rules: CompiledRules,
    config: RedactionConfig,
}

impl RegexEngine {
    pub fn new(config: RedactionConfig) -> Result<Self> {
        let compiled_rules = compile_rules(config.rules.clone())
            .context("Failed to compile redaction rules for RegexEngine")?;
        
        Ok(Self { compiled_rules, config })
    }
}

impl SanitizationEngine for RegexEngine {
    fn find_all_matches(&self, content: &str) -> Result<HashMap<String, Vec<RedactionMatch>>> {
        let stripped_bytes = strip(content.as_bytes());
        let stripped_input = String::from_utf8_lossy(&stripped_bytes).to_string();

        let mut all_matches: HashMap<String, Vec<RedactionMatch>> = HashMap::new();

        for compiled_rule in &self.compiled_rules.rules {
            let rule_name = &compiled_rule.name;
            for caps in compiled_rule.regex.captures_iter(&stripped_input) {
                let original_match = caps.get(0).unwrap();
                let original_string = original_match.as_str().to_string();

                let should_redact = if compiled_rule.programmatic_validation {
                    match rule_name.as_str() {
                        "us_ssn" => validators::is_valid_ssn_programmatically(&original_string),
                        "uk_nino" => validators::is_valid_uk_nino_programmatically(&original_string),
                        _ => {
                            debug!("Programmatic validation enabled for rule '{}', but no specific validator function found. Redacting by default.", rule_name);
                            true
                        }
                    }
                } else {
                    true
                };

                if should_redact {
                    let mut final_replacement = compiled_rule.replace_with.clone();
                    for i in 1..caps.len() {
                        if let Some(group) = caps.get(i) {
                            let placeholder = format!("${}", i);
                            final_replacement = final_replacement.replace(&placeholder, group.as_str());
                        }
                    }
                    
                    log_captured_match_debug("cleansh_core::engine", rule_name, &original_string);
                    all_matches.entry(rule_name.clone()).or_default().push(RedactionMatch {
                        rule_name: rule_name.clone(),
                        original_string: original_string.clone(),
                        sanitized_string: final_replacement,
                        start: original_match.start(),
                        end: original_match.end(),
                    });
                } else {
                    debug!("Rule '{}' matched '{}' but programmatic validation failed. Keeping original text.", rule_name, redact_sensitive(&original_string));
                }
            }
        }
        Ok(all_matches)
    }

    fn get_rules(&self) -> &RedactionConfig {
        &self.config
    }
}