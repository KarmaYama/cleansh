// src/commands/cleansh.rs

use anyhow::{Context, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use log::{debug, error, info, trace, warn};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config;
use crate::tools::sanitize_shell;
use crate::ui::{self, OutputTheme};

/// Executes the core cleansh sanitization logic.
///
/// This function orchestrates the entire process: loading rules, compiling them,
/// sanitizing the input content, and handling all various output methods
/// (diff view, clipboard, file output, or stdout).
///
/// # Arguments
/// * `input_content` - The raw string content to be sanitized.
/// * `clipboard_enabled` - If true, the sanitized content is copied to the clipboard.
/// * `diff_enabled` - If true, a diff view highlighting changes is printed to stderr.
/// * `config_path` - Optional path to a user-defined YAML configuration file.
/// * `output_path` - Optional path to a file where the sanitized content should be written.
pub fn run_cleansh(
    input_content: &str,
    clipboard_enabled: bool,
    diff_enabled: bool,
    config_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
) -> Result<()> {
    info!("Starting cleansh command execution.");
    debug!("Clipboard enabled: {}", clipboard_enabled);
    debug!("Diff enabled: {}", diff_enabled);
    debug!("Config path: {:?}", config_path);
    debug!("Output path: {:?}", output_path);

    // Get the default UI theme for consistent output formatting.
    let theme = OutputTheme::default();

    // 1. Load and Merge Redaction Rules
    info!("Loading default redaction rules...");
    let default_rules_config = config::load_default_rules()
        .context("Failed to load default redaction rules.")?;
    debug!("Default rules loaded: {} rules.", default_rules_config.rules.len());

    let final_rules_config = if let Some(path) = config_path {
        info!("Loading user-defined redaction rules from: {}", path.display());
        let user_rules_config = config::load_user_rules(&path)
            .with_context(|| format!("Failed to load user-defined rules from: {}", path.display()))?;
        debug!("User-defined rules loaded: {} rules.", user_rules_config.rules.len());
        
        info!("Merging user-defined rules with default rules.");
        let merged_rules = config::merge_rules(default_rules_config, Some(user_rules_config));
        debug!("Total rules after merge: {} rules.", merged_rules.rules.len());
        merged_rules
    } else {
        info!("No custom config specified. Using default rules only.");
        default_rules_config
    };

    // 2. Compile Redaction Rules
    info!("Compiling redaction rules...");
    let compiled_rules = sanitize_shell::compile_rules(final_rules_config)
        .context("Failed to compile redaction rules.")?;
    debug!("Successfully compiled {} redaction rules.", compiled_rules.len());
    trace!("Compiled rules: {:?}", compiled_rules);

    // 3. Perform Sanitization
    info!("Sanitizing content...");
    let (sanitized_content, redaction_summary_items) =
        sanitize_shell::sanitize_content(input_content, &compiled_rules);
    info!("Content sanitization complete. {} redactions made.", redaction_summary_items.len());

    // 4. Handle Output Modes

    // Always print a summary of redactions if any were made.
    if !redaction_summary_items.is_empty() {
        ui::print_redaction_summary(&redaction_summary_items, &theme);
    } else {
        ui::print_info_message("No sensitive information detected for redaction.", &theme);
    }


    // Handle diff view
    if diff_enabled {
        info!("Generating and printing diff view.");
        ui::print_diff_view(input_content, &sanitized_content, &theme);
    }

    // Handle clipboard output
    if clipboard_enabled {
        info!("Attempting to copy sanitized content to clipboard.");
        match ClipboardContext::new() {
            Ok(mut ctx) => {
                if let Err(e) = ctx.set_contents(sanitized_content.clone()) {
                    error!("Failed to copy to clipboard: {}", e);
                    ui::print_error_message(
                        &format!("Failed to copy to clipboard: {}", e),
                        &theme,
                    );
                } else {
                    ui::print_success_message("Sanitized content copied to clipboard.", &theme);
                }
            }
            Err(e) => {
                error!("Could not access clipboard: {}", e);
                ui::print_error_message(
                    &format!("Could not access clipboard: {}", e),
                    &theme,
                );
            }
        }
    }

    // Handle file output or stdout output (mutually exclusive)
    if let Some(path) = output_path {
        info!("Writing sanitized content to file: {}", path.display());
        fs::write(&path, &sanitized_content)
            .with_context(|| format!("Failed to write sanitized content to file: {}", path.display()))?;
        ui::print_success_message(
            &format!("Sanitized content written to: {}", path.display()),
            &theme,
        );
    } else if !diff_enabled {
        // Only print to stdout if not redirecting to file AND diff view is not the primary output.
        // If diff is enabled, the diff is the stdout output, not the raw sanitized content.
        info!("Printing sanitized content to stdout.");
        ui::print_content(&sanitized_content);
    }

    info!("cleansh command execution finished.");
    Ok(())
}