// tests/cleansh_integration_tests.rs
// This file contains integration tests for the cleansh application.

use anyhow::Result;
use anyhow::Context;
use std::collections::HashMap;
// Import strip_ansi_escapes
use strip_ansi_escapes;

// Import the specific function and types needed from the main crate
use cleansh::test_exposed::commands::run_cleansh;
use cleansh::test_exposed::config;
use cleansh::test_exposed::ui::theme::{self, ThemeEntry};

// This block ensures that logging (e.g., from pii_debug! macro) is set up for tests.
// It initializes env_logger exactly once per test run.
#[allow(unused_imports)] // Allow unused for clarity, as it's not always directly called
#[cfg(test)]
mod test_setup {
    use std::sync::Once;
    static INIT: Once = Once::new();

    pub fn setup_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .try_init()
                .ok(); // Ignore error if logger already initialized
        });
    }
}

// Helper function to get default theme map, moved here as it's only used by tests.
fn get_default_theme_map() -> HashMap<ThemeEntry, theme::ThemeStyle> {
    theme::ThemeStyle::default_theme_map()
}

#[test]
fn test_run_cleansh_basic_sanitization() -> Result<()> {
    test_setup::setup_logger(); // Initialize logger for this test
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

    // Create a temporary directory and file for output
    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output.txt");

    // Create a temporary config file for the test.
    let temp_config_file = temp_dir.path().join("test_rules.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    // Call the public function from the commands module
    run_cleansh(
        input,
        false, // clipboard_enabled
        false, // diff_enabled
        Some(temp_config_file), // config_path
        None, // rules_config_name: Added this argument with None
        Some(output_file_path.clone()), // output_path
        false, // no_redaction_summary (this means summary *should* be displayed on console/stderr, but not captured here)
        &get_default_theme_map(),
        vec![], // enable_rules
        vec![], // disable_rules
        None, // ADDED: input_file_path - no specific input file path for this test
    )?;

    let output_from_file = std::fs::read_to_string(&output_file_path)?;
    // Strip ANSI escape codes before assertions for robust comparison
    let output_stripped_from_file = strip_ansi_escapes::strip_str(&output_from_file);

    // Check output: Should ONLY contain sanitized content, as summary goes to stderr/console.
    // Assert that both email and SSN (which passes programmatic validation) are redacted.
    assert_eq!(output_stripped_from_file.trim(), "email: [EMAIL]. My SSN is [US_SSN_REDACTED].");

    // As noted, direct capture of stdout/stderr from `run_cleansh` isn't feasible here.
    // The `cli_integration_tests.rs` suite handles this aspect.

    Ok(())
}

#[test]
fn test_run_cleansh_no_redaction_summary() -> Result<()> {
    test_setup::setup_logger();
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
        None, // rules_config_name: Added this argument with None
        Some(output_file_path.clone()),
        true, // no_redaction_summary = true
        &get_default_theme_map(),
        vec![],
        vec![],
        None, // ADDED: input_file_path
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    // Email should be redacted, but the invalid SSN should *not* be redacted due to programmatic validation failing.
    assert_eq!(output_stripped.trim(), "email: [EMAIL]. Invalid SSN: 000-12-3456.");
    // Summary should not be present in the file output.
    assert!(!output_stripped.contains("--- Redaction Summary ---"));

    Ok(())
}

#[test]
#[cfg(feature = "clipboard")] // Only run if clipboard feature is enabled
fn test_run_cleansh_clipboard_copy() -> Result<()> {
    test_setup::setup_logger();

    // Skip this test if running in a CI environment (headless, no display server)
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping test_run_cleansh_clipboard_copy in CI (no display/X11)");
        return Ok(());
    }

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
        None, // rules_config_name: Added this argument with None
        Some(output_file_path.clone()), // Output to file, *and* clipboard
        true, // No summary for cleaner test focus
        &get_default_theme_map(),
        vec![],
        vec![],
        None, // ADDED: input_file_path
    )?;

    let mut clipboard = arboard::Clipboard::new().context("Failed to get clipboard")?;
    let clipboard_content = clipboard.get_text().context("Failed to read clipboard")?;

    // Use trim to handle potential newline differences between OS/clipboard implementations
    assert_eq!(clipboard_content.trim(), "email: [EMAIL]"); 

    // Verify the file content as well, it should contain the sanitized output
    let output_from_file = std::fs::read_to_string(&output_file_path)?;
    let output_stripped_from_file = strip_ansi_escapes::strip_str(&output_from_file);
    assert_eq!(output_stripped_from_file.trim(), "email: [EMAIL]");

    Ok(())
}

#[test]
fn test_run_cleansh_diff_output() -> Result<()> {
    test_setup::setup_logger();
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
        None, // rules_config_name: Added this argument with None
        Some(output_file_path.clone()), // Output to file
        true, // No summary to focus on diff
        &get_default_theme_map(),
        vec![],
        vec![],
        None, // ADDED: input_file_path
    )?;

    let output = std::fs::read_to_string(&output_file_path)?;
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    // Assert that the diff output contains correctly formatted lines.
    // The `\n` characters should be interpreted as actual newlines, not literal strings.
    // We'll check for the presence of the exact lines expected in a multi-line diff.
    let expected_diff_output_part = vec![
        "-Original email: test@example.com",
        "+Original email: [EMAIL]",
        " Another line.",
    ].join("\n"); // Join with actual newlines

    assert!(output_stripped.contains(&expected_diff_output_part), "Expected diff part not found:\n{}", expected_diff_output_part);
    assert!(!output_stripped.contains("\\n"), "Diff output should not contain literal \\n sequences."); // Explicitly check for absence of literal \n

    assert!(!output_stripped.contains("--- Redaction Summary ---")); // Summary should not be in the diff file output.

    Ok(())
}