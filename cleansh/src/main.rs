// cleansh-workspace/cleansh/src/main.rs
//! # Cleansh CLI Application
//!
//! `cleansh` is the command-line interface application that allows users to
//! sanitize sensitive information from text content. This crate serves as
//! the main executable wrapper, orchestrating the parsing of command-line
//! arguments, managing input and output streams, handling application-specific
//! features like user-interaction (e.g., donation prompts), and integrating
//! with the core redaction logic provided by the `cleansh-core` library.
//!
//! ## Key Responsibilities of this Crate:
//! - **Argument Parsing:** Defines and parses all CLI options and subcommands
//!   using the `clap` crate.
//! - **Input/Output Management:** Handles reading content from stdin or specified
//!   files, and writing sanitized or statistical output to stdout, files, or
//!   the system clipboard.
//! - **Application State:** Manages persistent application state such as usage
//!   counts and prompt timings, leveraging the `utils::app_state` module.
//! - **User Interface:** Incorporates modules for theming, formatted output,
//!   redaction summaries, and diff viewing (`ui` module).
//! - **Command Execution:** Dispatches to specific command handlers (e.g., `stats`,
//!   `uninstall`) based on user input, found within the `commands` module.
//! - **Integration:** Acts as the bridge between user commands and the core
//!   redaction and validation functionalities exposed by `cleansh-core`.
//!
//! ## License
//!
//! Licensed under the Polyform Noncommercial License 1.0.0.

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, Read, Write, BufRead, IsTerminal};
use std::fs;
use std::env;
use log::{info, LevelFilter};
use dotenvy;

// Corrected imports: Refer to the `cleansh` library crate by its package name.
// This brings in the modules declared as `pub mod` in `src/lib.rs`.
use cleansh::commands;
use cleansh::logger;
use cleansh::ui;
use cleansh::utils::app_state::AppState;
use cleansh::cli::{Cli, Commands};

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Determine effective log level based on CLI flags
    let effective_log_level = if cli.quiet {
        Some(LevelFilter::Warn)
    } else if cli.debug && !cli.disable_debug {
        Some(LevelFilter::Debug)
    } else if cli.disable_debug {
        Some(LevelFilter::Info) // If debug is explicitly disabled, set to Info to suppress DEBUG logs
    } else {
        None // Let RUST_LOG or default logger settings apply
    };

    logger::init_logger(effective_log_level);
    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));

    // --- AppState Management ---
    let app_state_path = if let Some(dir) = dirs::data_dir() {
        dir.join("cleansh").join("state.json")
    } else {
        // Fallback for systems where data_dir is not available
        std::env::current_dir()?.join("cleansh_state.json")
    };

    let app_state = AppState::load(&app_state_path)?;
    // --- End AppState Management ---

    // Theme map building (common for all paths)
    let theme_map = ui::theme::build_theme_map(cli.theme.as_ref())?;

    // Determine color support for stderr once
    let stderr_supports_color = io::stderr().is_terminal();

    // Handle subcommands first
    match cli.command {
        Some(Commands::Stats(stats_opts)) => {
            // Stats command still needs to read full input upfront for analysis
            let input_content = if let Some(path) = cli.input_file.as_ref() {
                commands::cleansh::info_msg(format!("Reading input from file: {}", path.display()), &theme_map);
                fs::read_to_string(path)
                    .with_context(|| format!("Failed to read input from {}", path.display()))?
            } else if io::stdin().is_terminal() {
                // NEW: This branch now correctly handles the interactive TTY case
                commands::cleansh::info_msg(
                    "Reading input from stdin. Press Ctrl+Z then Enter to finish input.",
                    &theme_map,
                );
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)
                    .context("Failed to read from stdin")?;
                buffer
            } else {
                commands::cleansh::info_msg("Reading input from stdin for stats analysis...", &theme_map);
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)
                    .context("Failed to read from stdin")?;
                buffer
            };

            commands::stats::run_stats_command(
                &input_content,
                cli.config.clone(),
                cli.rules.clone(),
                &theme_map,
                cli.enable.clone(),
                cli.disable.clone(),
                stats_opts.json_file.clone(),
                stats_opts.json_stdout,
                stats_opts.sample_matches,
                stats_opts.fail_over_threshold,
                cli.disable_donation_prompts,
            )?;
        }
        Some(Commands::Uninstall { yes }) => {
            commands::uninstall::run_uninstall_command(yes, &theme_map)?;
        }
        None => {
            // Move compatibility checks inside the None arm
            // to ensure they apply only to the main sanitization flow.
            let effective_diff = cli.diff && !cli.no_diff;
            let effective_clipboard = cli.clipboard && !cli.no_clipboard;

            if cli.line_buffered {
                if effective_diff {
                    commands::cleansh::error_msg("Error: --line-buffered is incompatible with --diff.", &theme_map);
                    std::process::exit(1);
                }
                if effective_clipboard {
                    commands::cleansh::error_msg("Error: --line-buffered is incompatible with --clipboard.", &theme_map);
                    std::process::exit(1);
                }
                if cli.input_file.is_some() {
                    commands::cleansh::error_msg("Error: --line-buffered is incompatible with --input-file. Use piping for streaming input.", &theme_map);
                    std::process::exit(1);
                }
            }

            // Line-buffered mode for stdin
            if cli.line_buffered && cli.input_file.is_none() && !io::stdin().is_terminal() {
                let mut output_writer: Box<dyn Write> = if let Some(path) = &cli.output {
                    commands::cleansh::warn_msg(
                        "Warning: --line-buffered is intended for real-time console output. \
                         Outputting to a file (--output) will still buffer by line, \
                         but real-time benefits might be less apparent.",
                        &theme_map,
                    );
                    Box::new(std::fs::File::create(path)?)
                } else {
                    // Don't print this banner when --quiet is active
                    if !cli.quiet {
                        commands::cleansh::info_msg(
                            "Reading input from stdin in real-time, line-buffered mode...",
                            &theme_map,
                        );
                    }
                    Box::new(io::stdout())
                };

                let compiled_rules = commands::cleansh::load_and_compile(
                    cli.config.clone(),
                    cli.rules.clone(),
                    cli.enable.clone(),
                    cli.disable.clone(),
                )?;

                log::debug!("Compiled rules count: {}", compiled_rules.rules.len());
                if compiled_rules.rules.is_empty() {
                    log::warn!("No rules were compiled. Redactions will not occur.");
                }

                let mut all_redaction_matches = Vec::new();
                let stdin = io::stdin().lock(); // Now stdin is pristine

                for raw_line in stdin.lines() {
                    let line_content = raw_line?;

                    log::debug!("Processing line: {:?}", line_content);

                    let (sanitized_segment, mut line_matches) =
                        commands::cleansh::sanitize_single_line(&line_content, &compiled_rules);

                    log::debug!("Sanitized segment: {:?}", sanitized_segment);
                    log::debug!("Matches found for line: {}", line_matches.len());

                    output_writer.write_all(sanitized_segment.as_bytes())?;
                    output_writer.write_all(b"\n")?;
                    output_writer.flush()?;

                    all_redaction_matches.append(&mut line_matches);
                }

                if all_redaction_matches.is_empty() {
                    if !cli.no_summary {
                        commands::cleansh::info_msg(
                            "No redactions applied.",
                            &theme_map,
                        );
                    }
                } else {
                    if !cli.no_summary && !cli.quiet {
                        let summary = commands::cleansh::build_redaction_summary_from_matches(&all_redaction_matches);
                        // Ensures the summary header matches the test expectation
                        commands::cleansh::info_msg(
                            "Redaction Summary",
                            &theme_map,
                        );
                        ui::redaction_summary::print_summary(&summary, &mut io::stderr(), &theme_map, stderr_supports_color)?;
                    }
                }
            } else {
                // Regular, non-line-buffered flow: Consume all of stdin here
                let input_content = if let Some(path) = cli.input_file.as_ref() {
                    commands::cleansh::info_msg(format!("Reading input from file: {}", path.display()), &theme_map);
                    fs::read_to_string(path)
                        .with_context(|| format!("Failed to read input from {}", path.display()))?
                } else if io::stdin().is_terminal() {
                    commands::cleansh::info_msg("Reading input from stdin. Press Ctrl+Z then Enter to finish input.", &theme_map);
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)
                        .context("Failed to read from stdin")?;
                    buffer
                } else {
                    commands::cleansh::info_msg("Reading input from stdin...", &theme_map);
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)
                        .context("Failed to read from stdin")?;
                    buffer
                };

                let opts = commands::cleansh::CleanshOptions {
                    input: input_content,
                    clipboard: effective_clipboard,
                    diff: effective_diff,
                    config_path: cli.config.clone(),
                    rules_config_name: cli.rules.clone(),
                    output_path: cli.output.clone(),
                    no_redaction_summary: cli.no_summary,
                    enable_rules: cli.enable.clone(),
                    disable_rules: cli.disable.clone(),
                };

                if let Err(e) = commands::cleansh::run_cleansh_opts(opts, &theme_map) {
                    commands::cleansh::error_msg(
                        &format!("An error occurred: {}", e),
                        &theme_map,
                    );
                    std::process::exit(1);
                }
            }
        }
    }

    // Always attempt to save AppState before exiting
    app_state.save(&app_state_path)?;

    info!("cleansh finished successfully.");
    Ok(())
}