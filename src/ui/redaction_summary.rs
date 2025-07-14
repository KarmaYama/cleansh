// src/ui/redaction_summary.rs

use crate::config::RedactionSummaryItem;
use crate::ui::theme::{ThemeEntry, ThemeStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::Write;
use anyhow::Result; // Import Result for error handling

/// Prints a summary of redactions made to the given writer.
pub fn print_summary<W: Write>(
    summary: &[RedactionSummaryItem],
    writer: &mut W,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> Result<()> {
    if summary.is_empty() {
        writeln!(writer, "\n{}\n", get_styled_text("No redactions applied.", ThemeEntry::Info, theme_map))?;
        return Ok(());
    }

    let header = get_styled_text("\n--- Redaction Summary ---", ThemeEntry::Header, theme_map);
    writeln!(writer, "{}", header)?;

    for item in summary {
        let rule_name_styled = get_styled_text(&item.rule_name, ThemeEntry::SummaryRuleName, theme_map);
        let occurrences_styled = get_styled_text(
            &format!(" ({} occurrences)", item.occurrences),
            ThemeEntry::SummaryOccurrences,
            theme_map,
        );
        writeln!(writer, "{}{}", rule_name_styled, occurrences_styled)?;

        if !item.original_texts.is_empty() {
            writeln!(writer, "  {}", get_styled_text("Original Examples:", ThemeEntry::Info, theme_map))?;
            for text in &item.original_texts {
                writeln!(writer, "    - {}", text.red())?;
            }
        }

        if !item.sanitized_texts.is_empty() {
            writeln!(writer, "  {}", get_styled_text("Sanitized Examples:", ThemeEntry::Info, theme_map))?;
            for text in &item.sanitized_texts {
                writeln!(writer, "    - {}", text.green())?;
            }
        }
    }
    writeln!(writer, "{}\n", get_styled_text("-------------------------", ThemeEntry::Header, theme_map))?;
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