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
use std::collections::{HashMap, HashSet}; // ADDED HashSet here
use std::path::Path;
use log::{debug, info};
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
        info!("Loading custom rules from: {}", path.display());
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file {}", path.display()))?;
        let config: RedactionConfig = serde_yaml::from_str(&text)
            .with_context(|| format!("Failed to parse config file {}", path.display()))?;
        info!("Loaded {} rules from file {}.", config.rules.len(), path.display());
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
        debug!("Loading default rules from embedded string...");
        let default_yaml = include_str!("../config/default_rules.yaml");
        let config: RedactionConfig = serde_yaml::from_str(default_yaml).context("Failed to parse default rules")?;
        debug!("Loaded {} default rules.", config.rules.len());
        Ok(config)
    }

    /// Filters the rules within the configuration based on the provided lists of rules to enable or disable.
    ///
    /// This method modifies the `rules` vector in-place, removing rules that are either explicitly
    /// disabled or are opt-in rules that haven't been explicitly enabled.
    ///
    /// # Arguments
    ///
    /// * `enable_rules` - A slice of `String`s representing the names of rules to explicitly enable.
    ///                     Only opt-in rules need to be explicitly enabled.
    /// * `disable_rules` - A slice of `String`s representing the names of rules to explicitly disable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cleansh_core::config::{RedactionConfig, RedactionRule};
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut config = RedactionConfig::default();
    /// config.rules.push(RedactionRule { name: "default_rule".to_string(), pattern: "".to_string(), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false });
    /// config.rules.push(RedactionRule { name: "opt_in_rule".to_string(), pattern: "".to_string(), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: true, programmatic_validation: false });
    /// config.rules.push(RedactionRule { name: "another_default".to_string(), pattern: "".to_string(), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false });
    ///
    /// // Initially, there are 3 rules.
    /// assert_eq!(config.rules.len(), 3);
    ///
    /// // Only enable "opt_in_rule", disable "another_default".
    /// config.set_active_rules(&["opt_in_rule".to_string()], &["another_default".to_string()]);
    ///
    /// // Now, there should be only two active rules.
    /// assert_eq!(config.rules.len(), 2);
    /// assert!(config.rules.iter().any(|r| r.name == "default_rule"));
    /// assert!(config.rules.iter().any(|r| r.name == "opt_in_rule"));
    /// assert!(!config.rules.iter().any(|r| r.name == "another_default"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_active_rules(&mut self, enable_rules: &[String], disable_rules: &[String]) {
        let enable_set: HashSet<&str> = enable_rules.iter().map(String::as_str).collect();
        let disable_set: HashSet<&str> = disable_rules.iter().map(String::as_str).collect();

        debug!("Initial rules count before filtering: {}", self.rules.len());
        debug!("Rules to enable: {:?}", enable_rules);
        debug!("Rules to disable: {:?}", disable_rules);

        self.rules.retain(|rule| {
            let rule_name_str = rule.name.as_str();

            // Check for explicit disable first (highest priority)
            if disable_set.contains(rule_name_str) {
                debug!("Rule '{}' disabled by user configuration.", rule_name_str);
                return false;
            }

            // Check for opt-in rules that are not explicitly enabled
            if rule.opt_in && !enable_set.contains(rule_name_str) {
                debug!("Opt-in rule '{}' is not explicitly enabled.", rule_name_str);
                return false;
            }
            
            // Keep the rule if it's not disabled and either not opt-in or explicitly enabled
            debug!("Rule '{}' is active.", rule_name_str);
            true
        });
        
        debug!("Final active rules count after filtering: {}", self.rules.len());
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
///                     If `None`, only the `default_config` rules are used.
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
    debug!("merge_rules called. Initial default rules count: {}", initial_default_count);

    if let Some(user_cfg) = user_config {
        let user_rules_len = user_cfg.rules.len(); // FIX: Get length before the move
        debug!("User config provided. Merging {} user rules.", user_rules_len);
        
        let mut user_rules_map: HashMap<String, RedactionRule> = HashMap::new();
        for rule in user_cfg.rules.into_iter() {
            debug!("User rule to merge: '{}', Opt_in: {}", rule.name, rule.opt_in);
            user_rules_map.insert(rule.name.clone(), rule);
        }

        let mut final_rules: Vec<RedactionRule> = Vec::new();
        let mut overridden_count = 0;
        
        for default_rule in default_config.rules {
            if let Some(user_override) = user_rules_map.remove(&default_rule.name) {
                // An override exists; use the user's rule
                debug!("Default rule '{}' overridden by user configuration. Using user's rule.", default_rule.name);
                final_rules.push(user_override);
                overridden_count += 1;
            } else {
                // No override; keep the default rule
                debug!("Keeping default rule: '{}', Opt_in: {}", default_rule.name, default_rule.opt_in);
                final_rules.push(default_rule);
            }
        }
        
        // Add any remaining user rules that didn't override a default
        for (name, rule) in user_rules_map {
            debug!("Adding new user rule: '{}'", name);
            final_rules.push(rule);
        }

        default_config.rules = final_rules;
        
        debug!(
            "Merged rules summary: {} default rules initially, {} user rules processed. Overrode {} defaults, added {} new user rules. Final total rules: {}",
            initial_default_count,
            user_rules_len, // FIX: Use the variable
            overridden_count,
            user_rules_len - overridden_count,
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