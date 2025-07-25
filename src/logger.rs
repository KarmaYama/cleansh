// src/logger.rs
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::env;

/// Initializes the application's logger with an optional explicit log level.
///
/// It sets up `env_logger` to output logs to stderr.
/// The `explicit_level` parameter, if `Some`, will override any `RUST_LOG`
/// environment variable for the 'cleansh' crate and set the global minimum.
/// Otherwise, `RUST_LOG` will be parsed, defaulting to `LevelFilter::Warn`
/// for the 'cleansh' crate and globally if `RUST_LOG` is not set.
/// Logs are formatted to include level, module path, and message.
pub fn init_logger(explicit_level: Option<LevelFilter>) {
    let mut builder = Builder::new();

    // Always parse RUST_LOG from the environment first.
    // This establishes the base configuration from the environment.
    builder.parse_env("RUST_LOG"); 

    // If an explicit level is provided via CLI flags, it takes precedence.
    if let Some(level) = explicit_level {
        // Set the filter specifically for the 'cleansh' crate.
        // This ensures the CLI flag directly controls your application's verbosity.
        builder.filter_module("cleansh", level);

        // Also, ensure the overall minimum log level is at least what the CLI specified.
        // This helps catch logs from other modules if they are below this level,
        // and ensures the CLI flag provides a floor for all logging.
        builder.filter_level(level); 
    } else {
        // If no explicit level from CLI, and RUST_LOG was not set,
        // default to `Warn` for the 'cleansh' crate and globally.
        // If RUST_LOG *was* set, `parse_env` already configured it.
        if env::var_os("RUST_LOG").is_none() {
            builder.filter_level(LevelFilter::Warn);
            builder.filter_module("cleansh", LevelFilter::Warn);
        }
    }

    builder
        .target(Target::Stderr)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                record.level(),
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .try_init() // Attempt to initialize. This implicitly calls `build()`.
        .ok();     // Ignore error if already initialized (e.g., in a test harness).
}