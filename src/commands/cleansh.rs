use anyhow::{Context, Result};
use arboard::Clipboard;
use crate::config;
use crate::tools::sanitize_shell;
use crate::ui::output_format;
use crate::ui::theme::{ThemeEntry, ThemeStyle};
use log::{debug, error, info, warn};
use std::fs;
use std::io; // `Write` is no longer directly used by this module's functions
use std::path::PathBuf;
use std::collections::HashMap;

/// The main entry point for the `cleansh` command.
pub fn run_cleansh(
    input_content: &str,
    clipboard_enabled: bool,
    diff_enabled: bool,
    config_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    theme_map: &HashMap<ThemeEntry, ThemeStyle>,
) -> Result<()> {
    info!("Starting cleansh command.");
    debug!("Clipboard: {}, Diff: {}", clipboard_enabled, diff_enabled);

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
    let compiled = sanitize_shell::compile_rules(merged_cfg).context("Compiling rules")?;

    // 2. Sanitize
    let (sanitized, summary) = sanitize_shell::sanitize_content(input_content, &compiled);

    // 3. Print summary
    output_format::print_redaction_summary(&mut io::stdout(), &summary, theme_map);

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
        // Only print content to stdout if diff is not enabled
        output_format::print_content(&mut io::stdout(), &sanitized);
    }

    info!("cleansh finished.");
    Ok(())
}
