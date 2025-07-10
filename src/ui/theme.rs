// src/ui/theme.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use anyhow::{Context, Result};
use owo_colors::AnsiColors;

/// The different logical parts of your output that can be styled.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeEntry {
    Header,
    Success,
    Info,
    Warn,
    Error,
    RedactedText,
    DiffAdded,
    DiffRemoved,
    DiffHeader,
    SummaryRuleName,
    SummaryOccurrences,
}

/// Only named ANSI colors (the 16‑color standard).
/// RGB and 256‑color codes are no longer supported.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ThemeColor {
    Named(String),
}

/// Parsing errors for `ThemeColor`.
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
    /// Map the string name into the `AnsiColors` enum.
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

/// Holds the raw style configuration for each entry.
/// Now only a foreground color is supported.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeStyle {
    pub fg: Option<ThemeColor>,
}

impl ThemeStyle {
    /// Loads the YAML from disk and merges with defaults.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<ThemeEntry, ThemeStyle>> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file {}", path.display()))?;
        let mut custom: HashMap<ThemeEntry, ThemeStyle> =
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
        ] {
            custom.entry(entry).or_insert_with(|| ThemeStyle { fg: Some(ThemeColor::Named("white".into())) });
        }
        Ok(custom)
    }

    /// Returns a default theme map with all entries set to white.
    pub fn default_theme_map() -> HashMap<ThemeEntry, ThemeStyle> {
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