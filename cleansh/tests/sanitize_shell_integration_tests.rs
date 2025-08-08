// tests/sanitize_shell_integration_tests.rs
// This is an integration test, so we import from the crate root
use anyhow::Result;

// Corrected imports to use the crate root where the components are exposed.
use cleansh::test_exposed::config::{RedactionConfig, RedactionRule, MAX_PATTERN_LENGTH};
use cleansh_core::engine::{SanitizationEngine, RegexEngine};
use strip_ansi_escapes; // Import the crate to strip ANSI codes


// This block ensures that logging (e.g., from pii_debug! macro) is set up for tests.
// It initializes env_logger exactly once per test run.
#[allow(unused_imports)] // Allow unused for clarity, as it's not always directly called
#[cfg(test)]
mod test_setup {
    use std::sync::Once;
    static INIT: Once = Once::new();

    pub fn setup_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .try_init()
                .ok(); // Ignore error if logger already initialized
        });
    }
}

// Helper to create a basic rule for testing
fn create_test_rule(
    name: &str,
    pattern: &str,
    replace: &str,
    opt_in: bool,
    description: Option<&str>,
    multiline: bool,
    dot_matches_new_line: bool,
    programmatic_validation: bool, // Added for programmatic validation flag
) -> RedactionRule {
    RedactionRule {
        name: name.to_string(),
        pattern: pattern.to_string(),
        replace_with: replace.to_string(),
        description: description.map(|s| s.to_string()),
        multiline,
        dot_matches_new_line,
        opt_in,
        programmatic_validation,
    }
}

// Helper function to filter rules based on opt-in/disabled lists
fn filter_rules(
    rules: Vec<RedactionRule>,
    enabled: &[String],
    disabled: &[String],
) -> Vec<RedactionRule> {
    rules.into_iter().filter(|r| {
        let is_enabled_explicitly = enabled.contains(&r.name);
        let is_disabled_explicitly = disabled.contains(&r.name);

        if is_disabled_explicitly {
            false
        } else if r.opt_in {
            is_enabled_explicitly
        } else {
            true // Default rules are always included unless disabled
        }
    }).collect()
}

#[test]
fn test_compile_rules_basic() -> Result<()> {
    test_setup::setup_logger(); // Initialize logger for this test
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("ip", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IP]", false, None, false, false, false),
    ];
    let config = RedactionConfig { rules: rules_vec };
    let compiled = RegexEngine::new(config)?;
    assert_eq!(compiled.get_rules().rules.len(), 2); // Access .rules field
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_not_enabled() -> Result<()> {
    test_setup::setup_logger();
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let filtered_rules = filter_rules(rules_vec, &[], &[]);
    let config = RedactionConfig { rules: filtered_rules };
    let compiled = RegexEngine::new(config)?; // Not enabled
    assert_eq!(compiled.get_rules().rules.len(), 1);
    assert_eq!(compiled.get_rules().rules[0].name, "email");
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_missing_returns_empty() -> Result<()> {
    test_setup::setup_logger();
    let rules_vec = vec![
        create_test_rule("secret_key", r"secret_\w+", "[REDACTED]", true, None, false, false, false),
    ];
    let filtered_rules = filter_rules(rules_vec, &[], &[]);
    let config = RedactionConfig { rules: filtered_rules };
    let compiled = RegexEngine::new(config)?;
    assert_eq!(compiled.get_rules().rules.len(), 0);
    Ok(())
}


#[test]
fn test_compile_rules_opt_in_enabled() -> Result<()> {
    test_setup::setup_logger();
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let filtered_rules = filter_rules(rules_vec, &["aws_key".to_string()], &[]);
    let config = RedactionConfig { rules: filtered_rules };
    let compiled = RegexEngine::new(config)?;
    assert_eq!(compiled.get_rules().rules.len(), 2);
    assert!(compiled.get_rules().rules.iter().any(|r| r.name == "aws_key"));
    Ok(())
}

#[test]
fn test_compile_rules_disabled() -> Result<()> {
    test_setup::setup_logger();
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let filtered_rules = filter_rules(rules_vec, &["aws_key".to_string()], &["email".to_string()]);
    let config = RedactionConfig { rules: filtered_rules };
    let compiled = RegexEngine::new(config)?;
    assert_eq!(compiled.get_rules().rules.len(), 1);
    assert_eq!(compiled.get_rules().rules[0].name, "aws_key");
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_and_disabled_conflict() -> Result<()> {
    test_setup::setup_logger();
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("sensitive_data", "sensitive_text", "[REDACTED]", true, None, false, false, false),
    ];
    let filtered_rules = filter_rules(rules_vec, &["sensitive_data".to_string()], &["sensitive_data".to_string()]);
    let config = RedactionConfig { rules: filtered_rules };
    let compiled = RegexEngine::new(config)?;
    assert_eq!(compiled.get_rules().rules.len(), 0);
    Ok(())
}

#[test]
fn test_overlapping_rules_priority() -> Result<()> {
    test_setup::setup_logger();
    let rule_email = create_test_rule("email", r"(\w+)@example\.com", "[EMAIL]", false, None, false, false, false);
    let rule_generic = create_test_rule("example_match", r"example\.com", "[DOMAIN]", false, None, false, false, false);
    let config = RedactionConfig { rules: vec![rule_email, rule_generic] };
    let compiled = RegexEngine::new(config)?;

    let input = "user@example.com";
    let (sanitized, _summary) = compiled.sanitize(input)?;
    
    assert_eq!(sanitized, "[EMAIL]");

    Ok(())
}


#[test]
fn test_sanitize_content_basic() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL_REDACTED]", false, None, false, false, false);
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine to create the engine

    let input = "My email is test@example.com.";
    let (output, _summary) = compiled_rules.sanitize(input)?;
    
    assert_eq!(output, "My email is [EMAIL_REDACTED].");
    Ok(())
}

#[test]
fn test_sanitize_content_multiple_matches_same_rule() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL_REDACTED]", false, None, false, false, false);
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    let input = "test1@example.com and test2@example.com.";
    let (output, _summary) = compiled_rules.sanitize(input)?;
    
    assert_eq!(
        output,
        "[EMAIL_REDACTED] and [EMAIL_REDACTED]."
    );
    Ok(())
}

#[test]
fn test_sanitize_content_multiple_rules() -> Result<()> {
    test_setup::setup_logger();
    let email_rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false);
    let ip_rule = create_test_rule("ipv4_address", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IPV4]", false, None, false, false, false);

    let config = RedactionConfig { rules: vec![email_rule, ip_rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    let input = "Email: a@b.com, IP: 192.168.1.1.";
    let (output, _summary) = compiled_rules.sanitize(input)?;
    
    assert_eq!(output, "Email: [EMAIL], IP: [IPV4].");
    Ok(())
}

#[test]
fn test_sanitize_content_with_ansi_escapes() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false);
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    let input_with_ansi = "Hello \x1b[31mtest@example.com\x1b[0m world.";
    // The core change: strip ANSI escape codes before sanitizing
    let input_stripped = strip_ansi_escapes::strip_str(input_with_ansi);
    
    let (output, _summary) = compiled_rules.sanitize(&input_stripped)?;
    
    assert_eq!(output, "Hello [EMAIL] world.");
    Ok(())
}

// Tests for programmatic validation

#[test]
fn test_us_ssn_programmatic_validation_valid() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b", // Pattern with capturing groups
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Valid SSN - should be redacted
    let text_valid = "My SSN is 123-45-6789. Another is 789-12-3456.";
    let (sanitized_valid, _summary) = compiled_rules.sanitize(text_valid)?;

    assert_eq!(sanitized_valid, "My SSN is [US_SSN_REDACTED]. Another is [US_SSN_REDACTED].");
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_000() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid SSN (000 area) - should NOT be redacted programmatically, meaning no RedactionMatch is generated
    let text_invalid_area_000 = "Invalid SSN: 000-12-3456.";
    let (sanitized_invalid_area_000, _summary) = compiled_rules.sanitize(text_invalid_area_000)?;
    
    assert_eq!(sanitized_invalid_area_000, "Invalid SSN: 000-12-3456.");
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_666() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid SSN (666 area) - should NOT be redacted programmatically, meaning no RedactionMatch is generated
    let text_invalid_area_666 = "Another invalid: 666-78-9012.";
    let (sanitized_invalid_area_666, _summary) = compiled_rules.sanitize(text_invalid_area_666)?;

    assert_eq!(sanitized_invalid_area_666, "Another invalid: 666-78-9012.");
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_9xx() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid SSN (9XX area) - should NOT be redacted programmatically, meaning no RedactionMatch is generated
    let text_invalid_area_9xx = "Area 9: 900-11-2222.";
    let (sanitized_invalid_area_9xx, _summary) = compiled_rules.sanitize(text_invalid_area_9xx)?;
    
    assert_eq!(sanitized_invalid_area_9xx, "Area 9: 900-11-2222.");
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_group_00() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid SSN (00 group) - should NOT be redacted programmatically, meaning no RedactionMatch is generated
    let text_invalid_group_00 = "Group 00: 123-00-4567.";
    let (sanitized_invalid_group_00, _summary) = compiled_rules.sanitize(text_invalid_group_00)?;

    assert_eq!(sanitized_invalid_group_00, "Group 00: 123-00-4567.");
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_serial_0000() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid SSN (0000 serial) - should NOT be redacted programmatically, meaning no RedactionMatch is generated
    let text_invalid_serial_0000 = "Serial 0000: 123-45-0000.";
    let (sanitized_invalid_serial_0000, _summary) = compiled_rules.sanitize(text_invalid_serial_0000)?;
    
    assert_eq!(sanitized_invalid_serial_0000, "Serial 0000: 123-45-0000.");
    Ok(())
}

#[test]
fn test_uk_nino_programmatic_validation_valid() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "uk_nino",
        r"\b([A-CEGHJ-NPR-TW-Z]{2})\s?(\d{2})\s?(\d{2})\s?(\d{2})\s?([A-D])\b",
        "[UK_NINO_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Corrected input: Use a genuinely valid NINO with spaces
    let input = "Valid NINO: AB123456A. Valid Spaced NINO: AA 12 34 56 B.";
    let (sanitized, _summary) = compiled_rules.sanitize(input)?;
    
    assert_eq!(sanitized, "Valid NINO: [UK_NINO_REDACTED]. Valid Spaced NINO: [UK_NINO_REDACTED].");
    Ok(())
}

#[test]
fn test_uk_nino_programmatic_validation_invalid_prefix() -> Result<()> {
    test_setup::setup_logger();
    let rule = create_test_rule(
        "uk_nino",
        r"\b([A-CEGHJ-NPR-TW-Z]{2})\s?(\d{2})\s?(\d{2})\s?(\d{2})\s?([A-D])\b",
        "[UK_NINO_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let config = RedactionConfig { rules: vec![rule] };
    let compiled_rules = RegexEngine::new(config)?; // Use RegexEngine

    // Invalid prefixes: BG, GB, NK, KN, TN, NT, ZZ, and those starting with D, F, I, Q, U, V, O
    let input = "Invalid BG: BG123456A. Invalid GB: GB123456B. Invalid ZZ: ZZ123456C. Invalid DF: DF123456A. Invalid QV: QV123456B.";
    let (sanitized, _summary) = compiled_rules.sanitize(input)?;
    
    // These should NOT be redacted due to programmatic validation, meaning no RedactionMatch is generated
    assert_eq!(sanitized, "Invalid BG: BG123456A. Invalid GB: GB123456B. Invalid ZZ: ZZ123456C. Invalid DF: DF123456A. Invalid QV: QV123456B.");
    Ok(())
}

#[test]
fn test_compile_rules_invalid_regex_fails_fast() {
    test_setup::setup_logger();
    let rules_vec = vec![
        create_test_rule("valid_rule", "abc", "[REDACTED]", false, None, false, false, false),
        create_test_rule("invalid_rule", "[", "[ERROR]", false, None, false, false, false), // Invalid regex
    ];
    let config = RedactionConfig { rules: rules_vec };
    let result = RegexEngine::new(config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Failed to compile redaction rules for RegexEngine"));
}

#[test]
fn test_compile_rules_pattern_too_long_fails_fast() {
    test_setup::setup_logger();
    let long_pattern = "a".repeat(MAX_PATTERN_LENGTH + 1);
    let rules_vec = vec![
        create_test_rule("valid_rule", "abc", "[REDACTED]", false, None, false, false, false),
        create_test_rule("long_pattern_rule", &long_pattern, "[TOO_LONG]", false, None, false, false, false), // Corrected call with `None` for description
    ];
    let config = RedactionConfig { rules: rules_vec };
    let result = RegexEngine::new(config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Failed to compile redaction rules for RegexEngine"));
}