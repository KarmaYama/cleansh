//lib.rs
// Main library entry point for the cleansh application.

#![doc = include_str!("../README.md")]

use anyhow::Context;
use std::io::Read;
use std::path::PathBuf;
use std::collections::HashMap;
use clap::{Parser, ArgAction};
use anyhow::Result;
use std::env;
use std::fs;
use std::io;
use log::{info, LevelFilter};
use dotenvy;

// MODIFIED: Added `pub mod stats;` inside `commands` and `pub mod app_state;` inside `utils`
pub mod commands {
    pub mod cleansh; // Existing
    pub mod stats;   // NEW: For --stats-only logic
}
pub mod config;
pub mod logger;
pub mod tools;
pub mod ui;
pub mod utils {
    pub mod app_state; // NEW: For app state persistence (usage count, donation prompts)
    pub mod redaction; // Existing
}


/// CLI definition
#[derive(Parser, Debug)]
#[command(author = "Cleansh Technologies", version, about = "Sanitize your terminal output. One tool. One purpose.")]
pub struct Cli {
    #[arg(short, long, env = "CLIPBOARD_ENABLED", action = ArgAction::SetTrue)]
    pub clipboard: bool,
    #[arg(long = "no-clipboard", action = ArgAction::SetTrue)]
    pub disable_clipboard: bool,
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub diff: bool,
    #[arg(long = "no-diff", action = ArgAction::SetTrue)]
    pub disable_diff: bool,
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue)]
    pub debug: bool,
    #[arg(long = "no-debug", action = ArgAction::SetTrue)]
    pub disable_debug: bool,
    // ADDED: quiet flag
    #[arg(short = 'q', long, action = ArgAction::SetTrue, help = "Suppress informational output, only show warnings and errors.")]
    pub quiet: bool,

    #[arg(short = 'i', long = "input-file", value_name = "FILE", help = "Input file to sanitize via a named flag.")]
    pub input_file_flag: Option<PathBuf>,

    // Removed positional_input and input_string_arg. Input will now be expected either via --input-file or stdin.
    // This simplifies argument parsing to avoid conflicts with flags.

    #[arg(long, value_name = "FILE")]
    pub theme: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_redaction_summary: bool,
    #[arg(long, value_name = "RULE_NAMES", value_delimiter = ',')]
    pub enable_rules: Vec<String>,
    #[arg(long, value_name = "RULE_NAMES", value_delimiter = ',')]
    pub disable_rules: Vec<String>,

    // REINTRODUCED: --stats-only flag for analysis mode (free core feature)
    #[arg(long, action = ArgAction::SetTrue, help = "Only show redaction statistics; do not redact content or output sanitized data.")]
    pub stats_only: bool,
    // ADDED: --disable-donation-prompts flag (user preference)
    #[arg(long, action = ArgAction::SetTrue, help = "Disable prompts for donations.")]
    pub disable_donation_prompts: bool,

    // NEW: Pro feature flags, to be handled by the new `stats` command
    // Consistent with stats.rs, stats_json takes a PathBuf (for file output)
    #[arg(long, value_name = "FILE", help = "Pro: Export full scan summary to JSON file.")]
    pub stats_json_file: Option<PathBuf>, // RENAMED FROM stats_json
    // NEW: Flag for outputting JSON to stdout
    #[arg(long, action = ArgAction::SetTrue, help = "Pro: Export full scan summary to JSON on stdout.")]
    pub export_json_to_stdout: bool, // NEW FIELD
    #[arg(long, value_name = "N", help = "Pro: Show N unique examples per rule in stats output.")]
    pub sample_matches: Option<usize>,
    #[arg(long, value_name = "X", help = "Pro: Exit with non-zero code if total secrets exceed X.")]
    pub fail_over: Option<usize>,

    // ADDED: General rules flag for specifying rule config (e.g., 'default')
    #[arg(long, value_name = "RULES_CONFIG", help = "Specify which rules configuration to use (e.g., 'default', 'strict').")]
    pub rules: Option<String>,
}

/// Test-only exports
#[cfg(any(test, feature = "test-exposed"))]
pub mod test_exposed {
    // Re-export necessary modules/items for testing
    pub mod config {
        pub use crate::config::*;
    }
    pub mod tools {
        // ONLY re-export sanitize_shell, which internally uses validators.
        // This avoids ambiguous re-exports of the validation functions.
        pub use crate::tools::sanitize_shell::*;
        // Directly re-export validators for tests that specifically need them
        pub use crate::tools::validators; // Keep this, but access functions via `validators::is_valid_...`
    }
    pub mod commands {
        pub use crate::commands::cleansh::run_cleansh;
        pub use crate::commands::stats::run_stats_command;
    }
    pub mod ui {
        pub use crate::ui::theme;
        pub use crate::ui::output_format;
        pub use crate::ui::redaction_summary;
        pub use crate::ui::diff_viewer;
    }
    pub mod utils {
        pub use crate::utils::redaction::*;
        pub use crate::utils::app_state::*;
    }
}

/// Main library entry
pub fn run(cli: Cli) -> Result<()> {
    dotenvy::dotenv().ok();

    // Determine the effective debug logging level based on CLI flags.
    // Order of precedence: --quiet > --no-debug > --debug > RUST_LOG env var > default Warn
    let effective_log_level = if cli.quiet {
        // If --quiet is set, suppress all info/debug, only show Warn, Error, Trace.
        Some(LevelFilter::Warn)
    } else if cli.debug && !cli.disable_debug {
        // If --debug is set AND --no-debug is NOT, enable debug logging.
        Some(LevelFilter::Debug)
    } else if cli.disable_debug {
        // If --no-debug is specifically set, ensure no debug logs,
        // effectively setting a higher filter level (Info, Warn, Error, Trace).
        Some(LevelFilter::Info)
    } else {
        // No explicit verbosity flags, let RUST_LOG or the default Warn in logger::init_logger apply.
        None
    };

    logger::init_logger(effective_log_level);
    // This info message will now be suppressed by default due to LevelFilter::Warn in logger.rs
    // unless RUST_LOG is set to info/debug or --debug is used.
    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));

    let effective_clipboard = cli.clipboard && !cli.disable_clipboard;
    let effective_diff = cli.diff && !cli.disable_diff;

    // Theme map
    let theme_map: HashMap<ui::theme::ThemeEntry, ui::theme::ThemeStyle> =
        if let Some(t) = cli.theme.as_ref() {
            ui::theme::ThemeStyle::load_from_file(t).unwrap_or_else(|e| {
                // Corrected: Pass a *new* default theme map for the error message
                ui::output_format::print_warn_message(
                    &mut io::stderr(),
                    &format!("Failed to load theme from {}: {}", t.display(), e),
                    &ui::theme::ThemeStyle::default_theme_map(),
                );
                ui::theme::ThemeStyle::default_theme_map()
            })
        } else {
            ui::theme::ThemeStyle::default_theme_map()
        };

    // Read input
    let mut input_content = String::new();
    let input_path = cli.input_file_flag; // Only --input-file is considered
    if let Some(path) = input_path.as_ref() {
        // FIX: Conditionally print "Reading input from file" based on quiet flag
        if !cli.quiet {
            ui::output_format::print_info_message(
                &mut io::stderr(),
                &format!("Reading input from file: {}", path.display()),
                &theme_map,
            );
        }
        input_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read input from {}", path.display()))?;
    } else {
        // FIX: Conditionally print "Reading input from stdin..." based on quiet flag
        if !cli.quiet {
            ui::output_format::print_info_message(
                &mut io::stderr(),
                "Reading input from stdin...",
                &theme_map,
            );
        }
        io::stdin().read_to_string(&mut input_content)
            .context("Failed to read from stdin")?;
    }

    // NEW: Central dispatch based on CLI flags
    if cli.stats_only {
        // Delegate to the new `stats` command
        commands::stats::run_stats_command(
            &input_content,
            cli.config.clone(),
            cli.rules.clone(), // Pass the rules config name
            &theme_map,
            cli.enable_rules.clone(),
            cli.disable_rules.clone(),
            cli.stats_json_file.clone(), // Pass the PathBuf for JSON file output
            cli.export_json_to_stdout, // NEW: Pass the boolean for stdout JSON output
            cli.sample_matches,
            cli.fail_over,
            cli.disable_donation_prompts,
        )?;
    } else {
        // Delegate to the existing `cleansh` command for sanitization
        if let Err(e) = commands::cleansh::run_cleansh(
            &input_content,
            effective_clipboard,
            effective_diff,
            cli.config.clone(),
            cli.rules.clone(), // Pass the rules config name
            cli.out.clone(),
            cli.no_redaction_summary,
            &theme_map,
            cli.enable_rules.clone(),
            cli.disable_rules.clone(),
        ) {
            ui::output_format::print_error_message(
                &mut io::stderr(),
                &format!("An error occurred: {}", e),
                &theme_map,
            );
            std::process::exit(1);
        }
    }

    // This info message will also be suppressed by default
    info!("cleansh finished successfully.");
    Ok(())
}