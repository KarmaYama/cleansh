//! audit_log.rs - Handles the creation and management of a secure,
//! append-only audit log for all redaction events.
//!
//! This module defines the `AuditLog` struct and its associated methods for
//! writing `RedactionLog` entries to a file in a JSON Lines format. This
//! ensures an immutable and auditable record of every sanitization action
//! performed.

use crate::redaction_match::RedactionLog;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};

/// Manages an append-only audit log file for redaction events.
///
/// The `AuditLog` struct provides a simple interface for writing `RedactionLog`
/// entries to a specified file, ensuring each entry is a valid JSON object on
/// a new line. This design is crucial for auditability and compliance.
pub struct AuditLog {
    path: PathBuf,
    // Using BufWriter for buffered writes improves performance, especially with many small writes.
    writer: BufWriter<fs::File>,
}

impl AuditLog {
    /// Creates a new `AuditLog` instance, opening or creating the log file
    /// in append mode.
    ///
    /// This method is designed to be resilient. It will create the necessary
    /// parent directories if they don't exist and opens the file in a way
    /// that new entries are always added to the end.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path for the audit log.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `AuditLog` instance on success, or an error
    /// if the file cannot be created or opened.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directories for {}", parent.display())
            })?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_buf)
            .with_context(|| format!("Failed to open or create audit log at {}", path_buf.display()))?;

        Ok(Self {
            path: path_buf,
            writer: BufWriter::new(file),
        })
    }

    /// Appends a new `RedactionLog` entry to the audit log file.
    ///
    /// The entry is serialized to a JSON string and written to the file,
    /// followed by a newline character. This ensures the log remains
    /// a stream of valid JSON Lines, which is easy to parse.
    ///
    /// # Arguments
    ///
    /// * `log_entry` - The `RedactionLog` entry to be written.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the write operation.
    pub fn append(&mut self, log_entry: &RedactionLog) -> Result<()> {
        let json_line = serde_json::to_string(log_entry)
            .context("Failed to serialize RedactionLog to JSON")?;
        self.writer
            .write_all(json_line.as_bytes())
            .context("Failed to write log entry")?;
        self.writer
            .write_all(b"\n")
            .context("Failed to write newline")?;
        self.writer.flush().context("Failed to flush audit log after append")?;
        Ok(())
    }

    /// Forces a flush of any buffered data to disk.
    ///
    /// This can be called in long-running sessions to ensure logs are persisted
    /// before the `AuditLog` is dropped.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().context("Failed to flush audit log")
    }

    /// Returns the file path of the audit log.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// Ensure the buffer is flushed when the AuditLog is dropped.
impl Drop for AuditLog {
    fn drop(&mut self) {
        if let Err(e) = self.writer.flush() {
            log::error!("Failed to flush audit log writer: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction_match::RedactionLog;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_audit_log_new_and_append() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_path = temp_dir.path().join("audit.log");

        let mut audit_log = AuditLog::new(&log_path)?;

        let log_entry = RedactionLog {
            timestamp: "2025-08-09T13:00:00Z".to_string(),
            run_id: "test-run-123".to_string(),
            file_path: "/path/to/test_file.txt".to_string(),
            user_id: "test_user".to_string(),
            reason_for_redaction: "PII detected".to_string(),
            redaction_outcome: "redacted".to_string(),
            rule_name: "email".to_string(),
            input_hash: "hash123".to_string(),
            match_hash: "matchhash456".to_string(),
            start: 10,
            end: 25,
        };

        audit_log.append(&log_entry)?;
        audit_log.flush()?; // Ensure it’s persisted for the test

        let log_content = fs::read_to_string(&log_path)?;
        let expected_json = serde_json::to_string(&log_entry)?;
        
        assert_eq!(log_content, format!("{}\n", expected_json));

        Ok(())
    }
}
