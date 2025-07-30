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
use is_terminal::IsTerminal; // Keep IsTerminal to determine coloring for stderr

// Import from cleansh_core
use cleansh_core::{
    // config as core_config, // Removed: unused after checking
    sanitizer::{self, CompiledRules},
    redaction_match::{log_redaction_match_debug, log_captured_match_debug, log_redaction_action_debug, RedactionMatch},
    // RedactionConfig, // Removed: unused after checking
};

// Local imports
use crate::ui::{output_format, theme, redaction_summary};
use crate::utils::app_state::AppState;
// Updated import: Removed warn_msg as it's no longer in cleansh.rs
use crate::commands::cleansh::{info_msg, error_msg, load_and_compile}; // Import the message helpers and load_and_compile

// Import ThemeMap from the theme module
use crate::ui::theme::ThemeMap;

/// Runs the statistics-only mode logic for Cleansh.
///
/// This function processes the provided `input_content` to identify instances
/// of sensitive data based on configured redaction rules. It compiles the rules,
/// performs a scan, and then presents a summary of the findings. No content
/// is redacted or modified in this mode.
///
/// It also handles features like donation prompts, exporting results to JSON,
/// and an optional fail-over threshold for detected secrets.
///
/// # Arguments
///
/// * `input_content` - The string slice containing the content to be analyzed.
/// * `config_path` - An optional `PathBuf` to a custom YAML configuration file for redaction rules.
/// * `rules_config_name` - An optional `String` specifying a named rule configuration within the config files (e.g., "default", "strict").
/// * `theme_map` - A reference to a `HashMap` containing the application's theme styles.
/// * `enable_rules` - A `Vec<String>` of rule names to explicitly enable, overriding configuration settings.
/// * `disable_rules` - A `Vec<String>` of rule names to explicitly disable, overriding configuration settings.
/// * `stats_json_file_path` - An optional `PathBuf` where the full scan summary should be exported as a JSON file.
/// * `export_json_to_stdout` - A boolean flag; if `true`, the full scan summary will be exported to stdout as JSON.
/// * `sample_matches_count` - An optional `usize` specifying how many unique examples per rule to show in the console stats output.
/// * `fail_over_threshold` - An optional `usize` representing a threshold. If the total number of detected secrets exceeds this,
///                            the application will exit with a non-zero code.
/// * `cli_disable_donation_prompts` - A boolean flag; if `true`, it disables future donation prompts in this session and saves the state.
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) or an error (`Err(anyhow::Error)`)
/// if rule loading, compilation, or output operations fail.
#[allow(clippy::too_many_arguments)]
pub fn run_stats_command(
    input_content: &str,
    config_path: Option<PathBuf>,
    rules_config_name: Option<String>,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_rules: Vec<String>,
    disable_rules: Vec<String>,
    stats_json_file_path: Option<PathBuf>,
    export_json_to_stdout: bool,
    sample_matches_count: Option<usize>,
    fail_over_threshold: Option<usize>,
    cli_disable_donation_prompts: bool,
) -> Result<()> {
    info!("Starting cleansh --stats-only operation.");
    debug!("Starting stats-only operation.");
    debug!("[cleansh::commands::stats] Received enable_rules: {:?}", enable_rules);
    debug!("[cleansh::commands::stats] Received disable_rules: {:?}", disable_rules);

    // --- App State Management (Donation Prompts) ---
    let app_state_file_path = std::env::var("CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("cleansh");
            path.push("app_state.json");
            path
        });

    let mut app_state = AppState::load(&app_state_file_path)?;

    if cli_disable_donation_prompts {
        app_state.donation_prompts_disabled = true;
    }

    // Use the centralized `load_and_compile` function
    let compiled_rules = load_and_compile(
        config_path,
        rules_config_name,
        enable_rules,
        disable_rules,
    )?;
    debug!("[cleansh::commands::stats] Compiled {} rules successfully.", compiled_rules.rules.len());
    debug!("[cleansh::commands::stats] Names of compiled rules available for stats processing:");
    for rule in &compiled_rules.rules {
        debug!("[cleansh::commands::stats] - {}", rule.name);
    }

    let (_, all_redaction_matches) = sanitizer::sanitize_content(input_content, &compiled_rules);
    debug!("[cleansh::commands::stats] Analysis completed. Total individual matches (including those not programmatically validated for redaction): {}", all_redaction_matches.len());

    for m in &all_redaction_matches {
        log_captured_match_debug(
            "[cleansh::commands::stats]",
            &m.rule_name,
            &m.original_string,
        );
        log_redaction_match_debug(
            "[cleansh::commands::stats]",
            &m.rule_name,
            &m.original_string,
            &m.sanitized_string
        );
        log_redaction_action_debug(
            "[cleansh::commands::stats]",
            &m.original_string,
            &m.sanitized_string,
            &m.rule_name,
        );
    }

    if !all_redaction_matches.is_empty() {
        app_state.increment_stats_only_usage();
    }

    let stderr_supports_color = io::stderr().is_terminal(); // Determine color support for stderr

    if !app_state.donation_prompts_disabled && app_state.should_display_donation_prompt() {
        // This is a direct print message, not an info/warn/error, so it's kept as is.
        // It's a special message that doesn't fit the standard info/warn/error pattern.
        output_format::print_message(
            &mut io::stderr(),
            "Hey! You've used Cleansh's stats feature a few times. If you find it valuable, please consider donating at least $1 to Cleansh on GitHub Sponsors to motivate us: https://github.com/sponsors/KarmaYama",
            theme_map,
            Some(theme::ThemeEntry::Info),
            stderr_supports_color, // <--- Corrected: Pass stderr_supports_color here
        )?;
    }
    app_state.save(&app_state_file_path)?;

    display_statistics(
        &all_redaction_matches, // <-- Corrected: Pass a reference to the vector
        &compiled_rules,
        stats_json_file_path,
        export_json_to_stdout,
        sample_matches_count,
        fail_over_threshold,
        theme_map,
        stderr_supports_color, // <--- Pass enable_colors to display_statistics
    )?;

    info!("Cleansh --stats-only operation completed.");
    debug!("[cleansh::commands::stats] Cleansh stats-only operation completed.");
    Ok(())
}

/// Converts a rule name to its desired JSON key format (PascalCase, with special acronym handling).
///
/// This helper function ensures consistency in JSON output for rule names,
/// transforming snake_case names into a more conventional PascalCase, with
/// specific handling for common acronyms (e.g., "aws_access_key" becomes "AWSAccessKey").
/// The "us_ssn" rule is a specific exception to remain lowercase.
///
/// This function is made `pub(crate)` for use in `redaction_summary.rs`.
///
/// # Arguments
///
/// * `rule_name` - The original snake_case name of the rule.
///
/// # Returns
///
/// A `String` representing the formatted rule name suitable for JSON keys.
pub(crate) fn format_rule_name_for_json(rule_name: &str) -> String {
    match rule_name.to_lowercase().as_str() {
        "aws_access_key" => "AWSAccessKey".to_string(),
        "aws_secret_key" => "AWSSecretKey".to_string(),
        "ipv4_address" => "IPv4Address".to_string(),
        "us_ssn" => "us_ssn".to_string(), // Ensure it's always lowercase "us_ssn"
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

/// A struct representing the flat JSON output for redaction statistics.
///
/// This structure is used to serialize the aggregated counts of redaction
/// rule matches into a JSON format, where each key is a formatted rule name
/// and its value is the count of occurrences.
#[derive(Debug, Serialize)]
struct CountsOutput {
    /// A map where keys are formatted rule names (e.g., "AWSAccessKey") and values are their respective counts.
    redaction_summary: HashMap<String, usize>,
}

/// Helper function to display statistics based on the processed redaction matches and CLI options.
///
/// This function is responsible for formatting and outputting the statistics.
/// It can print a human-readable summary to stderr, export a JSON summary to a file,
/// or export a JSON summary to stdout, based on the command-line arguments.
/// It also implements the `fail_over_threshold` logic.
///
/// # Arguments
///
/// * `all_redaction_matches` - A slice of all `RedactionMatch` instances found during the scan.
/// * `compiled_rules` - A reference to the `CompiledRules` instance that was used for the scan.
/// * `stats_json_file_path` - An optional `PathBuf` where the JSON summary should be written.
/// * `export_json_to_stdout` - A boolean; if `true`, the JSON summary is printed to stdout.
/// * `sample_matches_count` - An optional `usize` indicating how many unique sample matches to display per rule.
/// * `fail_over_threshold` - An optional `usize` for the total secrets threshold.
/// * `theme_map` - A reference to a `HashMap` containing theme styles for output formatting.
/// * `enable_colors` - A boolean indicating whether ANSI colors should be applied to console output.
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) or an error (`Err(anyhow::Error)`)
/// if output operations fail or the `fail_over_threshold` is exceeded.
fn display_statistics(
    all_redaction_matches: &[RedactionMatch],
    compiled_rules: &CompiledRules,
    stats_json_file_path: Option<PathBuf>,
    export_json_to_stdout: bool,
    sample_matches_count: Option<usize>,
    fail_over_threshold: Option<usize>,
    theme_map: &ThemeMap, // Use ThemeMap alias
    enable_colors: bool, // <--- Added enable_colors parameter
) -> Result<()> {
    let mut aggregated_matches: HashMap<String, Vec<&RedactionMatch>> = HashMap::new();
    for m in all_redaction_matches {
        aggregated_matches.entry(m.rule_name.clone()).or_default().push(m);
    }

    let total_matches: usize = all_redaction_matches.len();
    debug!("[cleansh::commands::stats] Total matches found (including those failing programmatic validation): {}", total_matches);

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

    let mut counts_map: HashMap<String, usize> = HashMap::new();
    for (rule_name, matches_for_rule) in aggregated_matches.iter() {
        let json_rule_name = format_rule_name_for_json(rule_name);
        counts_map.insert(json_rule_name, matches_for_rule.len());
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

    // Only print human-readable summary if not exporting JSON to stdout
    if !export_json_to_stdout {
        info_msg("Redaction Statistics Summary:", theme_map);
        // Use io::stderr() directly here as it's always a terminal for error messages
        redaction_summary::print_summary_for_stats_mode(
            &aggregated_matches,
            compiled_rules,
            &mut io::stderr(),
            theme_map,
            sample_matches_count,
            enable_colors,
        )?;
    }

    Ok(())
}