// src/tools/sanitize_shells.rs


use anyhow::{Context, Result};
use log::{debug, trace};
use regex::{Regex, RegexBuilder};
use strip_ansi_escapes::strip as strip_ansi_escapes_fn;

use crate::config::{self, RedactionSummaryItem};

/// Represents a compiled sanitization rule with its associated regex and replacement logic.
pub struct SanitizationRule {
    pub name: String,
    compiled_regex: Regex,
    replace_with: String,
    original_pattern: String,
    description: Option<String>,
    dot_matches_new_line: bool,
}

impl std::fmt::Debug for SanitizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SanitizationRule")
            .field("name", &self.name)
            .field("pattern", &self.original_pattern)
            .field("replace_with", &self.replace_with)
            .field("description", &self.description)
            .field("dot_matches_new_line", &self.dot_matches_new_line)
            .finish()
    }
}

/// Compiles user-provided YAML rules into regex-ready SanitizationRules.
pub fn compile_rules(rules_config: config::RulesConfig) -> Result<Vec<SanitizationRule>> {
    let mut compiled_rules = Vec::with_capacity(rules_config.rules.len());

    for rule in rules_config.rules {
        debug!("Attempting to compile rule: {}", rule.name);
        trace!("Pattern: '{}'", rule.pattern);

        let mut builder = RegexBuilder::new(&rule.pattern);
        builder.multi_line(rule.multiline);

        if rule.dot_matches_new_line {
            builder.dot_matches_new_line(true);
        }

        let compiled_regex = builder
            .build()
            .with_context(|| {
                format!(
                    "Failed to compile regex for rule '{}' with pattern: '{}'",
                    rule.name, rule.pattern
                )
            })?;

        compiled_rules.push(SanitizationRule {
            name: rule.name,
            compiled_regex,
            replace_with: rule.replace_with,
            original_pattern: rule.pattern,
            description: rule.description,
            dot_matches_new_line: rule.dot_matches_new_line,
        });

        debug!("Successfully compiled rule: {}", compiled_rules.last().unwrap().name);
    }

    Ok(compiled_rules)
}

/// Applies sanitization rules to a string and produces a redaction summary.
pub fn sanitize_content(
    content: &str,
    rules: &[SanitizationRule],
) -> (String, Vec<RedactionSummaryItem>) {
    debug!("Starting content sanitization.");

    // ✔️ Correct usage for strip_ansi_escapes 0.2.1
    let stripped_bytes = strip_ansi_escapes_fn(content);
    let mut sanitized_content = String::from_utf8_lossy(&stripped_bytes).to_string();

    let mut all_redactions: Vec<RedactionSummaryItem> = Vec::new();

    for rule in rules {
        trace!(
            "Applying rule '{}' with pattern '{}'",
            rule.name,
            rule.original_pattern
        );

        let matches: Vec<regex::Match> = rule.compiled_regex.find_iter(&sanitized_content).collect();

        if !matches.is_empty() {
            debug!(
                "Found {} matches for rule '{}'.",
                matches.len(),
                rule.name
            );

            let mut current_rule_redactions = Vec::new();

            for m in &matches {
                let original = m.as_str().to_string();
                let sanitized = rule
                    .compiled_regex
                    .replace(&original, rule.replace_with.as_str())
                    .to_string();

                current_rule_redactions.push(RedactionSummaryItem {
                    rule_name: rule.name.clone(),
                    original_text: original,
                    sanitized_text: sanitized,
                    occurrences: 1,
                });
            }

            all_redactions.extend(current_rule_redactions);

            sanitized_content = rule
                .compiled_regex
                .replace_all(&sanitized_content, rule.replace_with.as_str())
                .to_string();
        } else {
            trace!("No matches found for rule '{}'.", rule.name);
        }
    }

    debug!(
        "Sanitization complete. {} individual redactions recorded.",
        all_redactions.len()
    );

    (sanitized_content, summarize_redaction_items(all_redactions))
}

/// Aggregates redactions by rule name and counts their occurrences.
fn summarize_redaction_items(items: Vec<RedactionSummaryItem>) -> Vec<RedactionSummaryItem> {
    use std::collections::HashMap;

    let mut summary_map: HashMap<String, RedactionSummaryItem> = HashMap::new();

    for item in items {
        summary_map
            .entry(item.rule_name.clone())
            .and_modify(|existing| {
                existing.occurrences += 1;
                existing.original_text = item.original_text.clone();
                existing.sanitized_text = item.sanitized_text.clone();
            })
            .or_insert(item);
    }

    let mut summarized: Vec<RedactionSummaryItem> = summary_map.into_values().collect();
    summarized.sort_by(|a, b| a.rule_name.cmp(&b.rule_name));
    summarized
}
