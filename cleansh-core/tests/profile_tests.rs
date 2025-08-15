// cleansh-core/tests/profile_tests.rs
use cleansh_core::profiles::*;
use cleansh_core::redaction_match::RedactionMatch;
use std::collections::HashSet;
use anyhow::Result;

// Correctly import RedactionConfig and RedactionRule from the cleansh-core crate
use cleansh_core::config::{RedactionConfig, RedactionRule};

#[test]
fn test_profile_validation_success() -> Result<()> {
    let default_config = RedactionConfig {
        rules: vec![
            RedactionRule {
                name: "email".to_string(),
                author: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                version: "".to_string(),
                pattern_type: "regex".to_string(),
                pattern: Some("email".to_string()),
                replace_with: "".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                programmatic_validation: false,
                enabled: Some(true),
                severity: None,
                tags: None,
                opt_in: false,
            },
            RedactionRule {
                name: "credit_card".to_string(),
                author: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                version: "".to_string(),
                pattern_type: "regex".to_string(),
                pattern: Some("credit_card".to_string()),
                replace_with: "".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                programmatic_validation: false,
                enabled: Some(true),
                severity: None,
                tags: None,
                opt_in: false,
            },
        ],
    };

    let profile = ProfileConfig {
        profile_name: "test_profile".to_string(),
        display_name: None,
        description: None,
        version: "v1.0".to_string(),
        profile_id: None,
        author: None,
        compliance_scope: None,
        revision_date: None,
        signature: None,
        signature_alg: None,
        rules: vec![
            ProfileRule { name: "email".to_string(), enabled: Some(false), severity: None },
            ProfileRule { name: "credit_card".to_string(), enabled: Some(true), severity: Some("high".to_string()) },
        ],
        samples: Some(SamplesConfig { max_per_rule: 3, max_total: 10 }),
        dedupe: None,
        post_processing: None,
        reporting: None,
    };

    profile.validate(&default_config)?;
    Ok(())
}

#[test]
fn test_profile_validation_fails_on_unknown_rule() {
    let default_config = RedactionConfig {
        rules: vec![
            RedactionRule {
                name: "email".to_string(),
                author: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                version: "".to_string(),
                pattern_type: "regex".to_string(),
                pattern: Some("email".to_string()),
                replace_with: "".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                programmatic_validation: false,
                enabled: Some(true),
                severity: None,
                tags: None,
                opt_in: false,
            },
        ],
    };

    let profile = ProfileConfig {
        profile_name: "test_profile".to_string(),
        display_name: None,
        description: None,
        version: "v1.0".to_string(),
        profile_id: None,
        author: None,
        compliance_scope: None,
        revision_date: None,
        signature: None,
        signature_alg: None,
        rules: vec![
            ProfileRule { name: "unknown_rule".to_string(), enabled: Some(true), severity: None },
        ],
        samples: None,
        dedupe: None,
        post_processing: None,
        reporting: None,
    };

    assert!(profile.validate(&default_config).is_err());
}

#[test]
fn test_profile_validation_fails_on_invalid_samples() {
    let default_config = RedactionConfig {
        rules: vec![
            RedactionRule {
                name: "email".to_string(),
                author: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                version: "".to_string(),
                pattern_type: "regex".to_string(),
                pattern: Some("email".to_string()),
                replace_with: "".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                programmatic_validation: false,
                enabled: Some(true),
                severity: None,
                tags: None,
                opt_in: false,
            },
        ],
    };

    let profile = ProfileConfig {
        profile_name: "test_profile".to_string(),
        display_name: None,
        description: None,
        version: "v1.0".to_string(),
        profile_id: None,
        author: None,
        compliance_scope: None,
        revision_date: None,
        signature: None,
        signature_alg: None,
        rules: vec![
            ProfileRule { name: "email".to_string(), enabled: Some(true), severity: None },
        ],
        samples: Some(SamplesConfig { max_per_rule: 10, max_total: 5 }),
        dedupe: None,
        post_processing: None,
        reporting: None,
    };

    assert!(profile.validate(&default_config).is_err());
}

#[test]
fn test_profile_validation_handles_unlimited_samples() -> Result<()> {
    let default_config = RedactionConfig {
        rules: vec![
            RedactionRule {
                name: "email".to_string(),
                author: "".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                version: "".to_string(),
                pattern_type: "regex".to_string(),
                pattern: Some("email".to_string()),
                replace_with: "".to_string(),
                description: None,
                multiline: false,
                dot_matches_new_line: false,
                programmatic_validation: false,
                enabled: Some(true),
                severity: None,
                tags: None,
                opt_in: false,
            },
        ],
    };

    let profile = ProfileConfig {
        profile_name: "test_profile".to_string(),
        display_name: None,
        description: None,
        version: "v1.0".to_string(),
        profile_id: None,
        author: None,
        compliance_scope: None,
        revision_date: None,
        signature: None,
        signature_alg: None,
        rules: vec![
            ProfileRule { name: "email".to_string(), enabled: Some(true), severity: None },
        ],
        samples: Some(SamplesConfig { max_per_rule: 3, max_total: 0 }),
        dedupe: None,
        post_processing: None,
        reporting: None,
    };

    assert!(profile.validate(&default_config).is_ok());
    Ok(())
}

#[test]
fn test_compute_run_seed_normalization() -> Result<()> {
    let seed1 = compute_run_seed("v1.0.0", "run-id-1", "CLEANSH-ENGINE-V2")?;
    let seed2 = compute_run_seed("V1.0.0 ", "RUN-ID-1", "cleansh-engine-v2 ")?;
    assert_eq!(seed1, seed2);
    Ok(())
}

#[test]
fn test_sample_score_bytes_determinism() -> Result<()> {
    let run_seed = compute_run_seed("v1", "run1", "v0.1")?;
    let score1 = sample_score_bytes(&run_seed, "file.txt", 10, 20)?;
    let score2 = sample_score_bytes(&run_seed, "file.txt", 10, 20)?;
    assert_eq!(score1, score2);
    Ok(())
}

#[test]
fn test_sample_score_bytes_differs_with_inputs() -> Result<()> {
    let run_seed = compute_run_seed("v1", "run1", "v0.1")?;
    let score1 = sample_score_bytes(&run_seed, "file.txt", 10, 20)?;
    let score2 = sample_score_bytes(&run_seed, "file.txt", 10, 21)?;
    assert_ne!(score1, score2);
    Ok(())
}

#[test]
fn test_select_samples_correctly_sorts_and_dedupes() -> Result<()> {
    let run_seed = compute_run_seed("v1", "run1", "v0.1")?;
    
    // Create a mock RedactionRule to avoid repetition.
    let mock_rule = RedactionRule {
        name: "email".to_string(),
        author: "".to_string(),
        created_at: "".to_string(),
        updated_at: "".to_string(),
        version: "".to_string(),
        pattern_type: "".to_string(),
        pattern: Some("".to_string()),
        replace_with: "".to_string(),
        description: None,
        multiline: false,
        dot_matches_new_line: false,
        programmatic_validation: false,
        enabled: None,
        severity: None,
        tags: None,
        opt_in: false,
    };

    let matches = vec![
        // Match 1: same hash as Match 2, should be deduped.
        RedactionMatch { 
            rule_name: "email".to_string(), 
            original_string: "".to_string(),
            sanitized_string: "".to_string(), 
            start: 100, 
            end: 110, 
            line_number: None, // Added missing field
            sample_hash: Some("hash_c".to_string()), 
            match_context_hash: None,
            timestamp: None,
            rule: mock_rule.clone(),
            source_id: "file1".to_string(),
        },
        // Match 2: same hash as Match 1, will be deduplicated.
        RedactionMatch { 
            rule_name: "email".to_string(), 
            original_string: "".to_string(),
            sanitized_string: "".to_string(), 
            start: 100, 
            end: 110, 
            line_number: None, // Added missing field
            sample_hash: Some("hash_c".to_string()), 
            match_context_hash: None,
            timestamp: None,
            rule: mock_rule.clone(),
            source_id: "file1".to_string(),
        },
        // Match 3: No hash, unique coordinates.
        RedactionMatch { 
            rule_name: "email".to_string(), 
            original_string: "".to_string(), 
            sanitized_string: "".to_string(),
            start: 200, 
            end: 210, 
            line_number: None, // Added missing field
            sample_hash: None, 
            match_context_hash: None,
            timestamp: None,
            rule: mock_rule.clone(),
            source_id: "file2".to_string(),
        },
        // Match 4: A unique hash.
        RedactionMatch { 
            rule_name: "email".to_string(), 
            original_string: "".to_string(), 
            sanitized_string: "".to_string(),
            start: 300, 
            end: 310, 
            line_number: None, // Added missing field
            sample_hash: Some("hash_a".to_string()), 
            match_context_hash: None,
            timestamp: None,
            rule: mock_rule.clone(),
            source_id: "file3".to_string(),
        },
    ];

    let selected_samples = select_samples_for_rule(&matches, &run_seed, 3);
    
    // There should be three unique samples after deduplication.
    assert_eq!(selected_samples.len(), 3);
    
    // Check that the correct unique hashes and coordinates are present.
    let mut found_hashes = HashSet::new();
    let mut found_coords = HashSet::new();

    for sample in &selected_samples {
        if let Some(hash) = &sample.sample_hash {
            found_hashes.insert(hash.clone());
        } else {
            found_coords.insert((sample.source_id.clone(), sample.start, sample.end));
        }
    }

    assert_eq!(found_hashes.len(), 2);
    assert!(found_hashes.contains("hash_c"));
    assert!(found_hashes.contains("hash_a"));
    assert_eq!(found_coords.len(), 1);
    assert!(found_coords.contains(&("file2".to_string(), 200, 210)));

    Ok(())
}