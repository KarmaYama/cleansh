// src/ui/diff_viewer.rs

use crate::ui::theme::{ThemeEntry, ThemeStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{self, Write};
use anyhow::Result;
use diffy::{create_patch, Line};

/// Prints a diff view of the original and sanitized content to the given writer.
pub fn print_diff<W: Write>(
    original_content: &str,
    sanitized_content: &str,
    writer: &mut W,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
    enable_colors: bool, // NEW PARAMETER: Indicates if the writer supports ANSI colors
) -> Result<()> {
    // Diff header always goes to stderr (console) and should be colored if stderr is a TTY.
    // The `get_styled_text` helper, when used here, will use the `enable_colors` parameter
    // for this specific diff_viewer call, which might be different from stderr's TTY status.
    // To ensure consistent coloring for stderr output messages (like headers),
    // it's generally best if those helpers always check `io::stderr().is_terminal()` internally
    // or are explicitly called to use a separate `stderr_supports_color` flag from `run_cleansh`.
    // For now, let's pass `true` to `get_styled_text` for these console messages,
    // assuming stderr is usually a TTY, and focus the `enable_colors` for the `writer`.
    let diff_header = get_styled_text("\n--- Diff View ---", ThemeEntry::DiffHeader, theme_map, true); // Always attempt colors for stderr header
    writeln!(io::stderr(), "{}", diff_header)?;

    let patch = create_patch(original_content, sanitized_content);

    for hunk in patch.hunks() {
        for line_change in hunk.lines() {
            match line_change {
                Line::Delete(s) => {
                    if enable_colors {
                        writeln!(writer, "{}{}", "-".red(), s.red())?; // Apply red color
                    } else {
                        writeln!(writer, "-{}", s)?; // Plain text
                    }
                }
                Line::Insert(s) => {
                    if enable_colors {
                        writeln!(writer, "{}{}", "+".green(), s.green())?; // Apply green color
                    } else {
                        writeln!(writer, "+{}", s)?; // Plain text
                    }
                }
                Line::Context(s) => {
                    writeln!(writer, " {}", s)?; // Context lines are never colored by `diffy`
                }
            }
        }
    }
    // Diff footer always goes to stderr (console) and should be colored if stderr is a TTY.
    writeln!(io::stderr(), "{}", get_styled_text("-----------------", ThemeEntry::DiffHeader, theme_map, true))?; // Always attempt colors for stderr footer
    Ok(())
}

// Helper function (copied from output_format.rs, as it's a private helper)
fn get_styled_text(
    text: &str,
    entry: ThemeEntry,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
    enable_colors: bool, // NEW PARAMETER: Use this to decide whether to apply colors
) -> String {
    if enable_colors {
        if let Some(style) = theme_map.get(&entry) {
            if let Some(color) = &style.fg {
                return text.color(color.to_ansi_color()).to_string();
            }
        }
        // Fallback to white if no specific theme color is found but colors are enabled
        text.color(owo_colors::AnsiColors::White).to_string()
    } else {
        // If colors are disabled, return the plain text
        text.to_string()
    }
}