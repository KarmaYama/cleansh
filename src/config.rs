// src/config.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use log::debug;

/// Maximum allowed length for a regex pattern string.
/// This prevents excessively large or potentially malicious regexes.
pub const MAX_PATTERN_LENGTH: usize = 500; // Example: 500 characters

/// Represents a single redaction rule.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RedactionRule {
    pub name: String,
    pub pattern: String,
    pub replace_with: String,
    #[serde(default)] // Make description optional, default to None
    pub description: Option<String>,
    #[serde(default)] // Defaults to false if not specified
    pub multiline: bool,
    #[serde(default)] // Defaults to false if not specified
    pub dot_matches_new_line: bool,
    #[serde(default)] // Defaults to false if not specified
    pub opt_in: bool,
    #[serde(default)] // Defaults to false if not specified
    pub programmatic_validation: bool, // New field for advanced validation logic
}

/// Represents the collection of redaction rules in a configuration file.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct RedactionConfig {
    pub rules: Vec<RedactionRule>,
}

/// Represents a single item in the redaction summary, including examples and occurrences.
#[derive(Debug, Clone, PartialEq, Eq)] // Derive PartialEq, Eq for easier testing/comparison
pub struct RedactionSummaryItem {
    pub rule_name: String,
    pub occurrences: usize,
    pub original_texts: Vec<String>, // Stores unique original matches
    pub sanitized_texts: Vec<String>, // Stores unique sanitized replacements
}


impl RedactionConfig {
    /// Loads redaction rules from a YAML file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file {}", path.display()))?;
        serde_yaml::from_str(&text)
            .with_context(|| format!("Failed to parse config file {}", path.display()))
    }

    /// Loads default redaction rules from an embedded string.
    pub fn load_default_rules() -> Result<Self> {
        // Correct path relative to src/config.rs
        let default_yaml = include_str!("../config/default_rules.yaml");
        serde_yaml::from_str(default_yaml).context("Failed to parse default rules")
    }
}

/// Merges user-defined rules with default rules.
/// User-defined rules override default rules with the same name.
pub fn merge_rules(
    mut default_config: RedactionConfig,
    user_config: Option<RedactionConfig>,
) -> RedactionConfig {
    let _initial_default_count = default_config.rules.len();

    if let Some(user_cfg) = user_config {
        let user_rules_map: HashMap<String, RedactionRule> = user_cfg
            .rules.clone()
            .into_iter()
            .map(|rule| (rule.name.clone(), rule))
            .collect();

        default_config.rules.retain(|default_rule| {
            if user_rules_map.contains_key(&default_rule.name) {
                debug!("Default rule '{}' overridden by user configuration.", default_rule.name);
                false
            } else {
                true
            }
        });

        default_config.rules.extend(user_rules_map.into_values());
        debug!(
            "Merged rules: {} default rules, {} user rules. Total rules: {}",
            _initial_default_count,
            user_cfg.rules.len(),
            default_config.rules.len()
        );
    } else {
        debug!(
            "No user configuration provided. Using {} default rules.",
            default_config.rules.len()
        );
    }
    default_config
}