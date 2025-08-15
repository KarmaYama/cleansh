//! Cleansh Uninstallation Command (`uninstall`).
//!
//! This module implements the `cleansh uninstall` command, providing a mechanism
//! for the self-deletion of the Cleansh application and the removal of its
//! associated user data (such as configuration and application state files).
//! It includes user confirmation and platform-specific logic to ensure proper cleanup.

use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::env;
use std::thread;
use std::time::Duration;
use log::{info, debug};
use is_terminal::IsTerminal;
use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStringExt;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};

// Correct WinAPI imports for direct process elevation
#[cfg(target_os = "windows")]
use winapi::um::shellapi::ShellExecuteW;
#[cfg(target_os = "windows")]
use winapi::um::winuser::SW_SHOWNORMAL;
#[cfg(target_os = "windows")]
use winapi::um::fileapi::{GetTempPathW, GetTempFileNameW};

use crate::ui::{output_format, theme};
use crate::commands::cleansh::info_msg;
use crate::ui::theme::ThemeMap;

// Global counter to prevent infinite loops in specific scenarios
static ELEVATION_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

/// Helper function to convert a Rust OsString to a wide string for WinAPI.
#[cfg(target_os = "windows")]
fn to_wide_string(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}

/// Performs a Windows-specific check to see if the process is running as an administrator.
#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    unsafe {
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION};
        
        let mut token_handle = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), winapi::um::winnt::TOKEN_QUERY, &mut token_handle) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size: u32 = 0;
        let success = GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut elevation as *mut _ as winapi::shared::minwindef::LPVOID,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        ) != 0;

        CloseHandle(token_handle);
        
        if success {
            return elevation.TokenIsElevated != 0;
        }

        false
    }
}

/// A dedicated function to handle elevation and uninstallation on Windows.
/// This is called directly from `main.rs` to ensure the logic flow is clean.
#[cfg(target_os = "windows")]
pub fn elevate_and_run_uninstall(yes_flag: bool, theme_map: &ThemeMap) -> Result<()> {
    if is_elevated() {
        // If already elevated, proceed directly to the uninstallation logic
        run_uninstaller_logic(yes_flag, theme_map)?;
    } else {
        let attempts = ELEVATION_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
        if attempts > 1 {
            return Err(anyhow!("Failed to elevate and run uninstallation. Exiting to prevent infinite loop."));
        }

        info_msg("Attempting to elevate for uninstallation...", theme_map);
        let exe_path = env::current_exe()?;
        let exe_path_wide: Vec<u16> = exe_path.to_str().unwrap().encode_utf16().chain(Some(0)).collect();
        
        // Pass original arguments and the uninstaller flag
        let mut args: Vec<String> = env::args().skip(1).collect();

        // Add the --yes flag to ensure the elevated process doesn't prompt for confirmation again.
        if yes_flag && !args.contains(&"--yes".to_string()) {
            args.push("--yes".to_string());
        }

        let args_string = args.join(" ");
        let args_wide: Vec<u16> = args_string.encode_utf16().chain(Some(0)).collect();

        let operation = "runas".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                exe_path_wide.as_ptr(),
                args_wide.as_ptr(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };

        if result as usize <= 32 {
            let err_code = io::Error::last_os_error();
            if err_code.raw_os_error() == Some(5) {
                // 5 is ERROR_ACCESS_DENIED, which is the UAC dialog being cancelled.
                return Err(anyhow!("Uninstallation cancelled by user."));
            }
            return Err(anyhow!("Failed to relaunch with admin privileges. ShellExecuteW failed with error code: {}. OS Error: {}", result as isize, err_code));
        }
        
        // The original process must exit immediately after launching the new one.
        std::process::exit(0);
    }
    Ok(())
}


/// The core uninstallation logic that runs once the process is elevated.
fn run_uninstaller_logic(yes_flag: bool, theme_map: &ThemeMap) -> Result<()> {
    info!("Starting cleansh uninstall operation.");
    debug!("[uninstall.rs] Uninstall command initiated.");
    let stderr_supports_color = io::stderr().is_terminal();

    // --- 1. User Confirmation (if not running with --yes) ---
    if !yes_flag {
        info_msg("WARNING: This will uninstall Cleansh and remove its associated data.", theme_map);
        output_format::print_message(
            &mut io::stderr(),
            "Are you sure you want to proceed? (y/N): ",
            theme_map,
            Some(theme::ThemeEntry::Prompt),
            stderr_supports_color,
        )?;
        io::stderr().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)
            .context("Failed to read confirmation input.")?;

        if confirmation.trim().to_lowercase() != "y" {
            info_msg("Uninstallation cancelled.", theme_map);
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
            if let Some(mut path) = dirs::data_dir() {
                path.push("cleansh");
                path.push("state.json");
                path
            } else {
                debug!("[uninstall.rs] Data directory not found, defaulting to current directory.");
                PathBuf::from("cleansh_state.json")
            }
        });

    let app_state_dir = app_state_file_path.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            debug!("[uninstall.rs] Could not determine parent directory for app state file. Defaulting to current directory.");
            PathBuf::from(".")
        });
    debug!("[uninstall.rs] App state directory: {:?}", app_state_dir);
    
    // --- 3. Spawn Platform-Specific Helper for Self-Deletion ---
    info_msg("Initiating self-deletion process...", theme_map);

    #[cfg(target_os = "windows")]
    {
        // Get a temporary file path for the helper script
        let mut temp_path_buf = vec![0u16; 260];
        let temp_path_len = unsafe { GetTempPathW(temp_path_buf.len() as u32, temp_path_buf.as_mut_ptr()) };
        let temp_dir = PathBuf::from(OsString::from_wide(&temp_path_buf[0..temp_path_len as usize]));

        let mut temp_file_path_buf = vec![0u16; 260];
        unsafe { GetTempFileNameW(temp_dir.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr(), to_wide_string(OsStr::new("ps1")).as_ptr(), 0, temp_file_path_buf.as_mut_ptr()) };
        let temp_ps1_path = PathBuf::from(OsString::from_wide(&temp_file_path_buf));
        debug!("[uninstall.rs] Generated temporary PowerShell script path: {:?}", temp_ps1_path);
        
        let current_pid = std::process::id();
        let current_exe_path_string = current_exe_path.to_string_lossy().replace("'", "''");
        let app_state_file_path_string = app_state_file_path.to_string_lossy().replace("'", "''");
        let app_state_dir_string = app_state_dir.to_string_lossy().replace("'", "''");
        let log_file_string = temp_dir.join(format!("cleansh_uninstall_{}.log", current_pid)).to_string_lossy().replace("'", "''");
        
        let powershell_script = format!(
            r#"
            # This script runs in a new process to delete the original executable and data.
            $pidToWait = {}
            $logFile = "{}"
            $exePath = "{}"
            $appStateFile = "{}"
            $appStateDir = "{}"

            function Log($m){{ "$((Get-Date).ToString('s')) - $m" | Out-File -FilePath $logFile -Append -Encoding utf8 }}

            Log "Helper script started. Target exe: $exePath"
            
            # Wait for the original cleansh process to exit
            Log "Waiting for process $pidToWait to exit..."
            try {{
                # Wait for up to 30 seconds for the original process to terminate
                Wait-Process -Id $pidToWait -Timeout 30 -ErrorAction Stop
                Log "Original process exited successfully."
            }} catch {{
                Log "Original process already exited or was not found, proceeding with uninstallation."
            }}
            
            # Increase wait time to ensure file handles are released
            Start-Sleep -Seconds 2

            # Try to delete the executable repeatedly with a longer sleep interval
            $deletionAttemptSucceeded = $false
            for ($i=0; $i -lt 30; $i++) {{
                try {{
                    if (Test-Path $exePath) {{
                        Remove-Item -Path $exePath -Force -ErrorAction Stop
                        Log "Executable deleted successfully on attempt $i."
                        $deletionAttemptSucceeded = $true
                        break
                    }} else {{
                        Log "Executable not found, skipping file deletion."
                        $deletionAttemptSucceeded = $true
                        break
                    }}
                }} catch {{
                    Log "Attempt $i: Failed to delete exe: $($_.Exception.Message)"
                    Start-Sleep -Seconds 1
                }}
            }}

            if (-not $deletionAttemptSucceeded) {{
                Log "Exe still present after multiple attempts. Scheduling deletion on next reboot."
                Add-Type -TypeDefinition @"
                using System;
                using System.Runtime.InteropServices;
                public class M {{
                    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
                    public static extern bool MoveFileEx(string lpExistingFileName, string lpNewFileName, int dwFlags);
                }}
                "@
                [M]::MoveFileEx($exePath, $null, 0x4) | Out-Null
                if ($LASTEXITCODE -ne 0) {{
                    Log "Failed to schedule deletion via MoveFileEx. OS Error Code: $LASTEXITCODE"
                }} else {{
                    Log "Scheduled deletion via MoveFileEx successfully."
                }}
            }}

            # Remove app state
            try {{
                if (Test-Path $appStateFile) {{ Remove-Item -Path $appStateFile -Force -ErrorAction Stop; Log "App state file deleted." }}
                if (Test-Path $appStateDir) {{ Remove-Item -Path $appStateDir -Recurse -Force -ErrorAction Stop; Log "App state dir deleted." }}
            }} catch {{ Log "Failed deleting app-state: $($_.Exception.Message)" }}

            Log "Helper script finished."
            Write-Output "Log file: $logFile"

            # Clean up the helper script itself
            Remove-Item -Path '{}' -Force -ErrorAction SilentlyContinue
            "#,
            current_pid,
            log_file_string,
            current_exe_path_string,
            app_state_file_path_string,
            app_state_dir_string,
            temp_ps1_path.to_string_lossy().replace("'", "''")
        );
        
        let mut file = File::create(&temp_ps1_path)
            .context("Failed to create temporary PowerShell script.")?;
        file.write_all(powershell_script.as_bytes())
            .context("Failed to write to temporary PowerShell script.")?;
        
        let mut command = Command::new("powershell.exe");
        command.arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-File").arg(&temp_ps1_path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        command.spawn()
            .context("Failed to spawn PowerShell process for uninstallation.")?;
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
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

        command.spawn()
            .context("Failed to spawn bash process for uninstallation.")?;
    }

    // Give the helper process a moment to detach before the main process exits
    thread::sleep(Duration::from_millis(100));

    info_msg("Cleansh is being uninstalled. You can close this terminal.", theme_map);

    // Exit the current process immediately so the helper can delete the executable
    std::process::exit(0);
}

/// The public entry point for the uninstall command. It determines if elevation is needed.
#[cfg(not(target_os = "windows"))]
pub fn elevate_and_run_uninstall(yes_flag: bool, theme_map: &ThemeMap) -> Result<()> {
    // For non-Windows systems, no elevation is needed.
    run_uninstaller_logic(yes_flag, theme_map)
}