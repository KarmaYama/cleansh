// tests/cleansh_integration_tests.rs

use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

// Import the specific function and types needed from the main crate
use cleansh::commands::cleansh;
use cleansh::config;
use cleansh::ui::theme::{self, ThemeEntry}; 

// Helper function to get default theme map, moved here as it's only used by tests.
fn get_default_theme_map() -> HashMap<ThemeEntry, theme::ThemeStyle> {
    theme::ThemeStyle::default_theme_map()
}

#[test]
fn test_run_cleansh_basic_sanitization() -> Result<()> {
    // Setup: Minimal configuration for testing
    let input = "email: test@example.com. My SSN is 123-45-6789.";
    let config = config::RedactionConfig {
        rules: vec![
            config::RedactionRule {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
                replace_with: "[EMAIL]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: false,
            },
            config::RedactionRule {
                name: "us_ssn".to_string(),
                pattern: r"\b(\d{3})-(\d{2})-(\d{4})\b".to_string(), // Pattern with capturing groups for SSN validation
                replace_with: "[US_SSN_REDACTED]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: true,
            },
        ],
    };

    // Create a temporary file for output
    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output.txt");

    // Create a temp config file for the test.
    let temp_config_file = temp_dir.path().join("test_rules.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    cleansh::run_cleansh( // Call the public function from the commands module
        input,
        false, // clipboard_enabled
        false, // diff_enabled
        Some(temp_config_file), // config_path
        Some(output_file_path.clone()), // output_path
        false, // no_redaction_summary
        &get_default_theme_map(),
        vec![], // enable_rules
        vec![], // disable_rules
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    // Check output: Should contain sanitized content + summary
    assert!(output.contains("email: [EMAIL]. My SSN is [US_SSN_REDACTED]."));
    assert!(output.contains("--- Redaction Summary ---"));
    assert!(output.contains("email (1 occurrences)"));
    assert!(output.contains("- test@example.com"));
    assert!(output.contains("- [EMAIL]"));
    assert!(output.contains("us_ssn (1 occurrences)"));
    assert!(output.contains("- 123-45-6789"));
    assert!(output.contains("- [US_SSN_REDACTED]"));


    Ok(())
}

#[test]
fn test_run_cleansh_no_redaction_summary() -> Result<()> {
    let input = "email: test@example.com. Invalid SSN: 000-12-3456.";
    let config = config::RedactionConfig {
        rules: vec![
            config::RedactionRule {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
                replace_with: "[EMAIL]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: false,
            },
            config::RedactionRule {
                name: "us_ssn".to_string(),
                pattern: r"\b(\d{3})-(\d{2})-(\d{4})\b".to_string(),
                replace_with: "[US_SSN_REDACTED]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: true,
            },
        ],
    };

    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output_no_summary.txt");
    let temp_config_file = temp_dir.path().join("test_rules_no_summary.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    cleansh::run_cleansh(
        input,
        false,
        false,
        Some(temp_config_file),
        Some(output_file_path.clone()),
        true, // no_redaction_summary = true
        &get_default_theme_map(),
        vec![],
        vec![],
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    assert!(output.contains("email: [EMAIL]. Invalid SSN: 000-12-3456.")); // SSN should not be redacted due to programmatic validation
    assert!(!output.contains("--- Redaction Summary ---")); // Summary should not be present

    Ok(())
}

#[test]
#[cfg(feature = "clipboard")] // Only run if clipboard feature is enabled
fn test_run_cleansh_clipboard_copy() -> Result<()> {
    let input = "email: test@example.com";
    let config = config::RedactionConfig {
        rules: vec![config::RedactionRule {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
            replace_with: "[EMAIL]".to_string(),
            description: None,
            multiline: false,
            dot_matches_new_line: false,
            opt_in: false,
            programmatic_validation: false,
        }],
    };

    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output_clipboard.txt");
    let temp_config_file = temp_dir.path().join("test_rules_clipboard.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    cleansh::run_cleansh(
        input,
        true, // clipboard_enabled = true
        false,
        Some(temp_config_file),
        Some(output_file_path.clone()),
        true, // No summary for easier clipboard content check
        &get_default_theme_map(),
        vec![],
        vec![],
    )?;

    let mut clipboard = arboard::Clipboard::new().context("Failed to get clipboard")?;
    let clipboard_content = clipboard.get_text().context("Failed to read clipboard")?;

    assert_eq!(clipboard_content.trim(), "email: [EMAIL]"); // Use trim due to potential newline differences

    Ok(())
}

#[test]
fn test_run_cleansh_diff_output() -> Result<()> {
    let input = "Original email: test@example.com\nAnother line.";
    let config = config::RedactionConfig {
        rules: vec![config::RedactionRule {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
            replace_with: "[EMAIL]".to_string(),
            description: None,
            multiline: false,
            dot_matches_new_line: false,
            opt_in: false,
            programmatic_validation: false,
        }],
    };

    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output_diff.txt");
    let temp_config_file = temp_dir.path().join("test_rules_diff.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    cleansh::run_cleansh(
        input,
        false,
        true, // diff_enabled = true
        Some(temp_config_file),
        Some(output_file_path.clone()),
        true, // No summary to focus on diff
        &get_default_theme_map(),
        vec![],
        vec![],
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    assert!(output.contains("-Original email: test@example.com"));
    assert!(output.contains("+Original email: [EMAIL]"));
    assert!(output.contains("Another line."));

    Ok(())
}