// src/lib.rs
// Main library entry point for the cleansh application.
// This module coordinates the various components and commands.
// It handles CLI parsing, logging setup, and dispatches to the appropriate command based on user input.
// It also manages the application state, including usage statistics and donation prompts.
// license: Polyform Noncommercial License 1.0.0


#![doc = include_str!("../README.md")]

use anyhow::Context;
use std::io::{self, Read, Write, BufRead}; // Added Write and BufRead for flushing and read_line
use std::path::PathBuf;
use std::collections::HashMap;
use clap::{Parser, ArgAction, Subcommand};
use anyhow::Result;
use std::env;
use std::fs;
use log::{info, LevelFilter};
use dotenvy;

pub mod commands {
    pub mod cleansh; // Existing
    pub mod stats;   // For --stats-only logic
    pub mod uninstall; // NEW: For uninstall command
}
pub mod config;
pub mod logger;
pub mod tools;
pub mod ui;
pub mod utils {
    pub mod app_state; // For app state persistence (usage count, donation prompts)
    pub mod redaction; // Existing
}


/// CLI definition
#[derive(Parser, Debug)]
#[command(author = "Cleansh Technologies", version, about = "Sanitize your terminal output. One tool. One purpose.")]
pub struct Cli {
    #[command(subcommand)] // Define subcommands
    pub command: Option<Commands>,
    // Existing top-level flags now apply to the default `cleansh` operation
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
    #[arg(long, value_name = "FILE", help = "Pro: Export full scan summary to JSON file.")]
    pub stats_json_file: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue, help = "Pro: Export full scan summary to JSON on stdout.")]
    pub export_json_to_stdout: bool,
    #[arg(long, value_name = "N", help = "Pro: Show N unique examples per rule in stats output.")]
    pub sample_matches: Option<usize>,
    #[arg(long, value_name = "X", help = "Pro: Exit with non-zero code if total secrets exceed X.")]
    pub fail_over: Option<usize>,

    // ADDED: General rules flag for specifying rule config (e.g., 'default')
    #[arg(long, value_name = "RULES_CONFIG", help = "Specify which rules configuration to use (e.g., 'default', 'strict').")]
    pub rules: Option<String>,

    // --- NEW: Add --line-buffered flag ---
    #[arg(long, action = ArgAction::SetTrue, help = "Enable real-time, line-buffered output. Incompatible with --diff and --clipboard.")]
    pub line_buffered: bool,
    // --- END NEW FLAG ---
}

// Define subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Uninstall the cleansh application and its associated data.
    Uninstall {
        #[arg(short, long, action = ArgAction::SetTrue, help = "Bypass confirmation prompt.")]
        yes: bool,
    },
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
        pub use crate::commands::uninstall::run_uninstall_command; // NEW: Expose uninstall command for testing
    }
    pub mod ui {
        pub use crate::ui::theme;
        pub use crate::ui::output_format;
        pub use crate::ui::redaction_summary;
        pub use crate::ui::diff_viewer;
    }
    pub mod utils {
        pub use crate::utils::redaction::*;
        pub use crate::utils::app_state::*; // Expose AppState
    }
}

/// Main library entry
pub fn run(cli: Cli) -> Result<()> {
    dotenvy::dotenv().ok();

    // Determine the effective debug logging level based on CLI flags.
    let effective_log_level = if cli.quiet {
        Some(LevelFilter::Warn)
    } else if cli.debug && !cli.disable_debug {
        Some(LevelFilter::Debug)
    } else if cli.disable_debug {
        Some(LevelFilter::Info)
    } else {
        None
    };

    logger::init_logger(effective_log_level);
    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::Uninstall { yes } => {
                // Pass theme_map to uninstall command for consistent output styling
                let theme_map = ui::theme::ThemeStyle::default_theme_map(); // Default theme for uninstaller
                return commands::uninstall::run_uninstall_command(yes, &theme_map);
            }
        }
    }

    // Existing logic for the default `cleansh` operation (when no subcommand is given)
    let effective_clipboard = cli.clipboard && !cli.disable_clipboard;
    let effective_diff = cli.diff && !cli.disable_diff;

    // --- NEW: Check for incompatible flags with --line-buffered ---
    if cli.line_buffered {
        if effective_diff {
            let _ = ui::output_format::print_error_message(
                &mut io::stderr(),
                "Error: --line-buffered is incompatible with --diff.",
                &ui::theme::ThemeStyle::default_theme_map(),
            );
            std::process::exit(1);
        }
        if effective_clipboard {
            let _ = ui::output_format::print_error_message(
                &mut io::stderr(),
                "Error: --line-buffered is incompatible with --clipboard.",
                &ui::theme::ThemeStyle::default_theme_map(),
            );
            std::process::exit(1);
        }
        // ADDED: Check for --line-buffered and --input-file incompatibility
        if cli.input_file_flag.is_some() {
            let _ = ui::output_format::print_error_message(
                &mut io::stderr(),
                "Error: --line-buffered is incompatible with --input-file. Use piping for streaming input.",
                &ui::theme::ThemeStyle::default_theme_map(),
            );
            std::process::exit(1);
        }
    }
    // --- END NEW CHECK ---

    // Theme map loading and error handling
    let theme_map: HashMap<ui::theme::ThemeEntry, ui::theme::ThemeStyle> =
        if let Some(theme_path_arg) = cli.theme.as_ref() {
            match ui::theme::ThemeStyle::load_from_file(theme_path_arg) {
                Ok(loaded_map) => loaded_map,
                Err(e) => {
                    // `e` is now correctly in scope here
                    let _ = ui::output_format::print_warn_message(
                        &mut io::stderr(),
                        &format!("Failed to load theme from {}: {}. Using default theme.", theme_path_arg.display(), e),
                        &ui::theme::ThemeStyle::default_theme_map(), // Pass a default map for styling the warning itself
                    );
                    ui::theme::ThemeStyle::default_theme_map()
                }
            }
        } else {
            ui::theme::ThemeStyle::default_theme_map()
        };

    // --- NEW: Conditional input reading logic ---
    if cli.stats_only {
        // Stats-only mode still needs to read full input for analysis
        let mut input_content = String::new();
        let input_path = cli.input_file_flag; // Use cli.input_file_flag for consistency
        if let Some(path) = input_path.as_ref() {
            if !cli.quiet {
                let _ = ui::output_format::print_info_message(
                    &mut io::stderr(),
                    &format!("Reading input from file: {}", path.display()),
                    &theme_map,
                );
            }
            input_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read input from {}", path.display()))?;
        } else {
            if !cli.quiet {
                let _ = ui::output_format::print_info_message(
                    &mut io::stderr(),
                    "Reading input from stdin for stats analysis...",
                    &theme_map,
                );
            }
            io::stdin().read_to_string(&mut input_content)
                .context("Failed to read from stdin")?;
        }

        commands::stats::run_stats_command(
            &input_content,
            cli.config.clone(),
            cli.rules.clone(),
            &theme_map,
            cli.enable_rules.clone(),
            cli.disable_rules.clone(),
            cli.stats_json_file.clone(),
            cli.export_json_to_stdout,
            cli.sample_matches,
            cli.fail_over,
            cli.disable_donation_prompts,
        )?;
    } else if cli.line_buffered && cli.input_file_flag.is_none() { // This branch is now only for stdin line-buffered
        // If --out was given, open that file for writing; otherwise write to stdout.
        let mut writer: Box<dyn Write> = if let Some(path) = &cli.out {
            // Warn even in quiet mode
            let _ = ui::output_format::print_warn_message(
                &mut io::stderr(),
                "Warning: --line-buffered is intended for real-time console output. \
                 Outputting to a file (--out) will still buffer by line, \
                 but real-time benefits might be less apparent.",
                &ui::theme::ThemeStyle::default_theme_map(),
            );
            Box::new(std::fs::File::create(path)?)
        } else {
            if !cli.quiet {
                let _ = ui::output_format::print_info_message(
                    &mut io::stderr(),
                    "Reading input from stdin in real-time, line-buffered mode...",
                    &theme_map,
                );
            }
            Box::new(io::stdout().lock())
        };

        let default_rules = config::RedactionConfig::load_default_rules()?;
        let user_rules = if let Some(path) = cli.config.as_ref() {
            Some(config::RedactionConfig::load_from_file(path)
                .with_context(|| format!("Failed to load custom configuration from '{}'", path.display()))?)
        } else {
            None
        };
        let mut merged_config = config::merge_rules(default_rules, user_rules);

        if let Some(name) = cli.rules.clone() {
            merged_config.set_active_rules_config(&name)?;
        }

        let compiled_rules = crate::tools::sanitize_shell::compile_rules(
            merged_config.rules,
            &cli.enable_rules,
            &cli.disable_rules,
        )?;

        let mut all_redaction_matches = Vec::new();
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin.lock());
        let mut line = String::new();
        
        // Read and sanitize line by line, writing each immediately to `writer`.
        while reader.read_line(&mut line).context("Failed to read line from stdin")? > 0 {
            let (sanitized_line, line_matches) =
                commands::cleansh::sanitize_single_line(&line, &compiled_rules);

            writeln!(writer, "{}", sanitized_line)?;
            writer.flush()?; // Force flush after each line

            all_redaction_matches.extend(line_matches);
            line.clear();
        }

        // --- NEW LOGIC FOR SUMMARY/NO-REDACTIONS MESSAGE IN LINE-BUFFERED MODE ---
        if all_redaction_matches.is_empty() {
            // If no redactions were applied, print this message unless --no-redaction-summary is active
            if !cli.no_redaction_summary {
                 let _ = ui::output_format::print_info_message(
                    &mut io::stderr(),
                    "No redactions applied.",
                    &theme_map,
                );
            }
        } else {
            // If redactions *were* applied, print the summary unless --no-redaction-summary or --quiet is active
            if !cli.no_redaction_summary && !cli.quiet {
                let summary = commands::cleansh::build_redaction_summary_from_matches(&all_redaction_matches);
                let _ = ui::output_format::print_info_message(
                    &mut io::stderr(),
                    "Displaying redaction summary for streaming input.",
                    &theme_map,
                );
                ui::redaction_summary::print_summary(&summary, &mut io::stderr(), &theme_map)?;
            }
        }
        // --- END NEW LOGIC ---

    } else {
        // --- Default batch processing mode (full input read) ---
        let mut input_content = String::new();
        let input_path = cli.input_file_flag.clone(); // Clone input_file_flag for passing
        if let Some(path) = input_path.as_ref() { // Use as_ref() because input_path is now Option<PathBuf>
            if !cli.quiet {
                // This message is handled by run_cleansh now.
                // let _ = ui::output_format::print_info_message(
                //     &mut io::stderr(),
                //     &format!("Reading input from file: {}", path.display()),
                //     &theme_map,
                // );
            }
            input_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read input from {}", path.display()))?;
        } else {
            if !cli.quiet {
                // This message is handled by run_cleansh now.
                // let _ = ui::output_format::print_info_message(
                //     &mut io::stderr(),
                //     "Reading input from stdin...",
                //     &theme_map,
                // );
            }
            io::stdin().read_to_string(&mut input_content)
                .context("Failed to read from stdin")?;
        }

        // Delegate to the existing `cleansh` command for sanitization
        if let Err(e) = commands::cleansh::run_cleansh(
            &input_content,
            effective_clipboard,
            effective_diff,
            cli.config.clone(),
            cli.rules.clone(),
            cli.out.clone(),
            cli.no_redaction_summary,
            &theme_map,
            cli.enable_rules.clone(),
            cli.disable_rules.clone(),
            input_path, // Pass the cloned input_path here
        ) {
            let _ = ui::output_format::print_error_message( // Wrapped with `let _ =`
                &mut io::stderr(),
                &format!("An error occurred: {}", e),
                &theme_map,
            );
            std::process::exit(1);
        }
    }
    

    info!("cleansh finished successfully.");
    Ok(())
}