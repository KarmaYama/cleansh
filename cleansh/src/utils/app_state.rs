// cleansh-workspace/cleansh/src/utils/app_state.rs
//! Module for managing application state, including usage counts and donation prompts.
//! This module provides functionality to load, save, and manage the application state
//! in a JSON file. It tracks the number of times the `--stats-only` mode
//! has been used, the last time a donation prompt was displayed, and whether donation prompts
//! are currently disabled. The state is designed to be easily serializable and deserializable
//! to/from a JSON file, allowing for persistent tracking across application runs.

use std::fs;
use std::io;
use std::path::Path; // Use std::path::Path directly for arguments
use serde::{Deserialize, Serialize};
use log::{warn, debug};
use chrono::{Utc, TimeZone};

/// Defines the AppState struct, holding application-wide state such as
/// usage counts and timestamps for prompts. This struct is designed to be
/// easily serializable to and from a JSON file.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub stats_only_usage_count: u64,
    pub last_prompt_timestamp: Option<u64>, // Unix timestamp of the last prompt
    pub donation_prompts_disabled: bool,
    // Add any other global state variables here
}

impl AppState {
    /// Creates a new, default `AppState`.
    pub fn new() -> Self {
        AppState {
            stats_only_usage_count: 0,
            last_prompt_timestamp: None,
            donation_prompts_disabled: false,
        }
    }

    /// Loads the `AppState` from a specified file path.
    /// If the file does not exist, a new default `AppState` is returned.
    /// In case of other I/O errors or deserialization errors, a warning is logged,
    /// and a new default `AppState` is returned to allow the application to continue.
    pub fn load(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(app_state) => Ok(app_state),
                    Err(e) => {
                        warn!("Failed to deserialize AppState from {}: {}. Starting with default state.", path.display(), e);
                        Ok(AppState::new())
                    }
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                debug!("App state file not found at {}. Starting with default state.", path.display());
                Ok(AppState::new())
            },
            Err(e) => {
                warn!("Failed to read app state file from {}: {}. Starting with default state.", path.display(), e);
                Ok(AppState::new())
            }
        }
    }

    /// Loads the `AppState` from a specified file path.
    /// If the file does not exist, a new default `AppState` is returned.
    /// This function returns a `Result` indicating success or failure.
    #[cfg_attr(not(feature = "test-exposed"), allow(dead_code))]
    pub fn load_from_path(path: &Path) -> anyhow::Result<Self> {
        let app_state = match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json)?,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => AppState::new(),
            Err(e) => return Err(e.into()), // Convert other IO errors to anyhow::Error
        };
        Ok(app_state)
    }

    /// Saves the current `AppState` to a specified file path.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        // Ensure the parent directory exists before writing the file
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Saves the current `AppState` to a specified file path.
    /// This function returns a `Result` indicating success or failure.
    #[cfg_attr(not(feature = "test-exposed"), allow(dead_code))]
    pub fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        // Ensure the parent directory exists before writing the file
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    // Helper for tests to reset counts without removing the file
    #[cfg(feature = "test-exposed")]
    pub fn reset_for_testing(&mut self) {
        self.stats_only_usage_count = 0;
        self.last_prompt_timestamp = None;
        self.donation_prompts_disabled = false;
    }

    /// Increments the `stats_only_usage_count`.
    pub fn increment_stats_only_usage(&mut self) {
        self.stats_only_usage_count += 1;
    }

    /// Checks if a donation prompt should be displayed based on usage count and time elapsed.
    /// Returns true if a prompt should be shown, false otherwise.
    pub fn should_display_donation_prompt(&mut self) -> bool {
        if self.donation_prompts_disabled {
            return false;
        }

        const STATS_PROMPT_THRESHOLD: u64 = 5; // Number of `--stats-only` runs before prompting
        const PROMPT_COOLDOWN_DAYS: i64 = 30; // Cooldown period in days

        let now = Utc::now().timestamp() as u64;

        // Check if usage count threshold is met
        if self.stats_only_usage_count >= STATS_PROMPT_THRESHOLD {
            // Check cooldown period
            if let Some(last_prompt) = self.last_prompt_timestamp {
                // Use Utc.timestamp_opt directly
                let last_prompt_date = Utc.timestamp_opt(last_prompt as i64, 0).single();
                let now_date = Utc.timestamp_opt(now as i64, 0).single();

                if let (Some(last_p_date), Some(n_date)) = (last_prompt_date, now_date) {
                    if (n_date - last_p_date).num_days() < PROMPT_COOLDOWN_DAYS {
                        // Still in cooldown period
                        debug!("Donation prompt cooldown active. Last prompt: {} days ago.", (n_date - last_p_date).num_days());
                        return false;
                    }
                } else {
                    // Fallback if timestamp conversion fails (unlikely)
                    warn!("Failed to convert timestamps for donation prompt cooldown. Displaying prompt.");
                }
            }

            // If threshold met and cooldown passed (or no previous prompt), display prompt and update timestamp
            debug!("Donation prompt conditions met. Displaying prompt.");
            self.last_prompt_timestamp = Some(now);
            true
        } else {
            debug!("Donation prompt threshold not met. Current count: {}", self.stats_only_usage_count);
            false
        }
    }
}