use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use anyhow::{Context, Result};

/// Represents a single redaction rule.
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
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RulesConfig {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionSummaryItem {
    pub rule_name: String,
    pub original_text: String,
    pub sanitized_text: String,
    pub occurrences: usize,
}

pub fn load_default_rules() -> Result<RulesConfig> {
    let default_rules_yaml = include_str!("../config/default_rules.yaml");
    serde_yaml::from_str(default_rules_yaml)
        .context("Failed to parse embedded default_rules.yaml")
}

pub fn load_user_rules<P: AsRef<Path>>(path: P) -> Result<RulesConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read user config file: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse user config file: {}", path.display()))
}

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
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let path = file.path();

        let config = load_user_rules(path).unwrap();
        assert_eq!(config.rules.len(), 2);
        assert!(config.rules.iter().any(|r| r.name == "custom_token"));
        assert!(config.rules.iter().any(|r| r.name == "email"));
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
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(user_yaml.as_bytes()).unwrap();
        let user_rules = load_user_rules(file.path()).unwrap();

        let merged_config = merge_rules(default_config.clone(), Some(user_rules));

        assert!(merged_config.rules.iter().any(|r| r.name == "custom_log_id"));

        let merged_email_rule = merged_config.rules.iter().find(|r| r.name == "email").unwrap();
        assert_eq!(merged_email_rule.replace_with, "[EXAMPLE_EMAIL_REDACTED]");
        if let Some(default_replace) = default_email_replace {
            assert_ne!(merged_email_rule.replace_with, default_replace);
        }

        let initial_default_count = default_config.rules.len();
        let expected_count = initial_default_count + 1; // One new, one override
        assert_eq!(merged_config.rules.len(), expected_count);
    }
}
