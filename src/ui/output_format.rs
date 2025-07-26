// src/ui/output_format.rs

use crate::ui::theme::{ThemeEntry, ThemeStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{self, Write}; // Import io::Result

/// Helper to get a styled string based on the theme.
/// Returns an owned String that implements Display.
/// Made pub(crate) for use by other ui modules.
pub(crate) fn get_styled_text(
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

/// Prints a general message to the given writer, with an optional theme entry for styling.
/// If `theme_entry` is `None`, it defaults to `ThemeEntry::Info`.
pub fn print_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
    theme_entry: Option<ThemeEntry>,
) -> io::Result<()> { // CHANGED: Return io::Result<()>
    let final_theme_entry = theme_entry.unwrap_or(ThemeEntry::Info); // Default to Info
    let styled_message = get_styled_text(&format!("{}\n", message), final_theme_entry, theme_map);
    write!(writer, "{}", styled_message) // CHANGED: Propagate error with `?`
}

/// Prints an informational message to the given writer, styled by the theme.
pub fn print_info_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> io::Result<()> { // CHANGED: Return io::Result<()>
    let styled_message = get_styled_text(&format!("{}\n", message), ThemeEntry::Info, theme_map);
    write!(writer, "{}", styled_message) // CHANGED: Propagate error with `?`
}

/// Prints an error message to the given writer, styled by the theme.
pub fn print_error_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> io::Result<()> { // CHANGED: Return io::Result<()>
    let styled_message = get_styled_text(&format!("ERROR: {}\n", message), ThemeEntry::Error, theme_map);
    write!(writer, "{}", styled_message) // CHANGED: Propagate error with `?`
}

/// Prints a warning message to the given writer, styled by the theme.
pub fn print_warn_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> io::Result<()> { // CHANGED: Return io::Result<()>
    let styled_message = get_styled_text(&format!("WARNING: {}\n", message), ThemeEntry::Warn, theme_map);
    write!(writer, "{}", styled_message) // CHANGED: Propagate error with `?`
}
