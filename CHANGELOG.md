# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.5] Phase 1: Refined Default Redaction Rules And More - 2025/07/25

### Added

* **New redaction patterns** for emerging/expanded formats:
    * **GitHub PATs** (`ghp_…`)
    * **GitHub fine‑grained PATs** (`github_pat_…`, 72 chars)
    * **Stripe keys** (`sk_live_…`, `sk_test_…`, `rk_live_…`)
    * **Google OAuth tokens** (`ya29.…`, 20–120 chars)
    * **IPv6 addresses** (full uncompressed form, 8×1–4 hex digits)
    * **US Social Security Numbers (SSN)**
    * **UK National Insurance Numbers (NINO)**
    * **South African ID Numbers**
    * **Windows Absolute Paths** (`C:\…`, `\\Server\Share\…`)
    * **Slack Webhook URLs** (`https://hooks.slack.com/services/T...`)
    * **HTTP Basic Auth Headers** (`Authorization: Basic ...`)

* **New CLI Flags for Enhanced Control:**
    * `--no-redaction-summary`: Suppress the display of the redaction summary at the end of the output.
    * `--enable-rules`: Allow users to explicitly enable opt-in redaction rules by name (comma-separated).
    * `--disable-rules`: Allow users to explicitly disable any redaction rules by name (comma-separated).
    * `--quiet` (`-q`): Suppress informational output, showing only warnings and errors.

* **ANSI Escape Stripping Layer:**
    * All input content is now **sanitized for ANSI escape codes** prior to applying redaction rules, to eliminate evasion via terminal formatting.
    * Uses [`strip-ansi-escapes`](https://crates.io/crates/strip-ansi-escapes) to remove colors, cursor movements, and other terminal decorations.
    * Adds resilience against malicious payloads disguised with ANSI codes.

* **Test-Only Feature Flags Introduced**:
    * Internal-only `test-exposed` feature gate allows targeted unit test access to private module logic without exporting in production.
    * Adds support for `clipboard` integration via feature flag toggling to isolate platform-specific code during builds.

* **Redaction Summary Enhancements**:
    * Output summaries now display **unique original and sanitized values** per rule.
    * Summary sorting and formatting is **deterministic** for CI and diff-based output validation.

* **Extensive Integration Tests**:
    * New integration tests validate:
        * ANSI-stripping effectiveness (`test_sanitize_content_with_ansi_escapes`)
        * Clipboard output (`test_run_cleansh_clipboard_copy`)
        * Rule opt-in and opt-out behaviors
        * Redaction summary toggling
        * Edge cases like overlapping rules and invalid formats

### Changed

* **Regex Patterns**
    * Added `\b` anchors or full start/end matches to all regex rules to reduce partial/substring false positives.
    * Restructured complex regexes for clarity, with new comments in YAML-based rule definitions.

* **Rule Management System**
    * Now respects `opt_in: true` and filters rules at runtime using `--enable-rules` and `--disable-rules`.
    * Unknown `--enable-rule` names are ignored with a debug warning, ensuring robust fail-safe behavior.

* **Filesystem Path Rules**
    * **Windows** path redaction now uses clear anchors and broader detection of drive letters.
    * **Linux/macOS** rules refocused to redacting only home/user directories (`/home/`, `/Users/`).

* **Programmatic Validation Improvements**
    * **US SSN**:
        * Rejects invalid area codes (`000`, `666`, `9xx`)
        * Rejects group code `00` and serial `0000`
    * **UK NINO**:
        * Filters invalid prefixes (`BG`, `GB`, `NK`, etc.)
        * Enforces middle numeric and suffix character rules

### Improved

* **Security & Reliability**
    * Avoids regex constructs that could trigger excessive backtracking (no nested quantifiers, lookbehinds).
    * Pattern compilation size-limited to 10 MB per regex to avoid runtime slowdowns or ReDoS conditions.

* **Diagnostics**
    * Rich debug logs now include:
        * Which rule matched what text
        * Whether programmatic validation passed or failed
        * Summary statistics per rule

* **Logging Setup**
    * **Default log level is now `WARN`**, suppressing `INFO` messages by default for a quieter operation in automated scripts and for successful runs.
    * Refined the internal logging configuration for tests, ensuring debug messages are **conditionally logged only when explicitly enabled** via the `CLEANSH_ALLOW_DEBUG_PII` environment variable. This significantly enhances security by preventing accidental Personal Identifiable Information (PII) leakage into logs in non-development environments.
    * Integrated the `test-log` crate (formerly `test-env-log`) to provide a robust and idiomatic solution for managing `env_logger` initialization in test suites. This allows tests to precisely control their logging output and avoids interference from external `RUST_LOG` settings, leading to more reliable and deterministic test results.
    * Wrapped environment variable modifications (`std::env::set_var`, `std::env::remove_var`) in `unsafe` blocks within test code and doctests, acknowledging the potential global side effects and adhering to Rust's safety guidelines.

* **Code Maintainability**
    * Refactored `sanitize_shell.rs` to separate `CompiledRule` and `CompiledRules` structs.
    * Clearer variable naming (`stripped_input`, `summary_map`, `should_redact`) improves code readability.
    * Better test coverage over configuration edge cases and redaction behavior.

* **Build System**
    * Improved feature flag usage (`test-exposed`, `clipboard`) to modularize platform-specific and test-only behaviors.
    * Now compiles cleanly under all `cargo test` and `cargo build --no-default-features` scenarios.

* **Changelog Process**
    * This release formalizes adherence to [Keep a Changelog](https://keepachagelogs.com) structure and detailed Markdown documentation for every behavioral change, feature, and security enhancement.

---

*This release represents a major leap forward in `cleansh`'s accuracy, testability, and secure-by-default foundation. It lays the groundwork for future enhancements such as entropy-based token detection, contextual redaction, and Luhn checksum integration in Phase 2.*

---

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