# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.5] - 2025-07-16 or earlier – Phase 1: Refined Default Redaction Rules

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

### Changed

* **Anchoring & Boundaries**
    * Added `\b` word‑boundaries or full anchors to *every* regex to eliminate partial / substring matches.
* **Contact Info**
    * **Email** – now supports uppercase, digits and up to 63‑char TLDs (`.[A-Za-z]{2,63}`) to cover modern gTLDs.
* **Network Identifiers**
    * **IPv4** – tightened per‑octet ranges to `0–255` with full validation.
    * **IPv6** – added uncompressed rule; description now notes compressed forms (`::`) are not covered in this phase.
* **Auth Tokens & Keys**
    * **JWT Token** - Description updated to use standard ASCII hyphen-minus (`-`).
    * **GitHub Access** – classic `ghp_…` rule remains; added `github_pat_…` for fine‑grained tokens.
    * **Stripe** – unified `sk_live_`, `sk_test_`, `rk_live_` under one rule (24 chars).
    * **AWS** – expanded prefixes to include both `AKIA` and `ASIA`; all keys now fully anchored.
    * **AWS Secret Access Key** – **Now opt-in only** due to high false positive risk from generic Base64 patterns.
    * **GCP** – `AIza…` rule unchanged but re‑anchored.
    * **Google OAuth** – length bound refined to 20–120 chars.
    * **SSH keys** – block pattern refined to include full BEGIN/END delimiters (`-----…-----`), uses `[\s\S]*?` for safe multiline matching.
* **Generic Secrets**
    * **32‑ and 64‑char hex** – exact‑length patterns, fully anchored. **Now opt-in only** due to high false positive risk from matching common hashes or IDs.
    * **Generic Token** – unchanged pattern, but description now flags as **opt‑in only** due to high false‑positive risk.
* **Identifiers & Financial**
    * **Credit Cards** – **Pattern significantly updated** to incorporate major BINs (Bank Identification Numbers) for enhanced precision, replacing the broad `13-16 digit` match; description now explicitly notes “no Luhn check.”
    * **US SSN / UK NINO** – remain highly precise, with hyphens or built‑in date/area exclusions.
    * **South African ID Numbers** – **Pattern refined** for more accurate format matching of YYMMDDSSSCCZ, including citizenship; no Luhn check.
* **Filesystem Paths**
    * **Linux/macOS** – refined to target common user home directories (`/home`, `/Users`) for sensitive path redaction.
    * **Windows** – unchanged but fully anchored, supports drive‑letter paths; description clarifies that UNC paths are not extensively covered by this rule.

### Improved

* **Regex Clarity & Maintainability**
    * Replaced greedy wildcards with specific quantifiers.
    * Modularized complex rules into comment‑annotated YAML entries.
    * Documented known limitations (e.g. full‑compression IPv6, no Luhn, specific Windows path types).
* **Performance & Security**
    * Ensured all patterns compile efficiently under Rust’s `regex` crate (RE2‑style, no backtracking pitfalls).
    * Introduced `opt_in: true` flag for high false-positive risk rules (AWS Secret Key, generic hex, generic token) to align with "secure by default" principle.
    * Avoided nested quantifiers or lookaround constructs that could risk ReDoS.
* **Future‑Proofing**
    * Prepared hooks for Phase 2: entropy thresholds, contextual analysis, Luhn–post‑processing.
    * Updated descriptions to guide opt‑in/opt‑out of broad or high‑risk rules.

---

*All notable changes for this release—building a robust “secure by default” foundation for cleansh’s evolving redaction engine.*

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