/// Helper function to redact sensitive strings for logging.
/// It redacts strings longer than MAX_LEN to a generic "[REDACTED]"
/// or "[REDACTED: X chars]" format, to prevent accidental PII logging.
///
/// This function is intended for internal logging purposes where
/// sensitive data should not be exposed, even in debug logs.
///
/// # Arguments
/// * `s` - The string slice to redact.
///
/// # Examples
/// ```
/// use cleansh::utils::redaction::redact_sensitive;
/// assert_eq!(redact_sensitive("short"), "[REDACTED]".to_string());
/// assert_eq!(redact_sensitive("longer_than_eight_chars"), "[REDACTED: 23 chars]".to_string());
/// ```
pub fn redact_sensitive(s: &str) -> String {
    const MAX_LEN: usize = 8;
    if s.len() <= MAX_LEN {
        "[REDACTED]".to_string()
    } else {
        format!("[REDACTED: {} chars]", s.len())
    }
}

/// Macro to conditionally log debug messages that might contain redacted PII.
/// Messages are only logged if the `CLEANSH_ALLOW_DEBUG_PII` environment variable is set.
/// This acts as a security guard to prevent accidental leakage of sensitive information
/// into logs in production environments.
///
/// # Usage
/// Use `pii_debug!` just like `log::debug!`. If `CLEANSH_ALLOW_DEBUG_PII` is not set,
/// the message will not be logged.
///
/// # Examples
/// ```no_run
/// // In a module that uses pii_debug!
/// use std::env; // This import is for the example, not the macro itself
/// use cleansh::utils::redaction::pii_debug;
///
/// // This message will only be logged if CLEANSH_ALLOW_DEBUG_PII is set.
/// // Env vars are global and can affect other tests/programs, so this is an unsafe operation.
/// unsafe {
///     env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1");
/// }
/// pii_debug!("Sensitive data: {}", "user@example.com");
/// 
/// // Clean up env var for other tests/runs. This is also an unsafe operation.
/// unsafe {
///     env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
/// }
///
/// // This message will NOT be logged.
/// pii_debug!("Another sensitive piece: {}", "password123");
/// ```
#[macro_export]
macro_rules! pii_debug {
    ($($arg:tt)*) => {
        // The `log::debug!` is fully qualified here, so no `use log::debug;` needed at top level for the macro itself.
        if std::env::var("CLEANSH_ALLOW_DEBUG_PII").is_ok() {
            log::debug!($($arg)*);
        }
    };
}

// Re-export the macro for crate-wide use without needing `#[macro_export]` everywhere
pub use pii_debug;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use test_log::test; // Changed from test_env_log::test to test_log::test

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
    #[test_log::test] // Changed from test_env_log::test to test_log::test
    fn test_pii_debug_macro_respects_env_enabled() {
        unsafe {
            env::remove_var("RUST_LOG");
            env::set_var("CLEANSH_ALLOW_DEBUG_PII", "1");
        }
        
        let test_message = "This should be logged.";
        pii_debug!("{}", test_message);

        unsafe {
            env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
            env::remove_var("RUST_LOG");
        }
    }

    #[test]
    #[test_log::test] // Changed from test_env_log::test to test_log::test
    fn test_pii_debug_macro_respects_env_disabled() {
        unsafe {
            env::remove_var("CLEANSH_ALLOW_DEBUG_PII");
            env::remove_var("RUST_LOG");
        }
        
        let test_message = "This should NOT be logged.";
        pii_debug!("{}", test_message);
    }
}