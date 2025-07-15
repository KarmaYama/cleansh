// src/logger.rs
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write; 

/// Initializes the application's logger.
///
/// It sets up `env_logger` to output logs to stderr,
/// with a **warn-level default filter** unless overridden by `RUST_LOG` environment variable.
/// Logs are formatted to include timestamp, level, and module path.
pub fn init_logger() {
    Builder::new()
        .filter_level(LevelFilter::Warn) // Changed default log level from Info to Warn
        .parse_env("RUST_LOG") // Allow RUST_LOG env var to override this default
        .target(Target::Stderr) // Log to standard error
        .format(|buf, record| {
            // Custom format: [LEVEL MODULE_PATH] MESSAGE
            // Note: Timestamp is typically added by env_logger if you use `format_timestamp_millis()` etc.
            // If you want a timestamp, you'd add `buf.timestamp_millis()` or similar here.
            // For now, matching your existing format.
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