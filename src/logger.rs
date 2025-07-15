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
        // If an explicit level is provided (e.g., from --debug, --no-debug, or --quiet),
        // use it and ensure it takes precedence.
        builder.filter_level(level);
    } else {
        // If no explicit level is provided, the default log level will be WARN.
        // This means INFO and DEBUG messages are suppressed by default.
        // However, the RUST_LOG environment variable can still override this default.
        builder.filter_level(LevelFilter::Warn); // <--- THIS IS THE KEY CHANGE FOR DEFAULT SILENCE
        builder.parse_env("RUST_LOG"); // Still allow RUST_LOG to override the default
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