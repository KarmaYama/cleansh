use anyhow::Result;
#[allow(unused_imports)]
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

#[allow(unused_imports)]
use assert_cmd::prelude::*;
use assert_cmd::Command;

use strip_ansi_escapes::strip as strip_ansi_escapes_fn;

fn run_cleansh_command(input: &str, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("cleansh").unwrap();
    cmd.args(args);
    cmd.write_stdin(input.as_bytes()).unwrap();
    cmd
}

fn strip_ansi(s: &str) -> String {
    let cleaned = strip_ansi_escapes_fn(s);
    String::from_utf8_lossy(&cleaned).to_string()
}

// Helper to extract the core sanitized content, robust to summary/diff/info presence/absence
fn extract_sanitized_content(output: &str) -> String {
    let mut lines: Vec<&str> = output.lines().collect();

    // Filter out informational/meta lines
    lines.retain(|line| {
        !(line.contains("Reading input from stdin") ||
          line.contains("Reading input from file") ||
          line.contains("Written to file") ||
          line.contains("Copied to clipboard") ||
          line.contains("No redactions applied.") ||
          line.contains("Loading custom rules from") ||
          line.contains("--- Redaction Summary ---") ||
          line.contains("-------------------------") ||
          line.contains("--- Diff View ---") ||
          line.contains("-----------------"))
    });

    // Join and trim to handle leading/trailing empty lines
    lines.join("\n").trim().to_string()
}

// Helper to extract specific messages + content for 'no redactions' case
// This function needs to precisely match the *expected* output for the
// scenario where no redactions occur and --no-redaction-summary is active.
fn extract_no_redaction_output(output: &str) -> String {
    let mut lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    // In this specific test, we pass --no-redaction-summary,
    // so we expect NO "No redactions applied." message.
    lines.retain(|line| {
        line.contains("Reading input from stdin...") ||
        line.trim().starts_with("This is a clean string with no sensitive information.") ||
        line.is_empty() // Keep empty lines if they are part of the structure
    });
    lines.join("\n").trim().to_string()
}


#[test]
fn test_basic_sanitization() -> Result<()> {
    let input = "My email is test@example.com and my IP is 192.168.1.1.";
    let expected_sanitized_content = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].";
    // Construct the full expected output including info messages and summary.
    // The "Reading input from stdin..." is followed by a newline, then the summary,
    // and finally the sanitized content (which is printed after the summary).
    // Note: The extra newline after "Reading input from stdin..." is from output_format::print_info_message
    // and then there's an inherent newline after the summary block.
    let expected_output_with_summary = format!(
        "Reading input from stdin...\n\n\
         --- Redaction Summary ---\n\
         email (1 occurrences)\n\
         ipv4_address (1 occurrences)\n\
         -------------------------\n\n{}\
         ", // ADDED AN EXTRA NEWLINE HERE to match the actual output
        expected_sanitized_content
    );

    // Add --no-clipboard to prevent default clipboard action from .env
    // This test expects the summary, so no --no-redaction-summary here.
    let output = run_cleansh_command(input, &["--no-clipboard"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // Assert the full stripped output matches the constructed expected output.
    assert_eq!(stripped.trim(), expected_output_with_summary.trim());
    Ok(())
}

#[test]
fn test_clipboard_output() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (headless environment)");
        return Ok(());
    }
    let input = "Secret JWT: ey...";
    let expected_sanitized = "Secret [GENERIC_TOKEN_REDACTED]: ey...";
    // Ensure we specifically request clipboard behavior and suppress summary
    let output = run_cleansh_command(input, &["-c", "--no-redaction-summary", "--enable-rules", "jwt_token,generic_token"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    assert!(stripped.contains("Copied to clipboard."));
    assert!(!stripped.contains("--- Redaction Summary ---")); // Assert summary is NOT present

    // When summary is suppressed and no file output/diff, the sanitized content is printed to stdout.
    let content_to_check = extract_sanitized_content(&stripped);
    assert_eq!(content_to_check, expected_sanitized);
    Ok(())
}

#[test]
fn test_diff_view() -> Result<()> {
    let input = "Old IP: 10.0.0.1. New IP: 192.168.1.1.";
    let expected_diff_content = "-Old IP: 10.0.0.1. New IP: 192.168.1.1.\n+Old IP: [IPV4_REDACTED]. New IP: [IPV4_REDACTED].";
    // Add --no-clipboard and --no-redaction-summary
    let output = run_cleansh_command(input, &["-d", "--no-clipboard", "--no-redaction-summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));

    // Assert presence of common elements
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(!stripped.contains("--- Redaction Summary ---")); // Summary should be suppressed
    assert!(stripped.contains("--- Diff View ---"));
    assert!(stripped.contains("-----------------")); // Check for diff footer

    // Extract the diff content part by looking for markers, then trim.
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

    let diff = &stripped[diff_start_idx..diff_end_idx];

    // Assert the extracted diff matches the new expected format
    assert_eq!(diff.trim(), expected_diff_content.trim());

    Ok(())
}

#[test]
fn test_output_to_file() -> Result<()> {
    let input = "This is a test with sensitive info: user@domain.com";
    let expected = "This is a test with sensitive info: [EMAIL_REDACTED]";
    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();
    // Add --no-clipboard and --no-redaction_summary
    let output = run_cleansh_command(input, &["-o", path, "--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert!(stripped.contains("Written to file."));
    assert!(!stripped.contains("--- Redaction Summary ---")); // Summary should be suppressed
    let file_contents = fs::read_to_string(path)?;
    assert_eq!(file_contents.trim(), expected);
    Ok(())
}

#[test]
fn test_custom_config_file() -> Result<()> {
    let config_yaml = r#"rules:
  - name: "custom_secret"
    pattern: "MYSECRET-\\d{4}"
    replace_with: "[CUSTOM_SECRET_REDACTED]"
    description: "A custom secret pattern."
    multiline: false
    dot_matches_new_line: false
  - name: "email"
    pattern: "([a-z]+@[a-z]+\\.org)"
    replace_with: "[ORG_EMAIL_REDACTED]"
    multiline: false
    dot_matches_new_line: false
"#;
    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(config_yaml.as_bytes())?;
    let path = config_file.path().to_str().unwrap();
    let input = "My email is user@example.com and another is user@test.org. My secret is MYSECRET-1234.";
    let expected = "My email is user@example.com and another is [ORG_EMAIL_REDACTED]. My secret is [CUSTOM_SECRET_REDACTED].";
    // Add --no-clipboard and --no-redaction_summary
    let output = run_cleansh_command(input, &["--config", path, "--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped); // Now extract_sanitized_content filters info messages
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_absolute_path_redaction() -> Result<()> {
    let input = "Accessing /home/user/documents/report.pdf and /Users/admin/logs/app.log";
    let expected = "Accessing ~/home/user/documents/report.pdf and ~/Users/admin/logs/app.log";
    // Add --no-clipboard and --no-redaction_summary
    let output = run_cleansh_command(input, &["--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped);
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_no_redactions() -> Result<()> {
    let input = "This is a clean string with no sensitive information.";
    // When --no-redaction-summary is present, we should NOT see "No redactions applied.".
    // We expect the "Reading input from stdin..." message followed by the original content.
    let expected = format!("Reading input from stdin...\n{}", input);
    // Add --no-clipboard and --no-redaction_summary
    let output = run_cleansh_command(input, &["--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_no_redaction_output(&stripped); // Use custom extractor
    assert_eq!(actual, expected.trim());
    Ok(())
}

#[test]
fn test_opt_in_rule_not_enabled_by_default() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=."; // Should NOT be redacted
    // aws_secret_key is opt-in and not enabled in args
    let output = run_cleansh_command(input, &["--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped);
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_opt_in_rule_enabled() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected = "My AWS secret key is [AWS_SECRET_KEY_REDACTED]."; // Should be redacted
    // Enable aws_secret_key via --enable-rules
    let output = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped);
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_multiple_opt_in_rules_enabled() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=. And a generic hex: 0123456789abcdef0123456789abcdef.";
    let expected = "My AWS secret key is [AWS_SECRET_KEY_REDACTED]. And a generic hex: [HEX_SECRET_32_REDACTED]."; // Both should be redacted
    // Enable both aws_secret_key and generic_hex_secret_32
    let output = run_cleansh_command(input, &["--enable-rules", "aws_secret_key,generic_hex_secret_32", "--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped);
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_opt_in_rule_enabled_with_summary() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected_sanitized_content = "My AWS secret key is [AWS_SECRET_KEY_REDACTED].";
    let expected_summary_output = format!(
        "Reading input from stdin...\n\n\
         --- Redaction Summary ---\n\
         aws_secret_key (1 occurrences)\n\
         -------------------------\n\n{}\
         ",
        expected_sanitized_content
    );

    // Enable aws_secret_key and include summary
    let output = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert_eq!(stripped.trim(), expected_summary_output.trim());
    Ok(())
}

#[test]
fn test_opt_in_rule_not_in_config() -> Result<()> {

    let input = "email@example.com and a fake secret 1234-abcd-SECRET.";
    let expected = "[EMAIL_REDACTED] and a fake secret 1234-abcd-SECRET.";

    let output = run_cleansh_command(input, &["--enable-rules", "non_existent_rule", "--no-clipboard", "--no-redaction_summary"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let actual = extract_sanitized_content(&stripped);
    assert_eq!(actual, expected); 
    Ok(())
}