//! Integration tests for the --line-buffered mode of Cleansh.
//!
//! These tests focus on verifying the real-time, line-buffered input/output
//! behavior, including interactions with stdin, stdout, and various
//! redaction scenarios.

// No more assert_cmd::Command as we are consistently using StdCmd
use predicates::prelude::*; // Provides .contains() and .not() on predicates
use std::process::{Command as StdCmd, Stdio}; // Use StdCmd for all tests requiring process interaction
use std::io::Write; // For writing to stdin manually
use tempfile::tempdir; // For creating temporary directories/files
use std::fs;
use std::path::PathBuf; // Import PathBuf for the create_test_config return type
use log::error; // Import log for debugging within the test suite
use strip_ansi_escapes::strip; // Added to strip ANSI escape codes for cleaner output assertions

/// Strip ANSI escapes *and* (if present) drop the first “Using line-buffered…” banner line.
fn clean_stdout(raw: &[u8]) -> String {
    let s = String::from_utf8_lossy(&strip(raw)).to_string();
    let mut lines: Vec<&str> = s.lines().collect();
    if let Some(first) = lines.first() {
        if first.contains("Using line-buffered mode.") {
            lines.remove(0);
        }
    }
    // always ensure trailing newline
    lines.join("\n") + "\n"
}

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
    // Use the run_cleansh_with_stdin helper for consistency and proper I/O handling
    let input = "This is an IP: 192.168.1.100\nAnother secret: SECRET_KEY=abc123def\nNo secret here.\n"; // Ensure input ends with newline
    let output_debug = run_cleansh_with_stdin(
        input,
        Some(&config_path),
        &[], // No additional args like --quiet for this case
    )?;

    if !output_debug.status.success() {
        error!("Cleansh process exited with non-success status: {:?}", output_debug.status);
        error!("Cleansh stdout (from failed run):\n{}", String::from_utf8_lossy(&output_debug.stdout));
        error!("Cleansh stderr (from failed run):\n{}", String::from_utf8_lossy(&output_debug.stderr));
    }

    assert!(output_debug.status.success(), "Cleansh process failed. Stderr: {}", String::from_utf8_lossy(&output_debug.stderr));
    
    // Add debug prints to inspect the captured stdout and stderr
    println!("DEBUG: Cleansh stdout:\n{}", String::from_utf8_lossy(&output_debug.stdout));
    println!("DEBUG: Cleansh stderr:\n{}", String::from_utf8_lossy(&output_debug.stderr));

    // The program should now print the warning to stderr and the sanitized content to stdout.
    assert_eq!(
        clean_stdout(&output_debug.stdout),
        "This is an IP: [IPV4_REDACTED]\nAnother secret: SECRET_KEY=[REDACTED]\nNo secret here.\n"
    );
    // Add a new assertion to check for the warning on stderr.
    let stderr_str = String::from_utf8_lossy(&output_debug.stderr);
    assert!(stderr_str.contains("Using line-buffered mode. Incompatible with --diff, --clipboard, and --input-file."));

    // Test case 2: --quiet should strip both banner & summary entirely.
    let output_quiet = run_cleansh_with_stdin(
        "This is an IP: 192.168.1.100\n",
        Some(&config_path),
        &["--quiet"], // --quiet is active here
    )?;

    assert!(output_quiet.status.success());
    // Assert stdout for the quiet case
    assert_eq!(
        clean_stdout(&output_quiet.stdout),
        "This is an IP: [IPV4_REDACTED]\n" // Expect the redaction and a single newline
    );
    let stderr_str_quiet = String::from_utf8_lossy(&output_quiet.stderr);
    // When --quiet is used, the summary should NOT be present.
    assert!(predicates::str::contains("Redaction Summary").not().eval(&stderr_str_quiet));
    assert!(predicates::str::contains("test_ip_address (1 occurrences)").not().eval(&stderr_str_quiet));
    // Since redactions *were* applied (assuming the app fix), and summary is suppressed, stderr should be empty.
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
        clean_stdout(&output.stdout),
        "Just a normal line\nAnother normal line\n" // Adjusted for single newlines
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
    // empty stdin + --quiet: no stdout, but we still get the “no redactions” summary.
    assert!(output.stdout.is_empty()); // Expect empty stdout
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
    // Should end with exactly one `\n`
    assert_eq!(clean_stdout(&output.stdout), "Last line with [IPV4_REDACTED] but no newline\n");
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty()); // In quiet mode with match, summary should be suppressed.

    Ok(())
}

#[test]
fn test_line_buffered_with_multiple_writes_to_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = create_test_config(&dir);

    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    // FIX: Assign the result of the spawn call to a new child variable.
    let mut child = StdCmd::new(exe)
        .arg("--line-buffered")
        .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
        .arg("--quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin for child process");

    stdin.write_all(b"First line 1.1.1.1\n")?;
    stdin.flush()?;

    stdin.write_all(b"Second line SECRET_KEY=xyz\n")?;
    stdin.flush()?;

    drop(stdin);

    let output = child.wait_with_output()?;
    assert!(output.status.success());

    let sanitized_stdout = clean_stdout(&output.stdout);
    // Confirmed single newlines
    let expected_stdout = "First line [IPV4_REDACTED]\nSecond line SECRET_KEY=[REDACTED]\n";
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
    // FIX: Assertion updated to match the single, comprehensive error message from the program's output.
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --diff, --clipboard, and --input-file."));
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
        // FIX: Assertion updated to match the single, comprehensive error message from the program's output.
        assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --diff, --clipboard, and --input-file."));
        assert!(String::from_utf8_lossy(&output.stdout).is_empty(), "Unexpected stdout output: {}", String::from_utf8_lossy(&output.stdout));
    }
    // This `Ok(())` is important even if clipboard feature is not enabled,
    // so the test passes on systems without clipboard feature.
    Ok(())
}


#[test]
fn test_line_buffered_with_out_flag_warns() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let output_file = dir.path().join("output.txt");
    let config_path = create_test_config(&dir);

    let exe = assert_cmd::cargo::cargo_bin("cleansh");
    let mut cmd = StdCmd::new(exe);
    cmd.arg("--line-buffered")
        .arg("--output").arg(output_file.to_str().expect("Failed to convert output_file to string")) // Changed --out to --output
        .arg("--config").arg(config_path.to_str().expect("Failed to convert config_path to string"))
        .arg("--quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    write!(stdin, "Line with 10.0.0.1\n")?;
    stdin.flush()?;

    drop(stdin);

    let output = child.wait_with_output()?;

    assert!(output.status.success());
    // Warnings are still printed even with --quiet as they are considered important
    // Updated the warning message to match the actual output from src/main.rs
    assert!(String::from_utf8_lossy(&output.stderr).contains("Warning: --line-buffered is intended for real-time console output. \
                                   Outputting to a file (--output) will still buffer by line, \
                                   but real-time benefits might be less apparent."));

    // Verify content written to file
    let file_content = fs::read_to_string(&output_file)?;
    // Confirmed single newline
    assert_eq!(file_content, "Line with [IPV4_REDACTED]\n");

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

    // We now expect the command to *fail* due to the incompatibility check.
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error: --line-buffered is incompatible with --diff, --clipboard, and --input-file."));
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
    // confirmed single newline
    assert_eq!(clean_stdout(&output.stdout), "Test with [IPV4_REDACTED] and no summary.\n");
    assert!(predicates::str::contains("Redaction Summary")
        .not()
        .eval(&String::from_utf8_lossy(&output.stderr)));

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
    // Confirmed single newlines
    assert_eq!(
        clean_stdout(&output.stdout),
        "Sensitive data: [IPV4_REDACTED] and SECRET_KEY=[REDACTED]\n"
    );
    // In quiet mode, summary should be suppressed
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty());

    Ok(())
}