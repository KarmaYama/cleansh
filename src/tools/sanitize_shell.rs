use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder};
use std::collections::{HashMap, HashSet};
use log::{debug, warn, error};
use strip_ansi_escapes::strip;

// Import the moved functions and macro from the new utility module
use crate::utils::redaction::{pii_debug, redact_sensitive};

use crate::config::{RedactionRule, RedactionSummaryItem, MAX_PATTERN_LENGTH};
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
                debug!("Rule '{}' compiled successfully.", rule_name_str);
            }
            Err(e) => {
                let error_msg = format!(
                    "Rule '{}': failed to compile regex pattern: {}",
                    rule_name_str, e // Removed rule.pattern from log
                );
                error!("Compilation error: {}", error_msg);
                compilation_errors.push(error_msg);
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
        error!("{}", full_error_message); // Use log::error! for the final error
        Err(anyhow!(full_error_message))
    } else {
        debug!("Finished compiling rules. Total compiled: {}", compiled_rules.len());
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
    // Step 1: Strip ANSI escape codes from the input content
    // strip() returns Vec<u8> directly.
    let stripped_bytes = strip(input_content.as_bytes());

    let stripped_input = match String::from_utf8(stripped_bytes) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                "Invalid UTF-8 after ANSI stripping: {}. Falling back to lossy conversion.",
                e
            );
            // Fallback to original content bytes for lossy conversion if the stripped bytes
            // are not valid UTF-8. This might mean the original input was already problematic,
            // or stripping introduced an issue (less likely, but possible).
            // The test expects "test@example.com" for original_texts, so we need to ensure
            // that the string passed to replace_all is clean. If the stripped_bytes are invalid,
            // we should probably still try to process them, but use the lossy conversion to get a String.
            String::from_utf8_lossy(e.as_bytes()).to_string() // Use the bytes from the error to convert lossily
        }
    };

    let mut sanitized_content = stripped_input.clone(); // Start with the stripped content
    let mut summary_map: HashMap<String, RedactionSummaryItem> = HashMap::new();

    debug!("sanitize_content called. Num compiled rules: {}", compiled_rules.rules.len());
    // Avoid logging entire input content
    debug!("Sanitize called. Input content length: {}", stripped_input.len());


    for compiled_rule in &compiled_rules.rules {
        let rule_name = &compiled_rule.name;
        let mut occurrences = 0;
        let mut original_matches: HashSet<String> = HashSet::new();
        let mut sanitized_replacements: HashSet<String> = HashSet::new(); // Use a HashSet to store unique sanitized values

        debug!("Applying rule: '{}'", rule_name);

        // Avoid logging full regex patterns
        debug!("Rule '{}' compiled.", rule_name);
        pii_debug!("Rule '{}' does pattern match input? {}", rule_name, compiled_rule.regex.is_match(&sanitized_content));


        sanitized_content = compiled_rule.regex.replace_all(&sanitized_content, |caps: &regex::Captures| {
            let original_match = caps.get(0).unwrap().as_str().to_string();

            // Redact sensitive values in logs
            pii_debug!("Rule '{}' captured match (original): {}", rule_name, redact_sensitive(&original_match));


            // Perform programmatic validation if enabled for the rule
            let should_redact: bool = if compiled_rule.programmatic_validation {
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
                original_matches.insert(original_match.clone()); // Store unique original matches
                sanitized_replacements.insert(compiled_rule.replace_with.clone()); // Store unique sanitized replacements
                // Redact sensitive values in logs
                pii_debug!("Redacting '{}' with '{}' for rule '{}'", redact_sensitive(&original_match), redact_sensitive(&compiled_rule.replace_with), rule_name);
                compiled_rule.replace_with.clone()
            } else {
                // Redact sensitive values in logs
                pii_debug!("Rule '{}' matched '{}' but programmatic validation failed. Keeping original text.", rule_name, redact_sensitive(&original_match));
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
            // CORRECTED: Populate original_texts and sanitized_texts
            item.original_texts.extend(original_matches.drain()); // Use drain to move ownership from HashSet
            item.sanitized_texts.extend(sanitized_replacements.drain()); // Use drain to move ownership from HashSet

            debug!("Rule '{}' resulted in {} redactions.", rule_name, occurrences);
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

    debug!("Sanitization complete. Final summary items: {}", summary.len());
    (sanitized_content, summary)
}