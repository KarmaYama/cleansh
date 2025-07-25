// tests/full_stats_tests.rs
// Comprehensive integration tests for the `cleansh --stats-only` command.

use cleansh::test_exposed::utils::AppState; // Access AppState for direct manipulation in tests
use std::fs;
use std::path::PathBuf;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use std::env; // For setting/removing env vars in unsafe blocks
use log::{debug, LevelFilter}; // Import debug and LevelFilter
use cleansh::logger; // Import your logger initialization function

// --- Custom Predicate for JSON Validation ---
// This function creates a custom predicate that checks if a string is valid JSON.
fn is_json() -> impl Predicate<str> {
    predicate::function(|s: &str| {
        // Attempt to parse the string as JSON. If successful, it's valid JSON.
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    })
}
// --- End Custom Predicate ---

// --- Helper Functions and Structures for Tests ---

/// Manages temporary directories and paths for each test, ensuring isolation.
struct TestPaths {
    _temp_dir: TempDir, // Held to ensure temp_dir lives until test ends
    app_state_file_path: PathBuf,
}

/// Creates a new, isolated temporary directory and initializes a default AppState file within it.
/// This ensures each test starts with a clean slate for app state persistence.
fn get_test_paths(test_name: &str) -> anyhow::Result<TestPaths> {
    // Initialize logger for the test. Only sets it if not already set.
    // Set to Debug level for tests that need detailed output.
    logger::init_logger(Some(LevelFilter::Debug));
    debug!("Test setup: Initializing test paths for {}", test_name);

    // Use CARGO_TARGET_TMPDIR for robust temp directory creation across platforms
    let mut temp_base_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    temp_base_dir.push("cleansh_full_stats_tests_data"); // Common temp directory for all tests in this file
    temp_base_dir.push(test_name); // Specific subdirectory for the current test

    // Ensure the base directory for the test's temp folder exists
    fs::create_dir_all(&temp_base_dir)?;

    let temp_dir = tempfile::tempdir_in(temp_base_dir)?;
    // Define the specific path for the app state file within this temporary directory
    let app_state_file_path = temp_dir.path().join("app_state.json");

    // Initialize a default AppState and save it to the test-specific path.
    // This ensures the file exists and is in a known state for the test.
    AppState::new().save_to_path(&app_state_file_path)?;
    debug!("Test setup: App state file created at {:?}", app_state_file_path);

    Ok(TestPaths {
        _temp_dir: temp_dir,
        app_state_file_path,
    })
}

/// Constructs a `Command` for the `cleansh` binary, configuring it to use a test-specific
/// AppState file via an environment variable. It also **clears relevant environment variables**
/// to ensure test isolation.
fn run_cleansh_cmd(app_state_file: &PathBuf) -> Command {
    let mut cmd = Command::cargo_bin("cleansh").unwrap();
    // Pass the test-specific app state file path via an environment variable.
    cmd.env("CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS", app_state_file.to_str().unwrap());

    // --- IMPORTANT: Clear potentially interfering environment variables for each command call ---
    // This ensures that previous tests (especially those using unsafe env::set_var)
    // don't leak state into the current test.
    // Note: RUST_LOG is explicitly cleared here, so individual tests need to set it if desired.
    cmd.env_remove("RUST_LOG");
    cmd.env_remove("CLEANSH_ALLOW_DEBUG_PII"); // Clear PII debug flag
    cmd.env("RUST_BACKTRACE", "1"); // Enable backtraces for debugging panics
    debug!("Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to {:?}", app_state_file);
    cmd
}

/// Returns the current Unix timestamp in seconds.
fn current_timestamp_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// --- Test Suite for `--stats-only` Command ---

#[test]
fn test_stats_only_no_matches() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_only_no_matches")?;
    debug!("Running test_stats_only_no_matches");

    // No --export-json-to-stdout here, so human-readable summary to stderr is expected.
    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("This is a clean string with no PII.")
        .arg("--stats-only")
        .arg("--rules")
        .arg("default")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for no_matches: \n{}", stderr);

    // Ensure the "No redaction matches found." message is present
    assert!(stderr.contains("No redaction matches found."));
    // Ensure no specific rule matches are reported
    assert!(!stderr.contains("EmailAddress:"));
    assert!(!stderr.contains("IPv4Address:"));

    Ok(())
}

#[test]
fn test_stats_only_with_simple_matches() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_only_with_simple_matches")?;
    debug!("Running test_stats_only_with_simple_matches");

    // No --export-json-to-stdout here, so human-readable summary to stderr is expected.
    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("My email is test@example.com and IP is 192.168.1.1.")
        .arg("--stats-only")
        .arg("--rules")
        .arg("default")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for simple_matches: \n{}", stderr);

    // Assert for the PascalCase format with special handling for "EmailAddress" and "IPv4Address"
    assert!(stderr.contains("EmailAddress: 1 match"));
    assert!(stderr.contains("IPv4Address: 1 match"));
    assert!(!stderr.contains("No redaction matches found."));

    Ok(())
}

#[test]
fn test_stats_only_programmatic_validation_regex_match_only() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_only_programmatic_validation_regex_match_only")?;
    debug!("Running test_stats_only_programmatic_validation_regex_match_only");

    // This SSN (000-12-3456) matches the regex pattern but is programmatically invalid.
    // For rules with `programmatic_validation: true`, a match is only counted for stats
    // if both the regex matches AND the programmatic validation passes.
    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("My SSN is 000-12-3456.")
        .arg("--stats-only")
        .arg("--rules")
        .arg("default")
        .output()?;

    assert!(output.status.success(), "Command failed with status: {:?}", output.status);
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for programmatic_validation_regex_match_only: \n{}", stderr);

    // Expect no redaction matches, as the programmatic validation for SSN_US fails for "000-12-3456".
    // This clarifies the expected behavior when programmatic validation prevents a count.
    assert!(
        stderr.contains("No redaction matches found."),
        "Expected 'No redaction matches found.' for programmatically invalid SSN. Actual stderr:\n{stderr}"
    );
    // Explicitly assert that us_ssn is NOT present in the summary, confirming it wasn't counted.
    assert!(
        !stderr.contains("us_ssn:"),
        "Expected 'us_ssn' to NOT be present in summary for programmatically invalid SSN. Actual stderr:\n{stderr}"
    );

    Ok(())
}

#[test]
fn test_stats_only_programmatic_validation_valid_match() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_only_programmatic_validation_valid_match")?;
    debug!("Running test_stats_only_programmatic_validation_valid_match");

    // This SSN (123-45-6789) matches the regex pattern and is programmatically valid.
    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("My SSN is 123-45-6789.")
        .arg("--stats-only")
        .arg("--rules")
        .arg("default")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for programmatic_validation_valid_match: \n{}", stderr);

    // This valid SSN match should definitely be counted.
    // Corrected assertion: Ensure 'us_ssn' is lowercase as per the actual output.
    assert!(stderr.contains("us_ssn: 1 match"));
    assert!(!stderr.contains("No redaction matches found."));

    Ok(())
}

#[test]
fn test_stats_only_with_sample_matches() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_only_with_sample_matches")?;
    debug!("Running test_stats_only_with_sample_matches");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("Email 1: test@example.com. Email 2: example@domain.com. Email 3: user@mail.org.")
        .arg("--stats-only")
        .arg("--rules")
        .arg("default")
        .arg("--sample-matches")
        .arg("2") // Request only 2 samples
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for sample_matches: \n{}", stderr);

    assert!(stderr.contains("EmailAddress: 3 matches"));
    // Verify that "Sample Matches:" header is present
    assert!(stderr.contains("Sample Matches:"));
    // Check that at least two of the original emails are present (order isn't guaranteed)
    let email1_present = stderr.contains("test@example.com");
    let email2_present = stderr.contains("example@domain.com");
    let email3_present = stderr.contains("user@mail.org");
    assert!((email1_present as u8 + email2_present as u8 + email3_present as u8) >= 2);
    // Check for the "more samples" indicator if total unique samples exceed requested samples
    assert!(stderr.contains("... (1 more unique samples)"));
    assert_eq!(stderr.matches("EmailAddress:").count(), 1, "Should only have one EmailAddress summary line");

    Ok(())
}

#[test]
fn test_stats_json_output_to_stdout() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_json_output_to_stdout")?;
    debug!("Running test_stats_json_output_to_stdout");

    // Fix for E0716: Store the Assert result in a binding
    let assert_result = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: user@example.com, ip: 192.168.1.1")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .arg("--export-json-to-stdout")
        .assert()
        .success()
        .stdout(is_json()); // Assert that stdout is valid JSON

    let output = assert_result.get_output(); // Get the full output from the binding

    // Read the JSON output from stdout and check its content
    let output_str = String::from_utf8(output.stdout.clone())?;
    debug!("Stdout JSON for json_output_to_stdout: \n{}", output_str);
    let stats: Value = serde_json::from_str(&output_str)?;

    assert!(stats["redaction_summary"].is_object());
    assert!(stats["redaction_summary"]["EmailAddress"].is_object());
    assert_eq!(stats["redaction_summary"]["EmailAddress"]["count"], 1);
    assert!(stats["redaction_summary"]["IPv4Address"].is_object());
    assert_eq!(stats["redaction_summary"]["IPv4Address"]["count"], 1);

    // Ensure AppState-related fields are NOT in the JSON output
    assert!(!stats.as_object().unwrap().contains_key("stats_only_usage_count"));
    assert!(!stats.as_object().unwrap().contains_key("last_prompt_timestamp"));
    assert!(!stats.as_object().unwrap().contains_key("donation_prompts_disabled"));

    // Ensure stderr does NOT contain human-readable summary if JSON is output to stdout
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for json_output_to_stdout: \n{}", stderr_str);
    assert!(!stderr_str.contains("Redaction Statistics Summary:"));
    assert!(!stderr_str.contains("EmailAddress: 1 match"));

    Ok(())
}

#[test]
fn test_stats_json_output_to_file() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_json_output_to_file")?;
    debug!("Running test_stats_json_output_to_file");
    let output_json_path = test_paths._temp_dir.path().join("stats_output.json");
    debug!("Output JSON path: {:?}", output_json_path);

    run_cleansh_cmd(&test_paths.app_state_file_path)
        .env("RUST_LOG", "debug") // Ensure debug logs for this command
        .write_stdin("secret_token: ABCDEF1234567890")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .arg("--enable-rules=generic_token") // Enable an opt-in rule for testing
        .arg("--stats-json-file").arg(&output_json_path)
        .assert()
        .success();

    // Verify the file was created and contains valid JSON
    assert!(output_json_path.exists());
    let file_content = fs::read_to_string(&output_json_path)?;
    debug!("Content of stats_output.json:\n{}", file_content);
    let stats: Value = serde_json::from_str(&file_content)?;
    debug!("Parsed JSON stats: {:?}", stats);

    assert!(stats["redaction_summary"].is_object());
    // The `format_rule_name_for_json` function should convert "generic_token" to "GenericToken"
    assert!(stats["redaction_summary"]["GenericToken"].is_object(),
        "Expected 'GenericToken' in JSON output, but was not found. Full JSON: {}", serde_json::to_string_pretty(&stats)?);
    assert_eq!(stats["redaction_summary"]["GenericToken"]["count"], 1);

    Ok(())
}

#[test]
fn test_stats_fail_over_triggered() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_fail_over_triggered")?;
    debug!("Running test_stats_fail_over_triggered");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        // Input to trigger 3 matches: Email, IPv4, AWS Access Key
        .write_stdin("Email: test@example.com. IP: 192.168.1.1. AWS Access Key: AKIAIOSFODNN7EXAMPLE.")
        .arg("--rules").arg("default")
        .arg("--enable-rules=aws_access_key") // Enable 'aws_access_key' to ensure 3 matches are found
        .arg("--stats-only")
        .arg("--fail-over").arg("2") // Set fail-over threshold to 2. We expect 3 matches
        .output()?;

    // Expect failure due to --fail-over threshold being exceeded (3 > 2)
    assert!(!output.status.success());
    assert_eq!(output.status.code().unwrap_or(0), 1); // Check for exit code 1

    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for fail_over_triggered: \n{}", stderr);
    assert!(stderr.contains("Fail-over triggered: Total secrets (3) exceeded threshold (2)."));

    Ok(())
}

#[test]
fn test_stats_fail_over_not_triggered() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_fail_over_not_triggered")?;
    debug!("Running test_stats_fail_over_not_triggered");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        // Input to trigger 3 matches: Email, IPv4, AWS Access Key
        .write_stdin("Email: test@example.com. IP: 192.168.1.1. AWS Access Key: AKIAIOSFODNN7EXAMPLE.")
        .arg("--rules").arg("default")
        .arg("--enable-rules=aws_access_key")
        .arg("--stats-only")
        .arg("--fail-over").arg("3") // Set fail-over threshold to 3. With 3 matches, it should pass
        .output()?;

    // Expect success
    assert!(output.status.success());
    assert_eq!(output.status.code().unwrap_or(0), 0);

    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for fail_over_not_triggered: \n{}", stderr);
    assert!(!stderr.contains("Fail-over triggered")); // Ensure fail-over message is NOT present
    assert!(stderr.contains("Total secrets (3) are below the fail-over threshold (3)."));

    Ok(())
}

#[test]
fn test_stats_rule_enable_and_disable() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_rule_enable_and_disable")?;
    debug!("Running test_stats_rule_enable_and_disable");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("Email: test@example.com. AWS Secret: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY.")
        .arg("--rules").arg("default")
        .arg("--enable-rules=aws_secret_key") // Enable opt-in AWS Secret Key
        .arg("--disable-rules=email") // Disable default Email rule
        .arg("--stats-only")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for rule_enable_and_disable: \n{}", stderr);

    // Email should NOT be present as it was disabled
    assert!(!stderr.contains("EmailAddress:"));
    // AWS Secret Key SHOULD be present as it was explicitly enabled
    assert!(stderr.contains("AWSSecretKey: 1 match"));

    Ok(())
}

#[test]
fn test_stats_app_state_usage_increment() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_app_state_usage_increment")?;
    debug!("Running test_stats_app_state_usage_increment");

    // Initial state check
    let initial_app_state = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("Initial app state usage count: {}", initial_app_state.stats_only_usage_count);
    assert_eq!(initial_app_state.stats_only_usage_count, 0);

    // Run 1: Should increment count to 1
    run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: test@example.com")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .assert().success();

    let app_state_1 = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("App state usage count after run 1: {}", app_state_1.stats_only_usage_count);
    assert_eq!(app_state_1.stats_only_usage_count, 1);

    // Run 2: Should increment count to 2
    run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("ip: 1.2.3.4")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .assert().success();

    let app_state_2 = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("App state usage count after run 2: {}", app_state_2.stats_only_usage_count);
    assert_eq!(app_state_2.stats_only_usage_count, 2);

    Ok(())
}

#[test]
fn test_stats_donation_prompt_trigger_and_cooldown() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_donation_prompt_trigger_and_cooldown")?;
    debug!("Running test_stats_donation_prompt_trigger_and_cooldown");

    // Prime AppState: Set usage count to 4, last prompt timestamp to over a month ago
    let mut app_state = AppState::default();
    app_state.stats_only_usage_count = 4; // One run away from threshold (5)
    app_state.last_prompt_timestamp = Some(current_timestamp_secs() - (24 * 60 * 60 * 31)); // > 30 days ago
    app_state.save_to_path(&test_paths.app_state_file_path)?;
    debug!("Initial app state for donation prompt test: {:?}", app_state);

    let initial_prompt_timestamp_check = current_timestamp_secs();

    // Run 1 (total count 5): Should trigger prompt
    let output1 = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: trigger@example.com")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .assert()
        .success()
        .stderr(predicate::str::contains("please consider donating").count(1));
    debug!("Stderr after first run (prompt expected): \n{}", String::from_utf8_lossy(&output1.get_output().stderr));

    let app_state_after_prompt = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("App state after first run: {:?}", app_state_after_prompt);
    assert_eq!(app_state_after_prompt.stats_only_usage_count, 5);
    assert!(app_state_after_prompt.last_prompt_timestamp.is_some());
    // Verify timestamp was updated to around now
    assert!(app_state_after_prompt.last_prompt_timestamp.unwrap() >= initial_prompt_timestamp_check);

    // Run 2 (total count 6): Should NOT trigger prompt due to cooldown
    let output2 = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: no_prompt@example.com")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .assert()
        .success()
        .stderr(predicate::str::contains("please consider donating").not());
    debug!("Stderr after second run (no prompt expected): \n{}", String::from_utf8_lossy(&output2.get_output().stderr));

    let app_state_after_cooldown = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("App state after second run: {:?}", app_state_after_cooldown);
    assert_eq!(app_state_after_cooldown.stats_only_usage_count, 6);
    // Timestamp should be the same as after the first prompt
    assert_eq!(app_state_after_cooldown.last_prompt_timestamp, app_state_after_prompt.last_prompt_timestamp);

    Ok(())
}

#[test]
fn test_stats_donation_prompt_suppression_flag() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_donation_prompt_suppression_flag")?;
    debug!("Running test_stats_donation_prompt_suppression_flag");

    // Prime AppState to trigger prompt conditions (count 4, old timestamp)
    let mut app_state = AppState::default();
    app_state.stats_only_usage_count = 4;
    app_state.last_prompt_timestamp = Some(current_timestamp_secs() - (24 * 60 * 60 * 31));
    app_state.save_to_path(&test_paths.app_state_file_path)?;
    debug!("Initial app state for suppression test: {:?}", app_state);

    // Run with `--disable-donation-prompts`
    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("Some input: test@example.com.")
        .arg("--rules").arg("default")
        .arg("--stats-only")
        .arg("--disable-donation-prompts")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for donation_prompt_suppression_flag: \n{}", stderr);
    assert!(!stderr.contains("If you find Cleansh useful"), "Donation prompt should be suppressed");

    let app_state_after_suppression = AppState::load_from_path(&test_paths.app_state_file_path)?;
    debug!("App state after suppression run: {:?}", app_state_after_suppression);
    assert_eq!(app_state_after_suppression.stats_only_usage_count, 5);
    assert!(app_state_after_suppression.donation_prompts_disabled);
    // Timestamp should NOT be updated when prompts are disabled by flag
    assert!(app_state_after_suppression.last_prompt_timestamp.unwrap() < current_timestamp_secs() - 30 * 24 * 60 * 60 + 100);

    Ok(())
}

#[test]
fn test_stats_quiet_flag_suppresses_info() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_quiet_flag_suppresses_info")?;
    debug!("Running test_stats_quiet_flag_suppresses_info");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: test@example.com")
        .arg("--stats-only")
        .arg("--rules").arg("default")
        .arg("-q") // Use the quiet flag
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for quiet_flag_suppresses_info: \n{}", stderr);

    // Should contain the stats summary, but NOT info messages like "Reading input from stdin..."
    assert!(stderr.contains("EmailAddress: 1 match"));
    assert!(!stderr.contains("Reading input from stdin..."));
    assert!(!stderr.contains("Starting cleansh --stats-only operation."));
    assert!(!stderr.contains("Cleansh --stats-only operation completed."));

    Ok(())
}

#[test]
fn test_stats_debug_flag_enables_debug_logs() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_debug_flag_enables_debug_logs")?;
    debug!("Running test_stats_debug_flag_enables_debug_logs");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: test@example.com")
        .arg("--stats-only")
        .arg("--rules").arg("default")
        .arg("--debug") // Enable debug logging
        .output()?;

    // --- TEMPORARY DEBUGGING START ---
    eprintln!("--- STDOUT from test_stats_debug_flag_enables_debug_logs ---");
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("--- STDERR from test_stats_debug_flag_enables_debug_logs ---");
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    eprintln!("--- STATUS from test_stats_debug_flag_enables_debug_logs ---");
    eprintln!("Success: {}", output.status.success());
    eprintln!("Code: {:?}", output.status.code());
    // --- TEMPORARY DEBUGGING END ---

    assert!(output.status.success(), "Command failed with status: {:?}", output.status);
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for debug_flag_enables_debug_logs: \n{}", stderr);

    // Should contain debug logs related to rule compilation and processing
    assert!(stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Starting stats-only operation."));
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] compile_rules called with"));
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully."));
    // Specific debug from sanitize_content - Adjusted to reflect typical Rust log format
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Sanitization complete. Total individual matches found: 1"));
    assert!(stderr.contains("EmailAddress: 1 match")); // Still has the summary

    Ok(())
}

#[test]
fn test_stats_no_debug_flag_disables_debug_logs() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_no_debug_flag_disables_debug_logs")?;
    debug!("Running test_stats_no_debug_flag_disables_debug_logs");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .write_stdin("email: test@example.com")
        .arg("--stats-only")
        .arg("--rules").arg("default")
        .arg("--no-debug") // Explicitly disable debug logging
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for no_debug_flag_disables_debug_logs: \n{}", stderr);

    // Should NOT contain debug logs
    assert!(!stderr.contains("[DEBUG]"));
    // Should still contain info messages (unless --quiet is also used)
    assert!(stderr.contains("Starting cleansh --stats-only operation."));
    assert!(stderr.contains("EmailAddress: 1 match"));

    Ok(())
}

#[test]
fn test_stats_pii_debug_env_var() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_pii_debug_env_var")?;
    debug!("Running test_stats_pii_debug_env_var");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .env("CLEANSH_ALLOW_DEBUG_PII", "1") // Set PII debug flag for this command
        .env("RUST_LOG", "debug") // Ensure RUST_LOG is debug for this command
        .write_stdin("My SSN is 123-45-6789. My email is test@example.com.")
        .arg("--stats-only")
        .arg("--rules").arg("default")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for pii_debug_env_var: \n{}", stderr);

    // When CLEANSH_ALLOW_DEBUG_PII is 1, these logs should show the ORIGINAL (unredacted) PII.
    assert!(stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='us_ssn', Original='123-45-6789', Sanitized='[US_SSN_REDACTED]'"));
    assert!(stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='email', Original='test@example.com', Sanitized='[EMAIL_REDACTED]'"));

    // Also verify the more general `sanitize_shell` debug logs show the original if PII debug is on.
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Rule 'email' captured match (original): test@example.com"));
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' captured match (original): 123-45-6789"));

    // Verify summary is still present
    assert!(stderr.contains("us_ssn: 1 match"));
    assert!(stderr.contains("EmailAddress: 1 match"));

    Ok(())
}

#[test]
fn test_stats_pii_debug_env_var_not_set() -> anyhow::Result<()> {
    let test_paths = get_test_paths("test_stats_pii_debug_env_var_not_set")?;
    debug!("Running test_stats_pii_debug_env_var_not_set");

    let output = run_cleansh_cmd(&test_paths.app_state_file_path)
        .env("RUST_LOG", "debug") // Set RUST_LOG to debug for this command
        .write_stdin("My SSN is 123-45-6789. My email is test@example.com.")
        .arg("--stats-only")
        .arg("--rules").arg("default")
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Stderr for pii_debug_env_var_not_set: \n{}", stderr);

    // PII debug logs from `cleansh::commands::stats` (which explicitly show original PII) should NOT be present.
    assert!(!stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='us_ssn'"));
    assert!(!stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='email'"));

    // However, the `sanitize_shell` module *will* still log "captured match (original):"
    // but the actual PII content should be REDACTED when CLEANSH_ALLOW_DEBUG_PII is NOT set.
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Rule 'email' captured match (original): [REDACTED: 16 chars]"));
    assert!(stderr.contains("[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' captured match (original): [REDACTED: 11 chars]"));


    assert!(stderr.contains("[DEBUG cleansh::commands::stats] [stats.rs] Starting stats-only operation."));

    // Summary should still be present
    assert!(stderr.contains("us_ssn: 1 match"));
    assert!(stderr.contains("EmailAddress: 1 match"));

    Ok(())
}