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
    let diff_header = get_styled_text("\n--- Diff View ---", ThemeEntry::DiffHeader, theme_map, true);
    writeln!(io::stderr(), "{}", diff_header)?;

    let patch = create_patch(original_content, sanitized_content);

    for hunk in patch.hunks() {
        for line_change in hunk.lines() {
            let content_str = match line_change {
                Line::Delete(s) => s,
                Line::Insert(s) => s,
                Line::Context(s) => s,
            };

            // FIX: Replace literal "\\n" with actual newlines AND then split the resulting string
            // into individual lines to ensure each segment is printed on its own line.
            // This handles cases where diffy might return `\n` literally for multi-line inputs,
            // or if the input itself contained literal `\n` characters.
            let s_with_actual_newlines = content_str.replace("\\n", "\n");

            // Split the string by actual newlines and print each segment.
            // This handles cases where a single `diffy::Line` might represent multiple
            // visual lines due to embedded `\n` characters.
            for segment in s_with_actual_newlines.lines() {
                match line_change {
                    Line::Delete(_) => {
                        if enable_colors {
                            writeln!(writer, "{}{}", "-".red(), segment.red())?;
                        } else {
                            writeln!(writer, "-{}", segment)?;
                        }
                    }
                    Line::Insert(_) => {
                        if enable_colors {
                            writeln!(writer, "{}{}", "+".green(), segment.green())?;
                        } else {
                            writeln!(writer, "+{}", segment)?;
                        }
                    }
                    Line::Context(_) => {
                        writeln!(writer, " {}", segment)?;
                    }
                }
            }
        }
    }
    writeln!(io::stderr(), "{}", get_styled_text("-----------------", ThemeEntry::DiffHeader, theme_map, true))?;
    Ok(())
}

// Helper function (copied from output_format.rs, as it's a private helper)
fn get_styled_text(
    text: &str,
    entry: ThemeEntry,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
    enable_colors: bool,
) -> String {
    if enable_colors {
        if let Some(style) = theme_map.get(&entry) {
            if let Some(color) = &style.fg {
                return text.color(color.to_ansi_color()).to_string();
            }
        }
        text.color(owo_colors::AnsiColors::White).to_string()
    } else {
        text.to_string()
    }
}