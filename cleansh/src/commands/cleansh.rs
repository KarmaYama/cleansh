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
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

// Import from cleansh_core
use cleansh_core::{
    RedactionSummaryItem,
    config as core_config,
    sanitizer,
    redaction_match::{log_redaction_match_debug, RedactionMatch},
};

// Local imports
use crate::ui::diff_viewer;
use crate::ui::redaction_summary;
use crate::ui::output_format;
use crate::ui::theme::{self, ThemeMap}; // Import ThemeMap
use is_terminal::IsTerminal; // Keep IsTerminal to determine coloring for stdout/stderr

// --- NEW TRAIT DEFINITION AND BLANKET IMPLEMENTATION ---
// This trait is still needed because `primary_output_writer` uses `IsTerminal`
// for its own type definition (to ensure it can be passed to `is_terminal::IsTerminal` for `stdout()`)
// and for the `is_terminal()` call when setting `output_supports_color`.
pub trait WriteAndTerminal: Write + IsTerminal {}

// Blanket implementation: Any type that implements both `Write` and `IsTerminal`
// will automatically implement `WriteAndTerminal`.
impl<T: Write + IsTerminal> WriteAndTerminal for T {}
// --- END NEW TRAIT DEFINITION ---

/// Grouped options for the new ergonomic API
pub struct CleanshOptions {
    pub input: String,
    pub clipboard: bool,
    pub diff: bool,
    pub config_path: Option<std::path::PathBuf>,
    pub rules_config_name: Option<String>,
    pub output_path: Option<std::path::PathBuf>,
    pub no_redaction_summary: bool,
    pub enable_rules: Vec<String>,
    pub disable_rules: Vec<String>,
}

/// Helper for printing info messages to stderr.
pub fn info_msg(msg: impl AsRef<str>, theme: &ThemeMap) { // Use ThemeMap alias
    // We explicitly decide if stderr supports colors here for consistent messaging
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_info_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Helper for printing error messages to stderr.
pub fn error_msg(msg: impl AsRef<str>, theme: &ThemeMap) { // Use ThemeMap alias
    // We explicitly decide if stderr supports colors here for consistent messaging
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_error_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Helper for printing warning messages to stderr.
pub fn warn_msg(msg: impl AsRef<str>, theme: &ThemeMap) { // Use ThemeMap alias
    let stderr_supports_color = io::stderr().is_terminal();
    let _ = output_format::print_warn_message(&mut std::io::stderr(), msg.as_ref(), theme, stderr_supports_color);
}

/// Load default + user rules, apply named config, compile.
/// Returns `CompiledRules`.
pub fn load_and_compile(
    config_path: Option<PathBuf>,
    profile: Option<String>,
    enable: Vec<String>,
    disable: Vec<String>,
) -> Result<sanitizer::CompiledRules> {
    // 1. Load defaults
    let default = cleansh_core::RedactionConfig::load_default_rules()?; // Use fully qualified name

    // 2. Load user if specified
    let user = if let Some(p) = config_path {
        Some(cleansh_core::RedactionConfig::load_from_file(&p) // Use fully qualified name
            .with_context(|| format!("Failed to load custom rules at `{}`", p.display()))?)
    } else {
        None
    };

    // 3. Merge & named config
    let mut cfg = core_config::merge_rules(default, user);
    if let Some(name) = profile {
        cfg.set_active_rules_config(&name)?;
    }

    // 4. Compile
    sanitizer::compile_rules(cfg.rules, &enable, &disable)
}

/// New, slim implementation entrypoint.
/// Contains all the core logic for running the cleansh operation.
#[allow(clippy::too_many_lines)]
pub fn run_cleansh_opts(
    opts: CleanshOptions,
    theme_map: &ThemeMap,
) -> Result<()> {
    info!("Starting cleansh operation.");
    debug!("[cleansh::commands::cleansh] Starting cleansh operation.");
    debug!("[cleansh::commands::cleansh] Received enable_rules: {:?}", opts.enable_rules);
    debug!("[cleansh::commands::cleansh] Received disable_rules: {:?}", opts.disable_rules);

    // Command compatibility checks (moved from main.rs to CleanshOptions handler)
    if opts.diff && opts.clipboard {
        error_msg("Error: --diff is incompatible with --clipboard. Please choose one.", theme_map);
        std::process::exit(1);
    }

    info_msg("Processing input content.", theme_map);
    debug!("[cleansh::commands::cleansh] Processing input content.");

    let compiled_rules = load_and_compile(
        opts.config_path,
        opts.rules_config_name,
        opts.enable_rules,
        opts.disable_rules,
    )?;
    debug!("Rules compiled successfully.");
    debug!("[cleansh::commands::cleansh] Compiled {} rules successfully in cleansh.", compiled_rules.rules.len());

    debug!("[cleansh::commands::cleansh] Names of compiled rules available for sanitization:");
    for rule in &compiled_rules.rules {
        debug!("[cleansh::commands::cleansh] - {}", rule.name);
    }

    let (sanitized_content, all_redaction_matches) =
        sanitizer::sanitize_content(&opts.input, &compiled_rules);
    debug!(
        "Content sanitized. Original length: {}, Sanitized length: {}",
        opts.input.len(),
        sanitized_content.len()
    );

    for m in &all_redaction_matches {
        log_redaction_match_debug(
            "[cleansh::commands::cleansh]",
            &m.rule_name,
            &m.original_string,
            &m.sanitized_string
        );
    }

    let summary = build_redaction_summary_from_matches(&all_redaction_matches);
    debug!("DEBUG_CLEANSH: Redaction summary (num items): {:?}", summary.len());

    // Determine output writer and its color support
    let stderr_supports_color = io::stderr().is_terminal(); // Determine here for consistent use
    let (mut primary_output_writer, output_supports_color): (Box<dyn WriteAndTerminal + Send + Sync>, bool) = if let Some(path) = opts.output_path.clone() {
        info_msg(format!("Writing sanitized content to file: {}", path.display()), theme_map);
        debug!("[cleansh::commands::cleansh] Outputting to file: {}", path.display());
        (
            Box::new(
                fs::File::create(&path)
                    .with_context(|| format!("Failed to create output file: {}", path.display()))?,
            ),
            false, // Files generally don't support ANSI colors
        )
    } else {
        info_msg("Writing sanitized content to stdout.", theme_map);
        debug!("[cleansh::commands::cleansh] Outputting to stdout.");
        let stdout = io::stdout();
        let supports_color = stdout.is_terminal(); // Check if stdout is a terminal
        (Box::new(stdout), supports_color)
    };

    if opts.diff {
        debug!("Generating and displaying diff.");
        info_msg("Generating and displaying diff.", theme_map);
        debug!("[cleansh::commands::cleansh] Diff enabled.");
        // Call print_diff with the explicit color support flag
        diff_viewer::print_diff(&opts.input, &sanitized_content, &mut primary_output_writer, theme_map, output_supports_color)?;
    } else {
        debug!("Printing sanitized content.");
        debug!("[cleansh::commands::cleansh] Diff disabled, printing sanitized content.");
        writeln!(primary_output_writer, "{}", sanitized_content)
            .context("Failed to write sanitized content")?;
    }

    if !opts.no_redaction_summary {
        debug!("Displaying redaction summary.");
        info_msg("Displaying redaction summary.", theme_map);
        debug!("[cleansh::commands::cleansh] Redaction summary enabled.");
        // Use io::stderr() directly here as it's always a terminal for error messages
        // And pass the explicit color support flag
        redaction_summary::print_summary(&summary, &mut io::stderr(), theme_map, stderr_supports_color)?;
    } else {
        debug!("Redaction summary display skipped per user request.");
        info_msg("Redaction summary display skipped per user request.", theme_map);
        debug!("[cleansh::commands::cleansh] Redaction summary skipped.");
    }

    if opts.clipboard {
        debug!("Attempting to copy sanitized content to clipboard.");
        debug!("[cleansh::commands::cleansh] Clipboard enabled.");
        match copy_to_clipboard(&sanitized_content) {
            Ok(_) => {
                info!("Sanitized content copied to clipboard successfully.");
                info_msg("Sanitized content copied to clipboard successfully.", theme_map);
            },
            Err(e) => {
                warn!("Failed to copy to clipboard: {}", e);
                // Use the warn_msg for warnings to ensure consistent styling
                warn_msg(&format!("Failed to copy to clipboard: {}", e), theme_map);
            }
        }
    }

    info!("Cleansh operation completed.");
    debug!("[cleansh::commands::cleansh] Cleansh operation completed.");
    Ok(())
}

/// Original API, preserved for tests.
/// This function acts as a compatibility layer for existing integration tests.
#[allow(clippy::too_many_arguments)] // This signature is for backward compatibility
pub fn run_cleansh(
    input_content: &str,
    clipboard_enabled: bool,
    diff_enabled: bool,
    config_path: Option<PathBuf>,
    rules_config_name: Option<String>,
    output_path: Option<PathBuf>,
    no_redaction_summary: bool,
    theme_map: &HashMap<theme::ThemeEntry, theme::ThemeStyle>,
    enable_rules: Vec<String>,
    disable_rules: Vec<String>,
    _input_file_path: Option<PathBuf>, // This argument is ignored as input is passed directly
) -> Result<()> {
    // Build the CleanshOptions struct from the old signature arguments
    let opts = CleanshOptions {
        input: input_content.to_string(),
        clipboard: clipboard_enabled,
        diff: diff_enabled,
        config_path,
        rules_config_name,
        output_path,
        no_redaction_summary,
        enable_rules,
        disable_rules,
    };

    // Delegate to the new, streamlined implementation
    // run_cleansh_opts calculates stderr_supports_color internally
    run_cleansh_opts(opts, theme_map)
}


/// Sanitizes a single line of input using the provided compiled rules.
///
/// This function is primarily used in line-buffered streaming mode. It takes a single
/// line of text and applies the pre-compiled redaction rules to it.
///
/// # Arguments
///
/// * `line` - The string slice representing a single line of input.
/// * `compiled_rules` - A reference to the `CompiledRules` instance containing the
///                      active and compiled redaction rules.
///
/// # Returns
///
/// A tuple containing:
/// * `String` - The sanitized version of the input line.
/// * `Vec<RedactionMatch>` - A vector of `RedactionMatch` instances found within this line.
pub fn sanitize_single_line(
    line: &str,
    compiled_rules: &sanitizer::CompiledRules,
) -> (String, Vec<RedactionMatch>) {
    sanitizer::sanitize_content(line, compiled_rules)
}

/// Helper function to copy content to the system clipboard.
///
/// This function attempts to copy the given string content to the system clipboard.
/// Its availability depends on the `clipboard` feature being enabled during compilation.
///
/// # Arguments
///
/// * `content` - The string slice to be copied to the clipboard.
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) or an error (`Err(anyhow::Error)`)
/// if the clipboard operation fails or the feature is not enabled.
#[cfg(feature = "clipboard")]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Attempting to acquire clipboard.");
    debug!("[cleansh::commands::cleansh] Acquiring clipboard.");
    let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
    debug!("Setting clipboard text.");
    debug!("[cleansh::commands::cleansh] Setting clipboard text.");
    clipboard.set_text(content.to_string()).context("Failed to set clipboard text")?;
    Ok(())
}

/// Placeholder function for when the "clipboard" feature is not enabled.
///
/// This function provides a fallback when `cleansh` is compiled without the
/// `clipboard` feature. It always returns an error indicating that the
/// clipboard functionality is not available.
///
/// # Arguments
///
/// * `content` - The string slice that would have been copied (unused in this case).
///
/// # Returns
///
/// An `Err(anyhow::Error)` indicating that the clipboard feature is not enabled.
#[cfg(not(feature = "clipboard"))]
#[allow(unused_variables)]
fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Clipboard feature not enabled. Skipping copy operation.");
    debug!("[cleansh::commands::cleansh] Clipboard feature not enabled.");
    Err(anyhow::anyhow!("Clipboard feature is not enabled. Compile with --features clipboard to enable functionality."))
}

/// Builds a `Vec<RedactionSummaryItem>` from a `Vec<RedactionMatch>`.
///
/// This function aggregates individual `RedactionMatch` instances into a summarized
/// list, grouped by the rule name. Each `RedactionSummaryItem` includes the rule name,
/// total occurrences, and unique original and sanitized texts.
///
/// # Arguments
///
/// * `matches` - A slice of `RedactionMatch` instances obtained from content sanitization.
///
/// # Returns
///
/// A `Vec<RedactionSummaryItem>` containing the aggregated and sorted redaction summaries.
pub fn build_redaction_summary_from_matches(
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
        if !item.original_texts.contains(&m.original_string) {
            item.original_texts.push(m.original_string.clone());
        }
        if !item.sanitized_texts.contains(&m.sanitized_string) {
            item.sanitized_texts.push(m.sanitized_string.clone());
        }
    }

    for item in summary_map.values_mut() {
        item.original_texts.sort();
        item.sanitized_texts.sort();
    }

    let mut summary: Vec<RedactionSummaryItem> = summary_map.into_values().collect();
    summary.sort_by(|a, b| a.rule_name.cmp(&b.rule_name));

    summary
}