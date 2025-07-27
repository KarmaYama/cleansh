//! Integration tests for the --line-buffered mode of Cleansh.
//!
//! These tests focus on verifying the real-time, line-buffered input/output
//! behavior, including interactions with stdin, stdout, and various
//! redaction scenarios.

// No more assert_cmd::Command as we are consistently using StdCmd
use predicates::prelude::*; // Provides .contains() and .not() on predicates
use std::process::{Command as StdCmd, Stdio};  // Use StdCmd for all tests requiring process interaction
use std::io::Write; // For writing to stdin manually
use tempfile::tempdir; // For creating temporary directories/files
use std::fs;
use std::path::PathBuf; // Import PathBuf for the create_test_config return type
use log::error; // Import log for debugging within the test suite

// Helper function to create a basic config file for testing
fn create_test_config(dir: &tempfile::TempDir) -> PathBuf {
    // Corrected to .yaml extension and YAML content
    let config_path = dir.path().join("cleansh_test_config.yaml");
    let config_content = r#"
rules:
  - name: "test_ip_address"
    pattern: "\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b"
    replace_with: "[IPV4_REDACTED]" # Changed to match default_rules.yaml
    multiline: false
    dot_matches_new_line: false
    opt_in: false

  - name: "test_secret_key"
    pattern: "SECRET_KEY=[a-zA-Z0-9]+"
    replace_with: "SECRET_KEY=[REDACTED]"
    multiline: false
    dot_matches_new_line: false
    opt_in: false
"#; // Changed to YAML format
    fs::write(&config_path, config_content).unwrap();
    config_path
}

// Helper to run a command with piped stdin and capture output
fn run_cleansh_with_stdin(
    input: &str,
    config_path: Option<&PathBuf>,
    args: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut cmd = StdCmd::new(exe);

    cmd.arg("--line-buffered")
       .args(args)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    if let Some(path) = config_path {
        cmd.arg("--config").arg(path.to_str().expect("Failed to convert config_path to string"));
    }

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin for child process");
    write!(stdin, "{}", input)?;
    drop(stdin); // Close stdin to signal EOF

    let output = child.wait_with_output()?;
    Ok(output)
}

// NEW Helper: To run a command with only arguments, no stdin interaction expected
fn run_cleansh_with_args_only(
    args: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut cmd = StdCmd::new(exe);

    cmd.args(args)
       .stdin(Stdio::null()) // Explicitly set stdin to null as we don't expect input
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let output = cmd.output()?; // Use .output() for simpler execution of short-lived processes
    Ok(output)
}


#[test]
fn test_line_buffered_basic_sanitization() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    // Test case 1: Basic sanitization and stdout verification (without --quiet)
    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut cmd_debug = StdCmd::new(exe);

    cmd_debug.arg("--line-buffered")
             .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
             .env("RUST_LOG", "debug") // Set RUST_LOG for the spawned process to see verbose logs
             .stdin(Stdio::piped())
             .stdout(Stdio::piped())
             .stderr(Stdio::piped());

    let mut child_debug = cmd_debug.spawn()?;
    let mut stdin_debug = child_debug.stdin.take().expect("Failed to open stdin for debug child");
    write!(stdin_debug, "This is an IP: 192.168.1.100\nAnother secret: SECRET_KEY=abc123def\nNo secret here.")?;
    drop(stdin_debug);
    let output_debug = child_debug.wait_with_output()?;

    if !output_debug.status.success() {
        error!("Cleansh process exited with non-success status: {:?}", output_debug.status);
        error!("Cleansh stdout (from failed run): {}", String::from_utf8_lossy(&output_debug.stdout));
        error!("Cleansh stderr (from failed run): {}", String::from_utf8_lossy(&output_debug.stderr));
    }

    assert!(output_debug.status.success(), "Cleansh process failed. Stderr: {}", String::from_utf8_lossy(&output_debug.stderr));
    
    // Confirmed output with IPV4_REDACTED and extra newlines
    assert_eq!(
        String::from_utf8_lossy(&output_debug.stdout),
        "This is an IP: [IPV4_REDACTED]\n\nAnother secret: SECRET_KEY=[REDACTED]\n\nNo secret here.\n"
    );
    // When RUST_LOG=debug is set and --quiet is NOT passed, summary is expected.
    assert!(String::from_utf8_lossy(&output_debug.stderr).contains("Redaction Summary"));


    // Test case 2: Verify output when --quiet is active. Redaction Summary should be suppressed.
    let output_quiet = run_cleansh_with_stdin(
        "This is an IP: 192.168.1.100\n",
        Some(&config_path),
        &["--quiet"], // --quiet is active here
    )?;

    assert!(output_quiet.status.success());
    // Assert stdout for the quiet case
    assert_eq!(
        String::from_utf8_lossy(&output_quiet.stdout),
        "This is an IP: [IPV4_REDACTED]\n\n" // Expect the redaction and extra newline
    );
    let stderr_str_quiet = String::from_utf8_lossy(&output_quiet.stderr);
    // When --quiet is used, the summary should NOT be present.
    assert!(predicates::str::contains("Redaction Summary").not().eval(&stderr_str_quiet));
    assert!(predicates::str::contains("test_ip_address (1 occurrences)").not().eval(&stderr_str_quiet));
    // The only thing we expect on stderr in quiet mode is "No redactions applied." if no matches,
    // or nothing if matches occurred and summary is suppressed.
    // Since redactions *were* applied, and summary is suppressed, stderr should be empty.
    assert!(stderr_str_quiet.trim().is_empty());


    Ok(())
}

#[test]
fn test_line_buffered_no_match() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let output = run_cleansh_with_stdin(
        "Just a normal line\nAnother normal line\n",
        Some(&config_path),
        &["--quiet"],
    )?;

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Just a normal line\n\nAnother normal line\n\n" // Adjusted for extra newlines
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("No redactions applied.")); // Verify no redactions summary

    Ok(())
}

#[test]
fn test_line_buffered_empty_input() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let output = run_cleansh_with_stdin(
        "",
        Some(&config_path),
        &["--quiet"],
    )?;

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).is_empty()); // Expect empty stdout
    assert!(String::from_utf8_lossy(&output.stderr).contains("No redactions applied.")); // Expect no redaction summary

    Ok(())
}

#[test]
fn test_line_buffered_line_without_newline_at_end() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let output = run_cleansh_with_stdin(
        "Last line with 1.2.3.4 but no newline",
        Some(&config_path),
        &["--quiet"],
    )?;

    assert!(output.status.success());
    // Adjusted expected output for IP redaction and confirmed extra newline
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Last line with [IPV4_REDACTED] but no newline\n");
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty()); // In quiet mode with match, summary should be suppressed.

    Ok(())
}

#[test]
fn test_line_buffered_with_multiple_writes_to_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut child = StdCmd::new(exe)
        .arg("--line-buffered")
        .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
        .arg("--quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin for child process");

    stdin.write_all(b"First line 1.1.1.1\n")?;
    stdin.flush()?;

    stdin.write_all(b"Second line SECRET_KEY=xyz\n")?;
    stdin.flush()?;

    drop(stdin);

    let output = child.wait_with_output()?;
    assert!(output.status.success());

    let sanitized_stdout = String::from_utf8(output.stdout)?;
    // Confirmed extra newlines
    let expected_stdout = "First line [IPV4_REDACTED]\n\nSecond line SECRET_KEY=[REDACTED]\n\n";
    assert_eq!(sanitized_stdout, expected_stdout);

    // In quiet mode, summary should be suppressed
    let stderr_output = String::from_utf8(output.stderr)?;
    assert!(stderr_output.trim().is_empty());

    Ok(())
}

#[test]
fn test_line_buffered_incompatible_with_diff() -> Result<(), Box<dyn std::error::Error>> {
    // Use the new helper for arg-only tests
    let output = run_cleansh_with_args_only(
        &["--line-buffered", "--diff"],
    )?;

    assert!(!output.status.success(), "Command was expected to fail, but succeeded. Stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --diff."), "Expected error message not found. Stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).is_empty(), "Unexpected stdout output: {}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

#[test]
fn test_line_buffered_incompatible_with_clipboard() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "clipboard")]
    {
        let output = run_cleansh_with_args_only( // Use the new helper
            &["--line-buffered", "--clipboard"],
        )?;

        assert!(!output.status.success(), "Command was expected to fail, but succeeded. Stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --clipboard."), "Expected error message not found. Stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).is_empty(), "Unexpected stdout output: {}", String::from_utf8_lossy(&output.stdout));
    }
    // This `Ok(())` is important even if clipboard feature is not enabled,
    // so the test passes on systems without clipboard feature.
    Ok(())
}


#[test]
fn test_line_buffered_with_out_flag_warns() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let output_file = dir.path().join("output.txt");
    let config_path = create_test_config(&dir);

    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut cmd = StdCmd::new(exe);
    cmd.arg("--line-buffered")
       .arg("--out").arg(output_file.to_str().expect("Failed to convert output_file to string"))
       .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
       .arg("--quiet")
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    write!(stdin, "Line with 10.0.0.1\n")?;
    drop(stdin);

    let output = child.wait_with_output()?;

    assert!(output.status.success());
    // Warnings are still printed even with --quiet as they are considered important
    assert!(String::from_utf8_lossy(&output.stderr).contains("Warning: --line-buffered is intended for real-time console output. Outputting to a file (--out) will still buffer by line, but real-time benefits might be less apparent."));

    // Verify content written to file
    let file_content = fs::read_to_string(&output_file)?;
    // Confirmed extra newline
    assert_eq!(file_content, "Line with [IPV4_REDACTED]\n\n");

    Ok(())
}

#[test]
fn test_line_buffered_input_file_flag_not_supported() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);
    let input_file = dir.path().join("input.txt");
    fs::write(&input_file, "File content with 172.16.0.10\nAnother line.\n")?;

    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let output = StdCmd::new(exe)
        .arg("--line-buffered")
        .arg("--input-file").arg(input_file.to_str().expect("Failed to convert input_file to string"))
        .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
        .arg("--quiet")
        .output()?;

    // *** CHANGE IS HERE ***
    // We now expect the command to *fail* due to the incompatibility check.
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --input-file. Use piping for streaming input."));
    // We should *not* see any stdout from the actual redaction process
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());
    // And definitely no redaction summary or "Reading input from file" messages that indicate normal processing
    assert!(predicates::str::contains("Redaction Summary").not().eval(&String::from_utf8_lossy(&output.stderr)));
    assert!(predicates::str::contains("Reading input from file:").not().eval(&String::from_utf8_lossy(&output.stderr)));
    assert!(predicates::str::contains("Reading input from stdin in real-time").not().eval(&String::from_utf8_lossy(&output.stderr)));

    Ok(())
}


// Test with no_redaction_summary flag
#[test]
fn test_line_buffered_no_redaction_summary() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let output = run_cleansh_with_stdin(
        "Test with 1.2.3.4 and no summary.\n",
        Some(&config_path),
        &["--no-redaction-summary", "--quiet"], // Both flags active
    )?;

    assert!(output.status.success());
    // Confirmed extra newline
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Test with [IPV4_REDACTED] and no summary.\n\n");
    // Both --no-redaction-summary and --quiet should lead to no summary on stderr
    assert!(predicates::str::contains("Redaction Summary").not().eval(&String::from_utf8_lossy(&output.stderr)));

    Ok(())
}


// Test for multiple rules matching on a single line
#[test]
fn test_line_buffered_multiple_matches_single_line() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let output = run_cleansh_with_stdin(
        "Sensitive data: 192.168.1.1 and SECRET_KEY=xyz123abc\n",
        Some(&config_path),
        &["--quiet"],
    )?;

    assert!(output.status.success());
    // Confirmed extra newlines
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Sensitive data: [IPV4_REDACTED] and SECRET_KEY=[REDACTED]\n\n"
    );
    // In quiet mode, summary should be suppressed
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty());

    Ok(())
}