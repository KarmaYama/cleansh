// src/logger.rs
use env_logger::{Builder, Target};
use log::LevelFilter; // This import is now used

/// Initializes the application's logger.
///
/// It sets up `env_logger` to output logs to stderr,
/// with an info-level default filter unless overridden by `RUST_LOG` environment variable.
pub fn init_logger() {
    Builder::new()
        .filter_level(LevelFilter::Info) // Default log level
        .parse_env("RUST_LOG") // Allow RUST_LOG env var to override
        .target(Target::Stderr) // Log to standard error
        .init();
}