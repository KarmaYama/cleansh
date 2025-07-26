// src/commands/cleansh.rs
//! Cleansh command implementation for sanitizing terminal output.
// This module handles the main functionality of Cleansh, including
//! reading input, applying redaction rules, and outputting sanitized content.

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::io::{self, Write, IsTerminal};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use crate::ui::diff_viewer;
use crate::ui::redaction_summary;


use crate::config::{self, RedactionConfig, RedactionSummaryItem};
use crate::tools::sanitize_shell;
use crate::ui::{output_format, theme};
// Import the centralized logging function for RedactionMatch
use crate::utils::redaction::{log_redaction_match_debug, RedactionMatch};

/// Runs the core sanitization logic.
///
/// This function orchestrates the loading of rules, content sanitization,
/// and output/clipboard operations based on user preferences.
#[allow(clippy::too_many_arguments)] // This is acceptable for a main command function
pub fn run_cleansh(
    input_content: &str,
    clipboard_enabled: bool,
    diff_enabled: bool,
    config_path: Option<PathBuf>,
    rules_config_name: Option<String>,
    output_path: Option<PathBuf>,
    no_redaction_summary: bool,
    theme_map: &std::collections::HashMap<theme::ThemeEntry, theme::ThemeStyle>,
    enable_rules: Vec<String>,
    disable_rules: Vec<String>,
) -> Result<()> {
    info!("Starting cleansh operation.");
    debug!("[cleansh.rs] Starting cleansh operation.");
    debug!("[cleansh.rs] Received enable_rules: {:?}", enable_rules);
    debug!("[cleansh.rs] Received disable_rules: {:?}", disable_rules);


    let default_rules = RedactionConfig::load_default_rules()?;
    debug!("[cleansh.rs] Loaded {} default rules in cleansh.", default_rules.rules.len());


    let user_rules = if let Some(path) = config_path {
        info!("Loading custom rules from: {}", path.display());
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            &format!("Loading custom rules from: {}", path.display()),
            theme_map,
        );
        debug!("[cleansh.rs] Attempting to load custom rules from: {}", path.display());
        let loaded_custom_rules = RedactionConfig::load_from_file(&path).with_context(|| {
            format!(
                "Failed to load custom configuration from '{}'",
                path.display()
            )
        })?;
        debug!("[cleansh.rs] Loaded {} custom rules from {} in cleansh.", loaded_custom_rules.rules.len(), path.display());
        Some(loaded_custom_rules)
    } else {
        debug!("[cleansh.rs] No custom config path provided in cleansh.");
        None
    };

    let mut merged_config = config::merge_rules(default_rules, user_rules);
    debug!("[cleansh.rs] Merged config contains {} rules before compilation in cleansh.", merged_config.rules.len());

    // Apply rule configuration name if provided
    if let Some(name) = rules_config_name {
        merged_config.set_active_rules_config(&name)?;
        debug!("[cleansh.rs] Active rules config set to: {}", name);
    }


    debug!("Compiling rules...");
    debug!("[cleansh.rs] Calling compile_rules with {} rules, enable_rules: {:?}, disable_rules: {:?}",
        merged_config.rules.len(), enable_rules, disable_rules);
    // Pass the merged rules directly to compile_rules
    let compiled_rules = sanitize_shell::compile_rules(
        merged_config.rules, // Pass the Vec<RedactionRule>
        &enable_rules,
        &disable_rules,
    )?;
    debug!("Rules compiled successfully.");
    debug!("[cleansh.rs] Compiled {} rules successfully in cleansh.", compiled_rules.rules.len());

    // --- NEW DEBUG LINE ---
    debug!("[cleansh.rs] Names of compiled rules available for sanitization:");
    for rule in &compiled_rules.rules {
        debug!("[cleansh.rs] - {}", rule.name);
    }
    // --- END NEW DEBUG LINE ---


    // Perform sanitization
    // sanitize_content no longer returns a Result, handles its own errors
    let (sanitized_content, all_redaction_matches) =
        sanitize_shell::sanitize_content(input_content, &compiled_rules);
    debug!(
        "Content sanitized. Original length: {}, Sanitized length: {}",
        input_content.len(),
        sanitized_content.len()
    );

    // MODIFIED DEBUG LOGGING FOR REDACTION MATCHES IN CLEASH COMMAND
    // Now uses the centralized `log_redaction_match_debug` function
    for m in &all_redaction_matches {
        log_redaction_match_debug(
            "[cleansh::commands::cleansh]",
            &m.rule_name,
            &m.original_string,
            &m.sanitized_string
        );
    }
    // END MODIFIED DEBUG LOGGING


    // Build the RedactionSummaryItem from the raw RedactionMatch vector
    let summary = build_redaction_summary_from_matches(&all_redaction_matches);
    debug!("DEBUG_CLEANSH: Redaction summary (num items): {:?}", summary.len());


    // Determine the primary output writer (stdout or file) and if it supports colors
    let (mut primary_output_writer, output_supports_color): (Box<dyn Write>, bool) = if let Some(path) = output_path {
        info!("Writing sanitized content to file: {}", path.display());
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            &format!("Writing sanitized content to file: {}", path.display()),
            theme_map,
        );
        debug!("[cleansh.rs] Outputting to file: {}", path.display());
        (
            Box::new(
                fs::File::create(&path)
                    .with_context(|| format!("Failed to create output file: {}", path.display()))?,
            ),
            false, // Files generally do not support ANSI colors, so explicitly set to false
        )
    } else {
        info!("Writing sanitized content to stdout.");
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            "Writing sanitized content to stdout.",
            theme_map,
        );
        debug!("[cleansh.rs] Outputting to stdout.");
        let stdout = io::stdout();
        let supports_color = stdout.is_terminal(); // Check if stdout is connected to a TTY
        (Box::new(stdout), supports_color)
    };

    // Output logic
    if diff_enabled {
        debug!("Generating and displaying diff.");
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            "Generating and displaying diff.",
            theme_map,
        );
        debug!("[cleansh.rs] Diff enabled.");
        // Pass the output_supports_color flag to print_diff
        diff_viewer::print_diff(input_content, &sanitized_content, &mut primary_output_writer, theme_map, output_supports_color)?;
    } else {
        debug!("Printing sanitized content.");
        debug!("[cleansh.rs] Diff disabled, printing sanitized content.");
        // When not in diff mode, just write the sanitized_content.
        // `sanitize_shell::sanitize_content` ensures the `sanitized_content` itself is plain text
        // (by stripping input ANSI), so no further stripping is needed here.
        writeln!(primary_output_writer, "{}", sanitized_content)
            .context("Failed to write sanitized content")?;
    }

    // Redaction Summary handling (always to stderr, so always check stderr's TTY)
    if !no_redaction_summary {
        debug!("Displaying redaction summary.");
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            "Displaying redaction summary.",
            theme_map,
        );
        debug!("[cleansh.rs] Redaction summary enabled.");
        redaction_summary::print_summary(&summary, &mut io::stderr(), theme_map)?;
    } else {
        debug!("Redaction summary display skipped per user request.");
        let _ = output_format::print_info_message( // Wrapped with `let _ =`
            &mut io::stderr(),
            "Redaction summary display skipped per user request.",
            theme_map,
        );
        debug!("[cleansh.rs] Redaction summary skipped.");
    }

    // Clipboard handling
    if clipboard_enabled {
        debug!("Attempting to copy sanitized content to clipboard.");
        debug!("[cleansh.rs] Clipboard enabled.");
        match copy_to_clipboard(&sanitized_content) {
            Ok(_) => {
                info!("Sanitized content copied to clipboard successfully.");
                let _ = output_format::print_info_message( // Wrapped with `let _ =`
                    &mut io::stderr(),
                    "Sanitized content copied to clipboard successfully.",
                    theme_map,
                );
            },
            Err(e) => {
                warn!("Failed to copy to clipboard: {}", e);
                let _ = output_format::print_warn_message( // Wrapped with `let _ =`
                    &mut io::stderr(),
                    &format!("Failed to copy to clipboard: {}", e),
                    theme_map,
                );
            }
        }
    }

    info!("Cleansh operation completed.");
    debug!("[cleansh.rs] Cleansh operation completed.");
    Ok(())
}

/// Helper function to copy content to the system clipboard.
/// This function is conditionally compiled based on the "clipboard" feature.
#[cfg(feature = "clipboard")]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Attempting to acquire clipboard.");
    debug!("[cleansh.rs] Acquiring clipboard.");
    let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
    debug!("Setting clipboard text.");
    debug!("[cleansh.rs] Setting clipboard text.");
    clipboard.set_text(content.to_string()).context("Failed to set clipboard text")?;
    Ok(())
}

/// Placeholder function for when the "clipboard" feature is not enabled.
#[cfg(not(feature = "clipboard"))]
#[allow(unused_variables)]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Clipboard feature not enabled. Skipping copy operation.");
    debug!("[cleansh.rs] Clipboard feature not enabled.");
    Err(anyhow::anyhow!("Clipboard feature is not enabled. Compile with --features clipboard to enable functionality."))
}

/// Builds a `Vec<RedactionSummaryItem>` from a `Vec<RedactionMatch>`.
/// This aggregates individual matches into a summary grouped by rule.
fn build_redaction_summary_from_matches(
    matches: &[RedactionMatch],
) -> Vec<RedactionSummaryItem> {
    let mut summary_map: HashMap<String, RedactionSummaryItem> = HashMap::new();

    for m in matches {
        let item = summary_map.entry(m.rule_name.clone()).or_insert_with(|| RedactionSummaryItem {
            rule_name: m.rule_name.clone(),
            occurrences: 0,
            original_texts: Vec::new(),
            sanitized_texts: Vec::new(),
        });
        item.occurrences += 1;
        // Only add unique original and sanitized strings
        if !item.original_texts.contains(&m.original_string) {
            item.original_texts.push(m.original_string.clone());
        }
        if !item.sanitized_texts.contains(&m.sanitized_string) {
            item.sanitized_texts.push(m.sanitized_string.clone());
        }
    }

    // Sort original_texts and sanitized_texts within each summary item for consistent output
    for item in summary_map.values_mut() {
        item.original_texts.sort();
        item.sanitized_texts.sort();
    }

    let mut summary: Vec<RedactionSummaryItem> = summary_map.into_values().collect();
    // Sort the overall summary by rule name for deterministic output/tests
    summary.sort_by(|a, b| a.rule_name.cmp(&b.rule_name));

    summary
}