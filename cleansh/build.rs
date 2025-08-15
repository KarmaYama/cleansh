// cleansh/build.rs

use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;
use toml; // Ensure this is a build dependency in Cargo.toml

fn main() {
    // Tell Cargo to re-run this script only if Cargo.toml or the marker file changes.
    // This ensures that if the license note is updated, the warning will appear again.
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Get the path to the current crate's manifest directory.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = Path::new(&manifest_dir);

    // Navigate up one level to the workspace root.
    let workspace_root = manifest_path.parent()
        .expect("Failed to get parent directory of manifest.");
    let marker_file_path = workspace_root.join(".cleansh_license_checked");
    
    // Check if the marker file already exists. If it does, we've already run this check.
    if marker_file_path.exists() {
        return; // Exit the build script early.
    }

    // Read the metadata from Cargo.toml to include in the warning
    let cargo_toml_path = manifest_path.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml for license warning");
    
    let toml_value: toml::Value = toml::from_str(&cargo_toml_content)
        .expect("Failed to parse Cargo.toml content.");
    
    let license_note = toml_value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("cleansh"))
        .and_then(|c| c.get("license_notes"))
        .and_then(|n| n.as_str())
        .unwrap_or("No specific license note found in Cargo.toml. Please refer to the LICENSE file.");

    // Perform the main check.
    let is_cloned_repo = workspace_root.join(".git").exists();

    if is_cloned_repo {
        println!("cargo:warning=--------------------------------------------------------------------------------------------------");
        println!("cargo:warning=You are building CleanSH from source. Please be aware of the following licensing conditions:");
        println!("cargo:warning=");
        println!("cargo:warning=The `cleansh` CLI is licensed under the Polyform Noncommercial License 1.0.0.");
        println!("cargo:warning=For-profit, commercial, and government use is strictly prohibited without a commercial license.");
        println!("cargo:warning=");
        println!("cargo:warning=If you intend to use this in a commercial setting, please contact us for a license at contact@obscuratech.tech");
        println!("cargo:warning=");
        println!("cargo:warning=NOTE: due to the rapid development and growing feature set of CleanSH, the CLI interface and the License notes may change frequently.");
        println!("cargo:warning=Please refer to the LICENSE and License notes in cleansh under cleansh workspace in the repository for the most up to date license terms.");
        println!("cargo:warning=");
        println!("cargo:warning={}", license_note); 
        println!("cargo:warning=--------------------------------------------------------------------------------------------------");

        // Create the marker file so this logic doesn't run on future builds.
        let mut file = File::create(&marker_file_path)
            .expect("Could not create marker file to track license check status.");
        writeln!(file, "License check already performed.").unwrap();
    }
}