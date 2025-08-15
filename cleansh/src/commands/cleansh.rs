//! Cleansh command implementation for sanitizing terminal output.
//!
//! This module handles the main functionality of the `cleansh` CLI application,
//! including reading input from various sources (stdin, files), applying redaction
//! rules, and outputting the sanitized content to different destinations (stdout, files, clipboard).
//! It orchestrates the flow of data through the redaction pipeline, leveraging
//! the core logic from the `cleansh-core` crate.

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::io::{self, Write};
use std::fs;
use std::collections::HashMap;

// Import from cleansh_core
use cleansh_core::{
    engine::SanitizationEngine, // Import the SanitizationEngine trait
    RedactionSummaryItem,
};

// Local imports
use crate::ui::diff_viewer;
use crate::ui::redaction_summary;
use crate::ui::output_format;
use crate::ui::theme::{ThemeMap};
use crate::utils::clipboard::copy_to_clipboard;
use is_terminal::IsTerminal;

/// Grouped options for the new ergonomic API
pub struct CleanshOptions {
    pub input: String,
    pub clipboard: bool,
    pub diff: bool,
    pub output_path: Option<std::path::PathBuf>,
    pub no_redaction_summary: bool,
    pub quiet: bool,
}

/// Helper for printing info messages to stderr.
pub fn info_msg(msg: impl AsRef<str>, theme: &ThemeMap) {
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_info_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Helper for printing error messages to stderr.
pub fn error_msg(msg: impl AsRef<str>, theme: &ThemeMap) {
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_error_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Helper for printing warning messages to stderr.
pub fn warn_msg(msg: impl AsRef<str>, theme: &ThemeMap) {
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_warn_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Handles writing sanitized content to the primary output destination (stdout or file).
fn handle_primary_output(
    opts: &CleanshOptions,
    sanitized_content: &str,
    theme_map: &ThemeMap,
) -> Result<()> {
    if let Some(path) = opts.output_path.clone() {
        info_msg(format!("Writing sanitized content to file: {}", path.display()), theme_map);
        debug!("[cleansh::commands::cleansh] Outputting to file: {}", path.display());
        let mut file = fs::File::create(&path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;
        
        if opts.diff {
            debug!("Generating and displaying diff.");
            diff_viewer::print_diff(&opts.input, sanitized_content, &mut file, theme_map, false)?;
        } else {
            writeln!(file, "{}", sanitized_content)
                .context("Failed to write sanitized content")?;
        }
    } else {
        info_msg("Writing sanitized content to stdout.", theme_map);
        debug!("[cleansh::commands::cleansh] Outputting to stdout.");
        let stdout = io::stdout();
        let mut writer = stdout.lock();
        let supports_color = stdout.is_terminal();
        
        if opts.diff {
            debug!("Generating and displaying diff.");
            diff_viewer::print_diff(&opts.input, sanitized_content, &mut writer, theme_map, supports_color)?;
        } else {
            writeln!(writer, "{}", sanitized_content)
                .context("Failed to write sanitized content")?;
        }
    };
    Ok(())
}

/// Handles copying sanitized content to the clipboard.
fn handle_clipboard_output(sanitized_content: &str, theme_map: &ThemeMap) {
    debug!("Attempting to copy sanitized content to clipboard.");
    match copy_to_clipboard(sanitized_content) {
        Ok(_) => {
            info!("Sanitized content copied to clipboard successfully.");
            info_msg("Sanitized content copied to clipboard successfully.", theme_map);
        },
        Err(e) => {
            warn!("Failed to copy to clipboard: {}", e);
            warn_msg(&format!("Failed to copy to clipboard: {}", e), theme_map);
        }
    }
}

/// Displays the redaction summary to stderr.
fn handle_redaction_summary(
    summary: &[RedactionSummaryItem],
    opts: &CleanshOptions,
    theme_map: &ThemeMap,
) -> Result<()> {
    if !opts.no_redaction_summary && !opts.quiet {
        info!("Displaying redaction summary.");
        let stderr_supports_color = io::stderr().is_terminal();
        redaction_summary::print_summary(&summary, &mut io::stderr(), theme_map, stderr_supports_color)?;
    } else {
        info!("Redaction summary display skipped per user request.");
    }
    Ok(())
}

/// New, slim implementation entrypoint.
/// Contains all the core logic for running the cleansh operation.
pub fn run_cleansh_opts(
    engine: &dyn SanitizationEngine,
    opts: CleanshOptions,
    theme_map: &ThemeMap,
) -> Result<()> {
    info!("Starting cleansh operation.");

    let (sanitized_content, summary) = engine.sanitize(
        &opts.input,
        "",
        "",
        "",
        "",
        "",
        "",
        None,
    )
    .context("Sanitization failed")?;

    debug!(
        "Content sanitized. Original length: {}, Sanitized length: {}",
        opts.input.len(),
        sanitized_content.len()
    );
    
    handle_primary_output(&opts, &sanitized_content, theme_map)?;

    if opts.clipboard {
        handle_clipboard_output(&sanitized_content, theme_map);
    }
    
    handle_redaction_summary(&summary, &opts, theme_map)?;
    
    info!("Cleansh operation completed.");
    Ok(())
}

/// Sanitizes a single line of input using the provided compiled rules, returning a map of matched rules.
///
/// This function is primarily used in line-buffered streaming mode. It takes a single
/// line of text and applies the pre-compiled redaction rules to it, returning both
/// the sanitized string and a map of rules and their match counts.
///
/// # Arguments
///
/// * `line` - The string slice representing a single line of input.
/// * `engine` - A reference to the `SanitizationEngine` instance to use for redaction.
///
/// # Returns
///
/// A tuple containing the sanitized version of the input line and a `HashMap<String, usize>`
/// of the redaction rule names and their match counts for that line.
pub fn sanitize_single_line_with_count(
    line: &str,
    engine: &dyn SanitizationEngine,
) -> (String, HashMap<String, usize>) {
    let (sanitized_content, summary) = engine.sanitize(
        line,
        "",
        "",
        "",
        "",
        "",
        "",
        None,
    )
    .unwrap_or_else(|_| (line.to_string(), Vec::new()));
    let mut counts: HashMap<String, usize> = HashMap::new();
    for item in summary {
        *counts.entry(item.rule_name).or_insert(0) += 1;
    }
    (sanitized_content, counts)
}

/// Sanitizes a single line of input using the provided compiled rules.
///
/// This function is primarily used in line-buffered streaming mode. It takes a single
/// line of text and applies the pre-compiled redaction rules to it.
///
/// # Arguments
///
/// * `line` - The string slice representing a single line of input.
/// * `engine` - A reference to the `SanitizationEngine` instance to use for redaction.
///
/// # Returns
///
/// The sanitized version of the input line.
pub fn sanitize_single_line(
    line: &str,
    engine: &dyn SanitizationEngine,
) -> String {
    let (sanitized_content, _) = engine.sanitize(
        line,
        "",
        "",
        "",
        "",
        "",
        "",
        None,
    )
    .unwrap_or_else(|_| (line.to_string(), Vec::new()));
    sanitized_content
}