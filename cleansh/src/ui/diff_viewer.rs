// cleansh-workspace/cleansh/src/ui/diff_viewer.rs
//! Module for displaying differences between original and sanitized content.
//!
//! This module provides functionality to generate and print a human-readable
//! diff view, highlighting added and removed lines, typically used to show
//! the changes made by the redaction process. It leverages the `diffy` crate
//! for patch generation and `owo-colors` for colored terminal output.

// Updated import: Removed ThemeStyle and HasIsTerminal, kept ThemeEntry and ThemeMap
use crate::ui::theme::{ThemeEntry, ThemeMap};
use std::io::{self, Write};
use anyhow::Result;
use diffy::{create_patch, Line};
use owo_colors::OwoColorize;

// Import get_styled_text from output_format
use crate::ui::output_format::get_styled_text;

/// Prints a diff view of the original and sanitized content to the given writer.
///
/// This function takes two string slices, `original_content` and `sanitized_content`,
/// computes their differences, and prints a standard diff-like output.
/// Lines present only in the original content are marked with '-' and colored red,
/// lines present only in the sanitized content are marked with '+' and colored green,
/// and common lines are shown without a prefix.
///
/// The diff header and footer are styled using `ThemeEntry::DiffHeader`.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`, allowing output to various destinations
///         like `io::stdout()`, `io::stderr()`, or a file.
///
/// # Arguments
///
/// * `original_content` - The original string content before any sanitization.
/// * `sanitized_content` - The string content after sensitive data has been (potentially) sanitized.
/// * `writer` - The output writer where the diff will be printed (e.g., `&mut io::stdout()`).
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s to apply colors to the output.
/// * `enable_colors` - A boolean flag indicating whether ANSI colors should be used in the output.
///
/// # Returns
///
/// A `Result` indicating `Ok(())` on successful write operations or an `Err`
/// if any writing to the `writer` fails.
pub fn print_diff<W: Write>( // <--- Trait bound changed to just `Write`
    original_content: &str,
    sanitized_content: &str,
    writer: &mut W,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool, // <--- New `enable_colors` argument
) -> Result<()> {
    // Determine enable_colors is now handled by the passed argument

    // Print diff header to stderr as per existing logic, assuming it's for user info.
    let diff_header = get_styled_text("\n--- Diff View ---", ThemeEntry::DiffHeader, theme_map, enable_colors);
    writeln!(io::stderr(), "{}", diff_header)?;

    let patch = create_patch(original_content, sanitized_content);

    for hunk in patch.hunks() {
        for line_change in hunk.lines() {
            let content_str = match line_change {
                Line::Delete(s) => s,
                Line::Insert(s) => s,
                Line::Context(s) => s,
            };

            // `diffy` might escape newlines as `\n` in content; replace them back to actual newlines
            let s_with_actual_newlines = content_str.replace("\\n", "\n");

            // Iterate over lines within a segment to handle multi-line segments correctly
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
                        // Context lines are prefixed with a space for alignment with diff output
                        writeln!(writer, " {}", segment)?;
                    }
                }
            }
        }
    }
    // Print diff footer to stderr.
    writeln!(io::stderr(), "{}", get_styled_text("-----------------", ThemeEntry::DiffHeader, theme_map, enable_colors))?;
    Ok(())
}