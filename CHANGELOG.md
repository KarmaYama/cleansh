# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-07-12 - Stability & Output Refinement

### Fixed

* **Resolved critical output formatting issues** in integration tests (`test_basic_sanitization` and `test_no_redactions`) ensuring the application's stdout behavior aligns perfectly with expectations.
* **Corrected an oversight in the application's output logic** (`src/commands/cleansh.rs`) where an "No redactions applied." message was incorrectly suppressed when using `--no-redaction-summary`. This message now correctly appears when no redactions occur and the summary is *not* suppressed.
* Eliminated an **unused variable warning** in `test_clipboard_output` in `tests/cleansh_integration.rs` to maintain a clean compilation.

### Changed

* Adjusted internal test expectations within `tests/cleansh_integration.rs` to precisely match the `cleansh` application's refined output behavior, particularly concerning newlines and summary messages.

---

## [0.1.1] - 2025-07-12 - Precision View

### Fixed

* Resolved a critical bug in the `--diff` view functionality that caused incorrect output formatting. The diff now accurately highlights line-by-line changes.

### Changed

* Upgraded the internal diff generation engine from `dissimilar` to `diffy` for more robust and visually appealing diff output.
* Updated `README.md` to reflect `v0.1.1` as the current version and include enhanced instructions for installation and updating via `cargo install --force`.
* Updated `Cargo.toml` version to `0.1.1`.

### Improved

* Enhanced the clarity and accuracy of the diff output when using the `--diff` flag.

---

## [0.1.0] - 2025-07-12 - Initial Public Release (Pre-release)

### Added

* Initial core sanitization capabilities for common sensitive data types: emails, IPv4 addresses, generic tokens, JWTs, AWS/GCP keys, SSH keys, and hex secrets.
* Absolute path redaction and normalization (Linux/macOS).
* Core CLI flags: `--clipboard (-c)`, `--diff (-d)`, `--config`, `--out`.
* Layered configuration system supporting `.env` runtime settings and user-defined YAML rules for custom patterns.
* Robust logging and error handling infrastructure using `log`, `env_logger`, `anyhow`, and `thiserror`.
* Comprehensive unit and integration test suites to ensure reliability.
* Initial project structure, `README.md`, and MIT License.