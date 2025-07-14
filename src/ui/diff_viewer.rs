// src/ui/diff_viewer.rs

use crate::ui::theme::{ThemeEntry, ThemeStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{self, Write}; // Import io for stderr
use anyhow::Result; // Import Result for error handling
use diffy::{create_patch, Line};

/// Prints a diff view of the original and sanitized content to the given writer.
pub fn print_diff<W: Write>(
    original_content: &str,
    sanitized_content: &str,
    writer: &mut W, // This writer receives the actual diff lines (+/-)
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> Result<()> {
    let diff_header = get_styled_text("\n--- Diff View ---", ThemeEntry::DiffHeader, theme_map);
    // Changed to io::stderr()
    writeln!(io::stderr(), "{}", diff_header)?;

    let patch = create_patch(original_content, sanitized_content);

    for hunk in patch.hunks() {
        for line_change in hunk.lines() {
            match line_change {
                Line::Delete(s) => {
                    writeln!(writer, "{}{}", "-".red(), s.red())?;
                }
                Line::Insert(s) => {
                    writeln!(writer, "{}{}", "+".green(), s.green())?;
                }
                Line::Context(s) => {
                    writeln!(writer, " {}", s)?; // Space prefix for equal lines
                }
            }
        }
    }
    // Changed to io::stderr()
    writeln!(io::stderr(), "{}", get_styled_text("-----------------", ThemeEntry::DiffHeader, theme_map))?;
    Ok(())
}

// Helper function (copied from output_format.rs, as it's a private helper)
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
    text.color(owo_colors::AnsiColors::White).to_string()
}