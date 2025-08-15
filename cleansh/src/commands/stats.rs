//! This module handles the `stats` subcommand, which analyzes content for redaction
//! matches without performing any redactions. It's an informational tool for
//! understanding the impact of a profile.
//!
//! License: Polyform Noncommercial License 1.0.0

use crate::cli::ScanCommand;
use crate::ui::theme::ThemeMap;
use crate::ui::redaction_summary;
use anyhow::{Result, Context, anyhow};
use std::io::{self, Read, Write};
use std::fs;
use is_terminal::IsTerminal;
use cleansh_core::engine::SanitizationEngine;
use cleansh_core::RedactionMatch;
use std::collections::HashMap;

/// The main entry point for the `cleansh stats` subcommand.
pub fn run_stats_command(opts: &ScanCommand, theme_map: &ThemeMap, engine: &dyn SanitizationEngine) -> Result<()> {
    // Determine if we should use colors based on the output stream's terminal status.
    // For human-readable summaries, we write to stderr.
    let enable_colors = io::stderr().is_terminal();

    // Read input content
    let input_content = if let Some(path) = &opts.input_file {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read input file: {}", path.display()))?
    } else {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        content
    };

    // Corrected: Provide a default source name when reading from stdin
    let source_name = opts.input_file.clone()
        .unwrap_or_default()
        .display()
        .to_string();
    let source_name = if source_name.is_empty() {
        "stdin".to_string()
    } else {
        source_name
    };

    let all_matches = engine.find_matches_for_ui(&input_content, &source_name)
        .context("Failed to analyze content for statistics")?;

    let mut aggregated_matches: HashMap<String, Vec<&RedactionMatch>> = HashMap::new();
    for m in &all_matches {
        aggregated_matches.entry(m.rule_name.clone()).or_insert_with(Vec::new).push(m);
    }
    
    // --- Fail-over logic for stats command
    // If a threshold is set and the number of matches exceeds it, return an error.
    if let Some(threshold) = opts.fail_over_threshold {
        if all_matches.len() > threshold {
            // Print the specific fail-over message before returning the error
            redaction_summary::print_stats_fail_over_message(
                threshold,
                all_matches.len(),
                &mut io::stderr(),
                theme_map,
                enable_colors,
            ).ok(); // Use .ok() to prevent this write from causing a non-zero exit status

            // Then return the error to trigger a non-zero exit code
            return Err(anyhow!("FAIL-OVER threshold exceeded."));
        }
    }
    // --- End fail-over logic

    // Serialize the summary to JSON, as it's needed for both --json-file and --json-stdout
    #[derive(serde::Serialize)]
    struct StatsSummary {
        redaction_summary: HashMap<String, usize>,
    }
    let summary_map: HashMap<String, usize> = aggregated_matches
        .iter()
        .map(|(rule_name, matches)| (rule_name.clone(), matches.len()))
        .collect();
    let json_output = serde_json::to_string_pretty(&StatsSummary { redaction_summary: summary_map })
        .context("Failed to serialize stats summary to JSON")?;

    if let Some(json_path) = &opts.json_file {
        fs::write(json_path, json_output.as_bytes())
            .with_context(|| format!("Failed to write JSON output to file: {}", json_path.display()))?;
    } else if opts.json_stdout {
        // Correctly handle `--json-stdout`
        io::stdout().write_all(json_output.as_bytes())
            .context("Failed to write JSON output to stdout")?;
        // Print a newline for a clean terminal output after the JSON
        io::stdout().write_all(b"\n")
            .context("Failed to write newline to stdout")?;
    } else {
        redaction_summary::print_summary_for_stats_mode(
            &aggregated_matches,
            engine.compiled_rules(),
            &mut io::stderr(),
            theme_map,
            opts.sample_matches,
            enable_colors,
        ).ok(); // Use .ok() to prevent this write from causing a non-zero exit status
    }

    Ok(())
}