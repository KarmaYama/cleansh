// tests/cli_integration_tests.rs
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

// Helper to run the cleansh command with given input and arguments
// Now captures both stdout and stderr and sets RUST_LOG for the child process.
fn run_cleansh_command(input: &str, args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("cleansh").unwrap();
    // CRITICAL: Set RUST_LOG for the *spawned cleansh process*
    // This ensures debug logs from your application are visible in the test output.
    cmd.env("RUST_LOG", "debug");
    // Allow PII debug logs for testing purposes
    // Setting this to "true" means the "Rule '{}' captured match (original): {}" log
    // will display the *original*, unredacted PII.
    cmd.env("CLEANSH_ALLOW_DEBUG_PII", "true");
    cmd.args(args);
    cmd.write_stdin(input.as_bytes()).unwrap();
    cmd.assert()
}

// Helper to strip ANSI escape codes from a string
fn strip_ansi(s: &str) -> String {
    let cleaned = strip_ansi_escapes_fn(s);
    String::from_utf8_lossy(&cleaned).to_string()
}

#[test]
fn test_basic_sanitization() -> Result<()> {
    let input = "My email is test@example.com and my IP is 192.168.1.1.";
    // FIX APPLIED HERE: Added '\n' to the end of the expected_stdout string.
    let expected_stdout = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].\n";
    let expected_stderr_contains_substrings = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "Writing sanitized content to stdout.".to_string(),
        "--- Redaction Summary ---".to_string(),
        "email (1 occurrences)".to_string(),
        "ipv4_address (1 occurrences)".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Starting cleansh operation.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.".to_string(),
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 54, Sanitized length: 58".to_string(),
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 2".to_string(),
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);

    for msg in expected_stderr_contains_substrings {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }

    // Updated assertions to match the new log prefixes from `log_captured_match_debug`
    // and `log_redaction_action_debug` in `src/utils/redaction.rs`, and `log_redaction_match_debug` in `cleansh.rs`.
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'test@example.com' for rule 'email'"),
        "Stderr missing expected original capture log for email.\nFull stderr:\n{}", stderr
    );
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='test@example.com', Redacted='[EMAIL_REDACTED]' for rule 'email'"),
        "Stderr missing expected redaction action log for email.\nFull stderr:\n{}", stderr
    );
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): '192.168.1.1' for rule 'ipv4_address'"),
        "Stderr missing expected original capture log for IP.\nFull stderr:\n{}", stderr
    );
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='192.168.1.1', Redacted='[IPV4_REDACTED]' for rule 'ipv4_address'"),
        "Stderr missing expected redaction action log for IP.\nFull stderr:\n{}", stderr
    );
    // These logs are now also coming from `log_redaction_match_debug` in `cleansh.rs`
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='email', Original='test@example.com', Sanitized='[EMAIL_REDACTED]'"),
        "Stderr missing expected RedactionMatch log for email.\nFull stderr:\n{}", stderr
    );
    assert!(
        stderr.contains("[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='ipv4_address', Original='192.168.1.1', Sanitized='[IPV4_REDACTED]'"),
        "Stderr missing expected RedactionMatch log for ipv4_address.\nFull stderr:\n{}", stderr
    );

    Ok(())
}

#[cfg(feature = "clipboard")]
#[test]
fn test_run_cleansh_clipboard_copy_to_file() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (no display)");
        return Ok(());
    }

    let input = "My email is test@example.com";
    let expected_stdout = "My email is [EMAIL_REDACTED]\n";
    let expected_stderr_contains = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "Writing sanitized content to file:".to_string(), // This is an INFO level log, matching the actual
        "Sanitized content copied to clipboard successfully.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Starting cleansh operation.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.".to_string(),
        // Expect original PII in logs because CLEANSH_ALLOW_DEBUG_PII is true
        // Updated to match the new centralized logging format
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'test@example.com' for rule 'email'".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='test@example.com', Redacted='[EMAIL_REDACTED]' for rule 'email'".to_string(),
        // This log is now from `cleansh.rs` using `log_redaction_match_debug`
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='email', Original='test@example.com', Sanitized='[EMAIL_REDACTED]'".to_string(),
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 28, Sanitized length: 28".to_string(),
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 1".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Outputting to file:".to_string(), // Updated this line
        "[DEBUG cleansh::commands::cleansh] Attempting to copy sanitized content to clipboard.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Cleansh operation completed.".to_string(),
    ];

    let config_yaml = r#"rules:
  - name: "email"
    pattern: "([a-z]+@[a-z]+\\.com)"
    replace_with: "[EMAIL_REDACTED]"
    description: "Email address."
    multiline: false
    dot_matches_new_line: false
    programmatic_validation: false
    opt_in: false
"#;
    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(config_yaml.as_bytes())?;
    let config_path = config_file.path().to_str().unwrap();

    let output_file = NamedTempFile::new()?;
    let output_path = output_file.path().to_str().unwrap();


    let assert_result = run_cleansh_command(input, &[
        "-c",
        "-o", output_path,
        "--config", config_path,
        "--no-redaction-summary",
    ]).success();

    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, "");

    for msg in expected_stderr_contains {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    // Corrected line: Use `{}` for `&str` directly in format!
    assert!(stderr.contains(&format!("Writing sanitized content to file: {}", output_path)));

    let file_contents = fs::read_to_string(output_path)?;
    assert_eq!(file_contents, expected_stdout);

    Ok(())
}


#[test]
fn test_clipboard_output_with_jwt() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (no display)");
        return Ok(());
    }

    let input = "Secret JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let expected_stdout = "Secret JWT: [JWT_REDACTED]\n";
    let expected_stderr_contains = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "Writing sanitized content to stdout.".to_string(),
        "Sanitized content copied to clipboard successfully.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Starting cleansh operation.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.".to_string(),
        // Expect original PII in logs because CLEANSH_ALLOW_DEBUG_PII is true
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c'".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c', Redacted='[JWT_REDACTED]' for rule 'jwt_token'".to_string(),
        // This log is now from `cleansh.rs` using `log_redaction_match_debug`
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='jwt_token', Original='eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c', Sanitized='[JWT_REDACTED]'".to_string(),
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 167, Sanitized length: 26".to_string(),
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 1".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Outputting to stdout.".to_string(),
        "[DEBUG cleansh::commands::cleansh] Attempting to copy sanitized content to clipboard.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Cleansh operation completed.".to_string(),
    ];


    let assert_result = run_cleansh_command(input, &["-c", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);

    for msg in expected_stderr_contains {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    assert!(!stderr.contains("--- Redaction Summary ---"));
    Ok(())
}

#[test]
fn test_diff_view() -> Result<()> {
    let input = "Old IP: 10.0.0.1. New IP: 192.168.1.1.";
    let expected_stdout_contains = vec![
        "-Old IP: 10.0.0.1. New IP: 192.168.1.1.".to_string(),
        "+Old IP: [IPV4_REDACTED]. New IP: [IPV4_REDACTED].".to_string(),
    ];
    let expected_stderr_contains = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "Writing sanitized content to stdout.".to_string(),
        "Generating and displaying diff.".to_string(),
        "--- Diff View ---".to_string(),
        "-----------------".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.".to_string(),
        // Adjusted to match actual log output for the first match, as it sometimes doesn't include char count
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): '10.0.0.1' for rule 'ipv4_address'".to_string(),
        "Added RedactionMatch for rule 'ipv4_address'. Current total matches: 1".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='10.0.0.1', Redacted='[IPV4_REDACTED]' for rule 'ipv4_address'".to_string(),
        // This log still has a character count, matching the provided stderr for the second match
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): '192.168.1.1' for rule 'ipv4_address'".to_string(),
        "Added RedactionMatch for rule 'ipv4_address'. Current total matches: 2".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='192.168.1.1', Redacted='[IPV4_REDACTED]' for rule 'ipv4_address'".to_string(),
        // Added Found RedactionMatch logs from cleansh::commands::cleansh context
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='ipv4_address', Original='10.0.0.1', Sanitized='[IPV4_REDACTED]'".to_string(),
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='ipv4_address', Original='192.168.1.1', Sanitized='[IPV4_REDACTED]'".to_string(),
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 38, Sanitized length: 49".to_string(),
        // Corrected to 1 as only one rule type (ipv4_address) is summarized
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 1".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Outputting to stdout.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Cleansh operation completed.".to_string(),
    ];

    let assert_result = run_cleansh_command(input, &["-d", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    for msg in expected_stdout_contains {
        assert!(stdout.contains(&msg), "Stdout missing: '{}'\nFull stderr:\n{}", msg, stdout);
    }
    assert!(!stdout.contains("--- Diff View ---"));
    assert!(!stdout.contains("-----------------"));
    assert!(!stdout.contains("--- Redaction Summary ---"));

    for msg in expected_stderr_contains {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_output_to_file() -> Result<()> {
    let input = "This is a test with sensitive info: user@domain.com";
    let expected_file_content = "This is a test with sensitive info: [EMAIL_REDACTED]\n";
    let expected_stderr_contains = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "--- Redaction Summary ---".to_string(),
        "email (1 occurrences)".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.".to_string(),
        // Expect original PII in logs because CLEANSH_ALLOW_DEBUG_PII is true
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'user@domain.com' for rule 'email'".to_string(),
        "Added RedactionMatch for rule 'email'. Current total matches: 1".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='user@domain.com', Redacted='[EMAIL_REDACTED]' for rule 'email'".to_string(),
        // Added Found RedactionMatch log from cleansh::commands::cleansh context
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='email', Original='user@domain.com', Sanitized='[EMAIL_REDACTED]'".to_string(),
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 51, Sanitized length: 52".to_string(),
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 1".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Outputting to file:".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Cleansh operation completed.".to_string(),
    ];

    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();

    let assert_result = run_cleansh_command(input, &["-o", path, "--no-clipboard"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, "");

    for msg in expected_stderr_contains {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    // Corrected line: Use `{}` for `&str` directly in format!
    assert!(stderr.contains(&format!("Writing sanitized content to file: {}", path)));

    let file_contents = fs::read_to_string(path)?;
    assert_eq!(file_contents, expected_file_content);
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
    programmatic_validation: false
    opt_in: false
  - name: "email"
    pattern: "([a-z]+@[a-z]+\\.org)"
    replace_with: "[ORG_EMAIL_REDACTED]"
    multiline: false
    dot_matches_new_line: false
    programmatic_validation: false
    opt_in: false
"#;
    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(config_yaml.as_bytes())?;
    let path = config_file.path().to_str().unwrap();

    let input = "My email is user@example.com and another is user@test.org. My secret is MYSECRET-1234.";
    let expected_stdout = "My email is user@example.com and another is [ORG_EMAIL_REDACTED]. My secret is [CUSTOM_SECRET_REDACTED].\n";

    let expected_stderr_contains: Vec<String> = vec![
        // MODIFIED: Updated expected stderr message to match actual log output
        "Reading input from stdin (batch mode).".to_string(),
        "Writing sanitized content to stdout.".to_string(),
        // Assert the presence of the Redaction Summary and its specific contents
        "--- Redaction Summary ---".to_string(),
        "custom_secret (1 occurrences)".to_string(),
        // FIX APPLIED HERE: Corrected the indentation to match the actual output.
        "    Original Values:\n        - MYSECRET-1234".to_string(),
        "    Sanitized Values:\n        - [CUSTOM_SECRET_REDACTED]".to_string(),
        "email (1 occurrences)".to_string(),
        // FIX APPLIED HERE: Corrected the indentation to match the actual output.
        "    Original Values:\n        - user@test.org".to_string(),
        "    Sanitized Values:\n        - [ORG_EMAIL_REDACTED]".to_string(),
        // Assert on specific log messages for custom config loading and rule merging
        format!("[INFO cleansh::commands::cleansh] Loading custom rules from: {}", path),
        format!("[DEBUG cleansh::commands::cleansh] [cleansh.rs] Attempting to load custom rules from: {}", path),
        format!("[DEBUG cleansh::config] [config.rs] Attempting to load config from file: {}", path),
        format!("[DEBUG cleansh::config] [config.rs] Loaded 2 rules from file {}.", path),
        "[DEBUG cleansh::config] [config.rs] File Rule - Name: custom_secret, Opt_in: false".to_string(),
        "[DEBUG cleansh::config] [config.rs] File Rule - Name: email, Opt_in: false".to_string(),
        format!("[DEBUG cleansh::commands::cleansh] [cleansh.rs] Loaded 2 custom rules from {} in cleansh.", path),
        "[DEBUG cleansh::config] Merged rules summary: 24 default rules initially, 2 user rules processed. Overrode 1 defaults, added 1 new user rules. Final total rules: 25".to_string(),
        // Assert on successful compilation of the custom and overridden email rules
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'custom_secret' compiled successfully.".to_string(),
        "[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.".to_string(),
        // Expect original PII in logs because CLEANSH_ALLOW_DEBUG_PII is true
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'user@test.org' for rule 'email'".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='user@test.org', Redacted='[ORG_EMAIL_REDACTED]' for rule 'email'".to_string(),
        // Updated to match the new centralized logging format for captured match
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Captured match (original): 'MYSECRET-1234' for rule 'custom_secret'".to_string(),
        // Updated to match the new centralized logging format for redaction action
        "[DEBUG cleansh::utils::redaction] [cleansh::tools::sanitize_shell] Redaction action: Original='MYSECRET-1234', Redacted='[CUSTOM_SECRET_REDACTED]' for rule 'custom_secret'".to_string(),
        // These logs are now from `cleansh.rs` using `log_redaction_match_debug`
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='email', Original='user@test.org', Sanitized='[ORG_EMAIL_REDACTED]'".to_string(),
        "[DEBUG cleansh::utils::redaction] [cleansh::commands::cleansh] Found RedactionMatch: Rule='custom_secret', Original='MYSECRET-1234', Sanitized='[CUSTOM_SECRET_REDACTED]'".to_string(),
        // Assert on the final state
        "[DEBUG cleansh::commands::cleansh] Content sanitized. Original length: 86, Sanitized length: 104".to_string(),
        "[DEBUG cleansh::commands::cleansh] DEBUG_CLEANSH: Redaction summary (num items): 2".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Outputting to stdout.".to_string(),
        "[DEBUG cleansh::commands::cleansh] [cleansh.rs] Cleansh operation completed.".to_string(),
    ];

    let assert_result = run_cleansh_command(input, &["--config", path, "--no-clipboard"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);

    for msg in expected_stderr_contains {
        assert!(stderr.contains(&msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    // Specific assertion for the dynamic path in the INFO log, ensuring it's present.
    assert!(stderr.contains(&format!("Loading custom rules from: {}", path)));

    Ok(())
}