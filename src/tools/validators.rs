// src/tools/validators.rs

/// Helper function to validate SSN based on US Social Security Administration rules.
/// This implementation aims for a robust programmatic check without external data.
/// It validates the structural components against known invalid patterns.
///
/// A real-world, fully compliant SSN validation might require access to historical
/// issuance patterns or verification services, which is beyond the scope of a
/// purely programmatic string validation.
pub fn is_valid_ssn_programmatically(ssn: &str) -> bool {
    let parts: Vec<&str> = ssn.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let area = parts[0];
    let group = parts[1];
    let serial = parts[2];

    // All parts must be purely numeric and have correct lengths
    if area.len() != 3 || !area.chars().all(|c| c.is_ascii_digit()) ||
       group.len() != 2 || !group.chars().all(|c| c.is_ascii_digit()) ||
       serial.len() != 4 || !serial.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }

    let area_num = area.parse::<u16>().unwrap_or(0);
    let group_num = group.parse::<u8>().unwrap_or(0);
    let serial_num = serial.parse::<u16>().unwrap_or(0);

    // Area Number (first 3 digits) - Comprehensive Invalid Ranges
    // - 000 is invalid
    // - 666 is invalid
    // - 900-999 are invalid (assigned after 1972)
    // - Ranges not yet assigned (e.g., 700-729 are not yet issued for SSN)
    //   This list is not exhaustive and can change, but covers common known invalidities.
    if area_num == 0 || area_num == 666 || (area_num >= 900 && area_num <= 999) ||
       (area_num >= 700 && area_num <= 729) // As of current rules, these blocks are not issued.
    {
        return false;
    }

    // Group Number (middle 2 digits) - Comprehensive Invalid Ranges
    // - 00 is invalid
    // - Should generally not be 00-09 if the area number is 001-222
    //   For a general programmatic check, ensuring it's not 00 is critical.
    if group_num == 0 {
        return false;
    }

    // Serial Number (last 4 digits) - Comprehensive Invalid Ranges
    // - 0000 is invalid
    // - For some historical groups, 0000-0999 might be invalid in combination with group.
    //   For a general programmatic check, ensuring it's not 0000 is critical.
    if serial_num == 0 {
        return false;
    }

    true
}

/// Helper function to validate UK National Insurance Number (NINO) based on HMRC rules.
/// This implementation aims for a robust programmatic check without external data.
/// It validates the structural components against known invalid patterns and characters.
pub fn is_valid_uk_nino_programmatically(nino: &str) -> bool {
    let nino = nino.to_uppercase().replace(" ", ""); // Normalize input: remove spaces, uppercase

    // 1. Basic length check (must be exactly 9 characters)
    if nino.len() != 9 {
        return false;
    }

    // 2. Format check: AA######A (2 letters, 6 digits, 1 letter)
    let chars: Vec<char> = nino.chars().collect();

    // First two characters must be letters
    if !chars[0].is_ascii_alphabetic() || !chars[1].is_ascii_alphabetic() {
        return false;
    }

    // Characters 3 to 8 (index 2 to 7) must be digits
    for i in 2..8 {
        if !chars[i].is_ascii_digit() {
            return false;
        }
    }

    // Last character (index 8) must be a letter
    if !chars[8].is_ascii_alphabetic() {
        return false;
    }

    let prefix = &nino[0..2];
    let suffix = chars[8];

    // 3. Check against known invalid prefixes (first two characters)
    // This list covers common explicit exclusions by HMRC.
    let specific_invalid_prefixes = [
        "BF", "BG", "EH", "GB", "JE", "NK", "KN", "LI", "NT", "TN", "ZZ",
        // These are also often cited as invalid for NINO prefixes.
        // It's important to note that NINO formats are strict.
    ];
    if specific_invalid_prefixes.contains(&prefix) {
        return false;
    }

    // 4. Check for invalid characters in the first two positions
    // HMRC states that the first two letters cannot use D, F, I, Q, U, V, O.
    // The previous implementation already covered this, but clarifying it's for *either* position.
    let invalid_prefix_chars = ['D', 'F', 'I', 'Q', 'U', 'V', 'O'];
    if invalid_prefix_chars.contains(&chars[0]) || invalid_prefix_chars.contains(&chars[1]) {
        return false;
    }

    // 5. Special prefix rules (e.g., 'O' cannot be the second letter if the first is 'A' or 'B', etc.)
    // The previous `invalid_prefix_chars` check already handles if 'O' is in *either* position,
    // so `AO` and `BO` would be caught. This is generally sufficient unless a more nuanced rule
    // explicitly allows 'O' in first position but not second, which is not the common NINO rule.

    // 6. Suffix check (last character)
    // The suffix must be one of A, B, C, D
    if !matches!(suffix, 'A' | 'B' | 'C' | 'D') {
        return false;
    }

    true
}