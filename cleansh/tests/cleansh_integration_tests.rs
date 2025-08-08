// tests/cleansh_integration_tests.rs
//! This file contains integration tests for the `cleansh` application.
//!
//! Integration tests verify the end-to-end functionality of the `cleansh` application
//! by simulating real-world usage scenarios, including applying redaction rules,
//! handling different output modes (file output, clipboard, diff view), and
//! managing the display of redaction summaries.
//!
//! These tests leverage `tempfile` for creating temporary configuration and output files,
//! `anyhow` for simplified error handling, and `strip-ansi-escapes` for robust
//! assertion against terminal output that might contain ANSI color codes.
//! The `test_exposed` feature is used to access internal functions for testing.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
// Import strip_ansi_escapes to remove ANSI color codes before assertions.
use strip_ansi_escapes;

// Import the specific functions and types needed from the main crate for testing.
use cleansh::test_exposed::commands::run_cleansh_opts;
use cleansh::test_exposed::ui::theme::{self, ThemeEntry};
use cleansh::test_exposed::config::{RedactionConfig, merge_rules};
use cleansh_core::engine::{SanitizationEngine, RegexEngine};
use cleansh::commands::cleansh::CleanshOptions;


/// This module ensures that logging (e.g., from `pii_debug!` macro) is set up for tests.
///
/// It initializes `env_logger` exactly once per test run, allowing debug and other
/// log messages to be captured and displayed during test execution if configured.
#[allow(unused_imports)] // Allow unused for clarity, as it's not always directly called
#[cfg(test)]
mod test_setup {
    use std::sync::Once;
    static INIT: Once = Once::new();

    /// Initializes the `env_logger` for tests.
    ///
    /// This function sets up a logger that filters messages based on the `RUST_LOG`
    /// environment variable, defaulting to "debug" if not specified.
    /// It ensures that the logger is initialized only once across all tests
    /// within the test suite to prevent conflicts.
    pub fn setup_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true) // Configures the logger for test environments
                .try_init() // Attempts to initialize, returns Ok(()) if successful, Err otherwise
                .ok(); // Ignore error if logger already initialized (e.g., by another test crate)
        });
    }
}

/// Helper function to retrieve the default theme map.
///
/// This function is used by tests to provide a consistent theme configuration
/// to the `run_cleansh` function, ensuring that styling attempts by `cleansh`
/// can be correctly handled and stripped if needed.
///
/// # Returns
///
/// A `HashMap` containing the default `ThemeEntry` to `ThemeStyle` mappings.
fn get_default_theme_map() -> HashMap<ThemeEntry, theme::ThemeStyle> {
    theme::ThemeStyle::default_theme_map()
}

/// Helper to create and configure a SanitizationEngine for tests.
fn create_test_engine(custom_config_path: Option<PathBuf>) -> Result<Box<dyn SanitizationEngine>> {
    let mut config = RedactionConfig::load_default_rules()
        .context("Failed to load default redaction rules")?;

    if let Some(path) = custom_config_path {
        let user_config = RedactionConfig::load_from_file(&path)
            .context("Failed to load user-defined configuration file")?;
        config = merge_rules(config, Some(user_config));
    }

    let engine = RegexEngine::new(config)?;
    Ok(Box::new(engine))
}

/// Tests basic sanitization functionality of `run_cleansh`.
///
/// This test sets up a simple redaction configuration with rules for email
/// addresses and US SSNs (including programmatic validation). It then calls
/// `run_cleansh` with sample input and asserts that the output file
/// contains the correctly sanitized content, with ANSI escape codes removed
/// for reliable string comparison. It specifically checks that the SSN is
/// redacted because it passes the programmatic validation.
///
/// # Test Steps:
/// 1. Initialize logger.
/// 2. Define sample `input` string containing an email and an SSN.
/// 3. Create a `RedactionConfig` with rules for email and US SSN,
///    marking the SSN rule for programmatic validation.
/// 4. Create a temporary directory and an output file path.
/// 5. Serialize the `RedactionConfig` to a temporary YAML file.
/// 6. Call `run_cleansh_opts` with the new `CleanshOptions` struct.
/// 7. Read the content from the temporary output file.
/// 8. Strip any ANSI escape codes from the read content.
/// 9. Assert that the stripped output matches the expected sanitized string.
///    The email and the valid SSN should both be redacted.
///
/// # Returns
///
/// `Ok(())` if the test passes, `Err` if any step fails.
#[test]
fn test_run_cleansh_basic_sanitization() -> Result<()> {
    test_setup::setup_logger(); // Initialize logger for this test
    // Setup: Minimal configuration for testing
    let input = "email: test@example.com. My SSN is 123-45-6789.";
    let config = cleansh::test_exposed::config::RedactionConfig {
        rules: vec![
            cleansh::test_exposed::config::RedactionRule {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
                replace_with: "[EMAIL]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: false,
            },
            cleansh::test_exposed::config::RedactionRule {
                name: "us_ssn".to_string(),
                pattern: r"\b(\d{3})-(\d{2})-(\d{4})\b".to_string(), // Pattern with capturing groups for SSN validation
                replace_with: "[US_SSN_REDACTED]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: true, // Enable programmatic validation for this rule
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

    // Create the sanitization engine using the new helper
    let engine = create_test_engine(Some(temp_config_file.clone()))?;

    // Create the new CleanshOptions struct
    let opts = CleanshOptions {
        input: input.to_string(),
        clipboard: false,
        diff: false,
        output_path: Some(output_file_path.clone()),
        no_redaction_summary: false,
        quiet: false, // Added the missing field
    };
    let theme_map = get_default_theme_map();

    // Call the new, refactored function
    run_cleansh_opts(&*engine, opts, &theme_map)?;

    let output_from_file = std::fs::read_to_string(&output_file_path)?;
    // Strip any ANSI escape codes from the read content.
    let output_stripped_from_file = strip_ansi_escapes::strip_str(&output_from_file);

    // Check output: Should ONLY contain sanitized content, as summary goes to stderr/console.
    // Assert that both email and SSN (which passes programmatic validation) are redacted.
    assert_eq!(output_stripped_from_file.trim(), "email: [EMAIL]. My SSN is [US_SSN_REDACTED].");

    Ok(())
}

/// Tests the `run_cleansh` functionality when `no_redaction_summary` is enabled.
///
/// This test verifies that when the `no_redaction_summary` flag is set to `true`,
/// the redaction summary is *not* included in the output file. It also includes
/// a rule with programmatic validation for an invalid SSN to ensure that
/// `run_cleansh` correctly handles validation failures (i.e., the invalid SSN
/// should not be redacted).
///
/// # Test Steps:
/// 1. Initialize logger.
/// 2. Define sample `input` with an email and an *invalid* SSN.
/// 3. Create a `RedactionConfig` for email and US SSN (with programmatic validation).
/// 4. Create temporary files for output and config.
/// 5. Call `run_cleansh_opts` with the new `CleanshOptions` struct.
/// 6. Read the content from the output file.
/// 7. Strip ANSI escape codes from the output.
/// 8. Assert that the email is redacted, but the invalid SSN is *not* redacted.
/// 9. Assert that the "Redaction Summary" header is explicitly *not* present in the output.
///
/// # Returns
///
/// `Ok(())` if the test passes, `Err` if any step fails.
#[test]
fn test_run_cleansh_no_redaction_summary() -> Result<()> {
    test_setup::setup_logger();
    let input = "email: test@example.com. Invalid SSN: 000-12-3456."; // SSN 000-12-3456 is invalid programmatically
    let config = cleansh::test_exposed::config::RedactionConfig {
        rules: vec![
            cleansh::test_exposed::config::RedactionRule {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
                replace_with: "[EMAIL]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: false,
            },
            cleansh::test_exposed::config::RedactionRule {
                name: "us_ssn".to_string(),
                pattern: r"\b(\d{3})-(\d{2})-(\d{4})\b".to_string(),
                replace_with: "[US_SSN_REDACTED]".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                opt_in: false,
                programmatic_validation: true, // Enable programmatic validation
            },
        ],
    };

    let temp_dir = tempfile::tempdir()?;
    let output_file_path = temp_dir.path().join("output_no_summary.txt");
    let temp_config_file = temp_dir.path().join("test_rules_no_summary.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&temp_config_file, config_yaml)?;

    // Create the sanitization engine using the new helper
    let engine = create_test_engine(Some(temp_config_file.clone()))?;

    // Create the new CleanshOptions struct
    let opts = CleanshOptions {
        input: input.to_string(),
        clipboard: false,
        diff: false,
        output_path: Some(output_file_path.clone()),
        no_redaction_summary: true, // This is the core of this test
        quiet: false, // Added the missing field
    };
    let theme_map = get_default_theme_map();

    // Call the new, refactored function
    run_cleansh_opts(&*engine, opts, &theme_map)?;

    let output = std::fs::read_to_string(&output_file_path)?;
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    // Email should be redacted, but the invalid SSN should *not* be redacted due to programmatic validation failing.
    assert_eq!(output_stripped.trim(), "email: [EMAIL]. Invalid SSN: 000-12-3456.");
    // Summary should not be present in the file output when `no_redaction_summary` is true.
    assert!(!output_stripped.contains("--- Redaction Summary ---"));

    Ok(())
}

/// Tests `run_cleansh` functionality when copying to clipboard is enabled.
///
/// This test verifies that when the `clipboard_enabled` flag is set to `true`,
/// the sanitized content is correctly copied to the system clipboard.
/// It also confirms that the output file still receives the sanitized content.
/// The test is conditionally compiled and skipped in CI environments because
/// clipboard interaction often requires a display server (e.g., X11).
///
/// # Pre-conditions:
/// - The `clipboard` feature must be enabled (`#[cfg(feature = "clipboard")]`).
/// - The test will be skipped if the `CI` environment variable is set.
///
/// # Test Steps:
/// 1. Initialize logger.
/// 2. Skip if in CI environment.
/// 3. Define sample `input` and `RedactionConfig` for email redaction.
/// 4. Create temporary files for output and config.
/// 5. Call `run_cleansh_opts` with the new `CleanshOptions` struct.
/// 6. Attempt to acquire a clipboard instance and read its content.
/// 7. Assert that the clipboard content matches the expected sanitized string (trimmed to handle OS differences).
/// 8. Read the content from the output file and strip ANSI codes.
/// 9. Assert that the file content also matches the expected sanitized string.
///
/// # Returns
///
/// `Ok(())` if the test passes (or is skipped), `Err` if any step fails (and not skipped).
#[test]
#[cfg(feature = "clipboard")] // Only run if clipboard feature is enabled in Cargo.toml
fn test_run_cleansh_clipboard_copy() -> Result<()> {
    test_setup::setup_logger();

    // Skip this test if running in a CI environment (headless, no display server)
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping test_run_cleansh_clipboard_copy in CI (no display/X11)");
        return Ok(());
    }

    let input = "email: test@example.com";
    let config = cleansh::test_exposed::config::RedactionConfig {
        rules: vec![cleansh::test_exposed::config::RedactionRule {
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

    // Create the sanitization engine using the new helper
    let engine = create_test_engine(Some(temp_config_file.clone()))?;

    // Create the new CleanshOptions struct
    let opts = CleanshOptions {
        input: input.to_string(),
        clipboard: true, // This is the core of this test
        diff: false,
        output_path: Some(output_file_path.clone()),
        no_redaction_summary: true, // No summary for cleaner test focus.
        quiet: false, // Added the missing field
    };
    let theme_map = get_default_theme_map();

    // Call the new, refactored function
    run_cleansh_opts(&*engine, opts, &theme_map)?;

    // Attempt to get clipboard content *after* run_cleansh has completed.
    // This part will only run if the CI check above allowed the test to proceed.
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

/// Tests `run_cleansh` functionality when diff output is enabled.
///
/// This test verifies that when `diff_enabled` is set to `true`, the output file
/// contains a standard diff format showing the changes between the original and
/// sanitized content. It specifically checks for the presence of expected
/// diff lines (marked with '-' for removed and '+' for added content) and ensures
/// that literal `\n` characters are not present (i.e., newlines are correctly interpreted).
///
/// # Test Steps:
/// 1. Initialize logger.
/// 2. Define sample `input` with an email and another line.
/// 3. Create a `RedactionConfig` for email redaction.
/// 4. Create temporary files for output and config.
/// 5. Call `run_cleansh_opts` with the new `CleanshOptions` struct.
/// 6. Read the content from the output file.
/// 7. Strip ANSI escape codes from the output.
/// 8. Construct the expected diff output fragment.
/// 9. Assert that the stripped output contains the expected diff fragment.
/// 10. Assert that the stripped output does *not* contain literal `\n` sequences,
///    confirming correct newline handling.
/// 11. Assert that the "Redaction Summary" header is *not* present in the diff output.
///
/// # Returns
///
/// `Ok(())` if the test passes, `Err` if any step fails.
#[test]
fn test_run_cleansh_diff_output() -> Result<()> {
    test_setup::setup_logger();
    let input = "Original email: test@example.com\nAnother line.";
    let config = cleansh::test_exposed::config::RedactionConfig {
        rules: vec![cleansh::test_exposed::config::RedactionRule {
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

    // Create the sanitization engine using the new helper
    let engine = create_test_engine(Some(temp_config_file.clone()))?;

    // Create the new CleanshOptions struct
    let opts = CleanshOptions {
        input: input.to_string(),
        clipboard: false,
        diff: true, // This is the core of this test
        output_path: Some(output_file_path.clone()),
        no_redaction_summary: true, // No summary to focus on diff output.
        quiet: false, // Added the missing field
    };
    let theme_map = get_default_theme_map();

    // Call the new, refactored function
    run_cleansh_opts(&*engine, opts, &theme_map)?;

    let output = std::fs::read_to_string(&output_file_path)?;
    let output_stripped = strip_ansi_escapes::strip_str(&output); // Strip ANSI escape codes

    // Assert that the diff output contains correctly formatted lines.
    let expected_diff_output_part = vec![
        "-Original email: test@example.com",
        "+Original email: [EMAIL]",
        " Another line.",
    ]
    .join("\n"); // Join with actual newlines to match expected file content

    assert!(output_stripped.contains(&expected_diff_output_part), "Expected diff part not found in output:\n'{}'\nActual output:\n'{}'", expected_diff_output_part, output_stripped);
    // Explicitly check for absence of literal \n sequences, confirming correct newline handling by diffy/printer.
    assert!(!output_stripped.contains("\\n"), "Diff output should not contain literal \\n sequences.");

    // Summary should not be present in the diff file output when `no_redaction_summary` is true.
    assert!(!output_stripped.contains("--- Redaction Summary ---"));

    Ok(())
}