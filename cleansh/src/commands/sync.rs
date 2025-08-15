//! This module handles the `sync-profiles` subcommand, which is an enterprise
//! feature for pulling the latest compliance profiles from a remote server.
//!
//! License: Polyform Noncommercial License 1.0.0

use crate::cli::SyncProfilesCommand;
use crate::ui::theme::ThemeMap;
use crate::ui::sync_ui;
use anyhow::{Result, anyhow, Context};
use std::fs;
use std::io;
use is_terminal::IsTerminal; // FIX: Changed `is_terminal` function to `IsTerminal` trait
use std::env;

/// The default URL for the users organization server.
const DEFAULT_SERVER_URL: &str = "https://your-org-server.com";

/// The main entry point for the `cleansh sync-profiles` subcommand.
///
/// It uses the provided API key and organization ID to authenticate and pull the latest profiles.
pub fn run_sync_profiles_command(opts: &SyncProfilesCommand, theme_map: &ThemeMap) -> Result<()> {
    // FIX: Calling the method is now correct since we imported the trait
    let enable_colors = io::stdout().is_terminal();

    sync_ui::print_sync_start(theme_map, enable_colors)?;

    // Use a consistent environment variable name, e.g., "CLEANSH_SERVER_URL"
    let server_url = env::var("ORG_SERVER_URL")
        .unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());
    
    let org_id = &opts.org_id;
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/orgs/{}/profiles", server_url, org_id);

    sync_ui::print_connection_attempt(&url, theme_map, enable_colors)?;
    
    let response = client
        .get(&url)
        .bearer_auth(&opts.org_key)
        .send()
        .with_context(|| {
            format!("Failed to connect to the organization server at: {}", url)
        })?;

    if response.status().is_success() {
        let profiles_yaml = response.text().context("Failed to read response body")?;
        
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("cleansh")
            .join("profiles");
            
        fs::create_dir_all(&config_dir)
            .context("Failed to create local profiles directory")?;

        let profile_path = config_dir.join("synced_profiles.yaml");
        fs::write(&profile_path, profiles_yaml.as_bytes())?;

        sync_ui::print_sync_success(&profile_path, theme_map, enable_colors)?;
    } else {
        let status_code = response.status().as_u16();
        match status_code {
            401 => {
                sync_ui::print_auth_failed_error(theme_map, enable_colors)?;
                return Err(anyhow!("Authentication Failed (401 Unauthorized)"));
            },
            404 => {
                sync_ui::print_server_not_found_error(org_id, &url, theme_map, enable_colors)?;
                return Err(anyhow!("Server Not Found (404 Not Found)"));
            },
            _ => {
                sync_ui::print_sync_failure_error(status_code, theme_map, enable_colors)?;
                return Err(anyhow!("Profile synchronization failed with status code: {}", status_code));
            },
        }
    }

    Ok(())
}