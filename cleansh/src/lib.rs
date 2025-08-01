// cleansh-workspace/cleansh/src/lib.rs
//! # Cleansh
//!
//! `cleansh` is the command-line interface (CLI) application for securely redacting
//! sensitive information from text content. It acts as a lightweight wrapper around the
//! powerful `cleansh-core` library, which contains the core logic for pattern matching,
//! sanitization, and data validation.
//!
//! This crate provides the user-facing executable, handling command-line argument parsing,
//! file system interactions, application state management (like usage statistics and
//! donation prompts), and user interface elements such as formatted output and diff viewing.
//!
//! For details on the redaction rules, sanitization algorithms, and validation logic,
//! please refer to the `cleansh-core` crate's documentation.
//!
//! ## License
//!
//! Licensed under the Polyform Noncommercial License 1.0.0.

#![doc = include_str!("../README.md")]

// Declare CLI-specific modules as public within the 'cleansh' crate's library.
// This makes them accessible to main.rs (as crate::commands, etc.) and
// allows them to be re-exported by test_exposed.
pub mod commands;
pub mod cli;
pub mod ui;
pub mod utils;
pub mod logger;

// Test-only exports
#[cfg(any(test, feature = "test-exposed"))]
pub mod test_exposed {
    /// Core config types & constants
    pub mod config {
        pub use cleansh_core::config::{
            MAX_PATTERN_LENGTH,
            RedactionConfig,
            RedactionRule,
            RedactionSummaryItem,
            RuleConfigNotFoundError,
            merge_rules,
        };
    }

    /// Core sanitizer functions
    pub mod sanitizer {
        pub use cleansh_core::sanitizer::{
            CompiledRule,
            CompiledRules,
            compile_rules,
            sanitize_content,
        };
    }

    /// Core redaction-match types
    pub mod redaction_match {
        pub use cleansh_core::redaction_match::{
            RedactionMatch,
            redact_sensitive,
            log_redaction_match_debug,
            log_captured_match_debug,
            log_redaction_action_debug,
        };
    }

    /// Core validators
    pub mod validators {
        pub use cleansh_core::validators::{
            is_valid_ssn_programmatically,
            is_valid_uk_nino_programmatically,
        };
    }

    /// CLI commands for testing
    pub mod commands {
        pub use crate::commands::cleansh::{run_cleansh, sanitize_single_line, build_redaction_summary_from_matches};
        pub use crate::commands::stats::run_stats_command;
        pub use crate::commands::uninstall::run_uninstall_command;
    }

    /// CLI UI modules for testing
    pub mod ui {
        pub use crate::ui::diff_viewer;
        pub use crate::ui::output_format;
        pub use crate::ui::redaction_summary;
        pub use crate::ui::theme;
    }

    /// CLI utility modules for testing
    pub mod utils {
        pub use crate::utils::app_state::*;
        pub use crate::utils::platform;
    }

    /// CLI logger for testing
    pub mod logger {
        pub use crate::logger::*;
    }
}