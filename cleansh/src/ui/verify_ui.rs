// cleansh-workspace/cleansh/src/ui/verify_ui.rs

//! This module provides UI-specific functions for the `verify-artifact` subcommand.
//! It centralizes all output messages related to artifact verification.

use crate::ui::output_format::{print_message, print_error_message};
use crate::ui::theme::{ThemeMap, ThemeEntry};
use anyhow::Result;
use std::io; // Corrected: removed unused `Write` import
use std::path::Path;

/// Prints a message indicating that the artifact verification is starting.
pub fn print_verify_start(artifact_path: &Path, public_key_path: &Path, theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stdout = io::stdout().lock();
    print_message(&mut stdout, &format!("Verifying artifact: {}", artifact_path.display()), theme_map, Some(ThemeEntry::Info), enable_colors)?;
    print_message(&mut stdout, &format!("Using public key from: {}", public_key_path.display()), theme_map, Some(ThemeEntry::Info), enable_colors)?;
    Ok(())
}

/// Prints a success message after a successful artifact verification.
pub fn print_verify_success(theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stdout = io::stdout().lock();
    print_message(&mut stdout, "Artifact signature verified successfully.", theme_map, Some(ThemeEntry::Success), enable_colors)?;
    Ok(())
}

/// Prints an error message for a failed artifact verification.
pub fn print_verify_failure_error(theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stderr = io::stderr().lock();
    print_error_message(&mut stderr, "Artifact signature verification FAILED.", theme_map, enable_colors)?;
    Ok(())
}