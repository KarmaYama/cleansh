// src/ui/output_format.rs

use crate::ui::theme::{ThemeEntry, ThemeStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::Write;
// Removed diffy imports as print_diff_view is moved

// Removed: /// Prints the content to the given writer.
// Removed: pub fn print_content<W: Write>(writer: &mut W, content: &str) {
// Removed:     let _ = write!(writer, "{}", content);
// Removed: }

/// Helper to get a styled string based on the theme.
/// Returns an owned String that implements Display.
fn get_styled_text(
    text: &str,
    entry: ThemeEntry,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> String {
    if let Some(style) = theme_map.get(&entry) {
        if let Some(color) = &style.fg {
            return text.color(color.to_ansi_color()).to_string();
        }
    }
    // Fallback if no specific style or color is found
    text.color(owo_colors::AnsiColors::White).to_string()
}


/// Prints an informational message to the given writer, styled by the theme.
pub fn print_info_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("{}\n", message), ThemeEntry::Info, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints an error message to the given writer, styled by the theme.
pub fn print_error_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("ERROR: {}\n", message), ThemeEntry::Error, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints a warning message to the given writer, styled by the theme.
pub fn print_warn_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("WARNING: {}\n", message), ThemeEntry::Warn, theme_map);
    let _ = write!(writer, "{}", styled_message);
}