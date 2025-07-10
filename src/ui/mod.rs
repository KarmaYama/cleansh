// src/ui/mod.rs

pub mod output_format;
pub mod theme;

// Re-export common structs and functions for easier access
pub use output_format::{
    print_content, print_diff_view, print_error_message, print_info_message,
    print_redaction_summary, print_success_message, print_warning_message,
    RedactionSummaryItem,
};
pub use theme::OutputTheme;
