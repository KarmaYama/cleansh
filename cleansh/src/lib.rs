//! Cleansh CLI Application
//!
//! `cleansh` is the command-line interface application that allows users to
//! sanitize sensitive information from text content. This crate serves as
//! the main executable wrapper, orchestrating the parsing of command-line
//! arguments, managing input and output streams, handling application-specific
//! features like user-interaction (e.g., donation prompts), and integrating
//! with the core redaction logic provided by the `cleansh-core` library.
//!
//! ## Key Responsibilities of this Crate:
//! - **Argument Parsing:** Defines and parses all CLI options and subcommands
//!   using the `clap` crate.
//! - **Input/Output Management:** Handles reading content from stdin or specified
//!   files, and writing sanitized or statistical output to stdout, files, or
//!   the system clipboard.
//! - **Application State:** Manages persistent application state such as usage
//!   counts and prompt timings, leveraging the `utils::app_state` module.
//! - **User Interface:** Incorporates modules for theming, formatted output,
//!   redaction summaries, and diff viewing (`ui` module).
//! - **Command Execution:** Dispatches to specific command handlers (e.g., `stats`,
//!   `uninstall`) based on user input, found within the `commands` module.
//! - **Integration:** Acts as the bridge between user commands and the core
//!   redaction and validation functionalities exposed by `cleansh-core`.
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
        };
    }

    /// Core redaction-match types
    pub mod redaction_match {
        pub use cleansh_core::redaction_match::{
            RedactionMatch,
            redact_sensitive,
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
        // Updated to reflect the refactoring in cleansh/src/commands/cleansh.rs
        pub use crate::commands::cleansh::{run_cleansh_opts, sanitize_single_line};
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