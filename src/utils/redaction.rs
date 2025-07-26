// src/utils/redaction.rs
use serde::{Serialize, Deserialize};
use log::debug; // Import log::debug
use std::env; // Import std::env for environment variables

/// Represents a single instance of a matched and potentially redacted string.
/// This struct is used to collect granular information about each redaction,
/// allowing for more detailed summaries and analysis in `--stats-only` mode.
///
/// In a larger system, instances of `RedactionMatch` would typically be collected
/// during a redaction process and then processed to generate reports or statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionMatch {
    pub rule_name: String,
    pub original_string: String,
    pub sanitized_string: String,
    // Add other relevant fields if needed, e.g., line number, column, etc.
}

/// Redacts sensitive information from a string for logging or display.
///
/// Based on the provided code, strings up to MAX_LEN (8 chars) are simply "[REDACTED]",
/// longer strings include their length.
pub fn redact_sensitive(s: &str) -> String {
    // Constant for the maximum length before a string's length is included in the redaction.
    const MAX_LEN: usize = 8;
    if s.len() <= MAX_LEN {
        "[REDACTED]".to_string()
    } else {
        format!("[REDACTED: {} chars]", s.len())
    }
}

/// Checks if the `CLEANSH_ALLOW_DEBUG_PII` environment variable is set.
fn is_pii_debug_allowed() -> bool {
    env::var("CLEANSH_ALLOW_DEBUG_PII").is_ok()
}

/// Logs a debug message for a `RedactionMatch`, conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
///
/// This function is intended for logging the final `RedactionMatch` object's details.
pub fn log_redaction_match_debug(
    module_path: &str, // Renamed from log_prefix to module_path for clarity
    rule_name: &str,
    original_sensitive_content: &str,
    sanitized_content: &str,
) {
    let content_to_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        // Convert the String returned by redact_sensitive to a &str
        &*redact_sensitive(original_sensitive_content)
    };

    debug!("{} Found RedactionMatch: Rule='{}', Original='{}', Sanitized='{}'",
        module_path,
        rule_name,
        content_to_log,
        sanitized_content // Sanitized content is always safe to log
    );
}

/// Logs a debug message for a 'captured match', conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
///
/// This function is intended for logging an intermediate 'match' found by a regex
/// before full `RedactionMatch` objects are finalized.
pub fn log_captured_match_debug(
    module_path: &str, // Renamed from log_prefix to module_path for clarity
    rule_name: &str,
    original_sensitive_content: &str,
) {
    let content_to_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        // Convert the String returned by redact_sensitive to a &str
        &*redact_sensitive(original_sensitive_content)
    };
    // *** Adjusted format string and argument order to match test expectations ***
    debug!("{} Captured match (original): '{}' for rule '{}'", module_path, content_to_log, rule_name);
}

/// Logs a debug message for a redaction action, conditionally redacting
/// the original sensitive content based on the `CLEANSH_ALLOW_DEBUG_PII`
/// environment variable.
///
/// This function is intended for logging when an actual string replacement occurs.
pub fn log_redaction_action_debug(
    module_path: &str, // Renamed from log_prefix to module_path for clarity
    original_sensitive_content: &str,
    sanitized_replacement: &str,
    rule_name: &str,
) {
    let original_for_log: &str = if is_pii_debug_allowed() {
        original_sensitive_content
    } else {
        // Convert the String returned by redact_sensitive to a &str
        &*redact_sensitive(original_sensitive_content)
    };

    debug!(
        "{} Redaction action: Original='{}', Redacted='{}' for rule '{}'", // Adjusted format string
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

    // These tests for logging functions are more conceptual, as `test_log` doesn't
    // provide a direct way to capture and assert on log output in unit tests.
    // For robust assertion on log content, integration tests (like the ones
    // in `tests/full_stats_tests.rs`) are preferred, where you can capture stderr.
    // Here, we just ensure the functions compile and run without panicking.

    #[test]
    #[test_log::test]
    fn test_log_redaction_match_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1"); }
        log_redaction_match_debug(
            "[test::redaction]", "email", "test@example.com", "[EMAIL_REDACTED]"
        );
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
        // In a real test, you'd assert the captured log contains "test@example.com"
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_match_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_redaction_match_debug(
            "[test::redaction]", "email", "test@example.com", "[EMAIL_REDACTED]"
        );
        // In a real test, you'd assert the captured log contains "[REDACTED: 16 chars]"
    }

    #[test]
    #[test_log::test]
    fn test_log_captured_match_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1"); }
        log_captured_match_debug("[test::redaction]", "ssn", "123-45-6789");
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
        // In a real test, you'd assert the captured log contains "123-45-6789"
    }

    #[test]
    #[test_log::test]
    fn test_log_captured_match_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_captured_match_debug("[test::redaction]", "ssn", "123-45-6789");
        // In a real test, you'd assert the captured log contains "[REDACTED: 11 chars]"
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_action_debug_pii_allowed() {
        unsafe { env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1"); }
        log_redaction_action_debug("[test::redaction]", "original_token", "REDACTED_TOKEN", "generic_token");
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); }
        // Assert log contains "original_token"
    }

    #[test]
    #[test_log::test]
    fn test_log_redaction_action_debug_pii_not_allowed() {
        unsafe { env::remove_var("CLEANSH_ALLOW_DEBUG_PII"); } // Ensure not set
        log_redaction_action_debug("[test::redaction]", "original_token", "REDACTED_TOKEN", "generic_token");
        // Assert log contains "[REDACTED: 14 chars]"
    }
}