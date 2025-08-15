//! Integration tests for the `verify-artifact` subcommand.
//! This file tests the cryptographic signature verification process with both
//! valid and invalid signatures.

use anyhow::{Result, anyhow};
use cleansh::cli::VerifyArtifactCommand;
use cleansh::commands::verify::run_verify_artifact_command;
use cleansh::ui::theme::{ThemeMap, ThemeStyle};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::json;
use std::fs;
use tempfile::tempdir;
use uuid::Uuid;

/// A simple mock theme map for testing purposes.
fn mock_theme_map() -> ThemeMap {
    ThemeStyle::default_theme_map()
}

/// Sets up an isolated test environment by setting the HOME directory to a temporary directory.
/// This prevents tests from interfering with a user's actual configuration files.
fn setup_test_environment() -> Result<tempfile::TempDir> {
    let dir = tempdir()?;
    unsafe { std::env::set_var("HOME", dir.path()); }
    Ok(dir)
}

#[test]
fn test_artifact_verification() -> Result<()> {
    // 1. Setup: Create a temporary directory for our files.
    let dir = setup_test_environment()?;

    // 2. Generate a mock Ed25519 key pair using the 'ring' crate.
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| anyhow!("Failed to generate key pair: {:?}", e))?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|e| anyhow!("Failed to parse key pair: {:?}", e))?;
    
    // The verification function expects raw public key bytes.
    let public_key_path = dir.path().join("public_key.pem");
    fs::write(&public_key_path, key_pair.public_key().as_ref())?;

    // 3. Create a mock artifact JSON object that matches the FSpec.
    let run_id = Uuid::new_v4().to_string();
    let artifact_data = json!({
        "artifact_version": "1.0",
        "run_id": run_id,
        "profile": { "name": "test-profile", "version": "1.0", "rule_set_version": "1.0" },
        "engine": { "engine_name": "TestEngine", "engine_version": "1.0", "entropy_enabled": false },
        "input_hash": { "algorithm": "sha256", "hex": "abcdef1234567890" },
        "summary": { "total_matches": 1, "unique_matches": 1, "by_severity": { "high": 1 }, "by_rule": { "credit_card_number": 1 } },
        "matches": [],
        "samples": [],
        "artifact_meta": { "user": { "name": "test-user", "id": "test-id" }, "host_fingerprint": "test-host", "cli_args": "--profile test" }
    });

    // Serialize the artifact data to get the canonical JSON string to sign.
    let canonical_artifact_json = serde_json::to_string(&artifact_data)?;

    // 4. Sign the canonical JSON string.
    let signature = key_pair.sign(canonical_artifact_json.as_bytes());

    // 5. Create the final artifact JSON object with the 'data' wrapper.
    let final_artifact_json = json!({
        "data": canonical_artifact_json,
        "signature": hex::encode(signature.as_ref())
    });

    // 6. Write the final, complete artifact JSON file.
    let artifact_path = dir.path().join("test_artifact.json");
    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&final_artifact_json)?.as_bytes(),
    )?;

    // 7. Run the verification command with the valid key and artifact.
    let verify_opts = VerifyArtifactCommand {
        verify_artifact: artifact_path.clone(),
        public_key: public_key_path.clone(),
    };

    let verify_result = run_verify_artifact_command(&verify_opts, &mock_theme_map());
    assert!(
        verify_result.is_ok(),
        "Verification should succeed but returned an error: {:?}",
        verify_result.unwrap_err()
    );

    // 8. Test with an invalid signature (tampered artifact data).
    let tampered_data_string = canonical_artifact_json.replace("total_matches\":1", "total_matches\":999");
    let tampered_artifact_json = json!({
        "data": tampered_data_string,
        "signature": hex::encode(signature.as_ref())
    });

    let tampered_artifact_path = dir.path().join("tampered_artifact.json");
    fs::write(
        &tampered_artifact_path,
        serde_json::to_string_pretty(&tampered_artifact_json)?.as_bytes(),
    )?;

    let tampered_verify_opts = VerifyArtifactCommand {
        verify_artifact: tampered_artifact_path.clone(),
        public_key: public_key_path.clone(),
    };

    // This should fail because the signature no longer matches the data.
    let result = run_verify_artifact_command(&tampered_verify_opts, &mock_theme_map());
    assert!(
        result.is_err(),
        "Verification should fail for a tampered artifact."
    );

    // Clean up.
    dir.close()?;
    Ok(())
}