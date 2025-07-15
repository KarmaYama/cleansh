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
use log::{info};
use dotenvy;

pub mod commands;
pub mod config;
pub mod logger;
pub mod tools;
pub mod ui;
pub mod utils; // NEW: Expose the new utils module

/// CLI definition
#[derive(Parser, Debug)]
#[command(author = "Cleansh Technologies LLC", version, about = "Sanitize your terminal output. One tool. One purpose.")]
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
    #[arg(short = 'i', long, value_name = "FILE")] // MODIFIED: Added short and long attributes
    pub input_file: Option<PathBuf>,
    #[arg(long, value_name = "FILE")]
    pub theme: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_redaction_summary: bool,
    #[arg(long = "enable-rules", value_delimiter = ',')]
    pub enable_rules: Vec<String>,
    #[arg(long = "disable-rules", value_delimiter = ',')]
    pub disable_rules: Vec<String>,
}

/// Test-only exports
#[cfg(any(test, feature = "test-exposed"))]
pub mod test_exposed {
    pub mod config {
        pub use crate::config::*;
    }
    pub mod tools {
        pub use crate::tools::sanitize_shell::*;
    }
    pub mod commands {
        pub use crate::commands::cleansh::run_cleansh;
    }
    pub mod ui {
        pub use crate::ui::theme;
        pub use crate::ui::output_format;
        pub use crate::ui::redaction_summary;
        pub use crate::ui::diff_viewer;
    }
    // NEW: Expose redaction utilities for testing
    pub mod utils {
        pub use crate::utils::redaction::*;
    }
}

/// Main library entry
pub fn run(cli: Cli) -> Result<()> {
    dotenvy::dotenv().ok();

    // Logging and flags
    let effective_debug = cli.debug && !cli.disable_debug;
    let effective_clipboard = cli.clipboard && !cli.disable_clipboard;
    let effective_diff = cli.diff && !cli.disable_diff;
    if effective_debug {
        unsafe { env::set_var("RUST_LOG", "debug") }
    } else if env::var("RUST_LOG").is_err() {
        if let Ok(val) = env::var("LOG_LEVEL") {
            unsafe { env::set_var("RUST_LOG", val) }
        }
    }
    logger::init_logger();
    info!("cleansh started. Version: {}", env!("CARGO_PKG_VERSION"));

    // Theme map
    let theme_map: HashMap<ui::theme::ThemeEntry, ui::theme::ThemeStyle> =
        if let Some(t) = cli.theme.as_ref() {
            ui::theme::ThemeStyle::load_from_file(t).unwrap_or_else(|e| {
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
    if let Some(path) = cli.input_file.as_ref() {
        // Changed to stderr
        ui::output_format::print_info_message(
            &mut io::stderr(),
            &format!("Reading input from file: {}", path.display()),
            &theme_map,
        );
        input_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read input from {}", path.display()))?;
    } else {
        // Changed to stderr
        ui::output_format::print_info_message(
            &mut io::stderr(),
            "Reading input from stdin...",
            &theme_map,
        );
        io::stdin().read_to_string(&mut input_content)
            .context("Failed to read from stdin")?;
    }

    // Delegate to core command
    if let Err(e) = commands::cleansh::run_cleansh(
        &input_content,
        effective_clipboard,
        effective_diff,
        cli.config.clone(),
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

    info!("cleansh finished successfully.");
    Ok(())
}