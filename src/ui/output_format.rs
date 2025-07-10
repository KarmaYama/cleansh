// src/ui/output_format.rs
use crate::config::RedactionSummaryItem;
use crate::ui::theme::{ThemeEntry, ThemeStyle}; // Removed ThemeColor, as it's not directly used
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::Write;
use dissimilar;

/// Prints the content to the given writer.
pub fn print_content<W: Write>(writer: &mut W, content: &str) {
    let _ = write!(writer, "{}", content);
}

/// Helper to get a styled string based on the theme.
/// Returns an owned String that implements Display.
fn get_styled_text(
    text: &str,
    entry: ThemeEntry,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> String {
    if let Some(style) = theme_map.get(&entry) {
        if let Some(color) = &style.fg {
            return text.color(color.to_ansi_color()).to_string(); // Convert to owned String
        }
    }
    // Fallback if no specific style or color is found
    text.color(owo_colors::AnsiColors::White).to_string() // Convert to owned String
}

/// Prints a success message to the given writer, styled by the theme.
pub fn print_success_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("✅ {}\n", message), ThemeEntry::Success, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints an informational message to the given writer, styled by the theme.
pub fn print_info_message<W: Write>( // Corrected: removed extra 'fn'
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("ℹ️ {}\n", message), ThemeEntry::Info, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints an error message to the given writer, styled by the theme.
pub fn print_error_message<W: Write>(
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("❌ ERROR: {}\n", message), ThemeEntry::Error, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints a warning message to the given writer, styled by the theme.
pub fn print_warn_message<W: Write>( // Corrected: removed extra 'fn'
    writer: &mut W,
    message: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let styled_message = get_styled_text(&format!("⚠️ WARNING: {}\n", message), ThemeEntry::Warn, theme_map);
    let _ = write!(writer, "{}", styled_message);
}

/// Prints a summary of redactions made.
pub fn print_redaction_summary<W: Write>(
    writer: &mut W,
    summary: &[RedactionSummaryItem],
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    if summary.is_empty() {
        let _ = write!(writer, "\n{}\n", get_styled_text("No redactions applied.", ThemeEntry::Info, theme_map));
        return;
    }

    let header = get_styled_text("\n--- Redaction Summary ---", ThemeEntry::Header, theme_map);
    let _ = writeln!(writer, "{}", header);

    for item in summary {
        let rule_name_styled = get_styled_text(&item.rule_name, ThemeEntry::SummaryRuleName, theme_map);
        let occurrences_styled = get_styled_text(
            &format!(" ({} occurrences)", item.occurrences),
            ThemeEntry::SummaryOccurrences,
            theme_map,
        );
        let _ = writeln!(writer, "{}{}", rule_name_styled, occurrences_styled);
    }
    let _ = writeln!(writer, "{}\n", get_styled_text("-------------------------", ThemeEntry::Header, theme_map));
}

/// Prints a diff view of the original and sanitized content.
pub fn print_diff_view<W: Write>(
    writer: &mut W,
    original_content: &str,
    sanitized_content: &str,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) {
    let diff_header = get_styled_text("\n--- Diff View ---", ThemeEntry::DiffHeader, theme_map);
    let _ = writeln!(writer, "{}", diff_header);

    let diff = dissimilar::diff(original_content, sanitized_content);

    for change in diff {
        match change {
            dissimilar::Chunk::Equal(s) => {
                let _ = write!(writer, " {}", s); // No styling for equal parts
            }
            dissimilar::Chunk::Delete(s) => {
                let styled_deleted = get_styled_text(&format!("-{}", s), ThemeEntry::DiffRemoved, theme_map);
                let _ = write!(writer, "{}", styled_deleted);
            }
            dissimilar::Chunk::Insert(s) => {
                let styled_inserted = get_styled_text(&format!("+{}", s), ThemeEntry::DiffAdded, theme_map);
                let _ = write!(writer, "{}", styled_inserted);
            }
        }
    }
    let _ = writeln!(writer, "\n{}", get_styled_text("-----------------", ThemeEntry::DiffHeader, theme_map));
}