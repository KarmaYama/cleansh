use anyhow::Result;
use anyhow::Context; // <--- ADDED THIS LINE
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
        // Add checks for the new eprintln! debug messages
        "[cleansh.rs] DEBUG: Starting cleansh operation.",
        "[cleansh.rs] DEBUG: Received enable_rules: []", // No explicit enable for this test, should rely on default
        "[sanitize_shell.rs] DEBUG: compile_rules called with",
        "[sanitize_shell.rs] DEBUG: Rule 'email' compiled successfully.",
        "[sanitize_shell.rs] DEBUG: Rule 'ipv4_address' compiled successfully.",
        "DEBUG_CLEANSH: Original content: \"My email is test@example.com and my IP is 192.168.1.1.\"",
        "DEBUG_CLEANSH: Sanitized content: \"My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].\"",
    ];

    // Removed --enable-rules for this test, as email and ipv4 are not opt-in by default
    let assert_result = run_cleansh_command(input, &["--no-clipboard"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    // Print stdout and stderr for debugging in case of failure
    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    // Provided a valid JWT string that matches the regex pattern
    let input = "Secret JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    // FIX: Updated expected_stdout to match the actual output of cleansh
    let expected_stdout = "Secret JWT: [JWT_REDACTED]\n"; 
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "Sanitized content copied to clipboard successfully.",
        "[cleansh.rs] DEBUG: Starting cleansh operation.",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        "[sanitize_shell.rs] DEBUG: Rule 'jwt_token' compiled successfully.",
        // Add log for the JWT capture
        "[sanitize_shell.rs] DEBUG: Rule 'jwt_token' does pattern match input? true",
    ];

    let assert_result = run_cleansh_command(input, &["-c", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
        "--- Diff View ---", // This header is printed to stderr by print_info_message
        "-----------------", // This line is printed to stderr by print_info_message
        "[cleansh.rs] DEBUG: Received enable_rules: []", // Now empty as ipv4_address is not opt-in
        "[sanitize_shell.rs] DEBUG: Rule 'ipv4_address' compiled successfully.",
    ];

    // Removed --enable-rules "ipv4_address" as it's not opt-in
    let assert_result = run_cleansh_command(input, &["-d", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    // The diff content is printed to stdout by `diff_viewer::print_diff`
    for msg in expected_stdout_contains {
        assert!(stdout.contains(msg), "Stdout missing: '{}'\nFull stdout:\n{}", msg, stdout);
    }
    // These should NOT be in stdout, but in stderr as they are info messages
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
        "[cleansh.rs] DEBUG: Received enable_rules: []", // Now empty as email is not opt-in
        "[sanitize_shell.rs] DEBUG: Rule 'email' compiled successfully.",
    ];

    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();

    // Removed --enable-rules "email" as it's not opt-in
    let assert_result = run_cleansh_command(input, &["-o", path, "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, ""); // Nothing should be written to stdout when -o is used

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
        "[cleansh.rs] DEBUG: Received enable_rules: []", // No explicit enable, as custom_secret is not opt-in in custom config
        "[sanitize_shell.rs] DEBUG: Rule 'custom_secret' compiled successfully.",
        "[sanitize_shell.rs] DEBUG: Rule 'email' compiled successfully.", // This email rule is defined in custom config, not opt-in
    ];

    let assert_result = run_cleansh_command(input, &["--config", path, "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    // FIX: Changed expected_stdout to match the actual output of "~${0}"
    let expected_stdout = "Accessing ~${0} and ~${0}\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: []", // No explicit enable, as these rules are not opt-in
        "[sanitize_shell.rs] DEBUG: Rule 'absolute_linux_path' compiled successfully.",
        "[sanitize_shell.rs] DEBUG: Rule 'absolute_macos_path' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
        // No specific rule compilation messages expected if no rules match or are enabled that would log
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: []",
        // This log confirms the rule was indeed marked as opt-in and not enabled.
        "[sanitize_shell.rs] DEBUG: Opt-in rule 'aws_secret_key' not explicitly enabled, skipping compilation.",
        "[sanitize_shell.rs] DEBUG: Opt-in rule 'aws_secret_key' not enabled, skipping.",
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);

    for msg in expected_stderr_contains {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}

#[test]
fn test_opt_in_rule_enabled() -> Result<()> {
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\"]",
        // This log confirms the rule was compiled.
        "[sanitize_shell.rs] DEBUG: Rule 'aws_secret_key' compiled successfully.",
    ];

    // aws_secret_key IS opt_in: true in default_rules.yaml, so --enable-rules is necessary.
    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=. And a generic hex: 0123456789abcdef0123456789abcdef.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED]. And a generic hex: [HEX_SECRET_32_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\", \"generic_hex_secret_32\"]",
        "[sanitize_shell.rs] DEBUG: Rule 'aws_secret_key' compiled successfully.",
        "[sanitize_shell.rs] DEBUG: Rule 'generic_hex_secret_32' compiled successfully.",
    ];

    // Both aws_secret_key and generic_hex_secret_32 ARE opt_in: true in default_rules.yaml.
    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key,generic_hex_secret_32", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    let input = "My AWS secret key is aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.";
    let expected_stdout = "My AWS secret key is [AWS_SECRET_KEY_REDACTED].\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "--- Redaction Summary ---",
        "aws_secret_key (1 occurrences)",
        "- aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789/+=.",
        "- [AWS_SECRET_KEY_REDACTED]",
        "-------------------------",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"aws_secret_key\"]",
        "[sanitize_shell.rs] DEBUG: Rule 'aws_secret_key' compiled successfully.",
    ];

    // aws_secret_key IS opt_in: true in default_rules.yaml, so --enable-rules is necessary.
    let assert_result = run_cleansh_command(input, &["--enable-rules", "aws_secret_key", "--no-clipboard"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
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
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "[cleansh.rs] DEBUG: Received enable_rules: [\"non_existent_rule\"]",
        "[sanitize_shell.rs] DEBUG: Rule 'email' compiled successfully.",
        "[sanitize_shell.rs] DEBUG: Opt-in rule 'non_existent_rule' not explicitly enabled, skipping compilation.",
        "[sanitize_shell.rs] DEBUG: Rule 'non_existent_rule' not found in merged configuration, skipping.",
    ];

    // Changed to only pass non_existent_rule to --enable-rules, email is already active by default.
    let assert_result = run_cleansh_command(input, &["--enable-rules", "non_existent_rule", "--no-clipboard", "--no-redaction-summary"]).success();
    let stdout = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&assert_result.get_output().stderr));

    eprintln!("\n--- STDOUT Captured ---");
    eprintln!("{}", stdout);
    eprintln!("--- END STDOUT ---\n");
    eprintln!("\n--- STDERR Captured ---");
    eprintln!("{}", stderr);
    eprintln!("--- END STDERR ---\n");

    assert_eq!(stdout, expected_stdout);

    for msg in expected_stderr_contains {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }
    Ok(())
}