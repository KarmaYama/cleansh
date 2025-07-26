# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.5] - 2025-07-26 — Phase 1: Refined Redaction, Stats Foundation & Rule Expansion

This release marks a significant leap forward for `cleansh`, introducing a **powerful new statistics mode for in-depth redaction analysis**. Alongside this flagship feature, we've implemented substantial improvements to **CLI usability, logging, and overall stability**, making `cleansh` even more robust and user-friendly.

### Added

* **Introducing Redaction Statistics Mode (`--stats-only`):**
    A brand-new mode specifically designed for **analyzing and summarizing redactions without modifying the input content or producing sanitized output**. This provides detailed counts of matched items per redaction rule, laying the groundwork for advanced analytics and deeper insights into your data.

* **Pro Feature: JSON Statistics Export (`--stats-json-file <FILE>`, `--export-json-to-stdout`):**
    Take your redaction analysis to the next level with programmatic access to statistics.
    * `--stats-json-file <FILE>`: **Export a comprehensive redaction summary**, including rule details and match occurrences, to a specified JSON file. Ideal for reporting and integration into other tools.
    * `--export-json-to-stdout`: **Output the full JSON redaction summary directly to standard output**, enabling seamless piping to other scripts or analysis tools.

* **Pro Feature: Sample Matches in Statistics (`--sample-matches <N>`):**
    **Enhance your redaction reports** by including up to `N` unique examples of both the original (unredacted) and sanitized (redacted) text for each matched rule. This provides **immediate context** for identified sensitive data, significantly improving the utility of your redaction reports.

* **Pro Feature: Fail-over Threshold (`--fail-over <X>`):**
    Integrate `cleansh` more robustly into your CI/CD pipelines. This crucial flag allows you to **specify a maximum number of secrets (`X`)**. If the total secrets detected exceed this threshold, `cleansh` will exit with a non-zero status code, signaling a potential security or compliance issue and preventing unintended deployments.

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

* **Suppressed Informational Output (`-q`, `--quiet`):**
    A new command-line flag to **suppress all informational messages**, displaying only warnings and errors. This is ideal for scripting, automated workflows, and producing cleaner, focused output.

* **Disable Donation Prompts (`--disable-donation-prompts`):**
    Added an option for users to **disable prompts for donations**, providing a more streamlined and uninterrupted experience for those who prefer not to see them.

* **Configurable Rule Sets (`--rules <NAME>`):**
    Introduced the ability to **specify different predefined rule configurations** (e.g., `'default'`, `'strict'`) if such sets are defined within the configuration system. This provides greater flexibility in applying specific redaction policies tailored to different needs.

* **ANSI Escape Stripping Layer:**
    * All input content is now **sanitized for ANSI escape codes** prior to applying redaction rules, to eliminate evasion via terminal formatting.
    * Adds resilience against malicious payloads disguised with ANSI codes.

* **Redaction Summary Enhancements**:
    * Output summaries now display **unique original and sanitized values** per rule.
    * Summary sorting and formatting is **deterministic** for CI and diff-based output validation.

* **Extensive Integration Tests**:
    * New integration tests validate ANSI-stripping effectiveness, clipboard output, rule opt-in and opt-out behaviors, redaction summary toggling, and edge cases like overlapping rules and invalid formats.

### Changed

* **Licensing Model:** **This version (`v0.1.5`) and all subsequent versions are now licensed under the PolyForm Noncommercial License 1.0.0.** Previous versions (`< v0.1.5`) remain under the MIT License. This change allows for greater control over commercial use for the project's sustainability.
* **Regex Patterns**
    * Added `\b` anchors or full start/end matches to all regex rules to reduce partial/substring false positives.
    * Restructured complex regexes for clarity, with new comments in YAML-based rule definitions.

* **Rule Management System**
    * Now respects `opt_in: true` and filters rules at runtime using `--enable-rules` and `--disable-rules`.
    * Unknown `--enable-rule` names are ignored with a debug warning, ensuring robust fail-safe behavior.

* **Unified Input Handling:** We've **streamlined the input mechanism** for clearer argument parsing. Content is now read exclusively from a specified file (`--input-file`) or standard input (stdin), removing ambiguity and making `cleansh`'s behavior more predictable.

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

* **Robust Logging System:** Consolidated all application logging to ensure **consistent log levels and proper adherence to verbosity settings** controlled by `--debug`, `--quiet`, and the `RUST_LOG` environment variable.

* **Logging Setup**
    * **Default log level is now `WARN`**, suppressing `INFO` messages by default for a quieter operation in automated scripts and for successful runs.
    * Refined the internal logging configuration for tests, ensuring debug messages are **conditionally logged only when explicitly enabled** for security, preventing accidental Personal Identifiable Information (PII) leakage into logs in non-development environments.

* **Enhanced Redaction Summary:** The redaction summary now provides **more granular insights**, including unique examples of original and sanitized texts for each rule. The `--no-redaction-summary` flag allows for suppressing this output when not needed.

* **Improved Diff View (`-d`, `--diff`):** Resolved various formatting and content discrepancies within the diff output. The diff view now **accurately and visually distinguishes changes** between original and sanitized content, with improved handling of redaction summary suppression for a cleaner comparison.

* **Clipboard Integration:** Refined clipboard copy operations for **smoother functionality**, particularly when used in conjunction with output file redirection or diff mode, enhancing the overall user experience.

* **Configuration Handling:** Improved the logic for **merging default and custom rule configurations**, and enhanced error reporting for issues encountered during theme file loading, making configuration more reliable.

* **Testing Infrastructure:** Implemented significant improvements across the integration test suite, particularly for CLI behavior, diff view, and quiet mode. This **rigorous testing has led to a higher confidence** in the application's stability and correctness, ensuring a robust release.

* **Code Quality & Maintainability:** Continued adherence to best practices, ensuring the codebase remains high-quality, scalable, and easy to maintain.

* **Build System**
    * Improved feature flag usage to modularize platform-specific and test-only behaviors.
    * Now compiles cleanly under various build scenarios.

* **Changelog Process**
    * This release formalizes adherence to [Keep a Changelog](https://keepachangelog.com) structure and detailed Markdown documentation for every behavioral change, feature, and security enhancement.

### Fixed

* Fixed an issue related to inconsistent `stderr` messages for redaction logs, **ensuring the `[REDACTED: X chars]` format is consistently applied** in debug output.
* Addressed a failure by ensuring that informational messages are **correctly suppressed when the `--quiet` flag is active**.
* Resolved potential conflicts in command-line argument parsing by **simplifying input method selection**, preventing unexpected behavior.

### CLI Quick Reference

| Flag | Purpose |
|:---|:---|
| `--stats-only` | Summary of matches without redaction |
| `--stats-json-file <FILE>` | Write stats JSON to file |
| `--export-json-to-stdout` | Dump stats JSON to stdout |
| `--sample-matches <N>` | Show N example matches per rule |
| `--fail-over <X>` | Exit non-zero if matches > X |
| `--enable-rules`, `--disable-rules` | Selectively enable or disable rules |
| `--quiet` / `-q` | Suppress informational logs |
| `--no-redaction-summary` | Suppress summary footer |
| `--rules <NAME>` | Use predefined rule set (`default`, `strict`, etc.) |
| `--disable-donation-prompts` | Turn off donation prompts |

---
---

*This release represents a major leap forward in `cleansh`'s accuracy, testability, and secure-by-default foundation. It lays the groundwork for future enhancements such as entropy-based token detection, contextual redaction, and Luhn checksum integration in Phase 2.*

---

## [0.1.2] - 2025-07-12 - Stability & Output Refinement

### Fixed

* **Resolved critical output formatting issues** ensuring the application's stdout behavior aligns perfectly with expectations.
* **Corrected an oversight in the application's output logic** where an "No redactions applied." message was incorrectly suppressed when using `--no-redaction-summary`. This message now correctly appears when no redactions occur and the summary is *not* suppressed.
* Eliminated an **unused variable warning** in test code to maintain a clean compilation.

### Changed

* Adjusted internal test expectations to precisely match the `cleansh` application's refined output behavior, particularly concerning newlines and summary messages.

---

## [0.1.1] - 2025-07-12 - Precision View

### Fixed

* Resolved a critical bug in the `--diff` view functionality that caused incorrect output formatting. The diff now accurately highlights line-by-line changes.

### Changed

* Upgraded the internal diff generation engine for more robust and visually appealing diff output.
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
* Layered configuration system supporting runtime settings and user-defined YAML rules for custom patterns.
* Robust logging and error handling infrastructure.
* Comprehensive unit and integration test suites to ensure reliability.
* Initial project structure, `README.md`, and MIT License.