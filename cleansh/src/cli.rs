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


use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Top-level CLI definition.
#[derive(Parser, Debug)]
#[command(
    name = "cleansh",
    author = "Obscura Team (Obscura Tech)", // Added author as per best practice
    version = env!("CARGO_PKG_VERSION"), // Use env! for version
    about = "Securely redact sensitive data from text",
    long_about = "Cleansh is a command-line utility for securely redacting sensitive information from text-based data. It helps you sanitize logs, code, documents, or terminal output to ensure that Personally Identifiable Information (PII) and other sensitive patterns are removed or obfuscated according to a configurable rule set.",
    // Crucial for cli_integration_tests:
    // Allow options to be placed anywhere on the command line, not just before subcommands
    arg_required_else_help = false, // Display help if no arguments, but don't require args if a subcommand is chosen
    allow_missing_positional = true, // Allows for flexible argument order without strict positional requirements
    allow_external_subcommands = true, // Allows for future custom commands if needed
)]
pub struct Cli {
    /// Copy sanitized output to clipboard
    #[arg(short = 'c', long = "clipboard", default_value = "false", help = "Copy sanitized output to the system clipboard.")]
    pub clipboard: bool,

    /// Show a unified diff between original and sanitized
    #[arg(short = 'D', long = "diff", default_value = "false", help = "Show a unified diff to highlight the changes made during the sanitization process.")] // CHANGED: short = 'D', refined help text
    pub diff: bool,

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
    #[arg(short = 'e', long = "enable", value_delimiter = ',', help = "Explicitly enable only these rule names (comma-separated).")] // ADDED: short 'e'
    pub enable: Vec<String>,

    /// Explicitly disable these rule names (comma-separated)
    #[arg(short = 'x', long = "disable", value_delimiter = ',', help = "Explicitly disable these rule names (comma-separated).")] // ADDED: short 'x'
    pub disable: Vec<String>,

    /// Select which sanitization engine to use
    #[arg(long = "engine", value_name = "ENGINE", default_value = "regex", help = "Select a sanitization engine (e.g., 'regex').")]
    pub engine: EngineChoice,

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

/// Enum for selecting the sanitization engine.
///
/// This provides a type-safe way to handle engine choices.
#[derive(Debug, Clone, ValueEnum)]
pub enum EngineChoice {
    /// The default regular expression engine.
    Regex,
    /// An example of another engine. This would be a future feature.
    Entropy,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze content for sensitive data without redacting it, providing statistics.
    /// This is essentially the `--stats-only` mode from earlier versions.
    #[command(about = "Analyze input for sensitive data and provide a summary without redacting it.")]
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