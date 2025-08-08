// cleansh-workspace/cleansh-core/src/validators.rs
//! Programmatic validation functions for specific sensitive data types.
//!
//! This module provides additional validation logic beyond regular expression matching
//! for sensitive information such as SSN and UK NINO. These functions help reduce
//! false positives by applying structural and known invalid pattern checks.
//! License: BUSL-1.1

/// Helper function to validate SSN based on US Social Security Administration rules.
///
/// This implementation aims for a robust programmatic check without external data.
/// It validates the structural components against known invalid patterns.
///
/// A real-world, fully compliant SSN validation might require access to historical
/// issuance patterns or verification services, which is beyond the scope of a
/// purely programmatic string validation.
///
/// # Arguments
///
/// * `ssn` - The SSN string slice to validate. Expected format "XXX-XX-XXXX".
///
/// # Returns
///
/// `true` if the SSN passes basic structural and invalid pattern checks, `false` otherwise.
pub fn is_valid_ssn_programmatically(ssn: &str) -> bool {
    // SSN parts must be exactly 3-2-4 digits.
    let parts: Vec<&str> = ssn.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    
    let area = parts[0];
    let group = parts[1];
    let serial = parts[2];

    // Ensure all parts are numeric and have the correct lengths.
    if area.len() != 3 || !area.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if group.len() != 2 || !group.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if serial.len() != 4 || !serial.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Safely parse parts into numbers for validation.
    let (area_num, group_num, serial_num) = match (area.parse::<u16>(), group.parse::<u8>(), serial.parse::<u16>()) {
        (Ok(a), Ok(g), Ok(s)) => (a, g, s),
        _ => return false, // This case should be unreachable due to previous checks, but serves as a safeguard.
    };

    // Apply specific validation rules for each part.
    // Invalid Area Numbers
    if area_num == 0 || area_num == 666 || (area_num >= 900) || (area_num >= 700 && area_num <= 729) {
        return false;
    }
    
    // Invalid Group and Serial Numbers
    if group_num == 0 || serial_num == 0 {
        return false;
    }

    // All checks passed.
    true
}

/// Helper function to validate UK National Insurance Number (NINO) based on HMRC rules.
///
/// This implementation aims for a robust programmatic check without external data.
/// It validates the structural components against known invalid patterns and characters.
///
/// # Arguments
///
/// * `nino` - The NINO string slice to validate. Expected format "AA######A" (where # are digits).
///
/// # Returns
///
/// `true` if the NINO passes basic structural and invalid pattern checks, `false` otherwise.
pub fn is_valid_uk_nino_programmatically(nino: &str) -> bool {
    // 1. Normalize and perform initial length check.
    const NINO_LENGTH: usize = 9;
    let nino_normalized: String = nino
        .to_uppercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    if nino_normalized.len() != NINO_LENGTH {
        return false;
    }

    let chars: Vec<char> = nino_normalized.chars().collect();
    let prefix = &nino_normalized[0..2];

    // 2. Validate format: AA######A (2 letters, 6 digits, 1 letter).
    if !chars[0].is_ascii_alphabetic() || !chars[1].is_ascii_alphabetic() {
        return false;
    }
    if !chars[2..8].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !chars[8].is_ascii_alphabetic() {
        return false;
    }

    // 3. Check for known invalid prefixes.
    const SPECIFIC_INVALID_PREFIXES: [&str; 11] = [
        "BF", "BG", "EH", "GB", "JE", "NK", "KN", "LI", "NT", "TN", "ZZ"
    ];
    if SPECIFIC_INVALID_PREFIXES.contains(&prefix) {
        return false;
    }

    // 4. Check for invalid characters in the first two positions.
    const INVALID_PREFIX_CHARS: [char; 7] = ['D', 'F', 'I', 'Q', 'U', 'V', 'O'];
    if INVALID_PREFIX_CHARS.contains(&chars[0]) || INVALID_PREFIX_CHARS.contains(&chars[1]) {
        return false;
    }

    // 5. Check for invalid suffix characters.
    const VALID_SUFFIX_CHARS: [char; 4] = ['A', 'B', 'C', 'D'];
    if !VALID_SUFFIX_CHARS.contains(&chars[8]) {
        return false;
    }

    // All checks passed.
    true
}