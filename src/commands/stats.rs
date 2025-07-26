// Statistics command for Cleansh
//! This module implements the `--stats-only` command for Cleansh,
//! which analyzes input content for sensitive data without performing redaction.
// src/commands/stats.rs

use anyhow::{Context, Result};
use log::{debug, info};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;
use serde::Serialize;

use crate::config::{self, RedactionConfig};
use crate::tools::sanitize_shell::{self, CompiledRules}; // Import CompiledRules
use crate::ui::{output_format, theme, redaction_summary};
use crate::utils::app_state::AppState;
use crate::utils::redaction::RedactionMatch;

/// Runs the statistics-only mode logic.
///
/// This function processes the input to identify rule matches and
/// displays a summary of these matches. It does not perform redaction.
#[allow(clippy::too_many_arguments)]
pub fn run_stats_command(
    input_content: &str,
    config_path: Option<PathBuf>,
    rules_config_name: Option<String>,
    theme_map: &std::collections::HashMap<theme::ThemeEntry, theme::ThemeStyle>,
    enable_rules: Vec<String>,
    disable_rules: Vec<String>,
    stats_json_file_path: Option<PathBuf>,
    export_json_to_stdout: bool,
    sample_matches_count: Option<usize>,
    fail_over_threshold: Option<usize>,
    cli_disable_donation_prompts: bool,
) -> Result<()> {
    info!("Starting cleansh --stats-only operation.");
    debug!("[stats.rs] Starting stats-only operation.");
    debug!("[stats.rs] Received enable_rules: {:?}", enable_rules);
    debug!("[stats.rs] Received disable_rules: {:?}", disable_rules);

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

    let default_rules = RedactionConfig::load_default_rules()?;
    debug!("[stats.rs] Loaded {} default rules.", default_rules.rules.len());

    let user_rules = if let Some(path) = config_path {
        info!("Loading custom rules from: {}", path.display());
        output_format::print_info_message(
            &mut io::stderr(),
            &format!("Loading custom rules from: {}", path.display()),
            theme_map,
        );
        debug!("[stats.rs] Attempting to load custom rules from: {}", path.display());
        let loaded_custom_rules = RedactionConfig::load_from_file(&path).with_context(|| {
            format!(
                "Failed to load custom configuration from '{}'",
                path.display()
            )
        })?;
        debug!("[stats.rs] Loaded {} custom rules from {}.", loaded_custom_rules.rules.len(), path.display());
        Some(loaded_custom_rules)
    } else {
        debug!("[stats.rs] No custom config path provided.");
        None
    };

    let mut merged_config = config::merge_rules(default_rules, user_rules);
    debug!("[stats.rs] Merged config contains {} rules before compilation.", merged_config.rules.len());

    if let Some(name) = rules_config_name {
        merged_config.set_active_rules_config(&name)?;
        debug!("[stats.rs] Active rules config set to: {}", name);
    }

    debug!("Compiling rules for stats mode...");
    let compiled_rules = sanitize_shell::compile_rules(
        merged_config.rules,
        &enable_rules,
        &disable_rules,
    )?;
    debug!("[stats.rs] Compiled {} rules successfully.", compiled_rules.rules.len());
    // --- NEW DEBUG LINE FOR STATS COMMAND ---
    debug!("[stats.rs] Names of compiled rules available for stats processing:");
    for rule in &compiled_rules.rules {
        debug!("[stats.rs] - {}", rule.name);
    }
    // --- END NEW DEBUG LINE FOR STATS COMMAND ---


    // --- Perform analysis (no redaction, just get all matches) ---
    // The `all_redaction_matches` now correctly contains all regex matches,
    // regardless of whether they pass programmatic validation or are ultimately redacted.
    let (_, all_redaction_matches) = sanitize_shell::sanitize_content(input_content, &compiled_rules);
    debug!("[stats.rs] Analysis completed. Total individual matches (including those not programmatically validated for redaction): {}", all_redaction_matches.len());
    // --- NEW DEBUG LINE FOR REDACTION MATCHES IN STATS COMMAND ---
    // Only emit detailed match logs if PII debug is explicitly enabled
    if std::env::var("CLEANSH_ALLOW_DEBUG_PII").is_ok() {
        for m in &all_redaction_matches {
            debug!("[stats.rs] Found RedactionMatch: Rule='{}', Original='{}', Sanitized='{}'",
                m.rule_name, m.original_string, m.sanitized_string);
        }
    }
    // --- END NEW DEBUG LINE ---

    // --- CONDITIONALLY INCREMENT STATS ONLY USAGE ---
    // Increment usage count ONLY if actual matches were found during the analysis.
    if !all_redaction_matches.is_empty() {
        app_state.increment_stats_only_usage();
    }

    if !app_state.donation_prompts_disabled && app_state.should_display_donation_prompt() {
        output_format::print_message(
            &mut io::stderr(),
            "Hey! You've used Cleansh's stats feature a few times. If you find it valuable, please consider donating at least $1 to Cleansh on GitHub Sponsors to motivate us: https://github.com/sponsors/KarmaYama",
            theme_map,
            Some(theme::ThemeEntry::Info),
        );
    }
    app_state.save(&app_state_file_path)?;
    // --- End App State Management ---


    // --- Process and display statistics ---
    display_statistics(
        &all_redaction_matches,
        &compiled_rules, // Pass compiled_rules here
        stats_json_file_path,
        export_json_to_stdout,
        sample_matches_count,
        fail_over_threshold,
        theme_map,
    )?;

    info!("Cleansh --stats-only operation completed.");
    debug!("[stats.rs] Cleansh stats-only operation completed.");
    Ok(())
}

/// Converts a rule name to its desired JSON key format (PascalCase, with special acronym handling).
/// This handles cases like "aws_access_key" -> "AWSAccessKey" and "ipv4_address" -> "IPv4Address".
/// This function is made `pub(crate)` for use in `redaction_summary.rs`.
pub(crate) fn format_rule_name_for_json(rule_name: &str) -> String {
    match rule_name.to_lowercase().as_str() {
        "aws_access_key" => "AWSAccessKey".to_string(),
        "aws_secret_key" => "AWSSecretKey".to_string(),
        "ipv4_address" => "IPv4Address".to_string(),
        "ssn_us" => "SSN_US".to_string(),
        "email" => "EmailAddress".to_string(),
        "jwt_token" => "JWTToken".to_string(),
        "uk_nino" => "UKNINO".to_string(),
        "sa_id" => "SAID".to_string(),
        // Generic PascalCase conversion for other snake_case names
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


/// Helper to display statistics based on the summary and CLI options.
fn display_statistics(
    all_redaction_matches: &[RedactionMatch],
    compiled_rules: &CompiledRules, // Add this parameter
    stats_json_file_path: Option<PathBuf>,
    export_json_to_stdout: bool,
    sample_matches_count: Option<usize>,
    fail_over_threshold: Option<usize>,
    theme_map: &std::collections::HashMap<theme::ThemeEntry, theme::ThemeStyle>,
) -> Result<()> {
    // Aggregate matches by rule name for easier processing.
    // This `aggregated_matches` map *already* contains all regex matches,
    // regardless of whether they pass programmatic validation or are ultimately redacted.
    let mut aggregated_matches: HashMap<String, Vec<&RedactionMatch>> = HashMap::new();
    for m in all_redaction_matches {
        aggregated_matches.entry(m.rule_name.clone()).or_default().push(m);
    }

    // Calculate total matches for --fail-over
    let total_matches: usize = all_redaction_matches.len();
    debug!("[stats.rs] Total matches found (including those failing programmatic validation): {}", total_matches);

    // --fail-over logic
    if let Some(threshold) = fail_over_threshold {
        if total_matches > threshold {
            output_format::print_error_message(
                &mut io::stderr(),
                &format!("Fail-over triggered: Total secrets ({}) exceeded threshold ({}).", total_matches, threshold),
                theme_map,
            );
            std::process::exit(1); // Exit with non-zero code
        } else {
            output_format::print_info_message(
                &mut io::stderr(),
                &format!("Total secrets ({}) are below the fail-over threshold ({}).", total_matches, threshold),
                theme_map,
            );
        }
    }

    // Prepare serializable summary
    let serializable_summary: HashMap<String, RuleStats> = aggregated_matches
        .iter()
        .map(|(rule_name, matches_for_rule)| {
            let mut unique_samples: Vec<String> = matches_for_rule
                .iter()
                .map(|m| m.original_string.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            unique_samples.sort();

            let samples_to_include: Vec<String> = if let Some(n) = sample_matches_count {
                unique_samples.into_iter().take(n).collect()
            } else {
                Vec::new()
            };

            // Use the new helper function to format the rule name for JSON
            let json_rule_name = format_rule_name_for_json(rule_name);

            (
                json_rule_name, // Use the specially formatted name as the key
                RuleStats {
                    count: matches_for_rule.len(), // This count includes all regex matches
                    samples: if samples_to_include.is_empty() { None } else { Some(samples_to_include) },
                },
            )
        })
        .collect();

    // Create a top-level JSON structure
    #[derive(Debug, Serialize)]
    struct FullStatsOutput {
        redaction_summary: HashMap<String, RuleStats>,
        // Add other top-level stats if needed, e.g., total_matches: usize,
    }

    let full_output = FullStatsOutput {
        redaction_summary: serializable_summary,
        // total_matches: total_matches, // Example of adding more top-level data
    };

    // Serialize to JSON string
    let json_content = serde_json::to_string_pretty(&full_output)
        .context("Failed to serialize scan summary to JSON")?;

    // --- TEMPORARY DEBUGGING: Print JSON to stderr ---
    // You can uncomment this block temporarily if you still encounter issues
    // and need to see the exact JSON output during test runs.
    // eprintln!("DEBUG JSON Output:\n{}", json_content);
    // --------------------------------------------------

    // --stats-json_file (file output)
    if let Some(json_path) = stats_json_file_path {
        info!("Exporting scan summary to JSON file: {}", json_path.display());
        output_format::print_info_message(
            &mut io::stderr(),
            &format!("Exporting scan summary to JSON file: {}", json_path.display()),
            theme_map,
        );
        fs::write(&json_path, &json_content)
            .with_context(|| format!("Failed to write JSON summary to file: {}", json_path.display()))?;
        info!("JSON summary exported to file successfully.");
    }

    // --export-json-to-stdout (stdout output)
    if export_json_to_stdout {
        // Output to stdout. Don't print info messages to stderr if stdout is being used for JSON.
        // This is crucial for proper piping and machine-readable output.
        // Only print if stdout is a TTY or if it's explicitly requested.
        // Given your tests rely on stdout, we'll write it directly.
        let mut stdout = io::stdout();
        stdout.write_all(json_content.as_bytes())?;
        stdout.write_all(b"\n")?; // Ensure a newline at the end
        info!("JSON summary exported to stdout successfully.");
    }

    // Display human-readable summary to stderr (unless JSON to stdout is explicitly requested,
    // in which case, we assume machine readability is primary and human output is suppressed).
    if !export_json_to_stdout { // Only print human readable if not exporting JSON to stdout
        output_format::print_info_message(
            &mut io::stderr(),
            "Redaction Statistics Summary:",
            theme_map,
        );
        // MODIFIED: Pass compiled_rules to print_summary_for_stats_mode
        redaction_summary::print_summary_for_stats_mode(
            &aggregated_matches,
            compiled_rules, // Pass compiled rules here
            &mut io::stderr(), // Print human-readable summary to stderr
            theme_map,
            sample_matches_count,
        )?;
    }

    Ok(())
}

/// Helper struct for JSON serialization of rule statistics.
#[derive(Debug, Serialize)]
struct RuleStats {
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    samples: Option<Vec<String>>,
}