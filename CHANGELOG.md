# Changelog
# Guide: "Fixed", "Changed", "Removed", "Deprecated", or "Security"
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-07-12 - Precision View

### Fixed

* Resolved a critical bug in the `--diff` view functionality that caused incorrect output formatting. The diff now accurately highlights line-by-line changes.

### Changed

* Upgraded the internal diff generation engine from `dissimilar` to `diffy` for more robust and visually appealing diff output.
* Updated `README.md` to reflect `v0.1.1` as the current version and include enhanced instructions for installation and updating via `cargo install --force`.
* Updated `Cargo.toml` version to `0.1.1`.

### Improved

* Enhanced the clarity and accuracy of the diff output when using the `--diff` flag.

## [0.1.0] - 2025-07-12 - Initial Public Release (Pre-release)

### Added

* Initial core sanitization capabilities for common sensitive data types: emails, IPv4 addresses, generic tokens, JWTs, AWS/GCP keys, SSH keys, and hex secrets.
* Absolute path redaction and normalization (Linux/macOS).
* Core CLI flags: `--clipboard (-c)`, `--diff (-d)`, `--config`, `--out`.
* Layered configuration system supporting `.env` runtime settings and user-defined YAML rules for custom patterns.
* Robust logging and error handling infrastructure using `log`, `env_logger`, `anyhow`, and `thiserror`.
* Comprehensive unit and integration test suites to ensure reliability.
* Initial project structure, `README.md`, and MIT License.