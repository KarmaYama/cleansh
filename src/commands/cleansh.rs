use anyhow::{Context, Result};
use arboard::Clipboard;
use crate::config;
use crate::tools::sanitize_shell;
use crate::ui::output_format;
use crate::ui::theme::{ThemeEntry, ThemeStyle};
use log::{debug, error, info, warn};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

/// The main entry point for the `cleansh` command's execution logic.
///
/// This function orchestrates the entire sanitization process, including:
/// - Loading and merging redaction rules (default and custom).
/// - Applying sanitization to the input content.
/// - Displaying a summary of redactions.
/// - Handling optional features like diff view, clipboard output, and file output.
///
/// # Arguments
///
/// * `input_content` - The string content to be sanitized.
/// * `clipboard_enabled` - A boolean indicating whether to copy the sanitized output to the clipboard.
/// * `diff_enabled` - A boolean indicating whether to display a diff view between original and sanitized content.
/// * `config_path` - An optional `PathBuf` to a custom YAML configuration file for redaction rules.
/// * `output_path` - An optional `PathBuf` to write the sanitized output to a file.
/// * `no_redaction_summary` - A boolean indicating whether to suppress the redaction summary output.
/// * `theme_map` - A `HashMap` containing the current theme's styling for various output elements.
/// * `opt_in_rules` - A vector of rule names that the user has explicitly opted into.
///
/// # Returns
///
/// Returns `Ok(())` on successful execution, or an `anyhow::Result` error if any step fails.
pub fn run_cleansh(
    input_content: &str,
    clipboard_enabled: bool,
    diff_enabled: bool,
    config_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    no_redaction_summary: bool,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
    opt_in_rules: Vec<String>, // Added opt_in_rules
) -> Result<()> {
    info!("Starting cleansh command.");
    debug!("Clipboard: {}, Diff: {}", clipboard_enabled, diff_enabled);
    debug!("No redaction summary: {}", no_redaction_summary);
    debug!("Opt-in rules: {:?}", opt_in_rules);

    // 1. Load and merge rules
    let default_cfg = config::load_default_rules().context("Loading default rules")?;
    let merged_cfg = if let Some(p) = config_path {
        output_format::print_info_message(
            &mut io::stdout(),
            &format!("Loading custom rules from: {}", p.display()),
            theme_map,
        );
        let user = config::load_user_rules(&p).context("Loading user rules")?;
        config::merge_rules(default_cfg, Some(user))
    } else {
        default_cfg
    };

    // Filter rules based on opt-in status
    let filtered_rules_config = config::RulesConfig {
        rules: merged_cfg.rules.into_iter().filter(|rule| {
            // Include rule if it's not opt-in, or if it is opt-in AND its name is in the opt_in_rules list
            !rule.opt_in || opt_in_rules.contains(&rule.name)
        }).collect(),
    };

    let compiled = sanitize_shell::compile_rules(filtered_rules_config).context("Compiling rules")?;

    // 2. Sanitize
    let (sanitized, summary) = sanitize_shell::sanitize_content(input_content, &compiled);

    // 3. Print summary (only if not suppressed)
    if !no_redaction_summary {
        if summary.is_empty() && !input_content.trim().is_empty() { // Check if input is not just whitespace
            output_format::print_info_message(
                &mut io::stdout(),
                "No redactions applied.",
                theme_map,
            );
        } else if !summary.is_empty() {
            output_format::print_redaction_summary(&mut io::stdout(), &summary, theme_map);
        }
    }

    // 4. Diff
    if diff_enabled {
        output_format::print_diff_view(&mut io::stdout(), input_content, &sanitized, theme_map);
    }

    // 5. Clipboard
    if clipboard_enabled {
        match Clipboard::new() {
            Ok(mut cb) => {
                match cb.set_text(sanitized.clone()) {
                    Ok(_) => {
                        output_format::print_success_message(
                            &mut io::stdout(),
                            "✅ Copied to clipboard.",
                            theme_map,
                        );
                    }
                    Err(e) => {
                        error!("Clipboard set_text error: {}", e);
                        warn!("Failed to copy to clipboard: {}", e);
                        output_format::print_error_message(
                            &mut io::stderr(),
                            "⚠️ Failed to copy to clipboard. \
                             On Linux, ensure `xclip`, `xsel`, or `wl-clipboard` is installed.",
                            theme_map,
                        );
                    }
                }
            }
            Err(e) => {
                error!("Clipboard initialization error: {}", e);
                warn!("Clipboard backend unavailable: {}", e);
                output_format::print_error_message(
                    &mut io::stderr(),
                    "⚠️ Clipboard is unavailable on this system.",
                    theme_map,
                );
            }
        }
    }

    // 6. File or stdout
    if let Some(path) = output_path {
        fs::write(&path, &sanitized).context("Writing output file")?;
        output_format::print_success_message(
            &mut io::stdout(),
            "✅ Written to file.",
            theme_map,
        );
    } else if !diff_enabled {
        output_format::print_content(&mut io::stdout(), &sanitized);
    }

    info!("cleansh finished.");
    Ok(())
}