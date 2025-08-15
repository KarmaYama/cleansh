// cleansh/src/utils/license.rs

use anyhow::{Context, Result, anyhow};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::convert::TryFrom;

/// Put your actual base64-encoded Ed25519 public key here (32 bytes -> base64).
/// This key is used to verify signatures on licenses.
const EMBEDDED_LICENSE_PUBLIC_KEY_BASE64: &str = "37R/FtgbH7IUIuHucFs1HnnGDneuDltNP/KjK0uczPM=";

/// Canonical license structure. Fields are straightforward and serde-deserializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub version: u32,
    #[serde(default)]
    pub license_id: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default)]
    pub features: HashMap<String, Option<u64>>, // None => unlimited, Some(n) => limit
    #[serde(default)]
    pub tier: Option<String>, // optional human-readable tier label, e.g. "pro", "team"
}

/// A parsed token that keeps the payload and signature bytes
pub struct LicenseToken {
    pub payload: LicensePayload,
    pub signature: Vec<u8>,
}

impl LicenseToken {
    /// Compute a stable fingerprint for this license token suitable for indexing in AppState.
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.signature);
        let out = hasher.finalize();
        hex::encode(&out[..8]) // short hex (16 chars) for readability
    }
}

/// Parse a token of form `BASE64(json) . '.' . BASE64(sig)`
/// Returns LicenseToken on success.
pub fn parse_compact_token(token: &str) -> Result<LicenseToken> {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid token format: expected two dot-separated parts"));
    }

    let json_b = general_purpose::STANDARD.decode(parts[0])
        .context("Failed to base64-decode license JSON part")?;
    let sig_b = general_purpose::STANDARD.decode(parts[1])
        .context("Failed to base64-decode signature part")?;

    let payload: LicensePayload = serde_json::from_slice(&json_b)
        .context("Failed to deserialize license JSON")?;

    Ok(LicenseToken { payload, signature: sig_b })
}

/// Deterministically canonicalize a serde_json::Value by sorting object keys recursively.
/// Returns a new `Value`.
fn canonicalize_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut kv: Vec<_> = map.iter().collect();
            kv.sort_by(|a, b| a.0.cmp(b.0));
            let mut out = serde_json::Map::new();
            for (k, val) in kv {
                out.insert(k.clone(), canonicalize_value(val));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_value).collect()),
        other => other.clone(),
    }
}

/// Produce canonical bytes for signing/verifying by parsing the JSON bytes,
/// canonicalizing it (sorted keys), then serializing without extra whitespace.
fn canonical_bytes_from_json_slice(src: &[u8]) -> Result<Vec<u8>> {
    let v: Value = serde_json::from_slice(src)
        .context("Failed to parse JSON when canonicalizing")?;
    let canon = canonicalize_value(&v);
    let bytes = serde_json::to_vec(&canon)?;
    Ok(bytes)
}

/// Verify an Ed25519 signature for the given license token using the embedded or env public key.
/// Returns Ok(()) if valid; Err otherwise.
pub fn verify_token_signature(token: &LicenseToken) -> Result<()> {
    // Obtain public key bytes (first check env var)
    let pub_b64 = std::env::var("CLEANSH_LICENSE_PUBLIC_KEY_BASE64")
        .unwrap_or_else(|_| EMBEDDED_LICENSE_PUBLIC_KEY_BASE64.to_string());

    if pub_b64 == "REPLACE_WITH_YOUR_BASE64_PUBKEY" || pub_b64.trim().is_empty() {
        return Err(anyhow!("No embedded public key found. Set CLEANSH_LICENSE_PUBLIC_KEY_BASE64 or populate EMBEDDED_LICENSE_PUBLIC_KEY_BASE64."));
    }

    let pub_bytes = general_purpose::STANDARD.decode(pub_b64)
        .context("Failed to base64-decode embedded public key")?;
    let public_key_bytes: [u8; 32] = pub_bytes.as_slice()
        .try_into()
        .map_err(|_| anyhow!("Public key length invalid: expected 32 bytes"))?;
    let public = VerifyingKey::from_bytes(&public_key_bytes)?;

    // Reconstruct canonical JSON bytes for verification.
    let json_bytes = serde_json::to_vec(&token.payload)?;
    let canonical = canonical_bytes_from_json_slice(&json_bytes)?;

    // Construct signature safely
    let signature_bytes: [u8; 64] = token.signature.as_slice()
        .try_into()
        .map_err(|_| anyhow!("Signature must be exactly 64 bytes"))?;
    let sig = Signature::try_from(&signature_bytes[..])
        .map_err(|_| anyhow!("Failed to construct ed25519 Signature from bytes"))?;

    // Verify
    public.verify(&canonical, &sig)
        .map_err(|e| anyhow!("Signature verification failed: {}", e))
}

/// Convenience: parse the compact token and verify signature & expiry checks and return the token.
pub fn parse_and_verify_compact(token_str: &str) -> Result<LicenseToken> {
    let token = parse_compact_token(token_str)?;
    verify_token_signature(&token)?;

    // expiry check
    let now = Utc::now();
    if token.payload.expires_at < now {
        return Err(anyhow!("License expired at {}", token.payload.expires_at));
    }
    Ok(token)
}
