// src/tools/sanitize_shell.rs

use anyhow::{Context, Result};
use log::{debug, trace, warn};
use regex::{Regex, RegexBuilder, Replacer};
use strip_ansi_escapes;

use crate::config::{self, Rule as ConfigRule}; // Alias config::Rule to ConfigRule to avoid name collision
use crate::ui::RedactionSummaryItem; // For tracking and reporting redactions

/// A compiled redaction rule ready for application.
/// This struct holds the compiled regex and its associated replacement string and metadata.
pub struct SanitizationRule {
    pub name: String,
    compiled_regex: Regex,
    replace_with: String,
    original_pattern: String, // Stored for debugging/logging purposes
    description: Option<String>,
}

impl std::fmt::Debug for SanitizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SanitizationRule")
            .field("name", &self.name)
            .field("pattern", &self.original_pattern)
            .field("replace_with", &self.replace_with)
            .field("description", &self.description)
            .finish()
    }
}


/// Compiles a set of `ConfigRule`s into `SanitizationRule`s.
///
/// This function converts the declarative rules from `config.rs` into
/// executable regex objects, handling potential regex compilation errors.
///
/// # Arguments
/// * `rules_config` - The `RulesConfig` containing the raw rule definitions.
///
/// # Returns
/// A `Result` containing a `Vec<SanitizationRule>` on success, or an `anyhow::Error`
/// if any regex pattern fails to compile.
pub fn compile_rules(rules_config: config::RulesConfig) -> Result<Vec<SanitizationRule>> {
    let mut compiled_rules = Vec::with_capacity(rules_config.rules.len());

    for rule in rules_config.rules {
        debug!("Attempting to compile rule: {}", rule.name);
        trace!("Pattern: '{}'", rule.pattern);

        let compiled_regex = RegexBuilder::new(&rule.pattern)
            .multi_line(rule.multiline) // Apply multiline flag from rule config
            .build()
            .with_context(|| {
                format!(
                    "Failed to compile regex for rule '{}' with pattern: '{}'",
                    rule.name, rule.pattern
                )
            })?; // Propagate compilation errors

        compiled_rules.push(SanitizationRule {
            name: rule.name,
            compiled_regex,
            replace_with: rule.replace_with,
            original_pattern: rule.pattern,
            description: rule.description,
        });
        debug!("Successfully compiled rule: {}", compiled_rules.last().unwrap().name);
    }
    Ok(compiled_rules)
}

/// Sanitizes the input content by applying a given set of `SanitizationRule`s.
///
/// This is the core redaction function. It first strips ANSI escape codes,
/// then iterates through each compiled rule, applying replacements.
///
/// # Arguments
/// * `content` - The raw input string to be sanitized.
/// * `rules` - A slice of `SanitizationRule`s to apply.
///
/// # Returns
/// A tuple containing:
/// 1. The `String` with sanitized content.
/// 2. A `Vec<RedactionSummaryItem>` detailing the redactions made.
pub fn sanitize_content(
    content: &str,
    rules: &[SanitizationRule],
) -> (String, Vec<RedactionSummaryItem>) {
    debug!("Starting content sanitization.");
    let mut sanitized_content = strip_ansi_escapes::strip_ansi_escapes(content)
        .into_string() // Convert Cow to String
        .unwrap_or_else(|_| {
            // strip_ansi_escapes returns Cow<str>, and into_string can fail on invalid UTF-8
            // which is unlikely for terminal output but handled gracefully.
            warn!("Failed to convert ANSI-stripped content to String. Proceeding with raw content.");
            content.to_string()
        });

    let original_content = sanitized_content.clone(); // Clone for diffing later
    let mut redaction_summary_items: Vec<RedactionSummaryItem> = Vec::new();

    // Iterate through each rule and apply its regex
    for rule in rules {
        trace!(
            "Applying rule '{}' with pattern '{}'",
            rule.name,
            rule.original_pattern
        );

        // Find all matches for the current rule
        let matches: Vec<(std::ops::Range<usize>, String)> = rule
            .compiled_regex
            .find_iter(&sanitized_content)
            .map(|m| (m.range(), m.as_str().to_string())) // Store original match string and range
            .collect();

        if matches.is_empty() {
            trace!("No matches found for rule '{}'.", rule.name);
            continue;
        }

        debug!(
            "Found {} matches for rule '{}'.",
            matches.len(),
            rule.name
        );

        // Perform replacement. Regex::replace_all is efficient for multiple matches.
        // It requires a custom replacer for advanced scenarios or can use a string.
        // For our case, `Replacer` trait needs to be implemented by a custom struct
        // if we want more control than just a simple string replacement.
        // `regex::bytes::Regex` has `replace_all` which takes `&impl Replacer`.
        // `regex::Regex` (for utf8 strings) also has `replace_all`.
        // `regex::Regex::replace_all` directly supports backreferences like `$1`.

        let replaced_content = rule.compiled_regex.replace_all(
            &sanitized_content,
            rule.replace_with.as_str() as &dyn Replacer // Cast to trait object
        ).to_string(); // Convert Cow to String

        // Only record redactions if content actually changed by this rule.
        // This is a simple heuristic; a more advanced system might track per-match changes.
        if replaced_content != sanitized_content {
            for (range, original_match_text) in &matches {
                // Determine a sensible snippet to show in the summary
                let start_idx = range.start.saturating_sub(10); // 10 chars before
                let end_idx = range.end.saturating_add(10);   // 10 chars after
                let original_snippet = &original_content[start_idx..original_content.len().min(end_idx)];

                let sanitized_snippet = rule.compiled_regex.replace(original_snippet, rule.replace_with.as_str()).to_string();


                redaction_summary_items.push(RedactionSummaryItem {
                    rule_name: &rule.name,
                    original_text: original_match_text, // Use the exact original matched text
                    sanitized_text: &rule.replace_with, // Use the replacement string
                });
            }
            sanitized_content = replaced_content;
        }
    }

    debug!("Sanitization complete. {} redactions recorded.", redaction_summary_items.len());
    (sanitized_content, redaction_summary_items)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Rule, RulesConfig};

    // Helper to create a simple RulesConfig for tests
    fn create_test_rules_config(rules_data: &[(&str, &str, &str, bool)]) -> RulesConfig {
        let rules = rules_data
            .iter()
            .map(|&(name, pattern, replace, multiline)| Rule {
                name: name.to_string(),
                pattern: pattern.to_string(),
                replace_with: replace.to_string(),
                description: None,
                multiline,
            })
            .collect();
        RulesConfig { rules }
    }

    #[test]
    fn test_compile_rules_success() {
        let rules_config = create_test_rules_config(&[
            ("test_email", r"test@example\.com", "[EMAIL]", false),
            ("test_ip", r"127\.0\.0\.1", "[IP]", false),
        ]);
        let compiled = compile_rules(rules_config).unwrap();
        assert_eq!(compiled.len(), 2);
        assert_eq!(compiled[0].name, "test_email");
        assert_eq!(compiled[1].name, "test_ip");
    }

    #[test]
    fn test_compile_rules_invalid_regex() {
        let rules_config = create_test_rules_config(&[
            ("invalid_regex", "[", "[BAD_REGEX]", false), // Invalid regex pattern
        ]);
        let err = compile_rules(rules_config).unwrap_err();
        assert!(err.to_string().contains("Failed to compile regex for rule 'invalid_regex'"));
    }

    #[test]
    fn test_sanitize_content_email() {
        let rules_config = create_test_rules_config(&[
            ("email", r"test@example\.com", "[REDACTED_EMAIL]", false),
        ]);
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = "My email is test@example.com.";
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        assert_eq!(sanitized, "My email is [REDACTED_EMAIL].");
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].rule_name, "email");
        assert_eq!(summary[0].original_text, "test@example.com");
        assert_eq!(summary[0].sanitized_text, "[REDACTED_EMAIL]");
    }

    #[test]
    fn test_sanitize_content_ip_address() {
        let rules_config = create_test_rules_config(&[
            ("ip_addr", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IP_ADDR]", false),
        ]);
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = "Server IP: 192.168.1.1, another: 10.0.0.5";
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        assert_eq!(sanitized, "Server IP: [IP_ADDR], another: [IP_ADDR]");
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].rule_name, "ip_addr");
        assert_eq!(summary[1].rule_name, "ip_addr");
    }

    #[test]
    fn test_sanitize_content_path_normalization() {
        let rules_config = create_test_rules_config(&[
            ("linux_path", r"(/home/[a-zA-Z0-9_.-]+(?:/[a-zA-Z0-9_.-]+)*)", "~$1"), // Note the $1 backreference
            ("macos_path", r"(/Users/[a-zA-Z0-9_.-]+(?:/[a-zA-Z0-9_.-]+)*)", "~$1"), // Note the $1 backreference
        ]);
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = "Path: /home/user/documents/report.txt and /Users/apple/projects.";
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        // The regex captures the full path including /home/ or /Users/
        // So "$1" will include it. My previous thought of "~$" was incorrect for `regex::replace_all`.
        // It correctly replaces with the literal "~" followed by the first captured group.
        assert_eq!(sanitized, "Path: ~/home/user/documents/report.txt and ~/Users/apple/projects.");
        // The summary items will still show the *original* full path.
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].original_text, "/home/user/documents/report.txt");
        assert_eq!(summary[0].sanitized_text, "~$1"); // This is the replacement *pattern*, not the final string
        assert_eq!(summary[1].original_text, "/Users/apple/projects.");
        assert_eq!(summary[1].sanitized_text, "~$1"); // This is the replacement *pattern*, not the final string
    }

    #[test]
    fn test_sanitize_content_ssh_key_multiline() {
        let rules_config = create_test_rules_config(&[(
            "ssh_key",
            "BEGIN OPENSSH PRIVATE KEY-----.*?-----END OPENSSH PRIVATE KEY",
            "[SSH_KEY_REDACTED]",
            true, // Multiline enabled
        )]);
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = r#"
This is a log.
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAAZXNjZHNhLXNoYTI...
-----END OPENSSH PRIVATE KEY-----
Another line here.
"#;
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        assert!(sanitized.contains("[SSH_KEY_REDACTED]"));
        assert!(!sanitized.contains("BEGIN OPENSSH PRIVATE KEY"));
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].rule_name, "ssh_key");
    }

    #[test]
    fn test_sanitize_content_strip_ansi() {
        let rules_config = create_test_rules_config(&[]); // No rules
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = "\x1b[31mHello\x1b[0m \x1b[32mWorld\x1b[0m";
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        assert_eq!(sanitized, "Hello World");
        assert!(summary.is_empty());
    }

    #[test]
    fn test_sanitize_content_no_matches() {
        let rules_config = create_test_rules_config(&[
            ("email", r"nonexistent@example\.com", "[EMAIL]", false),
        ]);
        let compiled_rules = compile_rules(rules_config).unwrap();
        let content = "Just some plain text.";
        let (sanitized, summary) = sanitize_content(content, &compiled_rules);
        assert_eq!(sanitized, content);
        assert!(summary.is_empty());
    }
}