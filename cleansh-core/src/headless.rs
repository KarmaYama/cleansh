// File: cleansh-core/src/headless.rs

//! `headless.rs`
//! Convenience wrappers for using core engines in headless mode (non-UI).
//! Provides helper functions for a full, one-shot sanitization of strings.

use anyhow::Result;
use crate::config::RedactionConfig;
use crate::profiles::EngineOptions;
use crate::engines::regex_engine::RegexEngine;
use crate::engine::SanitizationEngine;

/// Fully sanitizes an input string by finding and applying all redaction matches.
/// This function is the primary entry point for non-interactive (headless) use.
///
/// `config` is the merged RedactionConfig (defaults + optional user overrides).
/// `options` represents EngineOptions (run_seed, etc).
/// `content` is the string to be sanitized.
/// `source_id` is a stable identifier for the input (file path or pseudo id).
pub fn headless_sanitize_string(
    config: RedactionConfig,
    options: EngineOptions,
    content: &str,
    source_id: &str,
) -> Result<String> {
    let engine = RegexEngine::with_options(config, options)?;
    // The `sanitize` method takes audit log parameters, which we can provide as empty placeholders.
    let (sanitized_content, _) = engine.sanitize(
        content,
        source_id,
        "",
        "",
        "",
        "",
        "",
        None,
    )?;
    Ok(sanitized_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedactionRule;
    use crate::profiles::EngineOptions;
    use anyhow::Result;

    #[test]
    fn test_headless_sanitize_string() -> Result<()> {
        let content = "My email is test@example.com, and another is another@example.net.";
        let config = RedactionConfig {
            rules: vec![
                RedactionRule {
                    name: "email".to_string(),
                    pattern: Some("([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[A-Za-z]{2,})".to_string()),
                    enabled: Some(true),
                    severity: Some("high".to_string()),
                    replace_with: "[EMAIL]".to_string(),
                    description: Some("Matches email addresses".to_string()),
                    multiline: false,
                    dot_matches_new_line: false,
                    programmatic_validation: false,
                    opt_in: false,
                    tags: None,
                    pattern_type: "regex".to_string(),
                    version: "0.1.8".to_string(),
                    created_at: "2025-01-01T00:00:00Z".to_string(),
                    updated_at: "2025-01-01T00:00:00Z".to_string(),
                    author: "Obscura Team".to_string(),
                },
            ],
        };
        let options = EngineOptions::default();
        
        let sanitized_content = headless_sanitize_string(config, options, content, "test_input")?;
        
        let expected_output = "My email is [EMAIL], and another is [EMAIL].";
        assert_eq!(sanitized_content, expected_output);
        
        Ok(())
    }
}