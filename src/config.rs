use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use anyhow::{Context, Result};

/// Represents a single redaction rule.
///
/// Each rule defines a pattern to match, a string to replace it with,
/// and optional metadata like a description and regex flags.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub pattern: String,
    pub replace_with: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub multiline: bool,
    #[serde(default)]
    pub dot_matches_new_line: bool,
    #[serde(default)] // Added for opt-in rules
    pub opt_in: bool,
}

/// A container for a collection of `Rule`s.
///
/// This struct is used to deserialize the YAML configuration files
/// that define the redaction rules.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RulesConfig {
    pub rules: Vec<Rule>,
}

/// Represents an item in the redaction summary, detailing what was redacted.
///
/// This struct stores information about a specific rule that found matches,
/// including the original text, its sanitized version, and the number of occurrences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionSummaryItem {
    pub rule_name: String,
    pub original_text: String,
    pub sanitized_text: String,
    pub occurrences: usize,
}

/// Loads the default redaction rules embedded within the application.
///
/// These rules are compiled into the binary and provide the base set of
/// sensitive patterns `cleansh` will redact by default.
///
/// # Returns
///
/// A `Result` containing `RulesConfig` if parsing is successful,
/// or an `anyhow::Error` if the embedded YAML is malformed.
pub fn load_default_rules() -> Result<RulesConfig> {
    let default_rules_yaml = include_str!("../config/default_rules.yaml");
    serde_yaml::from_str(default_rules_yaml)
        .context("Failed to parse embedded default_rules.yaml")
}

/// Loads custom redaction rules from a specified YAML file.
///
/// This allows users to extend or override the default redaction behavior
/// with their own patterns.
///
/// # Arguments
///
/// * `path` - A reference to the path of the user's YAML configuration file.
///
/// # Returns
///
/// A `Result` containing `RulesConfig` if the file is read and parsed successfully,
/// or an `anyhow::Error` if the file cannot be read or the YAML is malformed.
pub fn load_user_rules<P: AsRef<Path>>(path: P) -> Result<RulesConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read user config file: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse user config file: {}", path.display()))
}

/// Merges default redaction rules with optional user-defined rules.
///
/// User-defined rules with the same `name` as a default rule will override the default.
/// New user-defined rules will be added. The merged rules are sorted by name for
/// deterministic order.
///
/// # Arguments
///
/// * `default_rules` - The base set of default redaction rules.
/// * `user_rules` - An `Option` containing user-defined rules. If `None`, only default rules are used.
///
/// # Returns
///
/// A `RulesConfig` containing the combined and sorted set of redaction rules.
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
            merged_rules_map.insert(user_rule.name.clone(), user_rule);
        }
    }

    let mut rules: Vec<Rule> = merged_rules_map.into_values().collect();
    rules.sort_by(|a, b| a.name.cmp(&b.name)); // for deterministic order
    RulesConfig { rules }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_default_rules() {
        let config = load_default_rules().unwrap();
        assert!(!config.rules.is_empty());
        // Verify a rule with opt_in status exists and is correctly parsed
        assert!(config.rules.iter().any(|r| r.name == "aws_secret_key" && r.opt_in));
        assert!(config.rules.iter().any(|r| r.name == "email" && !r.opt_in));
    }

    #[test]
    fn test_load_user_rules_success() {
        let user_yaml = r#"
rules:
  - name: "custom_token"
    pattern: "MY_CUSTOM_TOKEN_\\d+"
    replace_with: "[CUSTOM_TOKEN_REDACTED]"
    description: "A custom token pattern."
    multiline: false
    dot_matches_new_line: false
  - name: "email"
    pattern: "([a-z]+@[a-z]+\\.com)"
    replace_with: "[CUSTOM_EMAIL_REDACTED]"
    multiline: false
    dot_matches_new_line: false
  - name: "my_opt_in_rule"
    pattern: "OPT_IN_PATTERN"
    replace_with: "[OPT_IN_REDACTED]"
    opt_in: true
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let path = file.path();

        let config = load_user_rules(path).unwrap();
        assert_eq!(config.rules.len(), 3);
        assert!(config.rules.iter().any(|r| r.name == "custom_token"));
        assert!(config.rules.iter().any(|r| r.name == "email"));
        assert!(config.rules.iter().any(|r| r.name == "my_opt_in_rule" && r.opt_in));
        assert_eq!(
            config.rules.iter().find(|r| r.name == "email").unwrap().replace_with,
            "[CUSTOM_EMAIL_REDACTED]"
        );
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
      - "invalid"
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
        let merged_config = merge_rules(default_config.clone(), None);

        // Compare names as unordered sets
        let expected_names: HashSet<_> = default_config.rules.iter().map(|r| &r.name).collect();
        let actual_names: HashSet<_> = merged_config.rules.iter().map(|r| &r.name).collect();

        assert_eq!(expected_names, actual_names, "Rule names do not match");

        // Optional: Compare sorted rule contents
        let mut expected_sorted = default_config.rules.clone();
        let mut actual_sorted = merged_config.rules.clone();
        expected_sorted.sort_by(|a, b| a.name.cmp(&b.name));
        actual_sorted.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(expected_sorted, actual_sorted, "Rule contents differ after sort");
    }

    #[test]
    fn test_merge_rules_with_user_rules_additive_and_override() {
        let default_config = load_default_rules().unwrap();
        let default_email_replace = default_config.rules.iter().find(|r| r.name == "email").map(|r| r.replace_with.clone());

        let user_yaml = r#"
rules:
  - name: "custom_log_id"
    pattern: "LOG_ID:\\s+\\d+"
    replace_with: "[LOG_ID_REDACTED]"
    multiline: false
    dot_matches_new_line: false
  - name: "email"
    pattern: "([A-Za-z0-9._%+-]+@example\\.com)"
    replace_with: "[EXAMPLE_EMAIL_REDACTED]"
    multiline: false
    dot_matches_new_line: false
  - name: "new_opt_in_rule"
    pattern: "NEW_OPT_IN"
    replace_with: "[NEW_OPT_IN_REDACTED]"
    opt_in: true
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let user_rules = load_user_rules(file.path()).unwrap();

        let merged_config = merge_rules(default_config.clone(), Some(user_rules));

        assert!(merged_config.rules.iter().any(|r| r.name == "custom_log_id"));
        assert!(merged_config.rules.iter().any(|r| r.name == "new_opt_in_rule" && r.opt_in));


        let merged_email_rule = merged_config.rules.iter().find(|r| r.name == "email").unwrap();
        assert_eq!(merged_email_rule.replace_with, "[EXAMPLE_EMAIL_REDACTED]");
        if let Some(default_replace) = default_email_replace {
            assert_ne!(merged_email_rule.replace_with, default_replace);
        }

        let initial_default_count = default_config.rules.len();
        // Count how many default rules are NOT overridden by user rules
        let non_overridden_defaults = default_config.rules.iter()
            .filter(|d_rule| !["email"].contains(&d_rule.name.as_str()))
            .count();
        // Expected count = non_overridden_defaults + new user rules (custom_log_id, new_opt_in_rule) + overridden email rule
        let expected_count = non_overridden_defaults + 2 + 1; // 2 new, 1 override
        assert_eq!(merged_config.rules.len(), expected_count);
    }
}