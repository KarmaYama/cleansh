// src/main.rs

// Standard library imports for I/O and path manipulation
use std::io::{self, Read};
use std::path::PathBuf;
use std::fs;
use std::process::exit;
use std::collections::HashMap;
use std::env;

// Third-party crate imports for CLI parsing, logging, and error handling
use anyhow::{Context, Result};
use clap::{Parser, ArgAction};
use log::{debug, info};
use dotenvy;

// Internal module imports. These modules will be defined within the `src` directory
mod commands;
mod config;
mod logger;
mod tools;
mod ui;

/// `cleansh` - A high-trust, single-purpose CLI tool that sanitizes terminal output for safe sharing.
///
/// Secure by default. Zero config required. Extendable when needed.
///
/// This tool processes input from stdin or a specified file, redacting sensitive
/// information such as emails, IP addresses, various tokens (JWTs, AWS, GCP),
/// SSH keys, hex secrets, and normalizing absolute file paths.
///
/// It offers optional features like copying the sanitized output to the clipboard,
/// displaying a diff view of the changes, loading custom redaction rules from a
/// YAML file, and outputting to a file instead of stdout.
#[derive(Parser, Debug)]
#[command(
    author = "Cleansh Technologies LLC",
    version = env!("CARGO_PKG_VERSION"),
    about = "Sanitize your terminal output. One tool. One purpose.",
    long_about = "cleansh is a robust and secure command-line utility designed to redact sensitive information from your terminal output before sharing. It supports masking emails, IP addresses, various tokens (JWTs, AWS, GCP), SSH keys, hex secrets, and normalizing absolute paths. Secure by default, zero config required, and extendable when needed."
)]
struct Cli {
    /// Copy the sanitized result to the system clipboard.
    /// Short flag is `-c`. Default can be set via CLIPBOARD_ENABLED env var.
    #[arg(short, long, help = "Copy the sanitized result to the system clipboard.", env = "CLIPBOARD_ENABLED", action = ArgAction::SetTrue)]
    clipboard: bool,

    /// Do NOT copy the sanitized result to the system clipboard. Overrides CLIPBOARD_ENABLED env var.
    #[arg(long = "no-clipboard", help = "Do NOT copy the sanitized result to the system clipboard.", action = ArgAction::SetTrue)]
    disable_clipboard: bool,

    /// Show a detailed diff view highlighting all redactions made to the input.
    /// Short flag is `-d`.
    #[arg(short, long, help = "Show a detailed diff view highlighting redactions.", action = ArgAction::SetTrue)]
    diff: bool,

    /// Do NOT show a detailed diff view.
    #[arg(long = "no-diff", help = "Do NOT show a detailed diff view.", action = ArgAction::SetTrue)]
    disable_diff: bool,

    /// Specify a custom YAML configuration file for redaction rules.
    /// Long-only `--config` to avoid conflicts.
    #[arg(long, value_name = "FILE", help = "Load a custom YAML configuration file for redaction rules.")]
    config: Option<PathBuf>,

    /// Output the sanitized content to a specified file instead of printing to stdout.
    /// Short flag is `-o`.
    #[arg(short = 'o', long, value_name = "FILE", help = "Output the sanitized content to a specified file instead of stdout.")]
    out: Option<PathBuf>,

    /// Enable debug logging for more verbose output. Overrides LOG_LEVEL env var.
    #[arg(long, help = "Enable debug logging for more verbose output.", action = ArgAction::SetTrue)]
    debug: bool,

    /// Do NOT enable debug logging.
    #[arg(long = "no-debug", help = "Do NOT enable debug logging.", action = ArgAction::SetTrue)]
    disable_debug: bool,

    /// Optional input file path. Reads from stdin if not provided.
    #[arg(value_name = "INPUT", help = "Optional input file to read from. Reads from stdin if not provided.")]
    input_file: Option<PathBuf>,

    /// Specify a custom YAML theme file for output styling.
    #[arg(long, value_name = "FILE", help = "Load a custom YAML theme file for output styling.")]
    theme: Option<PathBuf>,

    /// Do not display the redaction summary at the end of the output.
    #[arg(long, help = "Do not display the redaction summary at the end of the output.", action = ArgAction::SetTrue)]
    no_redaction_summary: bool,

    /// Enable these opt-in rule names, comma-separated. E.g., `aws_secret_key,generic_token`.
    #[clap(long = "enable-rules", value_delimiter = ',', help = "Enable these opt-in rule names (comma-separated).")]
    enable_rules: Vec<String>,

    /// Disable these rule names, comma-separated. E.g., `email,ipv4_address`.
    #[clap(long = "disable-rules", value_delimiter = ',', help = "Disable these rule names (comma-separated).")]
    disable_rules: Vec<String>,
}

/// The main entry point of the `cleansh` application.
///
/// This function parses command-line arguments, initializes logging,
/// reads input content, and delegates to the core `run_cleansh` command logic.
/// It handles top-level errors by printing a user-friendly message and exiting.
///
/// # Returns
///
/// Returns `Ok(())` on successful application execution.
/// Exits with a non-zero status code on error.
fn main() -> Result<()> {
    // Load environment variables from .env file first.
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Determine effective values: 'disable' flags take precedence.
    let effective_debug = cli.debug && !cli.disable_debug;
    let effective_clipboard = cli.clipboard && !cli.disable_clipboard;
    let effective_diff = cli.diff && !cli.disable_diff;

    // 1. Initialize the logger.
    // Set RUST_LOG environment variable based on effective_debug or LOG_LEVEL env var
    if effective_debug {
        // FIX: Wrap env::set_var in unsafe block for Rust 2024 edition compatibility
        unsafe {
            env::set_var("RUST_LOG", "debug");
        }
    } else if env::var("RUST_LOG").is_err() {
        if let Ok(log_level_env) = env::var("LOG_LEVEL") {
            // FIX: Wrap env::set_var in unsafe block for Rust 2024 edition compatibility
            unsafe {
                env::set_var("RUST_LOG", log_level_env);
            }
        }
    }
    logger::init_logger(); // Initialize the logger after setting RUST_LOG

    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));
    debug!("Parsed CLI arguments: {:?}", cli); // `cli` is still fully available here

    // Clone necessary fields for ownership transfer to `run_cleansh`
    let cloned_config_path = cli.config.clone();
    let cloned_out_path = cli.out.clone();
    let cloned_enable_rules = cli.enable_rules.clone();
    let cloned_disable_rules = cli.disable_rules.clone();

    debug!("Effective Debug: {}, Effective Clipboard: {}, Effective Diff: {}", effective_debug, effective_clipboard, effective_diff);
    debug!("Explicitly enabled opt-in rules: {:?}", cloned_enable_rules);
    debug!("Explicitly disabled rules: {:?}", cloned_disable_rules);


    // Load theme from file or use embedded defaults
    let theme_map: HashMap<ui::theme::ThemeEntry, ui::theme::ThemeStyle> =
        if let Some(theme_path) = cli.theme {
            ui::theme::ThemeStyle::load_from_file(&theme_path)
                .unwrap_or_else(|e| {
                    ui::output_format::print_warn_message(
                        &mut io::stderr(),
                        &format!("Failed to load custom theme from {}: {}. Falling back to default white theme.", theme_path.display(), e),
                        &ui::theme::ThemeStyle::default_theme_map(),
                    );
                    log::warn!("Failed to load custom theme from {}: {}. Falling back to default white theme.", theme_path.display(), e);
                    ui::theme::ThemeStyle::default_theme_map()
                })
        } else {
            ui::theme::ThemeStyle::default_theme_map()
        };

    // 2. Determine the input source and read content.
    let mut input_content = String::new();
    if let Some(input_path) = cli.input_file {
        info!("Reading input from file: {}", input_path.display());
        ui::output_format::print_info_message(
            &mut io::stdout(),
            &format!("Reading input from file: {}", input_path.display()),
            &theme_map,
        );
        input_content = fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read input from file: {}", input_path.display()))?;
    } else {
        info!("Reading input from stdin...");
        ui::output_format::print_info_message(
            &mut io::stdout(),
            "Reading input from stdin...",
            &theme_map,
        );
        io::stdin()
            .read_to_string(&mut input_content)
            .context("Failed to read input from stdin")?;
    }

    // 3. Delegate the core sanitization workflow.
    if let Err(e) = commands::cleansh::run_cleansh(
        &input_content,
        effective_clipboard, // Use effective value
        effective_diff,      // Use effective value
        cloned_config_path,  // Pass cloned path
        cloned_out_path,     // Pass cloned path
        cli.no_redaction_summary,
        &theme_map,
        cloned_enable_rules, // Pass cloned rules
        cloned_disable_rules, // Pass cloned rules
    ) {
        ui::output_format::print_error_message(
            &mut io::stderr(),
            &format!("An error occurred: {}", e),
            &theme_map,
        );
        exit(1);
    }

    info!("cleansh finished successfully.");
    Ok(())
}