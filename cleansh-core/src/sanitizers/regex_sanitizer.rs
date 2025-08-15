//! Core regex sanitization engine for CleanSH.
//!
//! This module is responsible for applying compiled redaction rules to input content
//! and generating sanitized output. It manages the stripping of ANSI escape codes,
//! handles overlapping matches, and performs byte-level mapping for accurate
//! redaction.
//!
//! License: BUSL-1.1

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};
use log::debug;
use strip_ansi_escapes::strip;
use sha2::{Digest, Sha256};
use hex;
use chrono::Utc;

use crate::config::{RedactionConfig, RedactionSummaryItem, RedactionRule};
use crate::redaction_match::{RedactionMatch, log_captured_match_debug, redact_sensitive, RedactionLog, ensure_match_hashes};
use crate::profiles::EngineOptions;
use crate::engine::SanitizationEngine;
use crate::sanitizers::compiler::{get_or_compile_rules, CompiledRules, CompiledRule};
use crate::validators;

// --- Final, monotonic byte-based `StrippedIndexMapper` ---
/// A mapper to convert byte indices from a stripped string back to the original string.
/// This is crucial for handling ANSI escape codes, which are removed from the input
/// before regex matching.
#[derive(Debug)]
struct StrippedIndexMapper {
    map: Vec<usize>,
}

impl StrippedIndexMapper {
    fn new(original: &str) -> Self {
        // Strip ANSI codes to get the "clean" text for regex matching.
        let stripped_bytes = strip(original.as_bytes());
        let stripped_str = String::from_utf8_lossy(&stripped_bytes);

        let mut map: Vec<usize> = Vec::with_capacity(stripped_str.len() + 1);
        let mut orig_char_indices = original.char_indices();
        let mut current_orig_char = orig_char_indices.next();

        for stripped_char in stripped_str.chars() {
            while let Some((orig_index, orig_char)) = current_orig_char {
                if orig_char == stripped_char {
                    map.push(orig_index);
                    current_orig_char = orig_char_indices.next();
                    break;
                }
                current_orig_char = orig_char_indices.next();
            }
        }
        
        // Final sentinel entry for `map_index` to handle matches at the end of the string.
        map.push(original.len());

        Self { map }
    }

    /// Maps a byte index from the stripped string to the original string.
    fn map_index(&self, stripped_index: usize) -> usize {
        // Use `saturating_sub` to handle empty strings gracefully.
        let idx = stripped_index.min(self.map.len().saturating_sub(1));
        self.map[idx]
    }
}
// --- end mapper ---

// Using a small batch size for streaming to keep memory usage low.
pub const BATCH_SIZE: usize = 4096;

#[derive(Debug)]
pub struct RegexEngine {
    compiled_rules: Arc<CompiledRules>,
    config: RedactionConfig,
    options: EngineOptions,
}

impl RegexEngine {
    pub fn new(config: RedactionConfig) -> Result<Self> {
        Self::with_options(config, EngineOptions::default())
    }

    pub fn with_options(config: RedactionConfig, options: EngineOptions) -> Result<Self> {
        if config.rules.is_empty() {
            debug!("RedactionConfig contains no rules. The RegexEngine will perform no sanitization.");
        }
        
        let compiled_rules = get_or_compile_rules(&config)
            .context("Failed to get or compile redaction rules for RegexEngine")?;
            
        Ok(Self {
            compiled_rules,
            config,
            options,
        })
    }

    // A helper function to run programmatic validators.
    fn run_programmatic_validator(&self, compiled_rule: &CompiledRule, original_str: &str) -> bool {
        if !compiled_rule.programmatic_validation {
            return true;
        }

        match compiled_rule.name.as_str() {
            "us_ssn" => validators::is_valid_ssn_programmatically(original_str),
            "uk_nino" => validators::is_valid_uk_nino_programmatically(original_str),
            "visa_card" | "mastercard_card" | "amex_card" | "discover_card" => {
                validators::is_valid_credit_card_programmatically(original_str)
            }
            _ => {
                debug!("No validator for '{}', redacting by default.", compiled_rule.name);
                true
            }
        }
    }

    fn create_redaction_match(
        &self,
        rule_config: &RedactionRule,
        original_match_str: &str,
        start: u64,
        end: u64,
        replacement: String,
        stripped_input: &str,
        source_id: &str,
        line_number: Option<u64>,
    ) -> RedactionMatch {
        let mut sample_hash = None;
        let mut match_context_hash = None;

        let needs_sample_hash = self.options.post_processing.as_ref().map_or(false, |pp| pp.replace_with_token) ||
            self.options.samples_config.is_some();
        let needs_context_hash = self.options.dedupe_config.as_ref().map_or(false, |dedupe| dedupe.use_hash);

        if needs_sample_hash || needs_context_hash {
            let mut hasher = Sha256::new();
            if needs_sample_hash {
                hasher.update(original_match_str.as_bytes());
                sample_hash = Some(hex::encode(hasher.finalize_reset()));
            }
            if needs_context_hash {
                let window = self.options.dedupe_config.as_ref().map(|d| d.window_bytes).unwrap_or(0);
                let ctx_start = (start as usize).saturating_sub(window);
                let ctx_end = std::cmp::min(stripped_input.len(), (end as usize).saturating_add(window));
                let ctx = &stripped_input[ctx_start..ctx_end];
                hasher.update(ctx.as_bytes());
                match_context_hash = Some(hex::encode(hasher.finalize()));
            }
        }

        RedactionMatch {
            rule_name: rule_config.name.clone(),
            original_string: original_match_str.to_string(),
            sanitized_string: replacement,
            start,
            end,
            sample_hash,
            match_context_hash,
            timestamp: Some(Utc::now().to_rfc3339()),
            rule: rule_config.clone(),
            source_id: source_id.to_string(),
            line_number,
        }
    }

    /// Finds all matches in the content, running programmatic validators where applicable.
    fn find_matches(&self, content: &str, source_id: &str) -> Result<HashMap<String, Vec<RedactionMatch>>> {
        let stripped_bytes = strip(content.as_bytes());
        let stripped_input = String::from_utf8_lossy(&stripped_bytes);
        
        let original_rules_map: HashMap<&str, &RedactionRule> = self.config.rules.iter()
            .map(|rule| (rule.name.as_str(), rule))
            .collect();
    
        let mut all_matches: HashMap<String, Vec<RedactionMatch>> = HashMap::new();
    
        for compiled_rule in &self.compiled_rules.rules {
            if let Some(rule_config) = original_rules_map.get(compiled_rule.name.as_str()) {
                if let Some(false) = rule_config.enabled {
                    continue;
                }

                for caps in compiled_rule.regex.captures_iter(&stripped_input) {
                    let original_match = caps.get(0).ok_or_else(|| anyhow!("Regex captured a non-existent match group"))?;
                    
                    if self.run_programmatic_validator(compiled_rule, original_match.as_str()) {
                        let mut replacement = compiled_rule.replace_with.clone();
                        for i in 1..caps.len() {
                            if let Some(group) = caps.get(i) {
                                replacement = replacement.replace(&format!("${}", i), group.as_str());
                            }
                        }
                        log_captured_match_debug("cleansh_core::engine", &compiled_rule.name, original_match.as_str());

                        let redaction_match = self.create_redaction_match(
                            rule_config,
                            original_match.as_str(),
                            original_match.start() as u64,
                            original_match.end() as u64,
                            replacement,
                            &stripped_input,
                            source_id,
                            None,
                        );

                        all_matches.entry(compiled_rule.name.clone()).or_default().push(redaction_match);
                    } else {
                        debug!(
                            "Match for '{}' failed programmatic validation: '{}'",
                            compiled_rule.name,
                            redact_sensitive(original_match.as_str())
                        );
                    }
                }
            }
        }
        Ok(all_matches)
    }

    fn build_summary_from_matches(&self, all_matches: &HashMap<String, Vec<RedactionMatch>>) -> Vec<RedactionSummaryItem> {
        let mut summary_items = Vec::new();
        for (rule_name, matches) in all_matches.iter() {
            let original_texts: Vec<String> = matches.iter().map(|m| m.original_string.clone()).collect();
            let sanitized_texts: Vec<String> = matches.iter().map(|m| m.sanitized_string.clone()).collect();

            summary_items.push(RedactionSummaryItem {
                rule_name: rule_name.clone(),
                occurrences: matches.len(),
                original_texts,
                sanitized_texts,
            });
        }
        summary_items
    }
}

impl SanitizationEngine for RegexEngine {
    fn sanitize(
        &self,
        content: &str,
        source_id: &str,
        run_id: &str,
        input_hash: &str,
        user_id: &str,
        reason: &str,
        outcome: &str,
        mut audit_log: Option<&mut crate::audit_log::AuditLog>,
    ) -> Result<(String, Vec<RedactionSummaryItem>)> {
        let all_matches = self.find_matches(content, source_id)?;

        let mut sorted_matches: Vec<&RedactionMatch> = all_matches.values()
            .flatten()
            .collect();
        sorted_matches.sort_by_key(|m| m.start);

        let mapper = StrippedIndexMapper::new(content);

        let mut sanitized_content = String::with_capacity(content.len());
        let mut last_end = 0usize;

        for m in sorted_matches.iter() {
            let original_start_byte = mapper.map_index(m.start as usize);
            let original_end_byte = mapper.map_index(m.end as usize);

            // Handle overlaps
            if original_end_byte <= last_end {
                debug!("Skipping fully-overlapped match for '{}': mapped range {}..{} already covered (last_end={})",
                    m.rule_name, original_start_byte, original_end_byte, last_end);
                continue;
            }

            // Append the content between the last match and the current one,
            // handling partial overlaps by starting from the last match's end.
            let current_start = original_start_byte.max(last_end);
            sanitized_content.push_str(&content[last_end..current_start]);
            
            // Append the sanitized string
            sanitized_content.push_str(&m.sanitized_string);
            
            // Update the last_end pointer
            last_end = original_end_byte;

            if let Some(log) = audit_log.as_mut() {
                let rlog = RedactionLog {
                    timestamp: m.timestamp.clone().unwrap_or_default(),
                    run_id: run_id.to_string(),
                    file_path: source_id.to_string(),
                    user_id: user_id.to_string(),
                    reason_for_redaction: reason.to_string(),
                    redaction_outcome: outcome.to_string(),
                    rule_name: m.rule_name.clone(),
                    input_hash: input_hash.to_string(),
                    match_hash: m.sample_hash.clone().unwrap_or_default(),
                    start: m.start,
                    end: m.end,
                };
                log.append(&rlog)?;
            }
        }

        sanitized_content.push_str(&content[last_end..]);

        let summary = self.build_summary_from_matches(&all_matches);
        Ok((sanitized_content, summary))
    }

    fn analyze_for_stats(&self, content: &str, source_id: &str) -> Result<Vec<RedactionSummaryItem>> {
        let all_matches = self.find_matches(content, source_id)?;
        let summary = self.build_summary_from_matches(&all_matches);
        Ok(summary)
    }

    fn find_matches_for_ui(&self, content: &str, source_id: &str) -> Result<Vec<RedactionMatch>> {
        let all_map = self.find_matches(content, source_id)?;
        let mut out: Vec<RedactionMatch> = Vec::new();

        for (_rule, mut vec_matches) in all_map.into_iter() {
            out.append(&mut vec_matches);
        }

        ensure_match_hashes(&mut out);
        out.sort_by_key(|m| m.start);
        Ok(out)
    }

    fn compiled_rules(&self) -> &CompiledRules {
        &self.compiled_rules
    }

    fn get_rules(&self) -> &RedactionConfig {
        &self.config
    }

    fn get_options(&self) -> &EngineOptions {
        &self.options
    }
}