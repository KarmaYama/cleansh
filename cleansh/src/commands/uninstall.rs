// cleansh-workspace/cleansh/src/commands/uninstall.rs
//! Cleansh Uninstallation Command (`uninstall`).
//!
//! This module implements the `cleansh uninstall` command, providing a mechanism
//! for the self-deletion of the Cleansh application and the removal of its
//! associated user data (such as configuration and application state files).
//! It includes user confirmation and platform-specific logic to ensure proper cleanup.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::env;
use std::thread;
use std::time::Duration;
use log::{info, debug};
use is_terminal::IsTerminal; // Keep IsTerminal to determine coloring for stderr

use crate::ui::{output_format, theme};
use crate::commands::cleansh::info_msg; // Import the info_msg helper
// Updated import: Removed HasIsTerminal as it's no longer defined in theme.rs
use crate::ui::theme::ThemeMap; // Import ThemeMap only

/// Runs the uninstallation logic for the cleansh application.
///
/// This function guides the user through the uninstallation process. It first
/// prompts for confirmation (unless the `yes_flag` is set), then identifies
/// the paths for the current executable and application state data. Finally,
/// it spawns a detached, platform-specific helper script (PowerShell on Windows,
/// Bash on other systems) to perform the actual file deletions, allowing the
/// currently running executable to be removed.
///
/// # Arguments
///
/// * `yes_flag` - A boolean; if `true`, the uninstallation proceeds without a
///                 confirmation prompt from the user.
/// * `theme_map` - A reference to a `HashMap` containing the application's
///                 theme styles for colored output messages.
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) if the uninstallation process is
/// initiated successfully, or an error (`Err(anyhow::Error)`) if there's a
/// problem determining paths, reading user input, or spawning the helper process.
/// The function exits the process upon successful initiation of uninstallation.
pub fn run_uninstall_command(
    yes_flag: bool,
    theme_map: &ThemeMap, // Use ThemeMap alias
) -> Result<()> {
    info!("Starting cleansh uninstall operation.");
    debug!("[uninstall.rs] Uninstall command initiated.");

    let stderr_supports_color = io::stderr().is_terminal(); // Determine color support for stderr

    // --- 1. User Confirmation ---
    if !yes_flag {
        info_msg("WARNING: This will uninstall Cleansh and remove its associated data.", theme_map); // Updated
        output_format::print_message( // This is a specific prompt, not a standard info message
            &mut io::stderr(),
            "Are you sure you want to proceed? (y/N): ",
            theme_map,
            Some(theme::ThemeEntry::Prompt),
            stderr_supports_color, // <--- Added enable_colors argument
        )?;
        io::stderr().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)
            .context("Failed to read confirmation input.")?;

        if confirmation.trim().to_lowercase() != "y" {
            info_msg("Uninstallation cancelled.", theme_map); // Updated
            return Ok(());
        }
    }

    // --- 2. Determine Paths ---
    let current_exe_path = env::current_exe()
        .context("Failed to determine current executable path.")?;
    debug!("[uninstall.rs] Current executable path: {:?}", current_exe_path);

    let app_state_file_path = std::env::var("CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("cleansh");
            path.push("app_state.json");
            path
        });
    debug!("[uninstall.rs] App state file path: {:?}", app_state_file_path);

    let app_state_dir = app_state_file_path.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            debug!("[uninstall.rs] Could not determine parent directory for app state file. Defaulting to current directory.");
            PathBuf::from(".")
        });
    debug!("[uninstall.rs] App state directory: {:?}", app_state_dir);

    // --- 3. Spawn Platform-Specific Helper for Self-Deletion ---
    info_msg("Initiating self-deletion process...", theme_map); // Updated

    #[cfg(target_os = "windows")]
    {
        let powershell_script = format!(
            r#"
            Start-Sleep -Seconds 1
            $exePath = "{}"
            $appStateFile = "{}"
            $appStateDir = "{}"

            Write-Host "Attempting to delete executable: $exePath"
            try {{
                Remove-Item -Path $exePath -Force -ErrorAction Stop
                Write-Host "Executable deleted successfully."
            }} catch {{
                Write-Error "Failed to delete executable: $($_.Exception.Message)"
                exit 1
            }}

            Write-Host "Attempting to delete app state file: $appStateFile"
            try {{
                if (Test-Path $appStateFile) {{
                    Remove-Item -Path $appStateFile -Force -ErrorAction Stop
                    Write-Host "App state file deleted successfully."
                }} else {{
                    Write-Host "App state file not found, skipping deletion."
                }}
            }} catch {{
                Write-Error "Failed to delete app state file: $($_.Exception.Message)"
                exit 1
            }}

            Write-Host "Attempting to delete app state directory: $appStateDir"
            try {{
                if (Test-Path $appStateDir) {{
                    Remove-Item -Path $appStateDir -Recurse -Force -ErrorAction Stop
                    Write-Host "App state directory deleted successfully."
                }} else {{
                    Write-Host "App state directory not found, skipping deletion."
                }}
            }} catch {{
                Write-Error "Failed to delete app state directory: $($_.Exception.Message)"
                exit 1
            }}

            Write-Host "Cleansh uninstallation complete."
            exit 0
            "#,
            current_exe_path.to_string_lossy().replace("'", "''"),
            app_state_file_path.to_string_lossy().replace("'", "''"),
            app_state_dir.to_string_lossy().replace("'", "''")
        );

        debug!("[uninstall.rs] PowerShell script to execute:\n{}", powershell_script);

        let mut command = Command::new("powershell.exe");
        command.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(&powershell_script)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());

        let child = command.spawn()
            .context("Failed to spawn PowerShell process for uninstallation.")?;
        debug!("[uninstall.rs] PowerShell helper spawned with PID: {}", child.id());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let bash_script = format!(
            r#"
            #!/bin/bash
            sleep 1
            exe_path="{}"
            app_state_file="{}"
            app_state_dir="{}"

            echo "Attempting to delete executable: $exe_path"
            rm -f "$exe_path"
            if [ $? -ne 0 ]; then
                echo "Error: Failed to delete executable: $exe_path" >&2
                exit 1
            fi
            echo "Executable deleted successfully."

            echo "Attempting to delete app state file: $app_state_file"
            if [ -f "$app_state_file" ]; then
                rm -f "$app_state_file"
                if [ $? -ne 0 ]; then
                    echo "Error: Failed to delete app state file: $app_state_file" >&2
                    exit 1
                fi
                echo "App state file deleted successfully."
            else
                echo "App state file not found, skipping deletion."
            fi

            echo "Attempting to delete app state directory: $app_state_dir"
            if [ -d "$app_state_dir" ]; then
                rmdir "$app_state_dir" 2>/dev/null || true
                if [ $? -ne 0 ]; then
                    echo "Warning: App state directory '$app_state_dir' might not be empty or could not be removed." >&2
                else
                    echo "App state directory deleted successfully."
                fi
            else
                echo "App state directory not found, skipping deletion."
            fi

            echo "Cleansh uninstallation complete."
            exit 0
            "#,
            current_exe_path.to_string_lossy(),
            app_state_file_path.to_string_lossy(),
            app_state_dir.to_string_lossy()
        );

        debug!("[uninstall.rs] Bash script to execute:\n{}", bash_script);

        let mut command = Command::new("bash");
        command.arg("-c")
                .arg(&bash_script)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());

        let child = command.spawn()
            .context("Failed to spawn bash process for uninstallation.")?;
        debug!("[uninstall.rs] Bash helper spawned with PID: {}", child.id());
    }

    // Give the helper process a moment to detach before the main process exits
    thread::sleep(Duration::from_millis(100));

    info_msg("Cleansh is being uninstalled. You can close this terminal.", theme_map); // Updated

    // Exit the current process immediately so the helper can delete the executable
    std::process::exit(0);
}