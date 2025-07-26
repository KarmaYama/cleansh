// src/utils/redaction.rs

use serde::{Serialize, Deserialize};

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

#[macro_export]
macro_rules! pii_debug {
    ($($arg:tt)*) => {
        // This macro now unconditionally calls log::debug!.
        // The decision to show PII or redacted content is handled
        // at the call site (e.g., in sanitize_shell.rs) before calling pii_debug!.
        log::debug!($($arg)*);
    };
}

pub use pii_debug;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env; // This import is now only needed within the test module
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
    #[test_log::test]
    fn test_pii_debug_macro_always_logs_if_rust_log_debug() {
        // Temporarily set environment variables for the test
        // Use `unsafe` blocks for `set_var` and `remove_var`
        unsafe {
            // For testing log output, setting RUST_LOG here can be fragile if test_log
            // or other tests already configure it.
            // For this specific test, we're primarily focused on `CLEANSH_ALLOW_DEBUG_PII`'s
            // effect on the content, assuming RUST_LOG is generally set by the test runner.
            // If you need to ensure RUST_LOG is "debug", it's best set in the command line:
            // `RUST_LOG=debug cargo test ...`
            env::remove_var("RUST_LOG"); // Remove to ensure test_log can manage it or it defaults
            env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1");
        }
        
        let test_message = "This should always be logged if RUST_LOG=debug, regardless of CLEANSH_ALLOW_DEBUG_PII.";
        // In a real test where you assert log output, you'd capture stdout/stderr.
        // For this test, we're relying on `test_log` to indicate if logging happened
        // and that the macro call doesn't panic.
        pii_debug!("{}", test_message);

        // Clean up environment variables after the test
        unsafe {
            env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
            env::remove_var("RUST_LOG"); // Clean up RUST_LOG if set here
        }
    }

    #[test]
    #[test_log::test]
    fn test_pii_debug_macro_always_logs_if_rust_log_debug_with_pii_disabled() {
        // Ensure the environment variable CLEANSH_ALLOW_DEBUG_PII is not set for this test
        unsafe {
            env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
            env::remove_var("RUST_LOG"); // Remove to ensure test_log can manage it or it defaults
        }
        
        let test_message = "This should still be logged, with content determined by call site.";
        pii_debug!("{}", test_message);

        unsafe {
            env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
            env::remove_var("RUST_LOG"); // Clean up RUST_LOG if set here
        }
    }
}