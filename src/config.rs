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
    #[serde(default)] // Defaults to false if not specified in YAML
    pub multiline: bool,
    #[serde(default)] // Defaults to false if not specified in YAML
    pub dot_matches_new_line: bool,
    #[serde(default)] // Defaults to false if not specified in YAML
    pub opt_in: bool,
    #[serde(default)] // Defaults to false if not specified in YAML
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
        debug!("[config.rs] Attempting to load config from file: {}", path.display());
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file {}", path.display()))?;
        let config: RedactionConfig = serde_yaml::from_str(&text)
            .with_context(|| format!("Failed to parse config file {}", path.display()))?;

        debug!("[config.rs] Loaded {} rules from file {}.", config.rules.len(), path.display());
        for rule in &config.rules {
            debug!("[config.rs] File Rule - Name: {}, Opt_in: {}", rule.name, rule.opt_in);
        }
        Ok(config)
    }

    /// Loads default redaction rules from an embedded string.
    pub fn load_default_rules() -> Result<Self> {
        debug!("[config.rs] Loading default rules from embedded string...");
        // Correct path relative to src/config.rs
        let default_yaml = include_str!("../config/default_rules.yaml");
        let config: RedactionConfig = serde_yaml::from_str(default_yaml).context("Failed to parse default rules")?;

        debug!("[config.rs] Loaded {} default rules.", config.rules.len());
        for rule in &config.rules {
            debug!("[config.rs] Default Rule - Name: {}, Opt_in: {}", rule.name, rule.opt_in);
        }
        Ok(config)
    }
}

/// Merges user-defined rules with default rules.
/// User-defined rules override default rules with the same name.
pub fn merge_rules(
    mut default_config: RedactionConfig,
    user_config: Option<RedactionConfig>,
) -> RedactionConfig {
    let initial_default_count = default_config.rules.len();
    debug!("[config.rs] merge_rules called. Initial default rules count: {}", initial_default_count);

    if let Some(user_cfg) = user_config {
        debug!("[config.rs] User config provided. Merging {} user rules.", user_cfg.rules.len());
        let user_rules_map: HashMap<String, RedactionRule> = user_cfg
            .rules.clone()
            .into_iter()
            .map(|rule| {
                debug!("[config.rs] User rule to merge: '{}', Opt_in: {}", rule.name, rule.opt_in);
                (rule.name.clone(), rule)
            })
            .collect();

        let mut overridden_count = 0;
        // Retain default rules that are NOT overridden by user rules
        default_config.rules.retain(|default_rule| {
            if user_rules_map.contains_key(&default_rule.name) {
                debug!("Default rule '{}' overridden by user configuration.", default_rule.name);
                debug!("[config.rs] Default rule '{}' overridden by user. Skipping default.", default_rule.name);
                overridden_count += 1;
                false // Remove this default rule
            } else {
                debug!("[config.rs] Keeping default rule: '{}', Opt_in: {}", default_rule.name, default_rule.opt_in);
                true // Keep this default rule
            }
        });

        // Extend with all user rules (including those that overrode defaults)
        let added_user_rules_count = user_rules_map.len() - overridden_count;
        default_config.rules.extend(user_rules_map.into_values());
        
        debug!(
            "Merged rules summary: {} default rules initially, {} user rules processed. Overrode {} defaults, added {} new user rules. Final total rules: {}",
            initial_default_count,
            user_cfg.rules.len(),
            overridden_count,
            added_user_rules_count,
            default_config.rules.len()
        );
        debug!("[config.rs] Final merged rules count: {}", default_config.rules.len());
        for rule in &default_config.rules {
            debug!("[config.rs] Final Merged Rule - Name: {}, Opt_in: {}", rule.name, rule.opt_in);
        }

    } else {
        debug!(
            "No user configuration provided. Using {} default rules.",
            default_config.rules.len()
        );
        debug!("[config.rs] No user configuration to merge. Final rules count: {}", default_config.rules.len());
        for rule in &default_config.rules {
            debug!("[config.rs] Final Merged Rule (no user config) - Name: {}, Opt_in: {}", rule.name, rule.opt_in);
        }
    }
    default_config
}