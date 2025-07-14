use anyhow::Result;
use std::collections::HashMap;
// Import strip_ansi_escapes
use strip_ansi_escapes;

// Import the specific function and types needed from the main crate
// UPDATED: Now accessing via `cleansh::test_exposed::`
use cleansh::test_exposed::commands::run_cleansh;
use cleansh::test_exposed::config;
use cleansh::test_exposed::ui::theme::{self, ThemeEntry};

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

    run_cleansh( // Call the public function from the commands module
        input,
        false, // clipboard_enabled
        false, // diff_enabled
        Some(temp_config_file), // config_path
        Some(output_file_path.clone()), // output_path
        false, // no_redaction_summary (this means summary *should* be displayed on console/stderr)
        &get_default_theme_map(),
        vec![], // enable_rules
        vec![], // disable_rules
    )?;

    let output_from_file = std::fs::read_to_string(&output_file_path)?;
    // Strip ANSI escape codes before assertions
    let output_stripped_from_file = strip_ansi_escapes::strip_str(&output_from_file);

    // Check output: Should ONLY contain sanitized content, as summary goes to stderr/console.
    // Corrected assertion: Expect SSN to be redacted
    assert_eq!(output_stripped_from_file.trim(), "email: [EMAIL]. My SSN is [US_SSN_REDACTED].");

    // We cannot directly capture stdout/stderr when calling a function like this
    // unless `run_cleansh` explicitly returns them or takes `Write` traits.
    // For now, we trust that the summary is printed to the console as per its logic.
    // If you need to test the console output, you would need to refactor `run_cleansh`
    // to accept `std::io::Write` arguments for stdout and stderr, or use `assert_cmd`
    // as you did in `cli_integration_tests.rs`.

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

    run_cleansh(
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
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    assert!(output_stripped.contains("email: [EMAIL]. Invalid SSN: 000-12-3456.")); // SSN should not be redacted due to programmatic validation
    assert!(!output_stripped.contains("--- Redaction Summary ---")); // Summary should not be present in the file output.

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

    run_cleansh(
        input,
        true, // clipboard_enabled = true
        false,
        Some(temp_config_file),
        Some(output_file_path.clone()), // Output to file, *and* clipboard
        true, // No summary for easier clipboard content check (summary won't be in file anyway)
        &get_default_theme_map(),
        vec![],
        vec![],
    )?;

    let mut clipboard = arboard::Clipboard::new().context("Failed to get clipboard")?;
    let clipboard_content = clipboard.get_text().context("Failed to read clipboard")?;

    assert_eq!(clipboard_content.trim(), "email: [EMAIL]"); // Use trim due to potential newline differences

    // Verify the file content as well, it should contain the sanitized output
    let output_from_file = std::fs::read_to_string(&output_file_path)?;
    let output_stripped_from_file = strip_ansi_escapes::strip_str(&output_from_file);
    assert_eq!(output_stripped_from_file.trim(), "email: [EMAIL]");

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

    run_cleansh(
        input,
        false,
        true, // diff_enabled = true
        Some(temp_config_file),
        Some(output_file_path.clone()), // Output to file
        true, // No summary to focus on diff (summary won't be in file anyway)
        &get_default_theme_map(),
        vec![],
        vec![],
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    // When diff is enabled AND output is to a file, the diff content itself goes to the file.
    // The summary, if enabled, would still go to stderr/console.
    assert!(output_stripped.contains("-Original email: test@example.com"));
    assert!(output_stripped.contains("+Original email: [EMAIL]"));
    assert!(output_stripped.contains(" Another line.")); // Note the leading space for context lines
    assert!(!output_stripped.contains("--- Redaction Summary ---")); // Summary should not be in the diff file output.


    Ok(())
}