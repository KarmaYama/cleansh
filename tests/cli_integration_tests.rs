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
    let expected_stdout = "My email is [EMAIL_REDACTED] and my IP is [IPV4_REDACTED].\n";
    let expected_stderr_contains_substrings = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "--- Redaction Summary ---",
        "email (1 occurrences)",
        "ipv4_address (1 occurrences)",
        "-------------------------",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Starting cleansh operation.",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.",
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

    // Check for substrings as before
    for msg in expected_stderr_contains_substrings {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }

    // Updated assertion for input content log
    assert!(
        stderr.contains("Sanitize called. Input content length: 54"),
        "Stderr missing the expected 'Sanitize called. Input content length' log.\nFull stderr:\n{}", stderr
    );

    // Assert that redacted messages are present because CLEANSH_ALLOW_DEBUG_PII is true
    assert!(
        stderr.contains("Rule 'email' captured match (original): [REDACTED: 16 chars]"),
        "Stderr missing expected redacted capture log for email.\nFull stderr:\n{}", stderr
    );
    // Corrected expected string for email redaction log
    assert!(
        stderr.contains("Redacting '[REDACTED: 16 chars]' with '[REDACTED: 16 chars]' for rule 'email'"),
        "Stderr missing expected redacted redaction log for email.\nFull stderr:\n{}", stderr
    );
    assert!(
        stderr.contains("Rule 'ipv4_address' captured match (original): [REDACTED: 11 chars]"),
        "Stderr missing expected redacted capture log for IP.\nFull stderr:\n{}", stderr
    );
    // Corrected expected string for ipv4_address redaction log
    assert!(
        stderr.contains("Redacting '[REDACTED: 11 chars]' with '[REDACTED: 15 chars]' for rule 'ipv4_address'"),
        "Stderr missing expected redacted redaction log for IP.\nFull stderr:\n{}", stderr
    );

    // The 'Sanitized content' log from cleansh.rs
    assert!(
        stderr.contains("Content sanitized. Original length: 54, Sanitized length: 58"),
        "Stderr missing 'Content sanitized' log.\nFull stderr:\n{}", stderr
    );
    // The 'Redaction summary (num items)' log from cleansh.rs
    assert!(
        stderr.contains("DEBUG_CLEANSH: Redaction summary (num items): 2"),
        "Stderr missing 'DEBUG_CLEANSH: Redaction summary' log.\nFull stderr:\n{}", stderr
    );

    Ok(())
}

// Renamed from `test_clipboard_output` based on the log, or this is a new test.
// Apply the CI skip logic here.
#[cfg(feature = "clipboard")]
#[test]
fn test_run_cleansh_clipboard_copy() -> Result<()> {
    // Skip in CI (no GUI / no X11 clipboard)
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (no display)");
        return Ok(());
    }

    let input = "My email is test@example.com";
    let expected_stdout = "My email is [EMAIL_REDACTED]\n"; // Expected output based on your previous logs
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to file:", // This implies output to file
        "Sanitized content copied to clipboard successfully.",
        // Add other expected logs if necessary
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
        "-c", // Copy to clipboard
        "-o", output_path, // Output to file
        "--config", config_path, // Custom config
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

    assert_eq!(stdout, ""); // When outputting to file, stdout should be empty

    for msg in expected_stderr_contains {
        assert!(stderr.contains(msg), "Stderr missing: '{}'\nFull stderr:\n{}", msg, stderr);
    }

    let file_contents = fs::read_to_string(output_path)?;
    assert_eq!(file_contents, expected_stdout);

    // This part tries to read the clipboard, which will fail in CI even if the copy "succeeds" internally.
    // If you need to verify clipboard *content* locally, you'd do it here.
    // However, the test's purpose seems to be primarily about *invoking* the clipboard logic.
    // Since we are skipping in CI, this part is safe.

    Ok(())
}


#[test]
fn test_clipboard_output() -> Result<()> {
    // Skip in CI (no GUI / no X11 clipboard)
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping clipboard test in CI (no display)"); 
        return Ok(());
    }

    let input = "Secret JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let expected_stdout = "Secret JWT: [JWT_REDACTED]\n";
    let expected_stderr_contains = vec![
        "Reading input from stdin...",
        "Writing sanitized content to stdout.",
        "Sanitized content copied to clipboard successfully.",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Starting cleansh operation.",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.",
        // This log now comes from pii_debug! and contains redacted info
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'jwt_token' does pattern match input? true",
    ];

    // REMOVED THE FOLLOWING LINE:
    // let _clipboard = arboard::Clipboard::new()
    //     .with_context(|| "Failed to get clipboard")?;


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
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
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
        "--- Redaction Summary ---", // The summary will be printed to stderr even if output is to file.
        "email (1 occurrences)",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
    ];

    let file = NamedTempFile::new()?;
    let path = file.path().to_str().unwrap();

    let assert_result = run_cleansh_command(input, &["-o", path, "--no-clipboard"]).success(); // Removed --no-redaction-summary
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
        "Writing sanitized content to stdout.",
        "--- Redaction Summary ---", // Summary should still appear here
        "custom_secret (1 occurrences)",
        "email (1 occurrences)",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'custom_secret' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--config", path, "--no-clipboard"]).success(); // Removed --no-redaction-summary
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
        "--- Redaction Summary ---", // Summary should still appear here
        "absolute_linux_path (1 occurrences)",
        "absolute_macos_path (1 occurrences)",
        "[DEBUG] [cleansh::commands::cleansh] [cleansh.rs] Received enable_rules: []",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled successfully.",
        "[DEBUG] [cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled successfully.",
    ];

    let assert_result = run_cleansh_command(input, &["--no-clipboard"]).success(); // Removed --no-redaction-summary
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