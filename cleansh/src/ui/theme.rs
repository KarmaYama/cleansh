// cleansh-workspace/cleansh/src/ui/theme.rs
//! Module for managing the application's command-line interface (CLI) theme.
//!
//! This module defines the structure for theme configuration, allowing users
//! to customize the colors of various output elements. It supports 16-color
//! ANSI named colors for foreground styling and provides functionality to
//! load themes from YAML files and manage default theme settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf}; // Added PathBuf
use std::str::FromStr;
use anyhow::{Context, Result};
use owo_colors::AnsiColors;
// Removed: use is_terminal::IsTerminal; // This import is not directly used in this module

/// Type alias for the theme map, providing a consistent type definition.
pub type ThemeMap = HashMap<ThemeEntry, ThemeStyle>;

// Removed: `HasIsTerminal` trait and its implementations.
// Removed: `TerminalWrite` trait and its implementations.
// We will rely directly on `is_terminal::IsTerminal` and `std::io::Write`.


/// The different logical parts of your output that can be styled.
///
/// Each variant represents a distinct type of message or UI element
/// that can have a configurable foreground color in the theme.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeEntry {
    /// Style for prominent headers or section titles.
    Header,
    /// Style for successful operation messages.
    Success,
    /// Style for general informational messages.
    Info,
    /// Style for warning messages.
    Warn,
    /// Style for error messages.
    Error,
    /// Style for text that has been redacted.
    RedactedText,
    /// Style for lines added in a diff view.
    DiffAdded,
    /// Style for lines removed in a diff view.
    DiffRemoved,
    /// Style for the header/footer of a diff view.
    DiffHeader,
    /// Style for the name of a rule in a summary or statistics output.
    /// Style for the count of occurrences in a summary or statistics output.
    SummaryRuleName,
    SummaryOccurrences,
    /// Style for user prompts or confirmation questions.
    Prompt,
}

/// Represents an ANSI color that can be used in the theme.
///
/// Currently, only named 16-color ANSI standard colors are supported.
/// RGB and 256-color codes are intentionally not supported to keep the
/// theme configuration simple and broadly compatible.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ThemeColor {
    /// A named ANSI color (e.g., "red", "brightgreen").
    Named(String),
}

/// Error type for parsing an invalid `ThemeColor` string.
#[derive(Debug, Clone)]
pub struct ParseThemeColorError;

impl fmt::Display for ParseThemeColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid theme color; expected one of: black, red, green, yellow, blue, \
            magenta, cyan, white, brightblack, brightred, brightgreen, brightyellow, \
            brightblue, brightmagenta, brightcyan, brightwhite."
        )
    }
}

impl std::error::Error for ParseThemeColorError {}

impl FromStr for ThemeColor {
    type Err = ParseThemeColorError;

    /// Attempts to parse a string into a `ThemeColor`.
    ///
    /// Only recognizes exact matches (case-insensitive) of the 16 standard
    /// ANSI color names.
    ///
    /// # Arguments
    ///
    /// * `s` - The string slice to parse.
    ///
    /// # Returns
    ///
    /// `Ok(ThemeColor)` if the string is a valid named ANSI color,
    /// `Err(ParseThemeColorError)` otherwise.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept only exact matches of the 16 ANSI color names
        let lower = s.to_lowercase();
        match lower.as_str() {
            "black"
            | "red"
            | "green"
            | "yellow"
            | "blue"
            | "magenta"
            | "cyan"
            | "white"
            | "brightblack"
            | "brightred"
            | "brightgreen"
            | "brightyellow"
            | "brightblue"
            | "brightmagenta"
            | "brightcyan"
            | "brightwhite" => Ok(ThemeColor::Named(lower)),
            _ => Err(ParseThemeColorError),
        }
    }
}

impl ThemeColor {
    /// Converts the `ThemeColor` enum variant into its corresponding `owo_colors::AnsiColors` enum.
    ///
    /// This mapping allows the `ThemeColor` to be directly used with the `owo-colors` crate
    /// for applying terminal colors.
    ///
    /// # Returns
    ///
    /// The `AnsiColors` enum variant corresponding to the `ThemeColor`.
    /// Falls back to `AnsiColors::White` if an unrecognized named color somehow slips through
    /// (though `FromStr` should prevent this if strict).
    pub fn to_ansi_color(&self) -> AnsiColors {
        match self {
            ThemeColor::Named(name) => match name.as_str() {
                "black" => AnsiColors::Black,
                "red" => AnsiColors::Red,
                "green" => AnsiColors::Green,
                "yellow" => AnsiColors::Yellow,
                "blue" => AnsiColors::Blue,
                "magenta" => AnsiColors::Magenta,
                "cyan" => AnsiColors::Cyan,
                "white" => AnsiColors::White,
                "brightblack" => AnsiColors::BrightBlack,
                "brightred" => AnsiColors::BrightRed,
                "brightgreen" => AnsiColors::BrightGreen,
                "brightyellow" => AnsiColors::BrightYellow,
                "brightblue" => AnsiColors::BrightBlue,
                "brightmagenta" => AnsiColors::BrightMagenta,
                "brightcyan" => AnsiColors::BrightCyan,
                "brightwhite" => AnsiColors::BrightWhite,
                _ => AnsiColors::White, // fallback, though FromStr should prevent this if strict
            },
        }
    }
}

/// Represents the style configuration for a specific `ThemeEntry`.
///
/// Currently, `ThemeStyle` only supports a foreground color (`fg`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeStyle {
    /// An optional `ThemeColor` to apply as the foreground color.
    pub fg: Option<ThemeColor>,
}

// MODIFIED: `build_theme_map` is now a standalone function
/// Loads a theme configuration from a YAML file or returns the default theme.
///
/// If `theme_path` is provided, it attempts to load a custom theme from that path.
/// If `theme_path` is `None` or loading from the file fails, it falls back to
/// the default theme.
///
/// # Arguments
///
/// * `theme_path` - An optional `PathBuf` pointing to a custom theme YAML file.
///
/// # Returns
///
/// A `Result` containing a `ThemeMap` on success, or an `anyhow::Error` if
/// a custom theme is specified but cannot be loaded.
pub fn build_theme_map(theme_path: Option<&PathBuf>) -> Result<ThemeMap> {
    if let Some(path) = theme_path {
        // Attempt to load from file. If it fails, propagate the error.
        ThemeStyle::load_from_file(path)
    } else {
        // If no path is provided, return the default theme.
        Ok(ThemeStyle::default_theme_map())
    }
}

impl ThemeStyle {
    /// Loads a theme configuration from a YAML file on disk and merges it with default styles.
    ///
    /// This function reads a YAML file specified by `path`, parses it into a `HashMap`
    /// of `ThemeEntry` to `ThemeStyle`, and then ensures that all `ThemeEntry` variants
    /// have an associated style. If any entry is missing in the custom file, it's
    /// filled in with a default `ThemeStyle` (foreground color set to white).
    ///
    /// # Type Parameters
    ///
    /// * `P`: A type that can be converted into a `&Path` (e.g., `&str`, `String`, `PathBuf`).
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML theme configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap<ThemeEntry, ThemeStyle>` on success,
    /// or an `anyhow::Error` if the file cannot be read or parsed.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<ThemeMap> { // Use ThemeMap alias
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file {}", path.display()))?;
        let mut custom: ThemeMap = // Use ThemeMap alias
            serde_yaml::from_str(&text).with_context(|| format!("Failed to parse theme file {}", path.display()))?;
        // Fill in missing entries with default white.
        for entry in [
            ThemeEntry::Header,
            ThemeEntry::Success,
            ThemeEntry::Info,
            ThemeEntry::Warn,
            ThemeEntry::Error,
            ThemeEntry::RedactedText,
            ThemeEntry::DiffAdded,
            ThemeEntry::DiffRemoved,
            ThemeEntry::DiffHeader,
            ThemeEntry::SummaryRuleName,
            ThemeEntry::SummaryOccurrences,
            ThemeEntry::Prompt,
        ] {
            custom.entry(entry).or_insert_with(|| ThemeStyle { fg: Some(ThemeColor::Named("white".into())) });
        }
        Ok(custom)
    }

    /// Returns a default theme map where all `ThemeEntry` elements are styled with white foreground.
    ///
    /// This function provides a baseline theme that can be used if no custom
    /// theme file is provided or loaded.
    ///
    /// # Returns
    ///
    /// A `HashMap<ThemeEntry, ThemeStyle>` representing the default theme.
    pub fn default_theme_map() -> ThemeMap { // Use ThemeMap alias
        let mut default_theme = HashMap::new();
        for entry in [
            ThemeEntry::Header,
            ThemeEntry::Success,
            ThemeEntry::Info,
            ThemeEntry::Warn,
            ThemeEntry::Error,
            ThemeEntry::RedactedText,
            ThemeEntry::DiffAdded,
            ThemeEntry::DiffRemoved,
            ThemeEntry::DiffHeader,
            ThemeEntry::SummaryRuleName,
            ThemeEntry::SummaryOccurrences,
            ThemeEntry::Prompt,
        ] {
            default_theme.insert(entry, ThemeStyle { fg: Some(ThemeColor::Named("white".into())) });
        }
        default_theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_colors() {
        assert!("red".parse::<ThemeColor>().is_ok());
        assert!("BrightGreen".parse::<ThemeColor>().is_ok());
        assert!("unknown".parse::<ThemeColor>().is_err());
    }

    #[test]
    fn to_ansi_color_roundtrip() {
        let tc: ThemeColor = "blue".parse().unwrap();
        assert_eq!(tc.to_ansi_color(), AnsiColors::Blue);
        let tc: ThemeColor = "brightmagenta".parse().unwrap();
        assert_eq!(tc.to_ansi_color(), AnsiColors::BrightMagenta);
    }
}