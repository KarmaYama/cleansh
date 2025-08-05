// cleansh-workspace/cleansh-core/src/config.rs
//! Configuration management for `CleanSH-core`, including data structures for redaction rules
//! and methods for loading and merging rule sets.
//!
//! This module defines the core types `RedactionRule` and `RedactionConfig` which are central
//! to how sensitive data patterns are defined and managed within the CleanSH ecosystem.
//! It also provides utility functions for loading these configurations from files or
//! embedded defaults, and for merging multiple rule sets.
//! The `RedactionConfig` can be used to load rules from YAML files, apply default rules
//! embedded in the library, and manage active rule configurations for sanitization.
//! License: BUSL-1.1


use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use log::{debug, info}; // Added 'info' to imports
use std::fmt;

/// Maximum allowed length for a regex pattern string.
/// This prevents excessively large or potentially malicious regexes.
pub const MAX_PATTERN_LENGTH: usize = 500;

/// Represents a single redaction rule.
///
/// Each rule defines a regular expression pattern to search for, the text to replace
/// matches with, and various flags controlling its behavior.
///
/// # Fields
///
/// * `name`: A unique identifier for the rule (e.g., "email", "ipv4_address").
/// * `pattern`: The regular expression string to match sensitive data.
/// * `replace_with`: The string used to replace matches of the `pattern`.
/// * `description`: An optional, human-readable explanation of what the rule targets.
/// * `multiline`: If `true`, the regex `.` will match newlines, and `^`/`$` match line start/end.
/// * `dot_matches_new_line`: If `true`, the `.` character in the pattern matches newlines.
/// * `opt_in`: If `true`, this rule is only applied when explicitly enabled (e.g., in "strict" mode).
/// * `programmatic_validation`: If `true`, this rule requires additional, external programmatic
///                              validation beyond just regex matching (e.g., Luhn check for credit cards).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RedactionRule {
    pub name: String,
    pub pattern: String,
    pub replace_with: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub multiline: bool,
    #[serde(default)]
    pub dot_matches_new_line: bool,
    #[serde(default)]
    pub opt_in: bool,
    #[serde(default)]
    pub programmatic_validation: bool,
}

/// Represents the collection of redaction rules in a configuration file.
///
/// This struct holds a vector of `RedactionRule` instances and provides methods
/// for loading rule sets from various sources and managing their active state.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct RedactionConfig {
    pub rules: Vec<RedactionRule>,
}

/// Represents a single item in the redaction summary, including examples and occurrences.
///
/// This struct is used to report details about each type of sensitive data that was
/// detected and redacted during the sanitization process.
///
/// # Fields
///
/// * `rule_name`: The name of the redaction rule that was applied.
/// * `occurrences`: The total number of times this rule matched and redacted content.
/// * `original_texts`: A list of unique original text snippets that were redacted by this rule.
/// * `sanitized_texts`: A list of unique sanitized (replaced) text snippets corresponding to the original texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionSummaryItem {
    pub rule_name: String,
    pub occurrences: usize,
    pub original_texts: Vec<String>,
    pub sanitized_texts: Vec<String>,
}

/// Custom error type for when a specific rule configuration is not found.
#[derive(Debug)]
pub struct RuleConfigNotFoundError {
    pub config_name: String,
}

impl fmt::Display for RuleConfigNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rule configuration '{}' not found.", self.config_name)
    }
}

impl std::error::Error for RuleConfigNotFoundError {}


impl RedactionConfig {
    /// Loads redaction rules from a YAML file at the specified path.
    ///
    /// This function is typically used to load user-defined or custom rule sets.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the path of the YAML configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(RedactionConfig)` on success, or an `anyhow::Error`
    /// if the file cannot be read or parsed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The file specified by `path` does not exist or cannot be read.
    /// * The content of the file is not valid YAML or does not conform to the `RedactionConfig` structure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use cleansh_core::config::RedactionConfig;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let config = RedactionConfig::load_from_file("rules.yaml")?;
    /// println!("Loaded {} rules from file.", config.rules.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        // Changed logging level from debug! to info! to match test expectation
        info!("Loading custom rules from: {}", path.display()); 
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
    ///
    /// This function provides a baseline set of rules that are compiled directly
    /// into the `cleansh-core` library, making it self-contained for basic usage.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(RedactionConfig)` on success, or an `anyhow::Error`
    /// if the embedded rules cannot be parsed (indicates a library internal error).
    ///
    /// # Errors
    ///
    /// This function will return an error if the embedded YAML string is malformed,
    /// which should ideally not happen in a released version of the library.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cleansh_core::config::RedactionConfig;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let config = RedactionConfig::load_default_rules()?;
    /// println!("Loaded {} default rules.", config.rules.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_default_rules() -> Result<Self> {
        debug!("[config.rs] Loading default rules from embedded string...");
        let default_yaml = include_str!("../config/default_rules.yaml");
        let config: RedactionConfig = serde_yaml::from_str(default_yaml).context("Failed to parse default rules")?;

        debug!("[config.rs] Loaded {} default rules.", config.rules.len());
        for rule in &config.rules {
            debug!("[config.rs] Default Rule - Name: {}, Opt_in: {}", rule.name, rule.opt_in);
        }
        Ok(config)
    }

    /// Sets the active rule configuration based on the provided name.
    ///
    /// This method filters the `rules` vector in-place. Currently supports "default" and "strict"
    /// configurations. "default" passes all loaded rules to compilation, while "strict" activates
    /// all rules including those marked as `opt_in`.
    ///
    /// **Note:** This function currently only logs the chosen configuration. The actual filtering
    /// of opt-in rules based on "default" vs. "strict" modes happens within the `compile_rules`
    /// function in the `sanitizer` module, which takes `enable_rules` and `disable_rules` lists.
    /// This method mainly serves as a flag or intent setter for the overall `RedactionConfig` instance.
    ///
    /// # Arguments
    ///
    /// * `config_name` - The name of the configuration to apply ("default", "strict").
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on success, or a `RuleConfigNotFoundError` if the
    /// `config_name` is not recognized.
    ///
    /// # Errors
    ///
    /// Returns `RuleConfigNotFoundError` if the provided `config_name` does not match
    /// any predefined configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cleansh_core::config::RedactionConfig;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut config = RedactionConfig::load_default_rules()?;
    /// config.set_active_rules_config("default")?; // Intends to use default rules (non-opt-in)
    /// config.set_active_rules_config("strict")?;  // Intends to use all rules (including opt-in)
    /// // config.set_active_rules_config("invalid")?; // This would return an error
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_active_rules_config(&mut self, config_name: &str) -> Result<()> {
        debug!("[config.rs] Setting active rules configuration to: '{}'", config_name);
        match config_name {
            "default" => {
                // In this implementation, "default" means all rules loaded into this config
                // are passed to compile_rules, where opt-in rules will be filtered out
                // unless explicitly enabled via `enable_rules` parameter.
                debug!("[config.rs] 'default' config applied. All rules loaded from this config will be passed to compilation.");
            }
            "strict" => {
                // "strict" means all rules (including opt-in ones) within this config
                // are considered for compilation. This implies that compile_rules
                // should not filter out opt-in rules when 'strict' is in effect.
                debug!("[config.rs] 'strict' config applied. All rules ({} total) are active for compilation.", self.rules.len());
            }
            _ => {
                return Err(RuleConfigNotFoundError { config_name: config_name.to_string() }.into());
            }
        }
        Ok(())
    }
}

/// Merges user-defined rules with default rules.
///
/// User-defined rules will override default rules that have the same name.
/// Rules present only in the user configuration will be added.
///
/// # Arguments
///
/// * `default_config` - The base `RedactionConfig`, typically loaded from default rules.
/// * `user_config` - An `Option` containing a `RedactionConfig` with user-defined rules.
///                   If `None`, only the `default_config` rules are used.
///
/// # Returns
///
/// A new `RedactionConfig` containing the merged set of rules.
///
/// # Examples
///
/// ```
/// # use cleansh_core::config::{RedactionConfig, RedactionRule, merge_rules};
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// // Simulate default config
/// let mut default_config = RedactionConfig::default();
/// default_config.rules.push(RedactionRule {
///     name: "email".to_string(), pattern: ".*@.*".to_string(), replace_with: "[EMAIL]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false,
/// });
/// default_config.rules.push(RedactionRule {
///     name: "phone".to_string(), pattern: r"\d{3}-\d{3}-\d{4}".to_string(), replace_with: "[PHONE]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false,
/// });
///
/// // Simulate user config (overrides "phone", adds "ssn")
/// let mut user_config = RedactionConfig::default();
/// user_config.rules.push(RedactionRule {
///     name: "phone".to_string(), pattern: r"\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}".to_string(), replace_with: "[PHONE_NUMBER]".to_string(),
///     description: Some("More flexible phone number".to_string()), multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false,
/// });
/// user_config.rules.push(RedactionRule {
///     name: "ssn".to_string(), pattern: r"\d{3}-\d{2}-\d{4}".to_string(), replace_with: "[SSN]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: true, programmatic_validation: false,
/// });
///
/// let merged_config = merge_rules(default_config, Some(user_config));
///
/// assert_eq!(merged_config.rules.len(), 3); // email, phone (user's), ssn
/// assert_eq!(merged_config.rules.iter().find(|r| r.name == "phone").unwrap().replace_with, "[PHONE_NUMBER]");
/// assert!(merged_config.rules.iter().any(|r| r.name == "ssn"));
///
/// // Example with no user config
/// let default_config_no_user = RedactionConfig::load_default_rules()?;
/// let merged_no_user = merge_rules(default_config_no_user.clone(), None);
/// assert_eq!(merged_no_user.rules.len(), default_config_no_user.rules.len());
/// # Ok(())
/// # }
/// ```
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
        default_config.rules.retain(|default_rule| {
            if user_rules_map.contains_key(&default_rule.name) {
                debug!("Default rule '{}' overridden by user configuration. Skipping default.", default_rule.name);
                overridden_count += 1;
                false
            } else {
                debug!("[config.rs] Keeping default rule: '{}', Opt_in: {}", default_rule.name, default_rule.opt_in);
                true
            }
        });

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