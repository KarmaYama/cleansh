//! This module handles the `verify-artifact` subcommand, which validates the
//! cryptographic signature of an artifact JSON file.
//!
//! License: Polyform Noncommercial License 1.0.0

use crate::cli::VerifyArtifactCommand;
use crate::ui::theme::ThemeMap;
use crate::ui::verify_ui;
use anyhow::{Result, anyhow, Context};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde_json::Value;
use std::fs;
use std::io;
use is_terminal::IsTerminal;

/// The main entry point for the `cleansh verify-artifact` subcommand.
///
/// It takes the path to the artifact and the public key, then performs the
/// cryptographic signature check.
pub fn run_verify_artifact_command(opts: &VerifyArtifactCommand, theme_map: &ThemeMap) -> Result<()> {
    let enable_colors = io::stdout().is_terminal();
    // Corrected field names
    verify_ui::print_verify_start(&opts.verify_artifact, &opts.public_key, theme_map, enable_colors)?;

    // 1. Read the artifact file
    let artifact_content = fs::read_to_string(&opts.verify_artifact)
        .context("Failed to read artifact file")?;

    // 2. Parse the JSON
    let parsed_json: Value = serde_json::from_str(&artifact_content)
        .context("Failed to parse artifact JSON")?;

    // 3. Extract and canonicalize the 'data' field.
    // Support both: `"data": "<stringified-json>"` and `"data": { ... }`.
    let data_to_verify: Vec<u8> = match parsed_json.get("data") {
        Some(v) => {
            if let Some(s) = v.as_str() {
                // 'data' is a string containing JSON — use raw bytes as-is.
                s.as_bytes().to_vec()
            } else {
                // 'data' is a JSON object/value — serialize to canonical compact JSON string.
                serde_json::to_string(v)
                    .context("Failed to serialize 'data' object to canonical JSON string for verification")?
                    .into_bytes()
            }
        }
        None => return Err(anyhow!("Artifact JSON missing 'data' field")),
    };

    // 4. Extract signature (expect hex-encoded string)
    let signature = parsed_json.get("signature")
        .and_then(|v| v.as_str())
        .and_then(|s| hex::decode(s).ok())
        .context("Artifact JSON missing valid 'signature' field")?;

    // 5. Read the public key bytes
    let public_key_bytes = fs::read(&opts.public_key)
        .context("Failed to read public key file")?;

    // 6. Verify the signature using Ed25519
    let public_key = UnparsedPublicKey::new(&ED25519, &public_key_bytes);

    match public_key.verify(&data_to_verify, &signature) {
        Ok(_) => {
            verify_ui::print_verify_success(theme_map, enable_colors)?;
            Ok(())
        }
        Err(_) => {
            verify_ui::print_verify_failure_error(theme_map, enable_colors)?;
            Err(anyhow!("Artifact signature verification FAILED."))
        }
    }
}