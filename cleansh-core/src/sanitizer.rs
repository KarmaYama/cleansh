//! Core sanitization engine for CleanSH.
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
// Removed the unused `use std::collections::HashSet;` import.
use log::debug;

// Import types and functions from other modules within cleansh-core
use crate::config::{RedactionRule, MAX_PATTERN_LENGTH};

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
/// It enforces pattern length limits and compiles each regular expression.
/// Rules that fail to compile or exceed length limits are reported as errors.
///
/// # Arguments
///
/// * `rules_to_compile` - A `Vec<RedactionRule>` containing the raw rule definitions.
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
) -> Result<CompiledRules> {
    debug!("compile_rules called with {} rules.", rules_to_compile.len());

    let mut compiled_rules = Vec::new();
    let mut compilation_errors = Vec::new();

    for rule in rules_to_compile {
        let rule_name_for_debug = rule.name.clone();
        let rule_name_str = rule_name_for_debug.as_str();

        debug!("Processing rule: '{}'", rule_name_str);

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