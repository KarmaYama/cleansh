// src/config.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

/// Represents a single redaction rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Rule {
    /// A unique name for the rule (e.g., "email", "ipv4_address").
    pub name: String,
    /// The regular expression pattern to match.
    pub pattern: String,
    /// The string to replace the matched pattern with.
    pub replace_with: String,
    /// An optional description of the rule's purpose.
    #[serde(default)] // Allow this field to be optional in YAML
    pub description: Option<String>,
    /// Whether the regex pattern should be applied in multiline mode.
    #[serde(default)] // Defaults to false if not specified
    pub multiline: bool,
}

/// Represents the top-level structure of the rules configuration file.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RulesConfig {
    pub rules: Vec<Rule>,
}

/// Loads the default, embedded redaction rules from `config/default_rules.yaml`.
/// These rules are compiled into the binary.
pub fn load_default_rules() -> Result<RulesConfig> {
    // Use include_str! to embed the YAML file at compile time.
    // This ensures the default rules are always available and reduces I/O.
    let default_rules_yaml = include_str!("../config/default_rules.yaml");

    // Deserialize the YAML string into our RulesConfig struct.
    serde_yaml::from_str(default_rules_yaml)
        .context("Failed to parse embedded default_rules.yaml")
}

/// Loads user-defined redaction rules from a specified YAML file.
///
/// # Arguments
/// * `path` - The path to the user's YAML configuration file.
///
/// # Returns
/// A `Result` containing `RulesConfig` on success, or an `anyhow::Error` on failure.
pub fn load_user_rules<P: AsRef<Path>>(path: P) -> Result<RulesConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read user config file: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse user config file: {}", path.display()))
}

/// Merges user-defined rules with default rules.
///
/// If a user rule has the same `name` as a default rule, the user rule
/// replaces the default rule. Otherwise, user rules are appended.
///
/// # Arguments
/// * `default_rules` - The `RulesConfig` containing the default rules.
/// * `user_rules` - An optional `RulesConfig` containing user-defined rules.
///
/// # Returns
/// A new `RulesConfig` containing the merged set of rules.
pub fn merge_rules(
    default_rules: RulesConfig,
    user_rules: Option<RulesConfig>,
) -> RulesConfig {
    let mut merged_rules_map: HashMap<String, Rule> = default_rules
        .rules
        .into_iter()
        .map(|rule| (rule.name.clone(), rule))
        .collect();

    if let Some(user_conf) = user_rules {
        for user_rule in user_conf.rules {
            // User rule replaces default if name matches, otherwise it's added.
            merged_rules_map.insert(user_rule.name.clone(), user_rule);
        }
    }

    let mut rules: Vec<Rule> = merged_rules_map.into_values().collect();

    // For consistent behavior, sort rules by name if the order matters for regex application.
    // However, with RegexSet, order typically doesn't matter for *matching*, only for replacement.
    // Let's keep the order of user-provided rules relative to each other if possible,
    // or sort them to ensure deterministic behavior. For now, a simple sort by name
    // is sufficient after merging from the HashMap.
    rules.sort_by(|a, b| a.name.cmp(&b.name));

    RulesConfig { rules }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_default_rules() {
        let config = load_default_rules().unwrap();
        assert!(!config.rules.is_empty());
        assert!(config.rules.iter().any(|r| r.name == "email"));
        assert!(config.rules.iter().any(|r| r.name == "ssh_private_key_rsa"));
        assert!(config.rules.iter().any(|r| r.name == "absolute_linux_path"));
    }

    #[test]
    fn test_load_user_rules_success() {
        let user_yaml = r#"
rules:
  - name: "custom_token"
    pattern: "MY_CUSTOM_TOKEN_\\d+"
    replace_with: "[CUSTOM_TOKEN_REDACTED]"
    description: "A custom token pattern."
  - name: "email" # Overrides default email
    pattern: "([a-z]+@[a-z]+\\.com)"
    replace_with: "[CUSTOM_EMAIL_REDACTED]"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let path = file.path();

        let config = load_user_rules(path).unwrap();
        assert_eq!(config.rules.len(), 2);
        assert!(config.rules.iter().any(|r| r.name == "custom_token"));
        assert!(config.rules.iter().any(|r| r.name == "email"));
        assert_eq!(config.rules.iter().find(|r| r.name == "email").unwrap().replace_with, "[CUSTOM_EMAIL_REDACTED]");
    }

    #[test]
    fn test_load_user_rules_file_not_found() {
        let err = load_user_rules("non_existent_file.yaml").unwrap_err();
        assert!(err.to_string().contains("Failed to read user config file"));
    }

    #[test]
    fn test_load_user_rules_invalid_yaml() {
        let invalid_yaml = r#"
rules:
  - name: "bad_rule"
    pattern:
      - "invalid" # Pattern should be a string, not a list
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(invalid_yaml.as_bytes()).unwrap();
        let path = file.path();

        let err = load_user_rules(path).unwrap_err();
        assert!(err.to_string().contains("Failed to parse user config file"));
    }

    #[test]
    fn test_merge_rules_no_user_rules() {
        let default_config = load_default_rules().unwrap();
        let default_count = default_config.rules.len();

        let merged_config = merge_rules(default_config.clone(), None);
        assert_eq!(merged_config.rules.len(), default_count);
        assert_eq!(merged_config, default_config);
    }

    #[test]
    fn test_merge_rules_with_user_rules_additive_and_override() {
        let default_config = load_default_rules().unwrap();
        let default_email_replace = default_config.rules.iter().find(|r| r.name == "email").unwrap().replace_with.clone();
        
        let user_yaml = r#"
rules:
  - name: "custom_log_id"
    pattern: "LOG_ID:\\s+\\d+"
    replace_with: "[LOG_ID_REDACTED]"
  - name: "email" # Override default email rule
    pattern: "([A-Za-z0-9._%+-]+@example\\.com)"
    replace_with: "[EXAMPLE_EMAIL_REDACTED]"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let user_rules = load_user_rules(file.path()).unwrap();

        let merged_config = merge_rules(default_config.clone(), Some(user_rules));

        // Check for new additive rule
        assert!(merged_config.rules.iter().any(|r| r.name == "custom_log_id"));

        // Check if email rule was overridden
        let merged_email_rule = merged_config.rules.iter().find(|r| r.name == "email").unwrap();
        assert_eq!(merged_email_rule.replace_with, "[EXAMPLE_EMAIL_REDACTED]");
        assert_ne!(merged_email_rule.replace_with, default_email_replace); // Ensure it's different from original default

        // Ensure total count is default + new user rules
        // (default rules - overridden defaults) + user rules
        let initial_default_count = default_config.rules.len();
        let expected_count = initial_default_count + 1; // 1 new rule, 1 override
        assert_eq!(merged_config.rules.len(), expected_count);
    }
}