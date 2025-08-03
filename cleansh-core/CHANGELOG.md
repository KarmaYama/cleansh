# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.1] - 2025-08-03 — Fix: Backreference Handling & Enhanced Config Validation

This release addresses two key issues found during integration testing, ensuring the core sanitization engine correctly handles backreferences in replacement strings and gracefully recovers from malformed rule configurations.

### Fixed

* **Corrected Backreference Expansion:** The `replace_all` closure now correctly processes replacement strings with backreferences (e.g., `"$1"`). This ensures that rules like `absolute_linux_path` and `absolute_macos_path` can perform partial redactions, preserving non-sensitive portions of the matched text.

---

## [0.1.0] - 2025-07-31 — Initial Library Crate Release

This is the inaugural release of the `cleansh-core` library crate. This version encapsulates the core logic of the `cleansh` application, providing a robust and reusable engine for sensitive data redaction.

### Added

* **Core Sanitization Engine:** A dedicated, standalone library for identifying and redacting sensitive data.
* **Rule Management & Compilation:** Functionality to load, validate, and compile redaction rules from a configuration source into efficient regular expressions.
* **Programmatic Validation Hooks:** Support for advanced, non-regex validation checks (e.g., checksums) via the `programmatic_validation` flag in rules.
* **ANSI Escape Stripping:** A preprocessing layer to remove ANSI escape codes before pattern matching, ensuring reliable redaction.
* **Redaction Match Struct:** A structured format for reporting details of each redaction, including original and sanitized values.
* **Modular Design:** The crate is designed with a clear separation of concerns, making it easy to integrate into other applications or CLI tools.