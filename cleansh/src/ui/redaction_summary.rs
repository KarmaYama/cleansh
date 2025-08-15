//! Module for printing redaction summaries and statistics in Cleansh.
//!
//! This module provides functions to present the results of redaction operations
//! and statistical analyses in a human-readable format to the user. It supports
//! detailed summaries for actual redactions (showing original and sanitized values)
//! and statistics-only mode (counting matches and optionally showing samples).
//! Output can be colored based on the application's theme.

use crate::ui::theme::{ThemeEntry, ThemeMap};
use std::collections::HashMap;
use std::io::Write;
use anyhow::Result;

// Import from cleansh_core
use cleansh_core::{RedactionSummaryItem, RedactionMatch, CompiledRules};

// Local imports
use crate::ui::output_format;

/// Prints a summary of actual redactions made to the given writer.
///
/// This function is used for the standard redaction output mode, where content
/// has been modified. It iterates through `RedactionSummaryItem`s, displaying
/// each rule name, its total occurrences, and (optionally) lists of original
/// and sanitized values. Colors are applied based on the theme and whether `enable_colors` is true.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `summary` - A slice of `RedactionSummaryItem`s, each representing a rule
///               that resulted in redactions.
/// * `writer` - The output writer where the summary will be printed (e.g., `&mut io::stdout()`).
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling the output.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// A `Result` indicating `Ok(())` on successful write operations or an `Err`
/// if any writing to the `writer` fails.
pub fn print_summary<W: Write>(
    summary: &[RedactionSummaryItem],
    writer: &mut W,
    theme_map: &ThemeMap,
    enable_colors: bool,
) -> Result<()> {
    if summary.is_empty() {
        writeln!(writer, "\n{}\n", output_format::get_styled_text("No redactions applied.", ThemeEntry::Info, theme_map, enable_colors))?;
        return Ok(());
    }

    let header = output_format::get_styled_text("\n--- Redaction Summary ---", ThemeEntry::Header, theme_map, enable_colors);
    writeln!(writer, "{}", header)?;

    for item in summary {
        let rule_name_styled = output_format::get_styled_text(&item.rule_name, ThemeEntry::SummaryRuleName, theme_map, enable_colors);
        let occurrences_styled = output_format::get_styled_text(
            &format!(" ({} occurrences)", item.occurrences),
            ThemeEntry::SummaryOccurrences,
            theme_map,
            enable_colors,
        );
        writeln!(writer, "{}{}", rule_name_styled, occurrences_styled)?;

        if !item.original_texts.is_empty() {
            writeln!(writer, "    {}", output_format::get_styled_text("Original Values:", ThemeEntry::Info, theme_map, enable_colors))?;
            for text in &item.original_texts {
                let formatted_text = format!("- {}", text);
                let styled_text = output_format::get_styled_text(&formatted_text, ThemeEntry::DiffRemoved, theme_map, enable_colors);
                writeln!(writer, "        {}", styled_text)?;
            }
        }

        if !item.sanitized_texts.is_empty() {
            writeln!(writer, "    {}", output_format::get_styled_text("Sanitized Values:", ThemeEntry::Info, theme_map, enable_colors))?;
            for text in &item.sanitized_texts {
                let formatted_text = format!("+ {}", text);
                let styled_text = output_format::get_styled_text(&formatted_text, ThemeEntry::DiffAdded, theme_map, enable_colors);
                writeln!(writer, "        {}", styled_text)?;
            }
        }
        writeln!(writer)?; // Empty line for separation
    }
    writeln!(writer, "{}\n", output_format::get_styled_text("-------------------------", ThemeEntry::Header, theme_map, enable_colors))?;
    Ok(())
}

/// Prints a detailed summary for the `--stats-only` mode, including optional samples.
///
/// This function is specifically designed for the statistics-only command. It displays
/// the count of matches for each active redaction rule. If `sample_matches_count` is
/// provided and greater than zero, it also lists a specified number of unique
/// original matched strings as examples for each rule.
/// Rule names are formatted using `format_rule_name_for_json` for consistent display.
///
/// # Type Parameters
///
/// * `W`: A type that implements `std::io::Write`.
///
/// # Arguments
///
/// * `aggregated_matches` - A `HashMap` where keys are rule names (`String`) and values
///                          are vectors of references to `RedactionMatch` instances found for that rule.
/// * `compiled_rules` - A reference to the `CompiledRules` instance, used to get all active rule names.
/// * `writer` - The output writer where the statistics will be printed (e.g., `&mut io::stderr()`).
/// * `theme_map` - A `HashMap` containing the defined `ThemeStyle`s for styling the output.
/// * `sample_matches_count` - An `Option<usize>` specifying how many unique sample matches to display
///                          for each rule. If `None` or `0`, no samples are shown.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied.
///
/// # Returns
///
/// A `Result` indicating `Ok(())` on successful write operations or an `Err`
/// if any writing to the `writer` fails.
pub fn print_summary_for_stats_mode<W: Write>(
    aggregated_matches: &HashMap<String, Vec<&RedactionMatch>>,
    compiled_rules: &CompiledRules,
    writer: &mut W,
    theme_map: &ThemeMap,
    sample_matches_count: Option<usize>,
    enable_colors: bool,
) -> Result<()> {
    let header = output_format::get_styled_text("\n--- Redaction Statistics Summary ---", ThemeEntry::Header, theme_map, enable_colors);
    writeln!(writer, "{}", header)?;

    // Get all rule names that were compiled and active, and sort them for consistent output
    let mut active_rule_names: Vec<String> = compiled_rules.rules.iter()
        .map(|r| r.name.clone())
        .collect();
    active_rule_names.sort();

    let mut has_any_matches = false;

    // Iterate by reference for efficiency.
    for rule_name in &active_rule_names {
        let matches_for_rule = aggregated_matches.get(rule_name);
        let total_occurrences = matches_for_rule.map_or(0, |matches| matches.len());

        // Only display rules that actually had matches
        if total_occurrences == 0 {
            continue;
        }

        has_any_matches = true;

        let display_name = format_rule_name_for_json(rule_name);

        let match_plural = if total_occurrences == 1 { "match" } else { "matches" };

        let line_content = format!("{}: {} {}", display_name, total_occurrences, match_plural);
        let styled_line = output_format::get_styled_text(&line_content, ThemeEntry::SummaryRuleName, theme_map, enable_colors);
        writeln!(writer, "{}", styled_line)?;

        if let Some(matches) = matches_for_rule {
            if let Some(num_samples) = sample_matches_count {
                if num_samples > 0 {
                    writeln!(writer, "    {}", output_format::get_styled_text("Sample Matches:", ThemeEntry::Info, theme_map, enable_colors))?;

                    // Collect unique samples to avoid showing duplicates, then sort for consistent output
                    let mut unique_samples: Vec<String> = matches
                        .iter()
                        .map(|m| m.original_string.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();
                    unique_samples.sort();

                    for (i, sample) in unique_samples.iter().take(num_samples).enumerate() {
                        let formatted_sample = format!("- {}", sample);
                        let styled_sample = output_format::get_styled_text(&formatted_sample, ThemeEntry::DiffRemoved, theme_map, enable_colors);
                        writeln!(writer, "        {}", styled_sample)?;
                        
                        // Indicate if there are more unique samples than displayed
                        if i == num_samples - 1 && unique_samples.len() > num_samples {
                            writeln!(writer, "        ... ({} more unique samples)", unique_samples.len() - num_samples)?;
                        }
                    }
                }
            }
        }
    }

    // Message if no matches were found across any active rules
    if !has_any_matches {
        writeln!(writer, "\n{}\n", output_format::get_styled_text("No redaction matches found.", ThemeEntry::Info, theme_map, enable_colors))?;
    }

    writeln!(writer, "{}\n", output_format::get_styled_text("---------------------------------", ThemeEntry::Header, theme_map, enable_colors))?;
    Ok(())
}

/// Prints a styled message when a `--fail-over-threshold` is exceeded in stats mode.
pub fn print_stats_fail_over_message<W: Write>(
    threshold: usize,
    matches_found: usize,
    writer: &mut W,
    theme_map: &ThemeMap,
    enable_colors: bool,
) -> Result<()> {
    let fail_over_msg = format!(
        "FAIL-OVER triggered: Found {} redaction matches, which exceeds the specified threshold of {}.",
        matches_found, threshold
    );
    let styled_msg = output_format::get_styled_text(&fail_over_msg, ThemeEntry::Error, theme_map, enable_colors);
    writeln!(writer, "{}", styled_msg)?;
    Ok(())
}

// A private helper function to format rule names for display, keeping logic local.
fn format_rule_name_for_json(name: &str) -> String {
    name.replace("_", " ").split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}