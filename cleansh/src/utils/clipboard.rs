//src/utils/clipboard.rs
//! This module provides functionality to interact with the system clipboard.
//! It allows copying sanitized content to the clipboard, which can be useful
//! for quick access or further processing without needing to write to a file.


use anyhow::{Result, Context};
use arboard;
use log::debug;

pub fn copy_to_clipboard(content: &str) -> Result<()> {
    debug!("Attempting to acquire clipboard.");
    let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
    debug!("Setting clipboard text.");
    clipboard.set_text(content.to_string()).context("Failed to set clipboard text")?;
    Ok(())
}