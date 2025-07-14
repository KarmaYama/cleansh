// tests/sanitize_shell_integration_tests.rs

// This is an integration test, so we import from the crate root
use anyhow::Result;

// Only import what's directly used in this test file
use cleansh::test_exposed::config::{RedactionRule};
use cleansh::test_exposed::tools::{compile_rules, sanitize_content};

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

#[test]
fn test_compile_rules_basic() -> Result<()> {
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("ip", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IP]", false, None, false, false, false),
    ];
    let compiled = compile_rules(rules_vec, &[], &[]).unwrap();
    assert_eq!(compiled.rules.len(), 2); // Access .rules field
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_not_enabled() -> Result<()> {
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let compiled = compile_rules(rules_vec, &[], &[]).unwrap(); // Not enabled
    assert_eq!(compiled.rules.len(), 1);
    assert_eq!(compiled.rules[0].name, "email");
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_missing_returns_empty() -> Result<()> {
    let rules_vec = vec![
        create_test_rule("secret_key", r"secret_\w+", "[REDACTED]", true, None, false, false, false),
    ];
    let compiled = compile_rules(rules_vec, &[], &[])?;
    assert_eq!(compiled.rules.len(), 0);
    Ok(())
}


#[test]
fn test_compile_rules_opt_in_enabled() -> Result<()> {
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let compiled = compile_rules(
        rules_vec,
        &["aws_key".to_string()], // Enable aws_key
        &[],
    )
    .unwrap();
    assert_eq!(compiled.rules.len(), 2);
    assert!(compiled.rules.iter().any(|r| r.name == "aws_key"));
    Ok(())
}

#[test]
fn test_compile_rules_disabled() -> Result<()> {
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false),
        create_test_rule("aws_key", "AKIA[A-Z0-9]{16}", "[AWS_KEY]", true, None, false, false, false), // Opt-in
    ];
    let compiled = compile_rules(
        rules_vec,
        &["aws_key".to_string()],
        &["email".to_string()], // Disable email
    )
    .unwrap();
    assert_eq!(compiled.rules.len(), 1);
    assert_eq!(compiled.rules[0].name, "aws_key");
    Ok(())
}

#[test]
fn test_compile_rules_opt_in_and_disabled_conflict() -> Result<()> {
    let rules_vec = vec![ // Directly pass Vec<RedactionRule>
        create_test_rule("sensitive_data", "sensitive_text", "[REDACTED]", true, None, false, false, false),
    ];
    let compiled = compile_rules(
        rules_vec,
        &["sensitive_data".to_string()],
        &["sensitive_data".to_string()],
    )
    .unwrap();
    assert_eq!(compiled.rules.len(), 0);
    Ok(())
}

#[test]
fn test_overlapping_rules_priority() -> Result<()> {
    let rule_email = create_test_rule("email", r"(\w+)@example\.com", "[EMAIL]", false, None, false, false, false);
    let rule_generic = create_test_rule("example_match", r"example\.com", "[DOMAIN]", false, None, false, false, false);
    // Order matters here when compiling, assuming the `compile_rules` or `sanitize_content` logic
    // applies the first matching rule, or the "longest match".
    // If the email regex matches the entire string, it will likely take precedence.
    let compiled = compile_rules(vec![rule_email, rule_generic], &[], &[])?; 
    
    let input = "user@example.com";
    let (sanitized, summary) = sanitize_content(&input, &compiled);
    
    // Updated assertion: If "email" rule (which is a complete match) applies first/greedily,
    // the output will be "[EMAIL]". The summary should reflect only one redaction.
    assert_eq!(sanitized, "[EMAIL]"); 
    assert_eq!(summary.len(), 1); // Only one rule should fire if it's a full replacement
    
    // Additionally, assert the details of the single redaction for clarity
    assert_eq!(summary[0].rule_name, "email");
    assert_eq!(summary[0].original_texts, vec!["user@example.com"]);
    assert_eq!(summary[0].sanitized_texts, vec!["[EMAIL]"]);

    Ok(())
}


#[test]
fn test_sanitize_content_basic() -> Result<()> {
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL_REDACTED]", false, None, false, false, false);
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules to create CompiledRules struct

    let input = "My email is test@example.com.";
    let (output, summary) = sanitize_content(input, &compiled_rules);
    assert_eq!(output, "My email is [EMAIL_REDACTED].");
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].rule_name, "email");
    assert_eq!(summary[0].occurrences, 1);
    assert_eq!(summary[0].original_texts, vec!["test@example.com"]);
    assert_eq!(summary[0].sanitized_texts, vec!["[EMAIL_REDACTED]"]);
    Ok(())
}

#[test]
fn test_sanitize_content_multiple_matches_same_rule() -> Result<()> {
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL_REDACTED]", false, None, false, false, false);
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    let input = "test1@example.com and test2@example.com.";
    let (output, summary) = sanitize_content(input, &compiled_rules);
    assert_eq!(
        output,
        "[EMAIL_REDACTED] and [EMAIL_REDACTED]."
    );
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].rule_name, "email");
    assert_eq!(summary[0].occurrences, 2);
    // Sort for consistent assertion, as HashMap iteration order is not guaranteed (summary is sorted now)
    let mut expected_original_texts = vec!["test1@example.com".to_string(), "test2@example.com".to_string()];
    expected_original_texts.sort(); // Ensure local sort as well for comparison
    assert_eq!(summary[0].original_texts, expected_original_texts);
    assert_eq!(summary[0].sanitized_texts, vec!["[EMAIL_REDACTED]"]);
    Ok(())
}

#[test]
fn test_sanitize_content_multiple_rules() -> Result<()> {
    let email_rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false);
    let ip_rule = create_test_rule("ipv4_address", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "[IPV4]", false, None, false, false, false);

    let compiled_rules = compile_rules(vec![email_rule, ip_rule], &[], &[]).unwrap(); // Use compile_rules

    let input = "Email: a@b.com, IP: 192.168.1.1.";
    let (output, summary) = sanitize_content(input, &compiled_rules);
    assert_eq!(output, "Email: [EMAIL], IP: [IPV4].");
    assert_eq!(summary.len(), 2);
    // summary is already sorted by rule name in sanitize_content

    assert_eq!(summary[0].rule_name, "email");
    assert_eq!(summary[0].occurrences, 1);
    assert_eq!(summary[0].original_texts, vec!["a@b.com"]);
    assert_eq!(summary[0].sanitized_texts, vec!["[EMAIL]"]);

    assert_eq!(summary[1].rule_name, "ipv4_address");
    assert_eq!(summary[1].occurrences, 1);
    assert_eq!(summary[1].original_texts, vec!["192.168.1.1"]);
    assert_eq!(summary[1].sanitized_texts, vec!["[IPV4]"]);
    Ok(())
}

#[test]
fn test_sanitize_content_with_ansi_escapes() -> Result<()> {
    let rule = create_test_rule("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]", false, None, false, false, false);
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    let input = "Hello \x1b[31mtest@example.com\x1b[0m world.";
    let (output, summary) = sanitize_content(input, &compiled_rules);
    assert_eq!(output, "Hello [EMAIL] world.");
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].rule_name, "email");
    assert_eq!(summary[0].occurrences, 1);
    assert_eq!(summary[0].original_texts, vec!["test@example.com"]);
    assert_eq!(summary[0].sanitized_texts, vec!["[EMAIL]"]);
    Ok(())
}

// Tests for programmatic validation

#[test]
fn test_us_ssn_programmatic_validation_valid() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b", // Pattern with capturing groups
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Valid SSN - should be redacted
    let text_valid = "My SSN is 123-45-6789. Another is 789-12-3456.";
    let (sanitized_valid, summary) = sanitize_content(text_valid, &compiled_rules);
    assert_eq!(sanitized_valid, "My SSN is [US_SSN_REDACTED]. Another is [US_SSN_REDACTED].");
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].rule_name, "us_ssn");
    assert_eq!(summary[0].occurrences, 2);
    // Sort for consistent assertion
    let mut expected_original_texts = vec!["123-45-6789".to_string(), "789-12-3456".to_string()];
    expected_original_texts.sort();
    assert_eq!(summary[0].original_texts, expected_original_texts);
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_000() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid SSN (000 area) - should NOT be redacted programmatically
    let text_invalid_area_000 = "Invalid SSN: 000-12-3456.";
    let (sanitized_invalid_area_000, summary) = sanitize_content(text_invalid_area_000, &compiled_rules);
    assert_eq!(sanitized_invalid_area_000, "Invalid SSN: 000-12-3456.");
    assert!(summary.is_empty() || summary[0].occurrences == 0);
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_666() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid SSN (666 area) - should NOT be redacted programmatically
    let text_invalid_area_666 = "Another invalid: 666-78-9012.";
    let (sanitized_invalid_area_666, summary) = sanitize_content(text_invalid_area_666, &compiled_rules);
    assert_eq!(sanitized_invalid_area_666, "Another invalid: 666-78-9012.");
    assert!(summary.is_empty() || summary[0].occurrences == 0);
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_area_9xx() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid SSN (9XX area) - should NOT be redacted programmatically
    let text_invalid_area_9xx = "Area 9: 900-11-2222.";
    let (sanitized_invalid_area_9xx, summary) = sanitize_content(text_invalid_area_9xx, &compiled_rules);
    assert_eq!(sanitized_invalid_area_9xx, "Area 9: 900-11-2222.");
    assert!(summary.is_empty() || summary[0].occurrences == 0);
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_group_00() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid SSN (00 group) - should NOT be redacted programmatically
    let text_invalid_group_00 = "Group 00: 123-00-4567.";
    let (sanitized_invalid_group_00, summary) = sanitize_content(text_invalid_group_00, &compiled_rules);
    assert_eq!(sanitized_invalid_group_00, "Group 00: 123-00-4567.");
    assert!(summary.is_empty() || summary[0].occurrences == 0);
    Ok(())
}

#[test]
fn test_us_ssn_programmatic_validation_invalid_serial_0000() -> Result<()> {
    let rule = create_test_rule(
        "us_ssn",
        r"\b(\d{3})-(\d{2})-(\d{4})\b",
        "[US_SSN_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid SSN (0000 serial) - should NOT be redacted programmatically
    let text_invalid_serial_0000 = "Serial 0000: 123-45-0000.";
    let (sanitized_invalid_serial_0000, summary) = sanitize_content(text_invalid_serial_0000, &compiled_rules);
    assert_eq!(sanitized_invalid_serial_0000, "Serial 0000: 123-45-0000.");
    assert!(summary.is_empty() || summary[0].occurrences == 0);
    Ok(())
}

#[test]
fn test_uk_nino_programmatic_validation_valid() -> Result<()> {
    let rule = create_test_rule(
        "uk_nino",
        r"\b([A-CEGHJ-NPR-TW-Z]{2})\s?(\d{2})\s?(\d{2})\s?(\d{2})\s?([A-D])\b",
        "[UK_NINO_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Corrected input: Use a genuinely valid NINO with spaces
    let input = "Valid NINO: AB123456A. Valid Spaced NINO: AA 12 34 56 B.";
    let (sanitized, summary) = sanitize_content(input, &compiled_rules);

    assert_eq!(sanitized, "Valid NINO: [UK_NINO_REDACTED]. Valid Spaced NINO: [UK_NINO_REDACTED].");
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].rule_name, "uk_nino");
    assert_eq!(summary[0].occurrences, 2);
    // Sort for consistent assertion
    let mut expected_original_texts = vec!["AB123456A".to_string(), "AA 12 34 56 B".to_string()];
    expected_original_texts.sort();
    assert_eq!(summary[0].original_texts, expected_original_texts);
    Ok(())
}

#[test]
fn test_uk_nino_programmatic_validation_invalid_prefix() -> Result<()> {
    let rule = create_test_rule(
        "uk_nino",
        r"\b([A-CEGHJ-NPR-TW-Z]{2})\s?(\d{2})\s?(\d{2})\s?(\d{2})\s?([A-D])\b",
        "[UK_NINO_REDACTED]",
        false, None, false, false,
        true, // Enable programmatic validation
    );
    let compiled_rules = compile_rules(vec![rule], &[], &[]).unwrap(); // Use compile_rules

    // Invalid prefixes: BG, GB, NK, KN, TN, NT, ZZ, and those starting with D, F, I, Q, U, V, O
    let input = "Invalid BG: BG123456A. Invalid GB: GB123456B. Invalid ZZ: ZZ123456C. Invalid DF: DF123456A. Invalid QV: QV123456B.";
    let (sanitized, summary) = sanitize_content(input, &compiled_rules);

    // These should NOT be redacted due to programmatic validation
    assert_eq!(sanitized, "Invalid BG: BG123456A. Invalid GB: GB123456B. Invalid ZZ: ZZ123456C. Invalid DF: DF123456A. Invalid QV: QV123456B.");
    assert!(summary.is_empty()); // No redactions should have occurred
    Ok(())
}

#[test]
fn test_compile_rules_invalid_regex_fails_fast() {
    let rules_vec = vec![
        create_test_rule("valid_rule", "abc", "[REDACTED]", false, None, false, false, false),
        create_test_rule("invalid_rule", "[", "[ERROR]", false, None, false, false, false), // Invalid regex
    ];
    let result = compile_rules(rules_vec, &[], &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Failed to compile 1 rule(s)"));
    assert!(err_msg.contains("invalid_rule"));
    assert!(err_msg.contains("failed to compile regex pattern '['"));
}

#[test]
fn test_compile_rules_pattern_too_long_fails_fast() {
    use cleansh::test_exposed::config::MAX_PATTERN_LENGTH;
    let long_pattern = "a".repeat(MAX_PATTERN_LENGTH + 1);
    let rules_vec = vec![
        create_test_rule("valid_rule", "abc", "[REDACTED]", false, None, false, false, false),
        create_test_rule("long_pattern_rule", &long_pattern, "[TOO_LONG]", false, None, false, false, false),
    ];
    let result = compile_rules(rules_vec, &[], &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Failed to compile 1 rule(s)"));
    assert!(err_msg.contains("long_pattern_rule"));
    assert!(err_msg.contains(&format!("pattern length ({}) exceeds maximum allowed ({})", MAX_PATTERN_LENGTH + 1, MAX_PATTERN_LENGTH)));
}