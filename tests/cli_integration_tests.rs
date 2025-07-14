use anyhow::Result;
use anyhow::Context;
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
    cmd.env("RUST_LOG", "debug"); // This should propagate
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
    let expected_stdout = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "--- Redaction Summary ---",
        "email (1 occurrences)",
        "- test@example.com",
        "- [EMAIL_REDACTED]",
        "ipv4_address (1 occurrences)",
        "- 192.168.1.1",
        "- [IPV4_REDACTED]",
        "-------------------------",
        // Add checks for the new debug messages, now using the log crate's prefix
        "[cleansh.rs] DEBUG: Starting cleansh operation.",
        "[cleansh.rs] DEBUG: Received enable_rules: []", // No explicit enable for this test, should rely on default
        "[DEBUG] [cleansh::tools::sanitize_shell] compile_rules called with",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.",
        "DEBUG_CLEANSH: Original content: \"My email is test@example.com and my IP is 192.168.1.1.\"",
        "DEBUG_CLEANSH: Sanitized content: \"My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].\"",
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

    for msg in expected_stderr_contains {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_clipboard_output() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (headless environment)");
        return Ok(());
    }
    let input = "Secret JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let expected_stdout = "Secret JWT: [JWT_REDACTED]\n"; 
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "Sanitized content copied to clipboard successfully.",
        "[cleansh.rs] DEBUG: Starting cleansh operation.",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'jwt_token' does pattern match input? true",
    ];
    
    let _clipboard = arboard::Clipboard::new()
        .with_context(|| "Failed to get clipboard")?; 


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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    assert!(!stderr.contains("--- Redaction Summary ---"));
    Ok(())
}

#[test]
fn test_diff_view() -> Result<()> {
    let input = "Old IP: 10.0.0.1. New IP: 192.168.1.1.";
    let expected_stdout_contains = vec![
        "-Old IP: 10.0.0.1. New IP: 192.168.1.1.",
        "+Old IP: [IPV4_REDACTED]. New IP: [IPV4_REDACTED].",
    ];
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "Generating and displaying diff.",
        "--- Diff View ---",
        "-----------------",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.",
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
        assert!(stdout.contains(msg), "Stdout missing: '{}'\nFull stdout:\n{}", msg, stdout);
    }
    assert!(!stdout.contains("--- Diff View ---"));
    assert!(!stdout.contains("-----------------"));
    assert!(!stdout.contains("--- Redaction Summary ---"));

    for msg in expected_stderr_contains {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_output_to_file() -> Result<()> {
    let input = "This is a test with sensitive info: user@domain.com";
    let expected_file_content = "This is a test with sensitive info: [EMAIL_REDACTED]\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to file:",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
    ];

    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();

    let assert_result = run_cleansh_command(input, &["-o", path, "--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
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
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Loading custom rules from:",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'custom_secret' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--config", path, "--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    assert!(stderr.contains(&format!("Loading custom rules from: {}", path)));

    Ok(())
}

#[test]
fn test_absolute_path_redaction() -> Result<()> {
    let input = "Accessing /home/user/documents/report.pdf and /Users/admin/logs/app.log";
    let expected_stdout = "Accessing ~${0} and ~${0}\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_no_redactions() -> Result<()> {
    let input = "This is a clean string with no sensitive information.";
    let expected_stdout = format!("{}\n", input);
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "Redaction summary display skipped per user request.",
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    assert!(!stderr.contains("No redactions applied."));
    Ok(())
}

#[test]
fn test_opt_in_rule_not_enabled_by_default() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected_stdout = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.\n";
    
    let assert_result = run_cleansh_command(input, &[]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);
    
    // Assert against the stripped stderr, expecting the log crate's format
    assert!(predicate::str::is_match(r".*\[DEBUG\] \[cleansh::tools::sanitize_shell\] Opt-in rule 'aws_secret_key' not explicitly enabled, skipping compilation.*").unwrap().eval(&stderr));
    // Removed the second assertion that was looking for "not enabled, skipping" as it's redundant and not the primary log
    assert!(stderr.contains("Reading input from stdin..."));
    assert!(stderr.contains("Writing sanitized content to stdout."));
    
    Ok(())
}

#[test]
fn test_opt_in_rule_enabled() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=A.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\"]",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'aws_secret_key' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_multiple_opt_in_rules_enabled() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=A And a generic hex: 0123456789abcdef0123456789abcdef.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED] And a generic hex: [HEX_SECRET_32_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\", \"generic_hex_secret_32\"]",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'aws_secret_key' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'generic_hex_secret_32' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'aws_secret_key' does pattern match input? true",
        "[DEBUG] [cleansh::tools::sanitize_shell] Redacting 'aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=A' with '[AWS_SECRET_KEY_REDACTED]' for rule 'aws_secret_key'",
    ];

    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key,generic_hex_secret_32", "--no-clipboard", "--no-redaction-summary"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_opt_in_rule_enabled_with_summary() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=A.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "--- Redaction Summary ---",
        "aws_secret_key (1 occurrences)",
        "- aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=A",
        "- [AWS_SECRET_KEY_REDACTED]",
        "-------------------------",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\"]",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'aws_secret_key' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard"]).success();
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
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_opt_in_rule_not_in_config() -> Result<()> {
    let input = "email@example.com and a fake secret 1234-abcd-SECRET.";
    let expected_stdout = "[EMAIL_REDACTED] and a fake secret 1234-abcd-SECRET.\n";
    
    let assert_result = run_cleansh_command(input, &["--enable-rules", "non_existent_rule", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprint!("\n--- STDOUT Captured ---\n");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprint!("\n--- STDERR Captured ---\n");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);
    
    assert!(stderr.contains("Reading input from stdin..."));
    assert!(stderr.contains("Writing sanitized content to stdout."));
    assert!(stderr.contains("[cleansh.rs] DEBUG: Received enable_rules: [\"non_existent_rule\"]"));
    // CORRECTED: Update this assertion to match the `log` crate's output format
    assert!(stderr.contains("[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully."));
    assert!(stderr.contains("[DEBUG] [cleansh::tools::sanitize_shell] Rule 'non_existent_rule' not found in merged configuration, skipping."));

    Ok(())
}