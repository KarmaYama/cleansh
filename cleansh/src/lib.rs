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

use anyhow::Result;
use utils::app_state::AppState;
use utils::license as license_utils;
use std::path::Path;

#[cfg(not(feature = "test-exposed"))]
use anyhow::anyhow;
#[cfg(not(feature = "test-exposed"))]
use std::env;
#[cfg(not(feature = "test-exposed"))]
use std::fs;

#[cfg(not(feature = "test-exposed"))]
/// Try to load license token from environment or a token file next to state.json
fn load_license_token_from_env_or_file(state_path: &Path) -> Option<String> {
    if let Ok(tok) = env::var("CLEANSH_LICENSE") {
        return Some(tok);
    }
    if let Some(parent) = state_path.parent() {
        let license_file = parent.join("license.token");
        if license_file.exists() {
            if let Ok(s) = fs::read_to_string(&license_file) {
                return Some(s.trim().to_string());
            }
        }
    }
    None
}

#[cfg(not(feature = "test-exposed"))]
/// Helper to compute the upgrade/purchase URL to show to user on invalid license.
fn license_url() -> String {
    env::var("CLEANSH_LICENSE_URL").unwrap_or_else(|_| "https://your-site.example/upgrade".to_string())
}

#[cfg(not(feature = "test-exposed"))]
/// Ensure a valid license exists and may be used for `feature`.
/// Returns parsed LicenseToken on success. Exits (process::exit) with code 2 on denial.
///
/// Per-feature logic:
/// - If the license payload contains a wildcard "*" feature, that applies to any feature.
/// - If the license maps the requested `feature` to Some(limit), we ensure the used count is < limit.
/// - If the license maps `feature` to None => unlimited for that feature.
/// - If the feature is absent (and "*" absent) => deny.
fn require_license_for_feature(feature: &str, state_path: &Path, app_state: &mut AppState, theme_map: &ui::theme::ThemeMap) -> Result<license_utils::LicenseToken> {
    // try to get token
    let tok = load_license_token_from_env_or_file(state_path)
        .ok_or_else(|| anyhow!("No license provided"))?;

    // verify signature & expiry
    let parsed = match license_utils::parse_and_verify_compact(&tok) {
        Ok(p) => p,
        Err(e) => {
            commands::cleansh::error_msg(format!("License validation failed: {}. Visit {}", e, license_url()), theme_map);
            std::process::exit(2);
        }
    };

    let fp = parsed.fingerprint();

    // If license already globally consumed (rare), deny immediately
    if app_state.is_license_consumed(&fp) {
        commands::cleansh::error_msg(format!("License appears fully consumed. Visit {}", license_url()), theme_map);
        std::process::exit(2);
    }

    // helper: check feature presence or wildcard
    let feature_entry = parsed.payload.features.get(feature)
        .or_else(|| parsed.payload.features.get("*"));

    match feature_entry {
        Some(opt_limit) => {
            if let Some(limit) = opt_limit {
                // finite limit
                let used = app_state.get_license_feature_usage(&fp, feature);
                if used >= *limit {
                    commands::cleansh::error_msg(format!("No remaining uses for feature '{}' on this license (used {}/{}). Visit {}", feature, used, limit, license_url()), theme_map);
                    std::process::exit(2);
                } else {
                    commands::cleansh::info_msg(format!("License validated — '{}' unlocked. Expires: {}. Usage for '{}': {}/{}", feature, parsed.payload.expires_at, feature, used, limit), theme_map);
                }
            } else {
                // unlimited
                commands::cleansh::info_msg(format!("License validated — '{}' unlocked (unlimited). Expires: {}", feature, parsed.payload.expires_at), theme_map);
            }
        }
        None => {
            // No feature granted by this license
            commands::cleansh::error_msg(format!("This license does not grant access to feature '{}'. Visit {}", feature, license_url()), theme_map);
            std::process::exit(2);
        }
    }

    Ok(parsed)
}

/// After a successful gated operation, increment per-feature usage and persist app state.
/// If a finite limit is reached because of this increment, mark the license consumed (fully)
/// only if **all** finite features are exhausted. That helps track fully-spent licenses.
pub fn consume_license_post_success(token: &license_utils::LicenseToken, feature: &str, app_state: &mut AppState, state_path: &Path, theme_map: &ui::theme::ThemeMap) {
    let fp = token.fingerprint();

    // increment usage
    let used_before = app_state.get_license_feature_usage(&fp, feature);
    app_state.increment_license_feature_usage(&fp, feature);
    let used_after = used_before.saturating_add(1);

    // If this feature had a finite limit and we've reached it, check whether every finite feature in the payload is now exhausted.
    if let Some(opt_limit) = token.payload.features.get(feature).cloned().flatten() {
        if used_after >= opt_limit {
            // check all finite features: if all finite features are exhausted, mark consumed true
            let mut all_exhausted = true;
            for (feat_name, feat_limit_opt) in &token.payload.features {
                if let Some(limit) = feat_limit_opt {
                    let used = app_state.get_license_feature_usage(&fp, feat_name.as_str());
                    if used < *limit {
                        all_exhausted = false;
                        break;
                    }
                }
            }
            if all_exhausted {
                app_state.mark_license_consumed(&fp);
            }
        }
    }

    // persist immediately
    if let Err(e) = app_state.save(state_path) {
        commands::cleansh::warn_msg(format!("Failed to persist app state after license usage: {}", e), theme_map);
    } else {
        commands::cleansh::info_msg(format!("Recorded license usage for feature '{}'. (fingerprint: {})", feature, fp), theme_map);
    }
}

/// The new public function for license checking
pub fn check_license_for_feature(
    feature: &str,
    state_path: &Path,
    app_state: &mut AppState,
    theme_map: &ui::theme::ThemeMap,
) -> Result<Option<license_utils::LicenseToken>> {
    #[cfg(feature = "test-exposed")]
    {
        // In test mode, we bypass the license check and return None.
        // This is safe because this code is only compiled with the "test-exposed" feature.
        commands::cleansh::info_msg("License check bypassed in test mode.", theme_map);
        let _ = (feature, state_path, app_state); // Mark parameters as used to silence warnings
        Ok(None)
    }

    #[cfg(not(feature = "test-exposed"))]
    {
        // This is the production path. The license check is required here.
        let token = require_license_for_feature(feature, state_path, app_state, theme_map)?;
        Ok(Some(token))
    }
}

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
        // The types must be re-exported from the top-level crate.
        pub use cleansh_core::{
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
        pub use crate::commands::uninstall::elevate_and_run_uninstall;
    }

    /// CLI UI modules for testing
    pub mod ui {
        pub use crate::ui::diff_viewer;
        pub use crate::ui::output_format;
        pub use crate::ui::redaction_summary;
        pub use crate::ui::theme;
        pub use crate::ui::verify_ui;
        pub use crate::ui::sync_ui;
    }

    /// CLI utility modules for testing
    pub mod utils {
        pub use crate::utils::app_state::*;
        pub use crate::utils::platform::*;
        pub use crate::utils::clipboard::*;
        pub use crate::utils::license::*;
    }

    /// CLI logger for testing
    pub mod logger {
        pub use crate::logger::*;
    }
}