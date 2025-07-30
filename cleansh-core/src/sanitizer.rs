// cleansh-workspace/cleansh-core/src/sanitizer.rs
//! Core sanitization engine for Cleansh.
//!
//! This module is responsible for compiling redaction rules into efficient regular expressions
//! and applying them to input content. It handles the actual process of identifying sensitive
//! data, performing programmatic validation where necessary, and replacing the matched content
//! with a specified replacement string. It also manages the stripping of ANSI escape codes
//! to ensure accurate pattern matching on raw text.
//!
//! This module works closely with `config` (for rule definitions), `validators` (for
//! advanced pattern validation), and `redaction_match` (for logging and result types).
//! License: BUSL-1.1


use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use log::debug;
use strip_ansi_escapes::strip;

// Import types and functions from other modules within cleansh-core
use crate::redaction_match::{log_captured_match_debug, log_redaction_action_debug, RedactionMatch, redact_sensitive};
use crate::config::{RedactionRule, MAX_PATTERN_LENGTH};
use crate::validators; // Import the validators module

/// Represents a single compiled redaction rule.
///
/// This struct holds a compiled regular expression along with its associated
/// replacement text and metadata, ready for efficient application to content.
#[derive(Debug)]
pub struct CompiledRule {
    /// The compiled regular expression used for matching.
    pub regex: Regex,
    /// The string to replace matches of this rule's pattern with.
    pub replace_with: String,
    /// The unique name of the redaction rule.
    pub name: String,
    /// A flag indicating if this rule requires additional programmatic validation
    /// beyond just regex matching (e.g., Luhn check for credit cards).
    pub programmatic_validation: bool,
}

/// Represents a collection of all compiled rules for efficient sanitization.
///
/// This struct acts as the primary container for the set of rules that will be
/// applied during a sanitization operation.
#[derive(Debug)]
pub struct CompiledRules {
    /// A vector of `CompiledRule` instances ready for application.
    pub rules: Vec<CompiledRule>,
}

/// Compiles a list of `RedactionRule`s into `CompiledRules` for efficient matching.
///
/// This function processes a vector of raw `RedactionRule` definitions,
/// filtering them based on explicit `enable_rules` and `disable_rules` lists.
/// It enforces pattern length limits and compiles each regular expression.
/// Rules that fail to compile or exceed length limits are reported as errors.
/// Opt-in rules are only compiled if explicitly present in `enable_rules`.
///
/// # Arguments
///
/// * `rules_to_compile` - A `Vec<RedactionRule>` containing the raw rule definitions.
/// * `enable_rules` - A slice of `String`s representing the names of rules to explicitly enable.
///                    Only opt-in rules need to be explicitly enabled.
/// * `disable_rules` - A slice of `String`s representing the names of rules to explicitly disable.
///
/// # Returns
///
/// A `Result` which is `Ok(CompiledRules)` on successful compilation of all active rules,
/// or an `anyhow::Error` if any rule fails to compile or violates constraints.
/// The error message will aggregate details of all compilation failures.
///
/// # Errors
///
/// Returns an `anyhow::Error` if:
/// * A rule's pattern exceeds `MAX_PATTERN_LENGTH`.
/// * A rule's regex pattern is syntactically invalid and fails to compile.
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
    let mut found_rules_in_config: HashSet<String> = HashSet::new();

    for rule in rules_to_compile {
        let rule_name_for_debug = rule.name.clone();
        let rule_name_str = rule_name_for_debug.as_str();

        found_rules_in_config.insert(rule_name_str.to_string());

        debug!("Processing rule: '{}', opt_in: {}", rule_name_str, rule.opt_in);

        if disable_set.contains(rule_name_str) {
            debug!("Rule '{}' disabled by user, skipping compilation.", rule_name_str);
            continue;
        }

        if rule.opt_in && !enable_set.contains(rule_name_str) {
            debug!("Opt-in rule '{}' not explicitly enabled, skipping compilation.", rule_name_str);
            continue;
        }

        if rule.pattern.len() > MAX_PATTERN_LENGTH {
            let error_msg = format!(
                "Rule '{}': pattern length ({}) exceeds maximum allowed ({})",
                rule_name_str,
                rule.pattern.len(),
                MAX_PATTERN_LENGTH
            );
            debug!("Compilation error: {}", error_msg);
            compilation_errors.push(error_msg);
            continue;
        }

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
                debug!("Rule '{}' compiled successfully.", rule_name_str);
            }
            Err(e) => {
                let error_msg = format!(
                    "Rule '{}': failed to compile regex pattern: {}",
                    rule_name_str, e
                );
                debug!("Compilation error: {}", error_msg);
                compilation_errors.push(error_msg);
                continue;
            }
        }
    }

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
        Err(anyhow!(full_error_message))
    } else {
        debug!("Finished compiling rules. Total compiled: {}", compiled_rules.len());
        Ok(CompiledRules { rules: compiled_rules })
    }
}

/// Sanitizes the input content using the provided compiled rules.
///
/// This function processes the `input_content` by first stripping ANSI escape codes,
/// then iterating through the `compiled_rules` to find and redact sensitive patterns.
/// For rules requiring programmatic validation, it calls the appropriate validator
/// functions from the `validators` module.
///
/// It collects and returns a detailed list of all redaction matches found.
///
/// # Arguments
///
/// * `input_content` - The string content to be sanitized.
/// * `compiled_rules` - A reference to `CompiledRules` containing the active and compiled redaction rules.
///
/// # Returns
///
/// A tuple containing:
/// * `String`: The sanitized content with sensitive data replaced.
/// * `Vec<RedactionMatch>`: A vector of `RedactionMatch` instances, detailing each
///   original match, its rule, and its sanitized replacement.
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

    let mut sanitized_content = stripped_input.clone();
    let mut all_redaction_matches: Vec<RedactionMatch> = Vec::new();

    debug!("sanitize_content called. Num compiled rules: {}", compiled_rules.rules.len());
    debug!("Sanitize called. Input content length: {}", stripped_input.len());

    for compiled_rule in &compiled_rules.rules {
        let rule_name = &compiled_rule.name;
        let replace_with_val = compiled_rule.replace_with.clone();

        debug!("Applying rule: '{}'", rule_name);
        debug!("Rule '{}' compiled.", rule_name);

        debug!("Rule '{}' does pattern match input? {}", rule_name, compiled_rule.regex.is_match(&sanitized_content));

        sanitized_content = compiled_rule.regex.replace_all(&sanitized_content, |caps: &regex::Captures| {
            let original_match = caps.get(0).unwrap().as_str().to_string();

            // Centralized PII logging for 'captured match'
            log_captured_match_debug(
                "cleansh_core::sanitizer", // Updated module path
                rule_name,
                &original_match
            );

            let should_redact: bool = if compiled_rule.programmatic_validation {
                match rule_name.as_str() {
                    "us_ssn" => validators::is_valid_ssn_programmatically(&original_match),
                    "uk_nino" => validators::is_valid_uk_nino_programmatically(&original_match),
                    _ => {
                        debug!("Programmatic validation enabled for rule '{}', but no specific validator function found. Redacting by default.", rule_name);
                        true
                    }
                }
            } else {
                true
            };

            if should_redact {
                all_redaction_matches.push(RedactionMatch {
                    rule_name: rule_name.clone(),
                    original_string: original_match.clone(),
                    sanitized_string: replace_with_val.clone(),
                });

                debug!("Added RedactionMatch for rule '{}'. Current total matches: {}", rule_name, all_redaction_matches.len());

                // Centralized PII logging for 'redaction action'
                log_redaction_action_debug(
                    "cleansh_core::sanitizer", // Updated module path
                    &original_match,
                    &replace_with_val,
                    rule_name
                );
                replace_with_val.clone()
            } else {
                debug!("Rule '{}' matched '{}' but programmatic validation failed. Keeping original text.", rule_name, redact_sensitive(&original_match));
                original_match
            }
        }).to_string();
    }

    debug!("Sanitization complete. Total individual matches found: {}", all_redaction_matches.len());
    (sanitized_content, all_redaction_matches)
}