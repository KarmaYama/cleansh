//! Provides core data structures and utility functions for managing redaction matches
//! and sensitive data logging within the `cleansh-core` library.
//!
//! This module defines `RedactionMatch` for detailed reporting of sanitization
//! operations and includes helper functions for conditionally redacting
//! sensitive information in debug logs based on environment variables,
//! ensuring PII is not accidentally exposed. It also defines the `RedactionLog`
//! for creating an auditable, immutable log of all redaction events.
//! License: BUSL-1.1

use serde::{Serialize, Deserialize};
use log::debug;
use crate::config::RedactionRule;

use lazy_static::lazy_static;
use sha2::{Sha256, Digest};
use hex;

lazy_static! {
    /// A static boolean that is initialized once to determine if PII is allowed in debug logs.
    /// This improves performance by avoiding repeated environment variable lookups.
    static ref PII_DEBUG_ALLOWED: bool = {
        std::env::var("CLEANSH_ALLOW_DEBUG_PII")
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    };
}

/// Represents a single instance of a matched and potentially redacted string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionMatch {
    pub rule_name: String,
    pub original_string: String,
    pub sanitized_string: String,
    // Changed to u64 for compatibility with file I/O
    pub start: u64,
    pub end: u64,
    #[serde(default)]
    pub line_number: Option<u64>, // Changed to u64
    #[serde(default)]
    pub sample_hash: Option<String>,
    #[serde(default)]
    pub match_context_hash: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub rule: RedactionRule,
    #[serde(default)]
    pub source_id: String,
}

/// Represents a single, auditable log entry for a redaction event.
#[derive(Debug, Serialize, Deserialize)]
pub struct RedactionLog {
    pub timestamp: String,
    pub run_id: String,
    pub file_path: String,
    pub user_id: String,
    pub reason_for_redaction: String,
    pub redaction_outcome: String,
    pub rule_name: String,
    pub input_hash: String,
    pub match_hash: String,
    // Changed to u64 for compatibility with file I/O
    pub start: u64,
    pub end: u64,
}

/// Redacts sensitive information from a string for logging or display purposes.
pub fn redact_sensitive(s: &str) -> String {
    const MAX_LEN: usize = 8;
    if s.len() <= MAX_LEN {
        "[REDACTED]".to_string()
    } else {
        format!("[REDACTED: {} chars]", s.len())
    }
}

/// Private helper to get the appropriate string for logging based on PII permission.
fn get_loggable_content(sensitive_content: &str) -> String {
    if *PII_DEBUG_ALLOWED {
        sensitive_content.to_string()
    } else {
        redact_sensitive(sensitive_content)
    }
}

/// Logs a debug message for a `RedactionMatch`, conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
pub fn log_redaction_match_debug(
    module_path: &str,
    rule_name: &str,
    original_sensitive_content: &str,
    sanitized_content: &str,
) {
    debug!("{} Found RedactionMatch: Rule='{}', Original='{}', Sanitized='{}'",
        module_path,
        rule_name,
        get_loggable_content(original_sensitive_content),
        sanitized_content
    );
}

/// Logs a debug message for a 'captured match', conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
pub fn log_captured_match_debug(
    module_path: &str,
    rule_name: &str,
    original_sensitive_content: &str,
) {
    debug!("{} Captured match (original): '{}' for rule '{}'",
        module_path,
        get_loggable_content(original_sensitive_content),
        rule_name
    );
}

/// Logs a debug message for a redaction action (i.e., when a replacement occurs),
/// conditionally redacting the original sensitive content based on `CLEANSH_ALLOW_DEBUG_PII`.
pub fn log_redaction_action_debug(
    module_path: &str,
    original_sensitive_content: &str,
    sanitized_replacement: &str,
    rule_name: &str,
) {
    debug!(
        "{} Redaction action: Original='{}', Redacted='{}' for rule '{}'",
        module_path,
        get_loggable_content(original_sensitive_content),
        sanitized_replacement,
        rule_name
    );
}

/// Produce a canonical hash for a matched snippet and rule.
/// Normalizes whitespace and case, includes rule id to avoid cross-rule collisions.
pub fn canonical_sample_hash(rule_id: &str, snippet: &str) -> String {
    // Normalization: trim, collapse whitespace to single spaces, lowercase
    let normalized = snippet
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut hasher = Sha256::new();
    hasher.update(rule_id.as_bytes());
    hasher.update(b":");
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

/// Ensure each RedactionMatch has a sample_hash. Populates sample_hash using canonical_sample_hash
/// if missing. This is safe to call after engine detection and before UI/ignore-store logic.
pub fn ensure_match_hashes(matches: &mut [RedactionMatch]) {
    for m in matches.iter_mut() {
        if m.sample_hash.is_none() {
            let hash = canonical_sample_hash(&m.rule_name, &m.original_string);
            m.sample_hash = Some(hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_redact_sensitive_short_string() {
        assert_eq!(redact_sensitive("abc"), "[REDACTED]".to_string());
        assert_eq!(redact_sensitive("12345678"), "[REDACTED]".to_string());
    }

    #[test]
    fn test_redact_sensitive_long_string() {
        assert_eq!(redact_sensitive("123456789"), "[REDACTED: 9 chars]".to_string());
        assert_eq!(redact_sensitive("long_sensitive_data"), "[REDACTED: 19 chars]".to_string());
    }

    #[test]
    fn test_canonical_sample_hash_consistency() {
        let h1 = canonical_sample_hash("email", "Test@Example.COM ");
        let h2 = canonical_sample_hash("email", "test@example.com");
        assert_eq!(h1, h2, "canonical hash should be stable across case/whitespace");
    }
}