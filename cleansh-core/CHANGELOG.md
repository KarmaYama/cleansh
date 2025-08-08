# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.2] - 2025-08-08 — Core Engine Refactoring, Engine Abstraction & Improved Rule Management

This release introduces a major architectural refactoring of the core sanitization engine. It abstracts the redaction logic behind a trait, enabling multiple backends and improving the application's extensibility. The update also streamlines CLI flags and centralizes key logic into dedicated helper functions, making the codebase more modular, maintainable, and robust. This version also refactors the programmatic validators for Social Security Numbers and UK National Insurance Numbers for improved clarity and accuracy.

### Added

* **Engine Abstraction (`SanitizationEngine` Trait):** A new trait, `SanitizationEngine`, has been introduced to define a common interface for different sanitization backends. The existing regex-based logic is now encapsulated in a concrete implementation, `RegexEngine`, which adheres to this trait.
* **New `engine.rs` Module:** A new module has been created to house the `SanitizationEngine` trait, the `RegexEngine` implementation, and the `SanitizationContext` helper struct. This modular design separates the core sanitization logic from the rule compilation process.
* **`SanitizationContext` Struct:** A new helper struct that centralizes the boilerplate logic for resolving overlapping matches, building the final sanitized string, and generating the summary.
* **`RedactionMatch` Position Data:** The `RedactionMatch` struct has been enhanced with two new fields, `start` and `end`, to record the byte indices of a match. This is crucial for accurately handling overlapping matches and reconstructing the sanitized string.
* **`RedactionConfig::set_active_rules` Method:** A new function that allows for explicit, programmatic control over which rules are active by accepting separate lists of rules to enable and disable. This replaces the less flexible "default" and "strict" configurations.
* **`HashSet` Dependency:** The `std::collections::HashSet` has been added to improve the efficiency of managing and checking rule names.

### Changed

* **Sanitization Logic Flow:** The core sanitization process has been refactored. The `sanitize_content` function has been removed and its logic is now part of the `SanitizationEngine::sanitize` method and the `SanitizationContext` struct. This moves the match aggregation and string reconstruction logic to a single, centralized location.
* **Rule Compilation (`compile_rules`):** The signature of `compile_rules` has been simplified to only accept `rules_to_compile`, as the filtering logic for enabling and disabling rules is now handled earlier in the `RedactionConfig` module.
* **Core API & `lib.rs`:** The top-level `lib.rs` now re-exports the new `SanitizationEngine` and `RegexEngine` types, which represent the new primary public API. The older, lower-level functions like `sanitize_content` are no longer publicly exposed.
* **Backreference Handling:** The `replace_all` closure within the regex matching logic has been updated to correctly process replacement strings containing backreferences (e.g., `$1`).
* **Logging:** Logging statements have been streamlined across the configuration module. Redundant log prefixes have been removed for cleaner output, and the final loaded rule count is now reported at a more appropriate `info!` level.
* **`RedactionMatch` Example:** An inaccuracy in the `redact_sensitive` function's documentation example has been corrected to reflect the actual character count of the example string.
* **SSN Validation Logic:** The programmatic validation for US Social Security Numbers (`is_valid_ssn_programmatically`) has been rewritten to be more readable and structured. The long, chained conditional checks were replaced with a series of distinct, easy-to-read checks for each validation criterion. The parsing of SSN parts now uses a `match` statement for safer, more explicit error handling.
* **UK NINO Validation Logic:** The programmatic validation for UK National Insurance Numbers (`is_valid_uk_nino_programmatically`) was refactored for clarity. The validation steps are now clearly separated and use more explicit methods like array lookups (`contains`) instead of macros where appropriate.

### Removed

* **`sanitize_content` Function:** The top-level `sanitize_content` function has been removed. Its core logic has been refactored and integrated into the new `SanitizationEngine` trait and `SanitizationContext` helper struct.
* **`RedactionConfig::set_active_rules_config` Method:** This method has been removed in favor of the more flexible and explicit `set_active_rules` method.

---

## [0.1.1] - 2025-08-03 — Fix: Backreference Handling & Enhanced Config Validation

This release addresses two key issues found during integration testing, ensuring the core sanitization engine correctly handles backreferences in replacement strings and gracefully recovers from malformed rule configurations.

### Fixed

* **Corrected Backreference Expansion:** The `replace_all` closure now correctly processes replacement strings with backreferences (e.g., `"$1"`). This ensures that rules like `absolute_linux_path` and `absolute_macos_path` can perform partial redactions, preserving non-sensitive portions of the matched text.

---

## [0.1.0] - 2025-07-31 — Initial Library Crate Release

This is the inaugural release of the `CleanSH-core` library crate. This version encapsulates the core logic of the `CleanSH` application, providing a robust and reusable engine for sensitive data redaction.

### Added

* **Core Sanitization Engine:** A dedicated, standalone library for identifying and redacting sensitive data.
* **Rule Management & Compilation:** Functionality to load, validate, and compile redaction rules from a configuration source into efficient regular expressions.
* **Programmatic Validation Hooks:** Support for advanced, non-regex validation checks (e.g., checksums) via the `programmatic_validation` flag in rules.
* **ANSI Escape Stripping:** A preprocessing layer to remove ANSI escape codes before pattern matching, ensuring reliable redaction.
* **Redaction Match Struct:** A structured format for reporting details of each redaction, including original and sanitized values.
* **Modular Design:** The crate is designed with a clear separation of concerns, making it easy to integrate into other applications or CLI tools.