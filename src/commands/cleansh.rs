/// Contains the main command logic for `cleansh`.
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

    let default_rules = RedactionConfig::load_default_rules()?;
    let user_rules = if let Some(path) = config_path {
        info!("Loading custom rules from: {}", path.display());
        output_format::print_info_message(
            &mut io::stdout(),
            &format!("Loading custom rules from: {}", path.display()),
            theme_map,
        );
        Some(RedactionConfig::load_from_file(&path).with_context(|| {
            format!(
                "Failed to load custom configuration from '{}'",
                path.display()
            )
        })?)
    } else {
        None
    };

    let merged_config = config::merge_rules(default_rules, user_rules);

    debug!("Compiling rules...");
    // Pass the merged rules directly to compile_rules
    let compiled_rules = sanitize_shell::compile_rules(
        merged_config.rules, // Pass the Vec<RedactionRule>
        &enable_rules,
        &disable_rules,
    )?;
    debug!("Rules compiled successfully.");

    // Perform sanitization
    // sanitize_content no longer returns a Result, handles its own errors
    let (sanitized_content, summary) =
        sanitize_shell::sanitize_content(input_content, &compiled_rules);
    debug!(
        "Content sanitized. Original length: {}, Sanitized length: {}",
        input_content.len(),
        sanitized_content.len()
    );

    // Output handling
    let mut output_writer: Box<dyn Write> = if let Some(path) = output_path {
        info!("Writing sanitized content to file: {}", path.display());
        output_format::print_info_message(
            &mut io::stdout(),
            &format!("Writing sanitized content to file: {}", path.display()),
            theme_map,
        );
        Box::new(
            fs::File::create(&path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?,
        )
    } else {
        info!("Writing sanitized content to stdout.");
        output_format::print_info_message(
            &mut io::stdout(),
            "Writing sanitized content to stdout.",
            theme_map,
        );
        Box::new(io::stdout())
    };

    if diff_enabled {
        debug!("Generating and displaying diff.");
        diff_viewer::print_diff(input_content, &sanitized_content, &mut output_writer, theme_map)?;
    } else {
        debug!("Printing sanitized content.");
        writeln!(output_writer, "{}", sanitized_content)
            .context("Failed to write sanitized content")?;
    }

    if !no_redaction_summary {
        debug!("Displaying redaction summary.");
        redaction_summary::print_summary(&summary, &mut output_writer, theme_map)?;
    } else {
        debug!("Redaction summary display skipped per user request.");
    }

    // Clipboard handling
    if clipboard_enabled {
        debug!("Attempting to copy sanitized content to clipboard.");
        match copy_to_clipboard(&sanitized_content) {
            Ok(_) => info!("Sanitized content copied to clipboard successfully."),
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
    Ok(())
}

/// Helper function to copy content to the system clipboard.
/// This function is conditionally compiled based on the "clipboard" feature.
#[cfg(feature = "clipboard")]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Attempting to acquire clipboard.");
    let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
    debug!("Setting clipboard text.");
    clipboard.set_text(content.to_string()).context("Failed to set clipboard text")?;
    Ok(())
}

/// Placeholder function for when the "clipboard" feature is not enabled.
#[cfg(not(feature = "clipboard"))]
#[allow(unused_variables)]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Clipboard feature not enabled. Skipping copy operation.");
    Err(anyhow::anyhow!("Clipboard feature is not enabled. Compile with --features clipboard to enable functionality."))
}