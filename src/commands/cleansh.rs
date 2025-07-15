// src/commands/cleansh.rs


use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;
use crate::ui::diff_viewer;
use crate::ui::redaction_summary;


use crate::config::{self, RedactionConfig};
use crate::tools::sanitize_shell;
use crate::ui::{output_format, theme};

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
        output_format::print_info_message(
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

    let merged_config = config::merge_rules(default_rules, user_rules);
    debug!("[cleansh.rs] Merged config contains {} rules before compilation in cleansh.", merged_config.rules.len());


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
    let (sanitized_content, summary) =
        sanitize_shell::sanitize_content(input_content, &compiled_rules);
    debug!(
        "Content sanitized. Original length: {}, Sanitized length: {}",
        input_content.len(),
        sanitized_content.len()
    );

    
    debug!("DEBUG_CLEANSH: Redaction summary (num items): {:?}", summary.len());


    // Determine the primary output writer (stdout or file)
    let mut primary_output_writer: Box<dyn Write> = if let Some(path) = output_path {
        info!("Writing sanitized content to file: {}", path.display());
        output_format::print_info_message(
            &mut io::stderr(),
            &format!("Writing sanitized content to file: {}", path.display()),
            theme_map,
        );
        debug!("[cleansh.rs] Outputting to file: {}", path.display());
        Box::new(
            fs::File::create(&path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?,
        )
    } else {
        info!("Writing sanitized content to stdout.");
        output_format::print_info_message(
            &mut io::stderr(),
            "Writing sanitized content to stdout.",
            theme_map,
        );
        debug!("[cleansh.rs] Outputting to stdout.");
        Box::new(io::stdout())
    };

    // Output logic
    if diff_enabled {
        debug!("Generating and displaying diff.");
        output_format::print_info_message(
            &mut io::stderr(),
            "Generating and displaying diff.",
            theme_map,
        );
        debug!("[cleansh.rs] Diff enabled.");
        diff_viewer::print_diff(input_content, &sanitized_content, &mut primary_output_writer, theme_map)?;
    } else {
        debug!("Printing sanitized content.");
        debug!("[cleansh.rs] Diff disabled, printing sanitized content.");
        writeln!(primary_output_writer, "{}", sanitized_content)
            .context("Failed to write sanitized content")?;
    }

    // Redaction Summary handling
    if !no_redaction_summary {
        debug!("Displaying redaction summary.");
        output_format::print_info_message(
            &mut io::stderr(),
            "Displaying redaction summary.",
            theme_map,
        );
        debug!("[cleansh.rs] Redaction summary enabled.");
        redaction_summary::print_summary(&summary, &mut io::stderr(), theme_map)?;
    } else {
        debug!("Redaction summary display skipped per user request.");
        output_format::print_info_message(
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
        // Replaced `?` with a `match` block to gracefully handle clipboard errors
        match copy_to_clipboard(&sanitized_content) {
            Ok(_) => {
                info!("Sanitized content copied to clipboard successfully.");
                output_format::print_info_message(
                    &mut io::stderr(),
                    "Sanitized content copied to clipboard successfully.",
                    theme_map,
                );
            },
            Err(e) => {
                warn!("Failed to copy to clipboard: {}", e);
                output_format::print_warn_message(
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