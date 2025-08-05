// cleansh-workspace/cleansh-core/src/redaction_match.rs
//! Provides core data structures and utility functions for managing redaction matches
//! and sensitive data logging within the `CleanSH-core` library.
//!
//! This module defines `RedactionMatch` for detailed reporting of sanitization
//! operations and includes helper functions for conditionally redacting
//! sensitive information in debug logs based on environment variables,
//! ensuring PII is not accidentally exposed.
//! License: BUSL-1.1


use serde::{Serialize, Deserialize};
use log::debug;
use std::env;

/// Represents a single instance of a matched and potentially redacted string.
///
/// This struct is used to collect granular information about each redaction,
/// allowing for more detailed summaries and analysis in `--stats-only` mode
/// or for integration with other reporting systems.
///
/// # Fields
///
/// * `rule_name`: The name of the redaction rule that was applied (e.g., "email", "ipv4_address").
/// * `original_string`: The original text that was matched and redacted.
/// * `sanitized_string`: The string used to replace the original text after redaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionMatch {
    pub rule_name: String,
    pub original_string: String,
    pub sanitized_string: String,
}

/// Redacts sensitive information from a string for logging or display purposes.
///
/// This function replaces the content of a string with a placeholder. For short
/// strings (up to 8 characters), it returns `"[REDACTED]"`. For longer strings,
/// it includes the character count, e.g., `"[REDACTED: 15 chars]"`.
///
/// This is a security-conscious utility to prevent accidental logging or display
/// of sensitive information during debugging or error reporting. It is publicly
/// exposed for potential utility in testing or custom logging scenarios where
/// immediate redaction of a single string is desired.
///
/// # Arguments
///
/// * `s` - The string slice to redact.
///
/// # Returns
///
/// A `String` containing the redacted representation.
///
/// # Examples
///
/// ```rust
/// use cleansh_core::redaction_match::redact_sensitive;
///
/// assert_eq!(redact_sensitive("abc"), "[REDACTED]".to_string());
/// assert_eq!(redact_sensitive("my_secret_password"), "[REDACTED: 18 chars]".to_string());
/// ```
pub fn redact_sensitive(s: &str) -> String {
    const MAX_LEN: usize = 8;
    if s.len() <= MAX_LEN {
        "[REDACTED]".to_string()
    } else {
        format!("[REDACTED: {} chars]", s.len())
    }
}

/// Checks if the `CLEANSH_ALLOW_DEBUG_PII` environment variable is set to "true".
///
/// This private helper function controls whether original sensitive information
/// is included in debug logs. When the environment variable is set to "true"
/// (case-insensitive), PII is allowed in debug logs; otherwise, it is redacted.
fn is_pii_debug_allowed() -> bool {
    env::var("CLEANSH_ALLOW_DEBUG_PII")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Logs a debug message for a `RedactionMatch`, conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
///
/// This function is intended for logging the details of a `RedactionMatch`
/// object, typically when a redaction has been successfully performed.
/// It integrates with the `log` crate's `debug!` macro.
///
/// # Arguments
///
/// * `module_path` - A string slice indicating the module or context from which the log is originating (e.g., `"[cleansh::core::sanitizer]"`).
/// * `rule_name` - The name of the redaction rule that triggered the match.
/// * `original_sensitive_content` - The actual sensitive string that was matched. This will be redacted if PII debug is not allowed.
/// * `sanitized_content` - The string that replaced the `original_sensitive_content`. This is always safe to log.
pub fn log_redaction_match_debug(
    module_path: &str,
    rule_name: &str,
    original_sensitive_content: &str,
    sanitized_content: &str,
) {
    let content_to_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        &*redact_sensitive(original_sensitive_content)
    };

    debug!("{} Found RedactionMatch: Rule='{}', Original='{}', Sanitized='{}'",
        module_path,
        rule_name,
        content_to_log,
        sanitized_content
    );
}

/// Logs a debug message for a 'captured match', conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
///
/// This function is intended for logging an intermediate match found by a regex
/// before a `RedactionMatch` object is fully finalized or validated.
/// It integrates with the `log` crate's `debug!` macro.
///
/// # Arguments
///
/// * `module_path` - A string slice indicating the module or context.
/// * `rule_name` - The name of the rule that captured the match.
/// * `original_sensitive_content` - The string content that was captured by the regex. This will be redacted if PII debug is not allowed.
pub fn log_captured_match_debug(
    module_path: &str,
    rule_name: &str,
    original_sensitive_content: &str,
) {
    let content_to_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        &*redact_sensitive(original_sensitive_content)
    };
    debug!("{} Captured match (original): '{}' for rule '{}'", module_path, content_to_log, rule_name);
}

/// Logs a debug message for a redaction action (i.e., when a replacement occurs),
/// conditionally redacting the original sensitive content based on `CLEANSH_ALLOW_DEBUG_PII`.
///
/// This function provides visibility into the actual transformation of sensitive
/// data during the sanitization process. It integrates with the `log` crate's `debug!` macro.
///
/// # Arguments
///
/// * `module_path` - A string slice indicating the module or context.
/// * `original_sensitive_content` - The original content that was replaced. This will be redacted if PII debug is not allowed.
/// * `sanitized_replacement` - The string used as the replacement. This is always safe to log.
/// * `rule_name` - The name of the rule that performed the redaction.
pub fn log_redaction_action_debug(
    module_path: &str,
    original_sensitive_content: &str,
    sanitized_replacement: &str,
    rule_name: &str,
) {
    let original_for_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        &*redact_sensitive(original_sensitive_content)
    };

    debug!(
        "{} Redaction action: Original='{}', Redacted='{}' for rule '{}'",
        module_path,
        original_for_log,
        sanitized_replacement,
        rule_name
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use test_log::test; // For `#[test_log::test]`

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
    #[test_log::test]
    fn test_log_redaction_match_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "true"); }
        log_redaction_match_debug(
            "[test::redaction]", "email", "test@example.com", "[EMAIL_REDACTED]"
        );
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_match_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_redaction_match_debug(
            "[test::redaction]", "email", "test@example.com", "[EMAIL_REDACTED]"
        );
    }

    #[test]
    #[test_log::test]
    fn test_log_captured_match_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "true"); }
        log_captured_match_debug("[test::redaction]", "ssn", "123-45-6789");
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
    }

    #[test]
    #[test_log::test]
    fn test_log_captured_match_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_captured_match_debug("[test::redaction]", "ssn", "123-45-6789");
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_action_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "true"); }
        log_redaction_action_debug("[test::redaction]", "original_token", "REDACTED_TOKEN", "generic_token");
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_action_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_redaction_action_debug("[test::redaction]", "original_token", "REDACTED_TOKEN", "generic_token");
    }
}