// cleansh-workspace/cleansh/src/cli.rs
//! This file defines the command-line interface (CLI) for the cleansh application,
//! including all available commands and their arguments.
//! It uses the `clap` library to parse command-line arguments and subcommands.
//! The CLI structure is designed to be extensible, allowing for future commands and options.
//! It also includes global flags that apply across commands, such as debug logging and theme customization.
//! The main command is `cleansh`, with subcommands like `stats` and `uninstall`.
//! This file is the entry point for the CLI, and it integrates with the `cleansh`
//! application logic to execute the appropriate commands based on user input.
//!
//! The CLI is designed to be user-friendly, with clear help messages and options for customization.
//! License: Polyform Noncommercial License 1.0.0


use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Top-level CLI definition.
#[derive(Parser, Debug)]
#[command(
    name = "cleansh",
    author = "Cleansh Technologies (Clean Shell)", // Added author as per best practice
    version = env!("CARGO_PKG_VERSION"), // Use env! for version
    about = "Securely redact sensitive data from text",
    long_about = "Cleansh is a command-line utility designed to help you sanitize sensitive information from your text-based data, such as logs, documents, or terminal output.",
    // Crucial for cli_integration_tests:
    // Allow options to be placed anywhere on the command line, not just before subcommands
    arg_required_else_help = false, // Display help if no arguments, but don't require args if a subcommand is chosen
    allow_missing_positional = true, // Allows for flexible argument order without strict positional requirements
    allow_external_subcommands = true, // Allows for future custom commands if needed
)]
pub struct Cli {
    /// Copy sanitized output to clipboard
    #[arg(short = 'c', long = "clipboard", conflicts_with = "no_clipboard", help = "Copy sanitized output to the system clipboard.")]
    pub clipboard: bool,

    /// Do not copy sanitized output to clipboard
    #[arg(long = "no-clipboard", conflicts_with = "clipboard", help = "Explicitly prevent copying output to clipboard.")]
    pub no_clipboard: bool, // NEW: Added for explicit disabling

    /// Show a unified diff between original and sanitized
    #[arg(short = 'D', long = "diff", conflicts_with = "no_diff", help = "Show a unified diff between original and sanitized content.")] // CHANGED: short = 'D'
    pub diff: bool,

    /// Do not show diff
    #[arg(long = "no-diff", conflicts_with = "diff", help = "Explicitly prevent showing a diff.")]
    pub no_diff: bool, // NEW: Added for explicit disabling

    /// Path to custom redaction config (YAML)
    #[arg(long = "config", value_name = "FILE", help = "Path to a custom redaction configuration file (YAML).")] // Added value_name for clarity
    pub config: Option<PathBuf>,

    /// Named rule set to apply (e.g. "strict")
    #[arg(long = "rules", value_name = "NAME", help = "Name of a specific rule set/profile to apply.")] // Added value_name
    pub rules: Option<String>,

    /// Write output to this file instead of stdout
    #[arg(short = 'o', long = "output", value_name = "FILE", help = "Write output to a specified file instead of stdout.")] // Added short 'o' and value_name
    pub output: Option<PathBuf>,

    /// Suppress redaction summary
    #[arg(long = "no-redaction-summary", help = "Suppress the redaction summary.")] // Changed name to match test convention
    pub no_summary: bool,

    /// Explicitly enable only these rule names (comma-separated)
    #[arg(long = "enable", value_delimiter = ',', help = "Explicitly enable only these rule names (comma-separated).")] // Added long name
    pub enable: Vec<String>,

    /// Explicitly disable these rule names (comma-separated)
    #[arg(long = "disable", value_delimiter = ',', help = "Explicitly disable these rule names (comma-separated).")] // Added long name
    pub disable: Vec<String>,

    // --- Global flags used across commands, or for the main command ---

    /// Path to an input file (reads from stdin if not provided)
    #[arg(long, short = 'i', value_name = "FILE", help = "Read input from a specified file instead of stdin.")] // Added value_name
    pub input_file: Option<PathBuf>,

    /// Process input line by line (incompatible with --diff, --clipboard, --input-file)
    #[arg(long, help = "Process input line by line (useful for streaming data from pipes).")]
    pub line_buffered: bool,

    /// Disable informational messages
    #[arg(long, short = 'q', help = "Suppress all informational and debug messages.")]
    pub quiet: bool,

    /// Enable debug logging (overrides RUST_LOG for 'cleansh' crate to DEBUG)
    #[arg(long, short = 'd', help = "Enable debug logging.")] // RETAINED: short = 'd' for debug
    pub debug: bool,

    /// Explicitly disable debug logging, even if RUST_LOG is set to DEBUG (useful for testing)
    #[arg(long = "disable-debug", help = "Disable debug logging, overriding RUST_LOG.")]
    pub disable_debug: bool,

    /// Custom theme for terminal output (YAML file path)
    #[arg(long = "theme", value_name = "FILE", help = "Specify the path to a custom YAML theme file.")] // Changed type to PathBuf for file path
    pub theme: Option<PathBuf>,

    /// Disable donation prompts that appear after certain usage thresholds
    #[arg(long = "disable-donation-prompts", help = "Disable future prompts for donations.")]
    pub disable_donation_prompts: bool,

    /// Subcommands like `stats` or `uninstall`
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze content for sensitive data without redacting it, providing statistics.
    /// This is essentially the `--stats-only` mode from earlier versions.
    #[command(about = "Analyze input for sensitive data and provide a summary without redacting it, providing statistics.")]
    Stats(StatsCommand),
    /// Uninstall cleansh (cleanup registry, etc.)
    #[command(about = "Uninstall cleansh and remove its associated files.")]
    Uninstall {
        /// Proceed with uninstallation without confirmation.
        #[arg(long, short = 'y', help = "Proceed with uninstallation without a confirmation prompt.")]
        yes: bool,
    },
}

/// Arguments specific to the `stats` subcommand.
#[derive(Parser, Debug)]
pub struct StatsCommand {
    /// Export scan summary to a JSON file.
    #[arg(long = "json-file", value_name = "FILE", help = "Export the redaction statistics to a JSON file.")]
    pub json_file: Option<PathBuf>,

    /// Print scan summary as JSON to stdout (conflicts with --json-file).
    #[arg(long = "json-stdout", conflicts_with = "json_file", help = "Export the redaction statistics to stdout as JSON.")] // Added conflicts_with
    pub json_stdout: bool,

    /// Limit the number of unique sample matches displayed per rule in console output.
    #[arg(long = "sample-matches", value_name = "N", help = "Display a sample of up to N unique matches per rule in the console output.")]
    pub sample_matches: Option<usize>,

    /// Exit with a non-zero code if the total number of detected secrets exceeds this threshold.
    #[arg(long = "fail-over-threshold", value_name = "N", help = "Exit with a non-zero code if the total number of detected secrets exceeds this threshold.")]
    pub fail_over_threshold: Option<usize>,
}