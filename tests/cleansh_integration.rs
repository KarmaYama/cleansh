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

#[test]
fn test_basic_sanitization() -> Result<()> {
    let input = "My email is test@example.com and my IP is 192.168.1.1.";
    let expected = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].";
    let output = run_cleansh_command(input, &[]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert!(stripped.contains("Reading input from stdin...\n"));
    assert!(stripped.contains("--- Redaction Summary ---"));
    assert!(stripped.contains("email (1 occurrences)"));
    assert!(stripped.contains("ipv4_address (1 occurrences)"));
    assert!(stripped.contains("-------------------------\n\n"));
    let content = &stripped[stripped.find("-------------------------\n\n").unwrap() + 27..];
    assert_eq!(content.trim_end(), expected);
    Ok(())
}

#[test]
fn test_clipboard_output() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (headless environment)");
        return Ok(());
    }
    let input = "Secret JWT: ey...";
    let expected = "Secret [GENERIC_TOKEN_REDACTED]: ey...";
    let output = run_cleansh_command(input, &["-c"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert!(stripped.contains("✅ Copied to clipboard."));
    let start = stripped.find("✅ Copied to clipboard.").unwrap() + "✅ Copied to clipboard.".len();
    let content = &stripped[start..];
    let content_trimmed = content.trim();
    assert_eq!(content_trimmed, expected);
    Ok(())
}

#[test]
fn test_diff_view() -> Result<()> {
    let input = "Old IP: 10.0.0.1. New IP: 192.168.1.1.";
    // FIX APPLIED HERE: Updated expected string to match the new diffy line-by-line format
    let expected = "-Old IP: 10.0.0.1. New IP: 192.168.1.1.\n+Old IP: [IPV4_REDACTED]. New IP: [IPV4_REDACTED].";
    let output = run_cleansh_command(input, &["-d"]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    
    // Assert presence of common elements
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

    let diff = &stripped[diff_start_idx..diff_end_idx];
    
    // Assert the extracted diff matches the new expected format
    assert_eq!(diff.trim(), expected.trim());
    
    // Removed the assertion about not containing the final sanitized content,
    // as the diff view explicitly shows both original and redacted parts.
    Ok(())
}

#[test]
fn test_output_to_file() -> Result<()> {
    let input = "This is a test with sensitive info: user@domain.com";
    let expected = "This is a test with sensitive info: [EMAIL_REDACTED]";
    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();
    let output = run_cleansh_command(input, &["-o", path]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert!(stripped.contains("✅ Written to file."));
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
    let output = run_cleansh_command(input, &["--config", path]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let idx = stripped.find("-------------------------\n\n").unwrap() + 27;
    let actual = &stripped[idx..];
    assert_eq!(actual.trim_end(), expected);
    Ok(())
}

#[test]
fn test_absolute_path_redaction() -> Result<()> {
    let input = "Accessing /home/user/documents/report.pdf and /Users/admin/logs/app.log";
    let expected = "Accessing ~/home/user/documents/report.pdf and ~/Users/admin/logs/app.log";
    let output = run_cleansh_command(input, &[]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    let idx = stripped.find("-------------------------\n\n").unwrap() + 27;
    let actual = &stripped[idx..];
    assert_eq!(actual.trim_end(), expected);
    Ok(())
}

#[test]
fn test_no_redactions() -> Result<()> {
    let input = "This is a clean string with no sensitive information.";
    let expected = format!("Reading input from stdin...\n\nNo redactions applied.\n{}", input);
    let output = run_cleansh_command(input, &[]).assert().success().get_output().stdout.clone();
    let stripped = strip_ansi(&String::from_utf8_lossy(&output));
    assert_eq!(stripped.trim_end(), expected.trim_end());
    Ok(())
}