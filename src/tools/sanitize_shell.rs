// src/tools/sanitize_shell.rs
// This module provides functions for sanitizing shell commands and their outputs.
// It compiles redaction rules, applies them to input content, and collects matches.
// It also handles ANSI escape codes and programmatic validation of sensitive data.
// This file is part of cleansh, a tool for sanitizing sensitive information in shell commands.


use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use log::{debug};
use strip_ansi_escapes::strip;

// Import the new functions from the redaction utility module.
// The `pii_debug` macro is removed as its logic is now within these functions.
use crate::utils::redaction::{log_captured_match_debug, log_redaction_action_debug, RedactionMatch, redact_sensitive};

use crate::config::{RedactionRule, MAX_PATTERN_LENGTH};
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
    pub rules: Vec<CompiledRule>,
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

    debug!("compile_rules called with {} rules.", rules_to_compile.len());
    debug!("enable_set: {:?}", enable_set);
    debug!("disable_set: {:?}", disable_set);


    let mut compiled_rules = Vec::new();
    let mut compilation_errors = Vec::new();
    let mut found_rules_in_config: HashSet<String> = HashSet::new(); // Track rules found in rules_to_compile

    for rule in rules_to_compile {
        let rule_name_for_debug = rule.name.clone();
        let rule_name_str = rule_name_for_debug.as_str();

        found_rules_in_config.insert(rule_name_str.to_string()); // Mark this rule as found in config

        debug!("Processing rule: '{}', opt_in: {}", rule_name_str, rule.opt_in);


        // Check if rule is disabled
        if disable_set.contains(rule_name_str) {
            debug!("Rule '{}' disabled by user, skipping compilation.", rule_name_str);
            continue;
        }

        // Check opt-in rules: only compile if explicitly enabled
        if rule.opt_in && !enable_set.contains(rule_name_str) {
            debug!("Opt-in rule '{}' not explicitly enabled, skipping compilation.", rule_name_str);
            continue;
        }

        // Enforce maximum pattern length to guard against runaway regexes
        if rule.pattern.len() > MAX_PATTERN_LENGTH {
            let error_msg = format!(
                "Rule '{}': pattern length ({}) exceeds maximum allowed ({})",
                rule_name_str,
                rule.pattern.len(),
                MAX_PATTERN_LENGTH
            );
            debug!("Compilation error: {}", error_msg); // Changed to debug for this specific error as it's an internal constraint
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
                debug!("Rule '{}' compiled successfully.", rule_name_str); // This is a general debug, not PII sensitive
            }
            Err(e) => {
                let error_msg = format!(
                    "Rule '{}': failed to compile regex pattern: {}",
                    rule_name_str, e
                );
                debug!("Compilation error: {}", error_msg); // Changed to debug for this specific error as it's an internal constraint
                compilation_errors.push(error_msg);
                continue; // Continue to next rule instead of returning early
            }
        }
    }

    // NEW LOGIC: Log rules from enable_set that were not found in the configuration
    for enabled_rule_name in enable_set.iter() {
        if !found_rules_in_config.contains(*enabled_rule_name) {
            debug!("Rule '{}' not found in merged configuration, skipping.", enabled_rule_name);
        }
    }

    if !compilation_errors.is_empty() {
        let full_error_message = format!(
            "Failed to compile {} rule(s):\n{}",
            compilation_errors.len(),
            compilation_errors.join("\n")
        );
        Err(anyhow!(full_error_message)) // Return a single anyhow error with all messages
    } else {
        debug!("Finished compiling rules. Total compiled: {}", compiled_rules.len());
        Ok(CompiledRules { rules: compiled_rules })
    }
}

/// Sanitizes the input content using the compiled rules.
///
/// Returns the sanitized content and a vector of all individual `RedactionMatch` instances found.
pub fn sanitize_content(
    input_content: &str,
    compiled_rules: &CompiledRules,
) -> (String, Vec<RedactionMatch>) {
    // Step 1: Strip ANSI escape codes from the input content
    let stripped_bytes = strip(input_content.as_bytes());

    let stripped_input = match String::from_utf8(stripped_bytes) {
        Ok(s) => s,
        Err(e) => {
            debug!(
                "Invalid UTF-8 after ANSI stripping: {}. Falling back to lossy conversion.",
                e
            );
            String::from_utf8_lossy(e.as_bytes()).to_string()
        }
    };

    let mut sanitized_content = stripped_input.clone(); // Start with the stripped content
    let mut all_redaction_matches: Vec<RedactionMatch> = Vec::new(); // NEW: Collect all individual matches

    debug!("sanitize_content called. Num compiled rules: {}", compiled_rules.rules.len());
    debug!("Sanitize called. Input content length: {}", stripped_input.len());


    for compiled_rule in &compiled_rules.rules {
        let rule_name = &compiled_rule.name;
        let replace_with_val = compiled_rule.replace_with.clone(); // Clone once per rule for the closure

        debug!("Applying rule: '{}'", rule_name); // This is a general debug, not PII sensitive
        debug!("Rule '{}' compiled.", rule_name); // This is a general debug, not PII sensitive

        // This debug! is not PII sensitive, so it doesn't need the redaction utility functions.
        debug!("Rule '{}' does pattern match input? {}", rule_name, compiled_rule.regex.is_match(&sanitized_content));

        sanitized_content = compiled_rule.regex.replace_all(&sanitized_content, |caps: &regex::Captures| {
            let original_match = caps.get(0).unwrap().as_str().to_string();

            // Centralized PII logging for 'captured match'
            log_captured_match_debug(
                "[cleansh::tools::sanitize_shell]", // Correct module path
                rule_name,
                &original_match
            );

            // Perform programmatic validation ONLY to decide on ACTUAL REDACTION
            let should_redact: bool = if compiled_rule.programmatic_validation {
                match rule_name.as_str() {
                    "us_ssn" => validators::is_valid_ssn_programmatically(&original_match),
                    "uk_nino" => validators::is_valid_uk_nino_programmatically(&original_match),
                    _ => {
                        debug!("Programmatic validation enabled for rule '{}', but no specific validator function found. Redacting by default.", rule_name);
                        true // Default to redacting if no specific validator is found
                    }
                }
            } else {
                true // No programmatic validation, always redact if regex matches
            };

            if should_redact {
                all_redaction_matches.push(RedactionMatch {
                    rule_name: rule_name.clone(),
                    original_string: original_match.clone(),
                    sanitized_string: replace_with_val.clone(),
                });

                // This debug is still useful, but if it contained PII, it would also be centralized.
                // For 'total matches', it's just a count, not PII.
                debug!("Added RedactionMatch for rule '{}'. Current total matches: {}", rule_name, all_redaction_matches.len());

                // Centralized PII logging for 'redaction action'
                log_redaction_action_debug(
                    "[cleansh::tools::sanitize_shell]", // Correct module path
                    &original_match,
                    &replace_with_val,
                    rule_name
                );
                replace_with_val.clone() // Return the replacement for `replace_all`
            } else {
                // Centralized PII logging for validation failure
                // Use redact_sensitive here because this log *is* directly showing the failed validation.
                debug!("Rule '{}' matched '{}' but programmatic validation failed. Keeping original text.", rule_name, redact_sensitive(&original_match));
                original_match // Keep original text if programmatic validation fails
            }
        }).to_string();
    }

    debug!("Sanitization complete. Total individual matches found: {}", all_redaction_matches.len());
    (sanitized_content, all_redaction_matches) // Return both sanitized content and all matches
}