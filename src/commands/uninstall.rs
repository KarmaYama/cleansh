// src/commands/uninstall.rs
//! Implements the `cleansh uninstall` command for self-deletion and cleanup.
// This command allows the user to uninstall the `cleansh` application and remove its associated data.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::env;
use std::thread;
use std::time::Duration;
use log::{info, debug};

use crate::ui::{output_format, theme};

/// Runs the uninstallation logic for the cleansh application.
///
/// This function handles user confirmation, determines paths for the executable
/// and application state, and spawns a platform-specific helper to perform
/// the actual deletion.
pub fn run_uninstall_command(
    yes_flag: bool,
    theme_map: &std::collections::HashMap<theme::ThemeEntry, theme::ThemeStyle>,
) -> Result<()> {
    info!("Starting cleansh uninstall operation.");
    debug!("[uninstall.rs] Uninstall command initiated.");

    // --- 1. User Confirmation ---
    if !yes_flag {
        output_format::print_info_message(
            &mut io::stderr(),
            "WARNING: This will uninstall Cleansh and remove its associated data.",
            theme_map,
        )?;
        output_format::print_message(
            &mut io::stderr(),
            "Are you sure you want to proceed? (y/N): ",
            theme_map,
            Some(theme::ThemeEntry::Prompt),
        )?;
        io::stderr().flush()?; // Ensure prompt is displayed before reading input

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)
            .context("Failed to read confirmation input.")?;

        if confirmation.trim().to_lowercase() != "y" {
            output_format::print_info_message(
                &mut io::stderr(),
                "Uninstallation cancelled.",
                theme_map,
            )?;
            return Ok(());
        }
    }

    // --- 2. Determine Paths ---
    let current_exe_path = env::current_exe()
        .context("Failed to determine current executable path.")?;
    debug!("[uninstall.rs] Current executable path: {:?}", current_exe_path);

    // Determine the app state file path using the same logic as `stats.rs`
    let app_state_file_path = std::env::var("CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("cleansh"); // The directory containing app_state.json
            path.push("app_state.json");
            path
        });
    debug!("[uninstall.rs] App state file path: {:?}", app_state_file_path);

    // The directory containing the app state file
    let app_state_dir = app_state_file_path.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Fallback if parent() somehow returns None (e.g., if app_state_file_path is just a filename)
            // This case should ideally not happen with dirs::config_dir()
            debug!("[uninstall.rs] Could not determine parent directory for app state file. Defaulting to current directory.");
            PathBuf::from(".")
        });
    debug!("[uninstall.rs] App state directory: {:?}", app_state_dir);


    // --- 3. Spawn Platform-Specific Helper for Self-Deletion ---
    output_format::print_info_message(
        &mut io::stderr(),
        "Initiating self-deletion process...",
        theme_map,
    )?;

    #[cfg(target_os = "windows")]
    {
        // On Windows, use a PowerShell script to wait and delete
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
                # Only remove directory if it's empty or contains only app_state.json (which is now deleted)
                # This is safer than -Recurse if other user files might be there, but we assume cleansh only puts app_state.json here.
                # For a more aggressive cleanup, -Recurse could be used, but it's risky.
                if (Test-Path $appStateDir) {{
                    # Check if directory is empty or only contains the app_state.json (which should be gone)
                    # This is a heuristic, a robust check would be more complex.
                    # For now, let's just try to remove it if it exists and is empty or contains only expected files.
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
            current_exe_path.to_string_lossy().replace("'", "''"), // Escape single quotes
            app_state_file_path.to_string_lossy().replace("'", "''"),
            app_state_dir.to_string_lossy().replace("'", "''")
        );

        debug!("[uninstall.rs] PowerShell script to execute:\n{}", powershell_script);

        // Spawn PowerShell process in a detached way
        let mut command = Command::new("powershell.exe");
        command.arg("-NoProfile")
               .arg("-NonInteractive")
               .arg("-Command")
               .arg(&powershell_script)
               .stdin(Stdio::null())
               .stdout(Stdio::null()) // Suppress output to avoid polluting user's terminal
               .stderr(Stdio::null()); // Suppress stderr as well

        let child = command.spawn()
            .context("Failed to spawn PowerShell process for uninstallation.")?;
        debug!("[uninstall.rs] PowerShell helper spawned with PID: {}", child.id());

        // Do not wait for the child process. The main cleansh process will exit,
        // allowing the PowerShell script to proceed with deletion.
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, use a bash script to wait and delete
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
                # rmdir only deletes empty directories. rm -rf is more aggressive but also more dangerous.
                # Given cleansh only creates app_state.json here, rmdir should be fine after file deletion.
                rmdir "$app_state_dir" 2>/dev/null || true # Suppress error if not empty, but try to remove
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

        // Spawn bash process in a detached way
        let mut command = Command::new("bash");
        command.arg("-c")
               .arg(&bash_script)
               .stdin(Stdio::null())
               .stdout(Stdio::null()) // Suppress output
               .stderr(Stdio::null()); // Suppress stderr

        let child = command.spawn()
            .context("Failed to spawn bash process for uninstallation.")?;
        debug!("[uninstall.rs] Bash helper spawned with PID: {}", child.id());
    }

    // Give the helper script a moment to start before the main process exits
    thread::sleep(Duration::from_millis(100));

    output_format::print_info_message(
        &mut io::stderr(),
        "Cleansh is being uninstalled. You can close this terminal.",
        theme_map,
    )?;

    // Exit the current process immediately so the helper can delete the executable.
    std::process::exit(0);
}