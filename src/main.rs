// Standard library imports for I/O and path manipulation
use std::io::{self, Read};
use std::path::PathBuf;
use std::fs;
use std::process::exit;
use std::collections::HashMap; // Needed for Theme map

// Third‑party crate imports for CLI parsing, logging, and error handling
use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};

// Internal module imports. These modules will be defined within the `src` directory
mod commands;
mod config;
mod logger;
mod tools;
mod ui;

/// cleansh - A high‑trust, single‑purpose CLI tool that sanitizes terminal output for safe sharing.
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
    /// Short flag is `-c` (unique).
    #[arg(short, long, help = "Copy the sanitized result to the system clipboard.")]
    clipboard: bool,

    /// Show a detailed diff view highlighting all redactions made to the input.
    /// Short flag is `-d`.
    #[arg(short, long, help = "Show a detailed diff view highlighting redactions.")]
    diff: bool,

    /// Specify a custom YAML configuration file for redaction rules.
    /// **Long-only** `--config` (no short) to avoid conflicts.
    #[arg(long, value_name = "FILE", help = "Load a custom YAML configuration file for redaction rules.")]
    config: Option<PathBuf>,

    /// Output the sanitized content to a specified file instead of printing to stdout.
    /// Short flag is `-o`.
    #[arg(short = 'o', long, value_name = "FILE", help = "Output the sanitized content to a specified file instead of stdout.")]
    out: Option<PathBuf>,

    /// Enable debug logging for more verbose output.
    #[arg(long, help = "Enable debug logging for more verbose output.")]
    debug: bool,

    /// Optional input file path. Reads from stdin if not provided.
    #[arg(value_name = "INPUT", help = "Optional input file to read from. Reads from stdin if not provided.")]
    input_file: Option<PathBuf>,

    /// Specify a custom YAML theme file for output styling.
    #[arg(long, value_name = "FILE", help = "Load a custom YAML theme file for output styling.")]
    theme: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Parse arguments once and store them.
    let cli = Cli::parse();

    // 1. Initialize the logger.
    // If `--debug` is enabled, and RUST_LOG is not already set, set it to "debug".
    // Because `set_var` is marked unsafe in this toolchain, wrap it accordingly.
    if cli.debug && std::env::var("RUST_LOG").is_err() {
        // Safety: We are setting an environment variable that is not used by other parts
        // of the program in a way that would cause unsoundness. This is a common pattern
        // for configuring logging.
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
    }
    logger::init_logger();

    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));
    debug!("Parsed CLI arguments: {:?}", cli);

    // Load theme from file or use embedded defaults
    let theme_map: HashMap<ui::theme::ThemeEntry, ui::theme::ThemeStyle> =
        if let Some(theme_path) = cli.theme {
            ui::theme::ThemeStyle::load_from_file(&theme_path)
                .unwrap_or_else(|e| {
                    // Use print_warn_message for user-facing warning
                    ui::output_format::print_warn_message(
                        &mut io::stderr(),
                        &format!("Failed to load custom theme from {}: {}. Falling back to default white theme.", theme_path.display(), e),
                        &ui::theme::ThemeStyle::default_theme_map(), // Use default theme for this warning message
                    );
                    log::warn!("Failed to load custom theme from {}: {}. Falling back to default white theme.", theme_path.display(), e);
                    ui::theme::ThemeStyle::default_theme_map()
                })
        } else {
            // Fallback to a default theme (all white, or a predefined simple one)
            ui::theme::ThemeStyle::default_theme_map()
        };


    // 2. Determine the input source and read content.
    let mut input_content = String::new();
    if let Some(input_path) = cli.input_file {
        info!("Reading input from file: {}", input_path.display());
        ui::output_format::print_info_message( // Use print_info_message
            &mut io::stdout(),
            &format!("Reading input from file: {}", input_path.display()),
            &theme_map,
        );
        input_content = fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read input from file: {}", input_path.display()))?;
    } else {
        info!("Reading input from stdin...");
        ui::output_format::print_info_message( // Use print_info_message
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
        cli.clipboard,
        cli.diff,
        cli.config,
        cli.out,
        &theme_map, // Pass the loaded theme map
    ) {
        // Print a user‑friendly error message (now styled)
        ui::output_format::print_error_message(
            &mut io::stderr(),
            &format!("An error occurred: {}", e),
            &theme_map, // Pass the theme map for error messages
        );
        exit(1);
    }

    info!("cleansh finished successfully.");
    Ok(())
}