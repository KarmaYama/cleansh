// src/tools/sanitize_shell.rs

use anyhow::{Result, Context, anyhow};
use regex::{Regex, RegexBuilder};
use std::collections::{HashMap, HashSet};
use log::{debug, warn, error};

use crate::config::{RedactionRule, RedactionSummaryItem, MAX_PATTERN_LENGTH}; // Import MAX_PATTERN_LENGTH
use crate::tools::validators; 

/// Represents a compiled redaction rule.
#[derive(Debug)]
pub struct CompiledRule {
    pub regex: Regex,
    pub replace_with: String,
    pub name: String,
    pub programmatic_validation: bool,
}

/// Represents all compiled rules for efficient sanitization.
#[derive(Debug)]
pub struct CompiledRules {
    pub rules: Vec<CompiledRule>, // Made public for testing and access
}

/// Compiles a list of `RedactionRule`s into `CompiledRules` for efficient matching.
///
/// This function filters rules based on `enable_rules` and `disable_rules` lists,
/// enforces pattern length limits, compiles regular expressions, and handles errors.
pub fn compile_rules(
    rules_to_compile: Vec<RedactionRule>,
    enable_rules: &[String],
    disable_rules: &[String],
) -> Result<CompiledRules> {
    let enable_set: HashSet<&str> = enable_rules.iter().map(String::as_str).collect();
    let disable_set: HashSet<&str> = disable_rules.iter().map(String::as_str).collect();

    let mut compiled_rules = Vec::new();
    let mut compilation_errors = Vec::new();

    for rule in rules_to_compile {
        let rule_name_str = rule.name.as_str();

        // Check if rule is disabled
        if disable_set.contains(rule_name_str) {
            debug!("Rule '{}' disabled by user, skipping compilation.", rule.name);
            continue;
        }

        // Check opt-in rules: only compile if explicitly enabled
        if rule.opt_in && !enable_set.contains(rule_name_str) {
            debug!("Opt-in rule '{}' not explicitly enabled, skipping compilation.", rule.name);
            continue;
        }

        // Enforce maximum pattern length to guard against runaway regexes
        if rule.pattern.len() > MAX_PATTERN_LENGTH {
            let error_msg = format!(
                "Rule '{}': pattern length ({}) exceeds maximum allowed ({})",
                rule.name,
                rule.pattern.len(),
                MAX_PATTERN_LENGTH
            );
            error!("Compilation error: {}", error_msg);
            compilation_errors.push(error_msg);
            continue;
        }

        // Build regex with specified options and a size limit to prevent ReDoS
        let regex_result = RegexBuilder::new(&rule.pattern)
            .multi_line(rule.multiline)
            .dot_matches_new_line(rule.dot_matches_new_line)
            .size_limit(10 * (1 << 20)) // 10 MB limit for compiled regex, example
            .build();

        match regex_result {
            Ok(regex) => {
                compiled_rules.push(CompiledRule {
                    regex,
                    replace_with: rule.replace_with,
                    name: rule.name,
                    programmatic_validation: rule.programmatic_validation,
                });
            }
            Err(e) => {
                let error_msg = format!(
                    "Rule '{}': failed to compile regex pattern '{}': {}",
                    rule.name, rule.pattern, e
                );
                error!("Compilation error: {}", error_msg); // Log at error level
                compilation_errors.push(error_msg);
            }
        }
    }

    if !compilation_errors.is_empty() {
        let full_error_message = format!(
            "Failed to compile {} rule(s):\n{}",
            compilation_errors.len(),
            compilation_errors.join("\n")
        );
        Err(anyhow!(full_error_message)) // Return aggregated error
    } else {
        Ok(CompiledRules { rules: compiled_rules })
    }
}

/// Sanitizes the input content using the compiled rules.
///
/// Returns the sanitized content and a summary of redactions made.
pub fn sanitize_content(
    input_content: &str,
    compiled_rules: &CompiledRules,
) -> (String, Vec<RedactionSummaryItem>) {
    let mut sanitized_content = input_content.to_string();
    let mut summary_map: HashMap<String, RedactionSummaryItem> = HashMap::new();

    for compiled_rule in &compiled_rules.rules {
        let rule_name = &compiled_rule.name;
        let mut occurrences = 0;
        let mut original_matches: HashSet<String> = HashSet::new();
        let mut sanitized_replacements: HashSet<String> = HashSet::new();

        sanitized_content = compiled_rule.regex.replace_all(&sanitized_content, |caps: &regex::Captures| {
            let original_match = caps.get(0).unwrap().as_str().to_string();

            // Perform programmatic validation if enabled for the rule
            let should_redact = if compiled_rule.programmatic_validation {
                match rule_name.as_str() {
                    "us_ssn" => validators::is_valid_ssn_programmatically(&original_match),
                    "uk_nino" => validators::is_valid_uk_nino_programmatically(&original_match),
                    _ => {
                        warn!("Programmatic validation enabled for rule '{}', but no specific validator function found. Redacting by default.", rule_name);
                        true // Default to redacting if no specific validator is found
                    }
                }
            } else {
                true // No programmatic validation, always redact if regex matches
            };

            if should_redact {
                occurrences += 1;
                original_matches.insert(original_match); // Store unique original matches
                sanitized_replacements.insert(compiled_rule.replace_with.clone()); // Store unique sanitized replacements
                compiled_rule.replace_with.clone()
            } else {
                debug!("Rule '{}' matched '{}' but programmatic validation failed. Keeping original text.", rule_name, original_match);
                original_match // Keep original text if programmatic validation fails
            }
        }).to_string();

        if occurrences > 0 {
            let item = summary_map.entry(rule_name.clone()).or_insert_with(|| RedactionSummaryItem {
                rule_name: rule_name.clone(),
                occurrences: 0,
                original_texts: Vec::new(),
                sanitized_texts: Vec::new(),
            });
            item.occurrences += occurrences;
            item.original_texts.extend(original_matches.into_iter());
            item.sanitized_texts.extend(sanitized_replacements.into_iter());
        }
    }

    // Sort original_texts and sanitized_texts within each summary item for consistent output
    for item in summary_map.values_mut() {
        item.original_texts.sort();
        item.sanitized_texts.sort();
    }

    let mut summary: Vec<RedactionSummaryItem> = summary_map.into_values().collect();
    // Sort the overall summary by rule name for deterministic output/tests
    summary.sort_by(|a, b| a.rule_name.cmp(&b.rule_name));


    (sanitized_content, summary)
}