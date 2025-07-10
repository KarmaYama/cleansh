// src/logger.rs

use env_logger::{Builder, Env};
use log::LevelFilter;

/// Initializes the application's logger.
///
/// The log level is determined by the `LOG_LEVEL` environment variable.
/// If `LOG_LEVEL` is not set, it defaults to `info`.
///
/// This function sets up `env_logger` to output logs to stderr.
/// It should be called once at the beginning of the application's execution.
pub fn init_logger() {
    Builder::from_env(Env::default().filter_or("LOG_LEVEL", "info"))
        .format_timestamp_millis() // Include milliseconds in timestamp for more detail
        .init(); // Initialize the global logger
}

// Optional: You could add a helper to set a specific log level programmatically for tests
// or specific debug modes if `--debug` flag is used in combination with env_logger.
// For now, env_logger handles CLI flags automatically if `RUST_LOG` is used.
// If you want a custom CLI flag `--debug` to override LOG_LEVEL, you'd handle it in main.rs
// and pass the desired level filter here.