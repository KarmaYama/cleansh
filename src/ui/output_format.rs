// src/ui/output_format.rs

use crate::ui::theme::OutputTheme;
use comfy_table::{presets::UTF8_FULL, Table};
use diffy::{diff, Patch};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use log::{info, warn, error}; // Used for internal logging of output actions


/// Prints a general informational message to stdout using the theme's info style.
pub fn print_info_message(message: &str, theme: &OutputTheme) {
    let _ = writeln!(io::stdout(), "{}", message.fg_rgb(theme.info.fg_color.unwrap().rgb()));
    info!("Printed info message: {}", message);
}

/// Prints a success message to stdout using the theme's success style.
pub fn print_success_message(message: &str, theme: &OutputTheme) {
    let _ = writeln!(io::stdout(), "{}", message.fg_rgb(theme.success.fg_color.unwrap().rgb()).bold());
    info!("Printed success message: {}", message);
}

/// Prints a warning message to stderr using the theme's warning style.
pub fn print_warning_message(message: &str, theme: &OutputTheme) {
    let _ = writeln!(io::stderr(), "{}", message.fg_rgb(theme.warn.fg_color.unwrap().rgb()).bold());
    warn!("Printed warning message: {}", message);
}

/// Prints an error message to stderr using the theme's error style.
pub fn print_error_message(message: &str, theme: &OutputTheme) {
    let _ = writeln!(io::stderr(), "{}", message.fg_rgb(theme.error.fg_color.unwrap().rgb()).bold());
    error!("Printed error message: {}", message);
}

/// Prints content (e.g., sanitized output) to stdout.
/// This is the primary way `cleansh` outputs its results.
pub fn print_content(content: &str) {
    let _ = write!(io::stdout(), "{}", content);
}

/// Represents a single item to be displayed in a redaction summary table.
/// This struct makes it easy to pass structured data to the table printer.
pub struct RedactionSummaryItem<'a> {
    pub rule_name: &'a str,
    pub original_text: &'a str,
    pub sanitized_text: &'a str,
}

/// Prints a tabular summary of redactions made.
pub fn print_redaction_summary(items: &[RedactionSummaryItem], theme: &OutputTheme) {
    if items.is_empty() {
        print_info_message("No redactions were performed.", theme);
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            "Rule",
            "Original Text (Snippet)",
            "Sanitized Text (Snippet)",
        ])
        .set_header_style(theme.highlight); // Apply header style

    for item in items {
        table.add_row(vec![
            item.rule_name.fg_rgb(theme.dim.fg_color.unwrap().rgb()).to_string(),
            // Truncate original text for table readability if it's too long
            format!("{}",
                item.original_text.fg_rgb(theme.diff_removed.fg_color.unwrap().rgb()).dimmed(),
            ),
            format!("{}",
                item.sanitized_text.fg_rgb(theme.diff_added.fg_color.unwrap().rgb()).bold(),
            ),
        ]);
    }

    let _ = writeln!(io::stdout(), "\n{}", table);
    info!("Printed redaction summary table.");
}

/// Prints a side-by-side or unified diff view of the original and sanitized content.
pub fn print_diff_view(original: &str, sanitized: &str, theme: &OutputTheme) {
    print_info_message("--- Diff View of Redactions ---", theme);

    let patch = diff(original, sanitized);
    for line in Patch::unified(&patch).lines() {
        if line.starts_with('+') {
            let _ = writeln!(io::stdout(), "{}", line.fg_rgb(theme.diff_added.fg_color.unwrap().rgb()));
        } else if line.starts_with('-') {
            let _ = writeln!(io::stdout(), "{}", line.fg_rgb(theme.diff_removed.fg_color.unwrap().rgb()));
        } else {
            let _ = writeln!(io::stdout(), "{}", line.fg_rgb(theme.diff_unchanged.fg_color.unwrap().rgb()));
        }
    }
    info!("Printed diff view.");
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::OutputTheme; // Need to import theme for tests
    use std::cell::RefCell;

    // A mock stdout/stderr to capture output for testing
    struct MockWriter {
        buffer: RefCell<Vec<u8>>,
    }

    impl MockWriter {
        fn new() -> Self {
            MockWriter {
                buffer: RefCell::new(Vec::new()),
            }
        }
        fn to_string(&self) -> String {
            String::from_utf8(self.buffer.borrow().clone()).unwrap()
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Override stdio for tests
    macro_rules! with_mock_io {
        ($body:expr) => {{
            let mut mock_stdout = MockWriter::new();
            let original_stdout = io::stdout();
            let _guard_stdout = io::set_print(Some(Box::new(&mut mock_stdout))).unwrap();

            let mut mock_stderr = MockWriter::new();
            let original_stderr = io::stderr();
            let _guard_stderr = io::set_print(Some(Box::new(&mut mock_stderr))).unwrap();

            let result = { $body };

            // Restore original stdout/stderr (important!)
            io::set_print(Some(original_stdout)).unwrap();
            io::set_print(Some(original_stderr)).unwrap();

            (result, mock_stdout.to_string(), mock_stderr.to_string())
        }};
    }


    #[test]
    fn test_print_info_message() {
        let theme = OutputTheme::default();
        let (_result, stdout, _stderr) = with_mock_io!({
            print_info_message("Test info", &theme);
        });
        assert!(stdout.contains("Test info"));
        // Check for basic ANSI escape codes for color (exact color code might vary by terminal)
        assert!(stdout.contains("\x1b[37mTest info\x1b[0m")); // Default white color
    }

    #[test]
    fn test_print_error_message() {
        let theme = OutputTheme::default();
        let (_result, _stdout, stderr) = with_mock_io!({
            print_error_message("Test error", &theme);
        });
        assert!(stderr.contains("Test error"));
        assert!(stderr.contains("\x1b[31mTest error\x1b[0m")); // Default red color
    }

    #[test]
    fn test_print_content() {
        let (_result, stdout, _stderr) = with_mock_io!({
            print_content("Raw output content");
        });
        assert_eq!(stdout, "Raw output content");
    }

    #[test]
    fn test_print_redaction_summary_empty() {
        let theme = OutputTheme::default();
        let (_result, stdout, _stderr) = with_mock_io!({
            print_redaction_summary(&[], &theme);
        });
        assert!(stdout.contains("No redactions were performed."));
    }

    #[test]
    fn test_print_redaction_summary_with_items() {
        let theme = OutputTheme::default();
        let items = vec![
            RedactionSummaryItem {
                rule_name: "email",
                original_text: "test@example.com",
                sanitized_text: "[EMAIL_REDACTED]",
            },
            RedactionSummaryItem {
                rule_name: "ip",
                original_text: "192.168.1.1",
                sanitized_text: "[IPV4_REDACTED]",
            },
        ];
        let (_result, stdout, _stderr) = with_mock_io!({
            print_redaction_summary(&items, &theme);
        });
        assert!(stdout.contains("Rule"));
        assert!(stdout.contains("Original Text (Snippet)"));
        assert!(stdout.contains("Sanitized Text (Snippet)"));
        assert!(stdout.contains("email"));
        assert!(stdout.contains("[EMAIL_REDACTED]"));
        assert!(stdout.contains("192.168.1.1"));
        assert!(stdout.contains("[IPV4_REDACTED]"));
    }

    #[test]
    fn test_print_diff_view() {
        let theme = OutputTheme::default();
        let original = "Line 1\nLine 2 sensitive\nLine 3";
        let sanitized = "Line 1\nLine 2 [REDACTED]\nLine 3";
        let (_result, stdout, _stderr) = with_mock_io!({
            print_diff_view(original, sanitized, &theme);
        });
        assert!(stdout.contains("--- Diff View of Redactions ---"));
        assert!(stdout.contains("- Line 2 sensitive"));
        assert!(stdout.contains("+ Line 2 [REDACTED]"));
        assert!(stdout.contains("Line 1")); // Unchanged line
    }
}