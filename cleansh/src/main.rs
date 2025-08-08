//! Cleansh CLI Application
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

use cleansh_core::{
    engine::{SanitizationEngine, RegexEngine}, config::{merge_rules, RedactionConfig}
};
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::io::{self, Read, Write, IsTerminal, BufReader, BufRead};
use std::fs;
use std::env;
use std::path::PathBuf;
use log::{info, LevelFilter};
use dotenvy;
use std::collections::HashMap;

use cleansh::commands;
use cleansh::logger;
use cleansh::ui;
use cleansh::utils::app_state::AppState;
use cleansh::utils::platform;
use cleansh::cli::{Cli, Commands, EngineChoice};

/// Creates a fully configured and compiled sanitization engine based on CLI arguments.
///
/// This helper function centralizes the logic for loading default rules,
/// merging with a custom configuration, and applying enable/disable filters.
/// It returns a `Box<dyn SanitizationEngine>`, allowing the application to
/// support different sanitization methods polymorphically.
fn create_sanitization_engine(
    config_path: Option<PathBuf>,
    engine_choice: EngineChoice,
    enable_rules: &[String],
    disable_rules: &[String],
) -> Result<Box<dyn SanitizationEngine>> {
    // 1. Load and merge rules
    let mut config = RedactionConfig::load_default_rules()
        .context("Failed to load default redaction rules")?;

    if let Some(path) = config_path {
        let user_config = RedactionConfig::load_from_file(&path)
            .context("Failed to load user-defined configuration file")?;
        config = merge_rules(config, Some(user_config));
    }

    // 2. Apply enable and disable filters
    config.set_active_rules(enable_rules, disable_rules);

    // 3. Instantiate the selected engine
    let engine: Box<dyn SanitizationEngine> = match engine_choice {
        EngineChoice::Regex => {
            Box::new(RegexEngine::new(config)
                .context("Failed to initialize RegexEngine")?)
        },
        EngineChoice::Entropy => {
            return Err(anyhow!("The 'entropy' engine is not yet implemented."));
        }
    };
    
    Ok(engine)
}

/// Reads input content from a file or stdin, handling both terminal and non-terminal cases.
///
/// This helper function centralizes the logic for reading input, avoiding code duplication.
fn read_input(input_file: &Option<PathBuf>, theme_map: &ui::theme::ThemeMap) -> Result<String> {
    if let Some(path) = input_file.as_ref() {
        commands::cleansh::info_msg(format!("Reading input from file: {}", path.display()), theme_map);
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read input from {}", path.display()))
    } else if io::stdin().is_terminal() {
        commands::cleansh::info_msg(
            &format!("Reading input from stdin. Press {} then Enter to finish input.", platform::eof_key_combo()),
            theme_map,
        );
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        Ok(buffer)
    } else {
        commands::cleansh::info_msg("Reading input from stdin...", theme_map);
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        Ok(buffer)
    }
}

/// A placeholder function for line-buffered mode.
/// This would be fully implemented to read input line-by-line and sanitize each line.
fn run_line_buffered_mode(engine: Box<dyn SanitizationEngine>, cli: &Cli) -> Result<()> {
    let stdin = io::stdin().lock();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    let mut total_matches = 0usize;
    let mut match_counts: HashMap<String, usize> = HashMap::new();

    // Use a Boxed writer to handle either stdout or a file
    let mut writer: Box<dyn Write> = if let Some(path) = cli.output.as_ref() {
        Box::new(fs::File::create(path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?)
    } else {
        Box::new(io::stdout().lock())
    };

    while reader.read_line(&mut line)? > 0 {
        let (sanitized_line, matches) =
            commands::cleansh::sanitize_single_line_with_count(&line, &*engine);
        
        // Write the sanitized line and a newline to the writer
        write!(writer, "{}", sanitized_line)?;
        
        // Aggregate counts from the returned HashMap
        for (rule, count) in matches {
            *match_counts.entry(rule).or_insert(0) += count;
            total_matches += count;
        }

        line.clear(); // Clear the buffer for the next line
    }

    // After streaming, print summary:
    if total_matches == 0 {
        eprintln!("No redactions applied.");
    } else if !cli.quiet && !cli.no_summary {
        eprintln!("--- Redaction Summary ---");
        for (rule, &count) in &match_counts {
            eprintln!("{}: {} occurrences", rule, count);
        }
    }

    Ok(())
}

/// Handles the main `cleansh` command without a subcommand.
/// This function encapsulates the core logic for sanitizing content.
fn handle_main_command(cli: &Cli, engine: Box<dyn SanitizationEngine>, theme_map: &ui::theme::ThemeMap) -> Result<()> {
    if cli.line_buffered {
        // banner always to stderr, but only if we're not in quiet mode
        if !cli.quiet {
            eprintln!("\x1B[1;33mUsing line-buffered mode. Incompatible with --diff, --clipboard, and --input-file.\x1B[0m");
        }
        // warn when output-to-file is in use (even if quiet)
        if cli.output.is_some() {
            eprintln!(
                "Warning: --line-buffered is intended for real-time console output. \
                 Outputting to a file (--output) will still buffer by line, \
                 but real-time benefits might be less apparent."
            );
        }
        run_line_buffered_mode(engine, cli)?;
    } else {
        let input_content = read_input(&cli.input_file, theme_map)?;

        let cleansh_options = commands::cleansh::CleanshOptions {
            input: input_content,
            clipboard: cli.clipboard,
            diff: cli.diff,
            output_path: cli.output.clone(),
            no_redaction_summary: cli.no_summary,
            quiet: cli.quiet,
        };
        commands::cleansh::run_cleansh_opts(&*engine, cleansh_options, theme_map)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // ── Honor test override for app state path ───────────────────────────────────
    let app_state_path: PathBuf = env::var("CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if let Some(dir) = dirs::data_dir() {
                dir.join("cleansh").join("state.json")
            } else {
                env::current_dir()
                    .expect("Failed to get current dir")
                    .join("cleansh_state.json")
            }
        });
    // ── End override block ─────────────────────────────────────────────────────

    // Determine log level
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

    // Load or create the AppState
    let mut app_state = AppState::load(&app_state_path)?;
    // Fix: Set the donation prompts disabled state after loading, so the value from the CLI
    // overwrites any previous state.
    app_state.donation_prompts_disabled = cli.disable_donation_prompts;

    let theme_map = ui::theme::build_theme_map(cli.theme.as_ref())?;

    match cli.command {
        Some(Commands::Stats(ref stats_opts)) => {
            let input_content = read_input(&cli.input_file, &theme_map)?;
            let engine = create_sanitization_engine(
                cli.config.clone(),
                cli.engine.clone(),
                &cli.enable,
                &cli.disable,
            )?;

            commands::stats::run_stats_command(
                &input_content,
                &*engine,
                &theme_map,
                stats_opts,
            )?;

            // Increment stats-only counter
            app_state.increment_stats_only_usage();

            // Check and prompt for donation, then save the state.
            if !app_state.donation_prompts_disabled {
                if let Err(e) = app_state.check_and_prompt_donation(&theme_map) {
                    commands::cleansh::error_msg(format!("Failed to handle donation prompt: {}", e), &theme_map);
                }
            }
            app_state.save(&app_state_path)?;
        }
        Some(Commands::Uninstall { yes }) => {
            commands::uninstall::run_uninstall_command(yes, &theme_map)?;
        }
        None => {
            if cli.line_buffered && (cli.diff || cli.clipboard || cli.input_file.is_some()) {
                commands::cleansh::error_msg(
                    "Error: --line-buffered is incompatible with --diff, --clipboard, and --input-file.",
                    &theme_map,
                );
                std::process::exit(1);
            }

            let engine = create_sanitization_engine(
                cli.config.clone(),
                cli.engine.clone(),
                &cli.enable,
                &cli.disable,
            )?;
            handle_main_command(&cli, engine, &theme_map)?;

            // Increment general usage counter
            app_state.increment_usage();

            // Check and prompt for donation, then save the state.
            if !app_state.donation_prompts_disabled {
                if let Err(e) = app_state.check_and_prompt_donation(&theme_map) {
                    commands::cleansh::error_msg(format!("Failed to handle donation prompt: {}", e), &theme_map);
                }
            }
            app_state.save(&app_state_path)?;
        }
    }

    info!("cleansh finished successfully.");
    Ok(())
}