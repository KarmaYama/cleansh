// src/main.rs

// Standard library imports for I/O and path manipulation
use std::io::{self, Read};
use std::path::PathBuf;

// Third-party crate imports for CLI parsing, logging, and error handling
use anyhow::{Context, Result}; // For robust error propagation and context
use clap::Parser; // For declarative command-line argument parsing
use log::{debug, info}; // For structured logging

// Internal module imports. These modules will be defined within the `src` directory
// and contain the specialized logic for configuration, logging, and command execution.
mod commands; // Contains the core CLI logic and command handlers (e.g., cleansh command)
mod config;   // Handles loading and merging redaction rules
mod logger;   // Manages the application's logging setup

/// cleansh - A high-trust, single-purpose CLI tool that sanitizes terminal output for safe sharing.
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
    version = "0.1.0",
    about = "Sanitize your terminal output. One tool. One purpose.",
    long_about = "cleansh is a robust and secure command-line utility designed to redact sensitive information from your terminal output before sharing. It supports masking emails, IP addresses, various tokens (JWTs, AWS, GCP), SSH keys, hex secrets, and normalizing absolute paths. Secure by default, zero config required, and extendable when needed."
)]
struct Cli {
    /// Copy the sanitized result to the system clipboard.
    /// This flag makes the clipboard action opt-in, enhancing security by default.
    #[arg(short, long, help = "Copy the sanitized result to the system clipboard.")]
    clipboard: bool,

    /// Show a detailed diff view highlighting all redactions made to the input.
    /// This helps users understand what was changed.
    #[arg(short, long, help = "Show a detailed diff view highlighting redactions.")]
    diff: bool,

    /// Specify a custom YAML configuration file for redaction rules.
    /// These rules will be merged with or override the default embedded rules.
    #[arg(short, long, value_name = "FILE", help = "Load a custom YAML configuration file for redaction rules.")]
    config: Option<PathBuf>,

    /// Output the sanitized content to a specified file instead of printing to standard output.
    #[arg(short, long, value_name = "FILE", help = "Output the sanitized content to a specified file instead of stdout.")]
    out: Option<PathBuf>,

    /// Enable debug logging for more verbose output.
    /// This flag overrides the `LOG_LEVEL` environment variable if set to a higher verbosity.
    #[arg(long, help = "Enable debug logging for more verbose output.")]
    debug: bool,

    /// Optional input file path. If not provided, cleansh reads from standard input (stdin).
    /// This allows for flexible input sources.
    #[arg(value_name = "INPUT", help = "Optional input file to read from. Reads from stdin if not provided.")]
    input_file: Option<PathBuf>,
}

/// The main entry point for the cleansh application.
///
/// This function initializes the logger, parses command-line arguments,
/// determines the input source (stdin or file), and then delegates
/// the core sanitization logic to the `commands::cleansh` module.
fn main() -> Result<()> {
    // 1. Initialize the logger.
    // The `debug` flag from CLI arguments directly controls the logger's verbosity.
    logger::init_logger(Cli::parse().debug); // Parse args once to get `debug` flag for logger init

    // Re-parse arguments for full processing after logger is initialized.
    // Clap is efficient enough that parsing twice here is negligible, but it cleanly separates
    // logger setup from main logic.
    let cli = Cli::parse();

    info!("cleansh started. Version: {}", clap::crate_version!());
    debug!("Parsed CLI arguments: {:?}", cli);

    // 2. Determine the input source and read content.
    let mut input_content = String::new();
    if let Some(input_path) = cli.input_file {
        info!("Reading input from file: {}", input_path.display());
        // Use `std::fs::read_to_string` for convenience to read the entire file.
        input_content = std::fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read input from file: {}", input_path.display()))?;
    } else {
        info!("Reading input from stdin...");
        // Read all available data from standard input until EOF.
        io::stdin()
            .read_to_string(&mut input_content)
            .context("Failed to read input from stdin")?;
    }

    // 3. Delegate the core sanitization workflow.
    // All specific logic for sanitization, output formatting, diffing,
    // and clipboard operations is handled within `commands::cleansh::run_cleansh`.
    commands::cleansh::run_cleansh(
        &input_content,
        cli.clipboard,
        cli.diff,
        cli.config,
        cli.out,
    )?;

    info!("cleansh finished successfully.");
    Ok(())
}