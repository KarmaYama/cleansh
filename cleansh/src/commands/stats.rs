//! Cleansh Statistics Command (`--stats-only`).
//!
//! This module implements the `--stats-only` command for Cleansh,
//! which analyzes input content for sensitive data without performing actual redaction.
//! It focuses on identifying and counting rule matches, providing options to
//! display a summary to the console or export it as a JSON file.
//! This mode is useful for auditing or understanding the potential impact of redaction rules
//! without modifying the original content.

use anyhow::{Context, Result};
use log::{debug, info};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;
use serde::Serialize;
use is_terminal::IsTerminal;

// Import from cleansh_core
use cleansh_core::{
    engine::SanitizationEngine, // Import the SanitizationEngine trait
    RedactionSummaryItem,
};

// Local imports
use crate::ui::{redaction_summary};
use crate::commands::cleansh::{info_msg, error_msg};
use crate::ui::theme::{ThemeMap};
use crate::cli::StatsCommand; // Import the StatsCommand struct

/// Runs the statistics-only mode logic for Cleansh.
pub fn run_stats_command(
    input_content: &str,
    engine: &dyn SanitizationEngine,
    theme_map: &ThemeMap,
    opts: &StatsCommand, // Pass the StatsCommand struct directly
) -> Result<()> {
    info!("Starting cleansh --stats-only operation.");
    debug!("Starting stats-only operation.");

    // The engine's sanitize method now returns the summary directly.
    let (_, summary) = engine.sanitize(input_content)?;

    debug!("[cleansh::commands::stats] Analysis completed. Total matches: {}", summary.iter().map(|item| item.occurrences).sum::<usize>());
    
    let stderr_supports_color = io::stderr().is_terminal();

    display_statistics(
        &summary,
        theme_map,
        &opts.json_file,
        opts.json_stdout,
        opts.sample_matches,
        opts.fail_over_threshold,
        stderr_supports_color,
    )?;

    info!("Cleansh --stats-only operation completed.");
    debug!("[cleansh::commands::stats] Cleansh stats-only operation completed.");
    Ok(())
}

/// Converts a rule name to its desired JSON key format (PascalCase, with special acronym handling).
pub(crate) fn format_rule_name_for_json(rule_name: &str) -> String {
    // This logic remains the same
    match rule_name.to_lowercase().as_str() {
        "aws_access_key" => "AWSAccessKey".to_string(),
        "aws_secret_key" => "AWSSecretKey".to_string(),
        "ipv4_address" => "IPv4Address".to_string(),
        "us_ssn" => "us_ssn".to_string(),
        "email" => "EmailAddress".to_string(),
        "jwt_token" => "JWTToken".to_string(),
        "uk_nino" => "UKNINO".to_string(),
        "sa_id" => "SAID".to_string(),
        _ => {
            rule_name.split('_')
                .map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<String>()
        }
    }
}

#[derive(Debug, Serialize)]
struct CountsOutput {
    redaction_summary: HashMap<String, usize>,
}

fn display_statistics(
    summary: &[RedactionSummaryItem],
    theme_map: &ThemeMap,
    stats_json_file_path: &Option<PathBuf>,
    export_json_to_stdout: bool,
    _sample_matches_count: Option<usize>,
    fail_over_threshold: Option<usize>,
    enable_colors: bool,
) -> Result<()> {
    
    let total_matches: usize = summary.iter().map(|item| item.occurrences).sum();
    debug!("[cleansh::commands::stats] Total matches found: {}", total_matches);
    
    // Fail-over logic
    if let Some(threshold) = fail_over_threshold {
        if total_matches > threshold {
            error_msg(
                &format!("Fail-over triggered: Total secrets ({}) exceeded threshold ({}).", total_matches, threshold),
                theme_map,
            );
            std::process::exit(1);
        } else {
            info_msg(
                &format!("Total secrets ({}) are below the fail-over threshold ({}).", total_matches, threshold),
                theme_map,
            );
        }
    }
    
    // JSON output logic
    let mut counts_map: HashMap<String, usize> = HashMap::new();
    for item in summary.iter() {
        let json_rule_name = format_rule_name_for_json(&item.rule_name);
        counts_map.insert(json_rule_name, item.occurrences);
    }
    
    let full_output = CountsOutput {
        redaction_summary: counts_map,
    };
    
    let json_content = serde_json::to_string_pretty(&full_output)
        .context("Failed to serialize scan summary to JSON")?;
    
    if let Some(json_path) = stats_json_file_path {
        info!("Exporting scan summary to JSON file: {}", json_path.display());
        info_msg(&format!("Exporting scan summary to JSON file: {}", json_path.display()), theme_map);
        fs::write(&json_path, &json_content)
            .with_context(|| format!("Failed to write JSON summary to file: {}", json_path.display()))?;
        info!("JSON summary exported to file successfully.");
    }
    
    if export_json_to_stdout {
        let mut stdout = io::stdout();
        stdout.write_all(json_content.as_bytes())?;
        stdout.write_all(b"\n")?;
        info!("JSON summary exported to stdout successfully.");
    }
    
    // Human-readable summary logic
    if !export_json_to_stdout {
        info_msg("Redaction Statistics Summary:", theme_map);
        redaction_summary::print_summary(&summary, &mut io::stderr(), theme_map, enable_colors)?;
    }

    Ok(())
}
