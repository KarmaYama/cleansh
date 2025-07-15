// src/logger.rs
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write; 

/// Initializes the application's logger with an optional explicit log level.
///
/// It sets up `env_logger` to output logs to stderr.
/// The `explicit_level` parameter, if `Some`, will override any `RUST_LOG`
/// environment variable. Otherwise, `RUST_LOG` will be parsed,
/// defaulting to `LevelFilter::Warn`.
/// Logs are formatted to include timestamp, level, and module path.
pub fn init_logger(explicit_level: Option<LevelFilter>) {
    let mut builder = Builder::new();

    if let Some(level) = explicit_level {
        // If an explicit level is provided, use it and clear RUST_LOG parsing
        // to ensure it takes precedence over environment variables.
        builder.filter_level(level);
    } else {
        // Otherwise, allow RUST_LOG env var to override the default Warn level
        builder.filter_level(LevelFilter::Warn);
        builder.parse_env("RUST_LOG");
    }

    builder
        .target(Target::Stderr) // Log to standard error
        .format(|buf, record| {
            // Custom format: [LEVEL MODULE_PATH] MESSAGE
            writeln!(
                buf,
                "[{}] [{}] {}",
                record.level(),
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .init();
}