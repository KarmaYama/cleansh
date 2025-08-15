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

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use log::{debug, info, warn};
use std::fmt;
use regex::Regex;
use std::hash::{Hash, Hasher}; // <-- Added for Hash implementation

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
/// * `enabled`: An optional boolean to explicitly enable or disable a rule, overriding default behavior.
/// * `severity`: An optional string indicating the severity of the rule.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct RedactionRule {
    pub name: String,
    pub description: Option<String>,
    pub pattern: Option<String>,
    pub pattern_type: String,
    pub replace_with: String,
    pub version: String,
    pub created_at: String,
    pub author: String,
    pub updated_at: String,
    pub multiline: bool,
    pub dot_matches_new_line: bool,
    pub opt_in: bool,
    pub programmatic_validation: bool,
    pub enabled: Option<bool>,
    pub severity: Option<String>,
    pub tags: Option<Vec<String>>,
}

// Manually implement the Hash trait for RedactionRule.
// This is necessary because Vec<String> and Option<Vec<String>> don't
// implement Hash, so #[derive(Hash)] won't work automatically.
// The Hash trait implementation is crucial for using `RedactionRule`
// within data structures like `HashMap` or `HashSet` that require hashing.
impl Hash for RedactionRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.description.hash(state);
        self.pattern.hash(state);
        self.pattern_type.hash(state);
        self.replace_with.hash(state);
        self.version.hash(state);
        self.created_at.hash(state);
        self.author.hash(state);
        self.updated_at.hash(state);
        self.multiline.hash(state);
        self.dot_matches_new_line.hash(state);
        self.opt_in.hash(state);
        self.programmatic_validation.hash(state);
        self.enabled.hash(state);
        self.severity.hash(state);
        // We're not hashing the tags since it's an Option<Vec<String>>
        // and we need to be careful with its Hash implementation.
        // For simplicity and correctness, we will omit it. If a more
        // complex logic for tags is needed in the future, it can be added here.
    }
}

impl Default for RedactionRule {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            pattern: None,
            pattern_type: "regex".to_string(),
            replace_with: "[REDACTED]".to_string(),
            version: "1.0.0".to_string(),
            created_at: "1970-01-01T00:00:00Z".to_string(),
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            author: "Obscura Team".to_string(),
            multiline: false,
            dot_matches_new_line: false,
            opt_in: false,
            programmatic_validation: false,
            enabled: None,
            severity: None,
            tags: None,
        }
    }
}

/// Represents the collection of redaction rules in a configuration file.
///
/// This struct holds a vector of `RedactionRule` instances and provides methods
/// for loading rule sets from various sources and managing their active state.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
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
        let config: RedactionConfig = serde_yml::from_str(&text)
            .with_context(|| format!("Failed to parse config file {}", path.display()))?;

        validate_rules(&config.rules)?;
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
        let config: RedactionConfig = serde_yml::from_str(default_yaml)
            .context("Failed to parse default rules")?;

        // No need to validate default rules as they are internal and trusted.
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
    ///                    Only opt-in rules need to be explicitly enabled.
    /// * `disable_rules` - A slice of `String`s representing the names of rules to explicitly disable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cleansh_core::config::{RedactionConfig, RedactionRule, merge_rules};
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut config = RedactionConfig::default();
    /// config.rules.push(RedactionRule { name: "default_rule".to_string(), pattern: Some("".to_string()), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()});
    /// config.rules.push(RedactionRule { name: "opt_in_rule".to_string(), pattern: Some("".to_string()), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: true, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()});
    /// config.rules.push(RedactionRule { name: "another_default".to_string(), pattern: Some("".to_string()), replace_with: "".to_string(), description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()});
    ///
    /// // Initially, there are 3 rules.
    /// assert_eq!(config.rules.len(), 3);
    ///
    /// // Only enable "opt_in_rule", disable "another_default".
    /// let enable_vec = vec!["opt_in_rule".to_string()];
    /// let disable_vec = vec!["another_default".to_string()];
    /// config.set_active_rules(&enable_vec, &disable_vec);
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
        
        // Find and warn about any rules in the enable/disable lists that don't exist
        let all_rule_names: HashSet<&str> = self.rules.iter().map(|r| r.name.as_str()).collect();

        for rule_name in enable_set.difference(&all_rule_names) {
            warn!("Rule '{}' in `enable_rules` list does not exist.", rule_name);
        }

        for rule_name in disable_set.difference(&all_rule_names) {
            warn!("Rule '{}' in `disable_rules` list does not exist.", rule_name);
        }

        self.rules.retain(|rule| {
            let rule_name_str = rule.name.as_str();

            // A rule is active if it's not explicitly disabled, and either
            // it's not an opt-in rule, or it is an opt-in rule that has been explicitly enabled.
            let is_active = !disable_set.contains(rule_name_str) && (!rule.opt_in || enable_set.contains(rule_name_str));

            if is_active {
                debug!("Rule '{}' is active.", rule_name_str);
            } else {
                debug!("Rule '{}' is inactive.", rule_name_str);
            }
            
            is_active
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
///                    If `None`, only the `default_config` rules are used.
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
///     name: "email".to_string(), pattern: Some(".*@.*".to_string()), replace_with: "[EMAIL]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()
/// });
/// default_config.rules.push(RedactionRule {
///     name: "phone".to_string(), pattern: Some(r"\d{3}-\d{3}-\d{4}".to_string()), replace_with: "[PHONE]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()
/// });
///
/// // Simulate user config (overrides "phone", adds "ssn")
/// let mut user_config = RedactionConfig::default();
/// user_config.rules.push(RedactionRule {
///     name: "phone".to_string(), pattern: Some(r"\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}".to_string()), replace_with: "[PHONE_NUMBER]".to_string(),
///     description: Some("More flexible phone number".to_string()), multiline: false, dot_matches_new_line: false, opt_in: false, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()
/// });
/// user_config.rules.push(RedactionRule {
///     name: "ssn".to_string(), pattern: Some(r"\d{3}-\d{2}-\d{4}".to_string()), replace_with: "[SSN]".to_string(),
///     description: None, multiline: false, dot_matches_new_line: false, opt_in: true, programmatic_validation: false, enabled: None, severity: None, tags: None, pattern_type: "regex".to_string(), version: "1.0.0".to_string(), created_at: "1970-01-01T00:00:00Z".to_string(), updated_at: "1970-01-01T00:00:00Z".to_string(), author: "Obscura Team".to_string()
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
    default_config: RedactionConfig,
    user_config: Option<RedactionConfig>,
) -> RedactionConfig {
    debug!("merge_rules called. Initial default rules count: {}", default_config.rules.len());
    
    let mut final_rules_map: HashMap<String, RedactionRule> = default_config.rules.into_iter()
        .map(|rule| (rule.name.clone(), rule))
        .collect();

    if let Some(user_cfg) = user_config {
        debug!("User config provided. Merging {} user rules.", user_cfg.rules.len());
        for user_rule in user_cfg.rules {
            if final_rules_map.contains_key(&user_rule.name) {
                debug!("Overriding default rule '{}' with user configuration.", user_rule.name);
            } else {
                debug!("Adding new user rule: '{}'", user_rule.name);
            }
            final_rules_map.insert(user_rule.name.clone(), user_rule);
        }
    } else {
        debug!("No user configuration provided. Using default rules.");
    }

    let final_rules: Vec<RedactionRule> = final_rules_map.into_values().collect();
    debug!("Final total rules after merge: {}", final_rules.len());

    RedactionConfig { rules: final_rules }
}

/// Validates a slice of `RedactionRule`s, checking for duplicate names,
/// empty names/patterns, and invalid replacement string syntax.
///
/// This function is intended to be called after a configuration has been loaded
/// to ensure its integrity before it is used.
fn validate_rules(rules: &[RedactionRule]) -> Result<()> {
    let mut rule_names = HashSet::new();
    let mut errors = Vec::new();
    let capture_group_regex = Regex::new(r"\$(\d+)").unwrap();

    for rule in rules {
        if rule.name.is_empty() {
            errors.push("A rule has an empty `name` field.".to_string());
        } else if !rule_names.insert(rule.name.clone()) {
            errors.push(format!("Duplicate rule name found: '{}'.", rule.name));
        }

        let pattern = match &rule.pattern {
            Some(p) => p,
            None => {
                errors.push(format!("Rule '{}' is missing the `pattern` field.", rule.name));
                continue;
            }
        };

        if pattern.is_empty() {
            errors.push(format!("Rule '{}' has an empty `pattern` field.", rule.name));
        }
        
        // Check for regex compilation errors
        if let Err(e) = Regex::new(pattern) {
            errors.push(format!("Rule '{}' has an invalid regex pattern: {}", rule.name, e));
            continue; // Skip further validation for this rule if the regex is invalid
        }
        
        // Count the number of capturing groups in the pattern.
        // We use a simplified approach that counts unescaped parentheses.
        let mut group_count = 0;
        let mut is_escaped = false;
        for c in pattern.chars() {
            match c {
                '\\' => is_escaped = !is_escaped,
                '(' if !is_escaped => group_count += 1,
                _ => is_escaped = false,
            }
        }

        // Validate the replacement string
        for cap in capture_group_regex.captures_iter(&rule.replace_with) {
            if let Some(group_num_str) = cap.get(1) {
                if let Ok(group_num) = group_num_str.as_str().parse::<usize>() {
                    // Check if the group number is valid.
                    // Group $0 is the full match, so we check against <= group_count.
                    if group_num > group_count {
                        errors.push(format!(
                            "Rule '{}': replacement string references non-existent capture group '${}'. Pattern has only {} capturing groups.",
                            rule.name, group_num, group_count
                        ));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        let full_error_message = format!("Rule validation failed:\n{}", errors.join("\n"));
        Err(anyhow!(full_error_message))
    } else {
        Ok(())
    }
}