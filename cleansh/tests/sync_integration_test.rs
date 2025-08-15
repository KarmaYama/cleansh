//! Integration tests for the `sync-profiles` subcommand.
//! This file uses `mockito` to mock an HTTP server and test the profile
//! synchronization process under different conditions.

use anyhow::{Context, Result, anyhow};
use cleansh::cli::SyncProfilesCommand;
use cleansh::commands::sync::run_sync_profiles_command;
use cleansh::ui::theme::{ThemeMap, ThemeStyle};
use mockito::Server;
use std::fs;
use tempfile::tempdir;
use std::sync::Mutex;

/// Serialize tests that touch global environment variables to avoid races.
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// A simple mock theme map for testing purposes.
fn mock_theme_map() -> ThemeMap {
    ThemeStyle::default_theme_map()
}

/// Sets up an isolated test environment by setting the HOME directory to a temporary directory.
fn setup_test_environment() -> Result<tempfile::TempDir> {
    let dir = tempdir()?;
    // Some toolchains require these calls in an unsafe block in your environment.
    unsafe { std::env::set_var("HOME", dir.path()); }
    Ok(dir)
}

/// A helper to run a test function with an environment variable set,
/// ensuring the variable is removed afterward, even on panic.
/// This variant also acquires the global TEST_MUTEX to serialize env mutations.
fn set_and_restore_env<F, T>(key: &str, value: &str, test_function: F) -> T
where
    F: FnOnce() -> T,
{
    // Acquire the global lock to prevent other tests from touching the env at the same time.
    let _guard = TEST_MUTEX.lock().unwrap();

    // Set env while holding the lock. Wrap in unsafe as required by your compiler.
    unsafe { std::env::set_var(key, value); }

    // run the passed closure while the lock is held
    let result = test_function();

    // remove the env var while still holding the lock (also unsafe per compiler)
    unsafe { std::env::remove_var(key); }

    // _guard is dropped here after function returns, allowing other tests to proceed
    result
}

#[test]
fn test_sync_profiles_success() -> Result<()> {
    let mut server = Server::new();
    let expected_api_key = "valid-api-key";

    // Mock expects Authorization header exactly as used by your code
    let _m = server
        .mock("GET", "/orgs/your-organization-id/profiles")
        .match_header("authorization", format!("Bearer {}", expected_api_key).as_str())
        .with_status(200)
        .with_header("content-type", "application/x-yaml")
        .with_body("profiles:\n  - name: test-profile\n    rules: []")
        .create();

    let _dir = setup_test_environment()?;

    // set_and_restore_env holds TEST_MUTEX for the duration of the closure.
    let result = set_and_restore_env("ORG_SERVER_URL", &server.url(), || {
        let sync_opts = SyncProfilesCommand {
            org_key: expected_api_key.to_string(),
            org_id: "your-organization-id".to_string(),
        };

        // Run sync and expect Ok
        run_sync_profiles_command(&sync_opts, &mock_theme_map())
            .with_context(|| "Sync command should have succeeded")?;

        // Verify profiles file was created and contains expected content
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory for test"))?;
        let profile_path = config_dir.join("cleansh").join("profiles").join("synced_profiles.yaml");

        assert!(
            profile_path.exists(),
            "Synced profiles file should exist at: {}", profile_path.display()
        );

        let content = fs::read_to_string(&profile_path)?;
        assert!(
            content.contains("test-profile"),
            "Synced profiles content is incorrect."
        );

        // Clean up
        fs::remove_file(&profile_path)?;

        Ok(())
    });

    result
}

#[test]
fn test_sync_profiles_auth_failure() -> Result<()> {
    let mut server = Server::new();

    // Mock returns 401 Unauthorized for any request to this path
    let _m = server
        .mock("GET", "/orgs/your-organization-id/profiles")
        .with_status(401)
        .create();

    let _dir = setup_test_environment()?;

    let result = set_and_restore_env("ORG_SERVER_URL", &server.url(), || {
        let sync_opts = SyncProfilesCommand {
            org_key: "invalid-api-key".to_string(),
            org_id: "your-organization-id".to_string(),
        };

        let result = run_sync_profiles_command(&sync_opts, &mock_theme_map());

        assert!(result.is_err(), "Sync command should fail with a 401 error.");

        let error_message = result.as_ref().unwrap_err().to_string();
        assert!(
            error_message.contains("Authentication Failed (401 Unauthorized)"),
            "Incorrect error message for 401."
        );

        // Clean up file if it somehow exists
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory for test"))?;
        let profile_path = config_dir.join("cleansh").join("profiles").join("synced_profiles.yaml");
        if profile_path.exists() {
            fs::remove_file(&profile_path)?;
        }

        Ok(())
    });

    result
}