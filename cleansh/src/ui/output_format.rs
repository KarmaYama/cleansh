// cleansh-workspace/cleansh/src/ui/output_format.rs
//! Module for consistent command-line output formatting in Cleansh.
//!
//! This module provides utility functions for printing various types of messages
//! to standard output or standard error, applying theme-based styling (colors)
//! when the terminal supports it. It centralizes text styling logic to ensure
//! a consistent user interface experience across the application.

use crate::ui::theme::{ThemeEntry, ThemeMap};
use owo_colors::OwoColorize;
use std::io::{self, Write};
// Removed: use is_terminal::IsTerminal; // Not needed in this module now as we pass `enable_colors` directly

/// Helper to get a styled string based on the theme.
///
/// This function applies ANSI color codes to a given `text` based on the
/// specified `ThemeEntry` and `theme_map`. If `enable_colors` is `false`
/// or if no specific style/color is found for the given `ThemeEntry`,
/// the original text is returned without any color codes.
///
/// It returns an owned `String` that implements `Display`.
/// Made `pub(crate)` for use by other UI modules within the crate.
///
/// # Arguments
///
/// * `text` - The string slice to apply styling to.
/// * `entry` - The `ThemeEntry` indicating which style to use from the `theme_map`
///             (e.g., `ThemeEntry::Info`, `ThemeEntry::Error`).
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for various `ThemeEntry`s.
/// * `enable_colors` - A boolean indicating whether ANSI colors should actually be applied.
///                     If `false`, the original text is returned without color codes,
///                     regardless of theme configuration.
///
/// # Returns
///
/// A `String` with ANSI color codes applied if `enable_colors` is true and a matching
/// theme color is found. If no specific color is found but colors are enabled,
/// it falls back to white. Otherwise (if colors are not enabled), the original `text`
/// is returned as a `String` without color codes.
pub(crate) fn get_styled_text(
    text: &str,
    entry: ThemeEntry,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool,
) -> String {
    if enable_colors {
        if let Some(style) = theme_map.get(&entry) {
            if let Some(color) = &style.fg {
                return text.color(color.to_ansi_color()).to_string();
            }
        }
        // Fallback if no specific style or color is found, but colors are enabled
        // This ensures some color is always applied if `enable_colors` is true
        text.color(owo_colors::AnsiColors::White).to_string()
    } else {
        // If colors are not enabled, return plain text
        text.to_string()
    }
}

/// Prints a general message to the given writer, with an optional theme entry for styling.
///
/// If `theme_entry` is `None`, it defaults to `ThemeEntry::Info`.
/// The message is automatically followed by a newline character.
/// Colors are applied only if `enable_colors` is true.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `writer` - The output writer (e.g., `&mut io::stdout()` or `&mut io::stderr()`).
/// * `message` - The string slice containing the message to print.
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling.
/// * `theme_entry` - An `Option<ThemeEntry>` specifying the desired style. If `None`, `ThemeEntry::Info` is used.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or an I/O error during writing.
pub fn print_message<W: Write>( // <--- Removed `+ IsTerminal` trait bound
    writer: &mut W,
    message: &str,
    theme_map: &ThemeMap, // Use ThemeMap alias
    theme_entry: Option<ThemeEntry>,
    enable_colors: bool, // <--- Added enable_colors parameter
) -> io::Result<()> {
    let final_theme_entry = theme_entry.unwrap_or(ThemeEntry::Info);
    let styled_message = get_styled_text(&format!("{}\n", message), final_theme_entry, theme_map, enable_colors);
    write!(writer, "{}", styled_message)
}

/// Prints an informational message to the given writer, styled by the theme.
///
/// This function uses `ThemeEntry::Info` for styling. The message is automatically
/// followed by a newline character. Colors are applied only if `enable_colors` is true.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `writer` - The output writer (e.g., `&mut io::stderr()`).
/// * `message` - The string slice containing the informational message.
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or an I/O error during writing.
pub fn print_info_message<W: Write>( // <--- Removed `+ IsTerminal` trait bound
    writer: &mut W,
    message: &str,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool, // <--- Added enable_colors parameter
) -> io::Result<()> {
    let styled_message = get_styled_text(&format!("{}\n", message), ThemeEntry::Info, theme_map, enable_colors);
    write!(writer, "{}", styled_message)
}

/// Prints an error message to the given writer, styled by the theme.
///
/// This function prefixes the message with "ERROR: " and uses `ThemeEntry::Error` for styling.
/// The message is automatically followed by a newline character. Colors are applied only
/// if `enable_colors` is true.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `writer` - The output writer (e.g., `&mut io::stderr()`).
/// * `message` - The string slice containing the error message.
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or an I/O error during writing.
pub fn print_error_message<W: Write>( // <--- Removed `+ IsTerminal` trait bound
    writer: &mut W,
    message: &str,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool, // <--- Added enable_colors parameter
) -> io::Result<()> {
    let styled_message = get_styled_text(&format!("ERROR: {}\n", message), ThemeEntry::Error, theme_map, enable_colors);
    write!(writer, "{}", styled_message)
}

/// Prints a warning message to the given writer, styled by the theme.
///
/// This function prefixes the message with "WARNING: " and uses `ThemeEntry::Warn` for styling.
/// The message is automatically followed by a newline character. Colors are applied only
/// if `enable_colors` is true.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `writer` - The output writer (e.g., `&mut io::stderr()`).
/// * `message` - The string slice containing the warning message.
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or an I/O error during writing.
pub fn print_warn_message<W: Write>( // <--- Removed `+ IsTerminal` trait bound
    writer: &mut W,
    message: &str,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool, // <--- Added enable_colors parameter
) -> io::Result<()> {
    let styled_message = get_styled_text(&format!("WARNING: {}\n", message), ThemeEntry::Warn, theme_map, enable_colors);
    write!(writer, "{}", styled_message)
}