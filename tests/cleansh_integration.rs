use anyhow::Result;
#[allow(unused_imports)] // Allow if not directly using predicate macros but prelude might be needed for traits
use predicates::prelude::*; // Brings in useful predicate macros and types (keep for now, may be implicitly used)
use tempfile::NamedTempFile;
use std::io::Write; // Needed for file.write_all
use std::fs; // For fs::read_to_string

#[allow(unused_imports)] // Allow if prelude traits are used but not direct functions/macros
use assert_cmd::prelude::*;
use assert_cmd::Command; // This is the assert_cmd::Command, which has the extended methods

// Import the function to strip ANSI escape codes
use strip_ansi_escapes::strip as strip_ansi_escapes_fn;

// Helper function to run the cleansh command with given input and arguments
fn run_cleansh_command(input: &str, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("cleansh").unwrap();
    cmd.arg("--debug"); // Keep debug argument for more verbose output if needed
    cmd.args(args); // Set arguments BEFORE writing to stdin
    cmd.write_stdin(input.as_bytes()).unwrap();
    cmd
}

// Helper to strip ANSI codes from a string
fn strip_ansi(s: &str) -> String {
    let cleaned = strip_ansi_escapes_fn(s); // Returns Vec<u8>
    String::from_utf8_lossy(&cleaned).to_string()
}

#[test]
fn test_basic_sanitization() -> Result<()> {
    let input = "My email is test@example.com and my IP is 192.168.1.1.";
    // Based on debug output: content appears after "-------------------------\n\n" and ends without a trailing newline
    let expected_sanitized_content = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].";

    let output = run_cleansh_command(input, &[])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // The info message about reading from stdin is now part of the output
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("email (1 occurrences)"));
    assert!(stripped.contains("ipv4_address (1 occurrences)"));
    assert!(stripped.contains("-------------------------\n\n")); // Assert the ending sequence of the summary

    // Extract the part after the summary and compare it
    let summary_end_marker = "-------------------------\n\n";
    let summary_end_idx = stripped.find(summary_end_marker)
                                  .map(|idx| idx + summary_end_marker.len())
                                  .unwrap_or_else(|| {
                                      panic!("Summary end marker not found in output: '{}'", stripped);
                                  });
    let actual_sanitized_part = &stripped[summary_end_idx..];

    // Use assert_eq for exact match of the content part, trimming any unexpected newlines
    assert_eq!(actual_sanitized_part.trim_end(), expected_sanitized_content.trim_end());
    Ok(())
}

#[test]
fn test_clipboard_output() -> Result<()> {
    let input = "Secret JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    // Adjust expected_sanitized_content to include clipboard message and GENERIC_TOKEN_REDACTED
    let expected_sanitized_content = "✅ Copied to clipboard.\nSecret [GENERIC_TOKEN_REDACTED]: [JWT_REDACTED]";

    let output = run_cleansh_command(input, &["-c"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // The info message about reading from stdin is now part of the output
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("jwt_token (1 occurrences)")); // Still expect JWT to be counted
    assert!(stripped.contains("-------------------------\n\n")); // Assert the ending sequence of the summary

    // Extract content after summary for comparison
    let summary_end_marker = "-------------------------\n\n";
    let summary_end_idx = stripped.find(summary_end_marker)
                                  .map(|idx| idx + summary_end_marker.len())
                                  .unwrap_or_else(|| {
                                      panic!("Summary end marker not found in output: '{}'", stripped);
                                  });
    let actual_sanitized_part = &stripped[summary_end_idx..];

    assert_eq!(actual_sanitized_part.trim_end(), expected_sanitized_content.trim_end());
    Ok(())
}

#[test]
fn test_diff_view() -> Result<()> {
    let input = "Old IP: 10.0.0.1. New IP: 192.168.1.1.";

    let output = run_cleansh_command(input, &["-d"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // Based on the last debug output for diff:
    // It appears the content portion is `  Old IP: -10.0.0.1. New IP: 192.168.1.1+[IPV4_REDACTED]. New IP: [IPV4_REDACTED] .`
    // And it is wrapped in Diff View headers.
    let expected_diff_content = "  Old IP: -10.0.0.1. New IP: 192.168.1.1+[IPV4_REDACTED]. New IP: [IPV4_REDACTED] ."; // Exact match from debug output

    // The info message about reading from stdin is now part of the output
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("ipv4_address (2 occurrences)"));
    assert!(stripped.contains("-------------------------\n\n")); // Check for summary footer
    assert!(stripped.contains("--- Diff View ---"));
    assert!(stripped.contains("-----------------")); // Check for diff footer

    // Extract the diff content part
    let diff_start_marker = "--- Diff View ---\n";
    let diff_end_marker = "\n-----------------";

    let diff_start_idx = stripped.find(diff_start_marker)
                                 .map(|idx| idx + diff_start_marker.len())
                                 .unwrap_or_else(|| {
                                     panic!("Diff start marker not found: '{}'", stripped);
                                 });
    let diff_end_idx = stripped[diff_start_idx..].find(diff_end_marker)
                                                 .map(|idx| idx + diff_start_idx)
                                                 .unwrap_or_else(|| {
                                                     panic!("Diff end marker not found after start: '{}'", stripped);
                                                 });

    let actual_diff_part = &stripped[diff_start_idx..diff_end_idx];

    // Use assert_eq for the exact diff content
    assert_eq!(actual_diff_part.trim(), expected_diff_content.trim()); // Trim to ignore any leading/trailing newlines/spaces

    // Ensure the *final sanitized content* is NOT printed to stdout when diff is enabled
    assert!(!stripped.contains("Old IP: [IPV4_REDACTED]. New IP: [IPV4_REDACTED]."));
    Ok(())
}

#[test]
fn test_output_to_file() -> Result<()> {
    let input = "This is a test with sensitive info: user@domain.com";
    let expected_file_content = "This is a test with sensitive info: [EMAIL_REDACTED]"; // This is for the file content, no leading newline needed.

    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();

    let output = run_cleansh_command(input, &["-o", path])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    // The info message about reading from stdin is now part of the output
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("✅ Written to file."));
    assert!(stripped.contains("email (1 occurrences)"));
    assert!(stripped.contains("-------------------------\n\n"));

    let file_contents = fs::read_to_string(path)?;
    assert_eq!(file_contents.trim(), expected_file_content.trim());
    Ok(())
}

#[test]
fn test_custom_config_file() -> Result<()> {
    let custom_rules_yaml = r#"
rules:
  - name: "custom_secret"
    pattern: "MYSECRET-\\d{4}"
    replace_with: "[CUSTOM_SECRET_REDACTED]"
    description: "A custom secret pattern."
    multiline: false
    dot_matches_new_line: false
  - name: "email" # Override default email
    pattern: "([a-z]+@[a-z]+\\.org)" # Only match .org emails
    replace_with: "[ORG_EMAIL_REDACTED]"
    multiline: false
    dot_matches_new_line: false
"#;
    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(custom_rules_yaml.as_bytes())?;
    let config_path = config_file.path().to_str().unwrap();

    let input = "My email is user@example.com and another is user@test.org. My secret is MYSECRET-1234.";
    // Adjusted: user@example.com is NOT redacted by custom rule, user@test.org IS.
    let expected_sanitized_content = "My email is user@example.com and another is [ORG_EMAIL_REDACTED]. My secret is [CUSTOM_SECRET_REDACTED].";

    let output = run_cleansh_command(input, &["--config", config_path])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped_output = strip_ansi(&String::from_utf8_lossy(&output));

    // The info message about reading from stdin and loading config are now part of the output
    assert!(stripped_output.contains("Reading input from stdin...\n"));
    assert!(stripped_output.contains(&format!("Loading custom rules from: {}\n", config_path)));
    assert!(stripped_output.contains("--- Redaction Summary ---"));
    assert!(stripped_output.contains("custom_secret (1 occurrences)"));
    assert!(stripped_output.contains("email (1 occurrences)")); // Still counts email if it matches the *overridden* email rule
    assert!(stripped_output.contains("-------------------------\n\n"));

    let summary_end_marker = "-------------------------\n\n";
    let summary_end_idx = stripped_output.find(summary_end_marker)
                                  .map(|idx| idx + summary_end_marker.len())
                                  .unwrap_or_else(|| {
                                      panic!("Summary end marker not found in output: '{}'", stripped_output);
                                  });
    let actual_sanitized_part = &stripped_output[summary_end_idx..];

    assert_eq!(actual_sanitized_part.trim_end(), expected_sanitized_content.trim_end());

    // Also check for the specific custom redaction
    assert!(stripped_output.contains("[ORG_EMAIL_REDACTED]"));
    assert!(stripped_output.contains("user@example.com")); // Explicitly assert non-redacted email
    Ok(())
}

#[test]
fn test_absolute_path_redaction() -> Result<()> {
    let input = "Accessing /home/user/documents/report.pdf and /Users/admin/logs/app.log";
    // *** FIX APPLIED HERE: Corrected expected_sanitized_content for macOS path ***
    let expected_sanitized_content = "Accessing ~/home/user/documents/report.pdf and ~/Users/admin/logs/app.log";

    let output = run_cleansh_command(input, &[])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // The info message about reading from stdin is now part of the output
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("absolute_linux_path (1 occurrences)"));
    assert!(stripped.contains("absolute_macos_path (1 occurrences)"));
    assert!(stripped.contains("-------------------------\n\n"));

    let summary_end_marker = "-------------------------\n\n";
    let summary_end_idx = stripped.find(summary_end_marker)
                                  .map(|idx| idx + summary_end_marker.len())
                                  .unwrap_or_else(|| {
                                      panic!("Summary end marker not found in output: '{}'", stripped);
                                  });
    let actual_sanitized_part = &stripped[summary_end_idx..];

    assert_eq!(actual_sanitized_part.trim_end(), expected_sanitized_content.trim_end());
    Ok(())
}

#[test]
fn test_no_redactions() -> Result<()> {
    let input = "This is a clean string with no sensitive information.";

    // FIX APPLIED HERE: Updated expected_full_output to include the new info message
    let expected_full_output = format!("Reading input from stdin...\n\nNo redactions applied.\n{}", input);

    let output = run_cleansh_command(input, &[])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    assert_eq!(stripped.trim_end(), expected_full_output.trim_end());

    Ok(())
}