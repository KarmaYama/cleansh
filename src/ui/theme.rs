// src/ui/theme.rs

use owo_colors::{AnsiColors, Style};
use serde::{Deserialize, Serialize};

/// Represents the color scheme and formatting preferences for cleansh's CLI output.
///
/// This struct allows for consistent styling across all output elements and
/// supports customization via configuration files.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OutputTheme {
    // --- General Text Styles ---
    /// Style for informational messages.
    pub info: Style,
    /// Style for debug messages.
    pub debug: Style,
    /// Style for warning messages.
    pub warn: Style,
    /// Style for error messages.
    pub error: Style,
    /// Style for successful operations or confirmations.
    pub success: Style,
    /// Style for highlighting key elements or changed parts (e.g., redacted text).
    pub highlight: Style,
    /// Style for secondary or less important text, making it appear muted.
    pub dim: Style,

    // --- Diff View Styles ---
    /// Style for lines added in a diff.
    pub diff_added: Style,
    /// Style for lines removed in a diff.
    pub diff_removed: Style,
    /// Style for unchanged lines in a diff.
    pub diff_unchanged: Style,

    // --- Table Styles ---
    // Note: `comfy-table` uses its own theme system, but we can map conceptual
    // styles from here or provide a default `comfy-table` theme based on these colors.
    // For now, these are conceptual. Actual `comfy-table` theme setup will be in `output_format.rs`.
}

impl Default for OutputTheme {
    /// Provides the default, aesthetically pleasing, and readable theme for cleansh.
    fn default() -> Self {
        OutputTheme {
            info: Style::new().fg(AnsiColors::White),
            debug: Style::new().fg(AnsiColors::BrightBlack).italic(),
            warn: Style::new().fg(AnsiColors::Yellow).bold(),
            error: Style::new().fg(AnsiColors::Red).bold(),
            success: Style::new().fg(AnsiColors::Green).bold(),
            highlight: Style::new().fg(AnsiColors::Cyan).bold(),
            dim: Style::new().fg(AnsiColors::BrightBlack), // Dark grey for less important text

            diff_added: Style::new().fg(AnsiColors::Green),
            diff_removed: Style::new().fg(AnsiColors::Red),
            diff_unchanged: Style::new().fg(AnsiColors::BrightBlack),
        }
    }
}

// TODO: Future enhancement: Implement `load_theme_from_file` if custom themes are desired.
// This would involve `serde_yaml` and deserializing a theme config.
// For MVP, we'll use the default theme.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_creation() {
        let theme = OutputTheme::default();
        assert_eq!(theme.info, Style::new().fg(AnsiColors::White));
        assert_eq!(theme.error, Style::new().fg(AnsiColors::Red).bold());
        assert_eq!(theme.diff_added, Style::new().fg(AnsiColors::Green));
    }
}