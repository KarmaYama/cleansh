// cleansh-workspace/cleansh/src/utils/platform.rs
//! This module provides platform-specific utilities and information.
//! It uses Rust's `cfg!` macro to handle OS-specific logic at compile time.

/// Returns the correct End-of-File (EOF) key combination for the current platform.
///
/// This is used for displaying the correct prompt to the user when reading from stdin
/// in an interactive terminal. The check is performed at compile-time for efficiency
/// and safety.
///
/// # Returns
///
/// * `"Ctrl+Z"` on Windows targets.
/// * `"Ctrl+D"` on non-Windows targets (Linux, macOS, etc.).
pub fn eof_key_combo() -> &'static str {
    if cfg!(windows) {
        "Ctrl+Z"
    } else {
        "Ctrl+D"
    }
}