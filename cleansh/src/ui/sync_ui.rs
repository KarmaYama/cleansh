//! This module provides UI-specific functions for the `sync-profiles` subcommand.
//! It centralizes all output messages related to profile synchronization.

use crate::ui::output_format::{print_message, print_error_message};
use crate::ui::theme::{ThemeMap, ThemeEntry};
use anyhow::Result;
use std::io;
use std::path::Path;

/// Prints a message indicating that the profile synchronization is starting.
pub fn print_sync_start(theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stdout = io::stdout().lock();
    print_message(&mut stdout, "Starting profile synchronization...", theme_map, Some(ThemeEntry::Info), enable_colors)?;
    Ok(())
}

/// Prints a message for a connection attempt.
pub fn print_connection_attempt(url: &str, theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stdout = io::stdout().lock();
    print_message(&mut stdout, &format!("Attempting to connect to: {}", url), theme_map, Some(ThemeEntry::Info), enable_colors)?;
    Ok(())
}

/// Prints a success message after a successful profile sync.
pub fn print_sync_success(profile_path: &Path, theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stdout = io::stdout().lock();
    print_message(&mut stdout, "Profile synchronization SUCCESSFUL.", theme_map, Some(ThemeEntry::Success), enable_colors)?;
    print_message(&mut stdout, &format!("Profiles saved to: {}", profile_path.display()), theme_map, Some(ThemeEntry::Success), enable_colors)?;
    Ok(())
}

/// Prints an error message for a failed authentication.
pub fn print_auth_failed_error(theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stderr = io::stderr().lock();
    print_error_message(&mut stderr, "Authentication FAILED. Please check your API key.", theme_map, enable_colors)?;
    Ok(())
}

/// Prints an error message for a server not found.
pub fn print_server_not_found_error(org_id: &str, url: &str, theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stderr = io::stderr().lock();
    let message = format!("Server not found for organization '{}' at: {}. Please check the ID and URL.", org_id, url);
    print_error_message(&mut stderr, &message, theme_map, enable_colors)?;
    Ok(())
}

/// Prints a generic error message for a sync failure with a status code.
pub fn print_sync_failure_error(status_code: u16, theme_map: &ThemeMap, enable_colors: bool) -> Result<()> {
    let mut stderr = io::stderr().lock();
    let message = format!("Profile synchronization FAILED with status code: {}.", status_code);
    print_error_message(&mut stderr, &message, theme_map, enable_colors)?;
    Ok(())
}