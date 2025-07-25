# 🧭 cleansh – Project Scope & Strategic Roadmap

> A high-trust, single-purpose CLI tool that securely sanitizes terminal output for safe sharing. Secure by default. Zero config required. Extendable when needed.

---

## 1. Current State (Achieved as of v0.1.5)

### 🎯 Core Capabilities:
`cleansh` currently provides robust sanitization of shell output (piped via stdin or loaded from a file), intelligently masking:

* **Emails**
* **IPv4 addresses**
* **Generic tokens, JWTs, AWS/GCP keys, SSH keys, and common hex secrets**
* **Absolute paths** (e.g., `/Users/alex/...`) which are normalized to `~/...`
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

### ✨ Key Features Implemented:
* **Clipboard Integration (`--clipboard` / `-c`):** Automatically copies sanitized output to the system clipboard.
* **Diff View (`--diff` / `-d`):** Displays a colored diff between original and redacted content, with improved accuracy.
* **Custom Redaction Config (`--config config.yaml`):** Allows loading user-defined YAML rules for extended pattern matching. These rules intelligently merge with built-in defaults.
* **Output to File (`--out result.txt` / `-o`):** Directs sanitized output to a specified file.
* **Runtime Configuration:** Supports configuration via environment variables for settings like logging verbosity and clipboard behavior.
* **Robust Error Handling & Logging:** Features comprehensive error management and flexible logging. The default log level is now `WARN`, providing a quieter experience by default. The `--quiet` (`-q`) flag is also fully supported.
* **Comprehensive Testing:** Includes extensive unit and integration tests for regex accuracy, path normalization, YAML parsing, I/O, flag behavior, and core redaction logic, ensuring high reliability.
* **Cross-Platform Distribution:** Packaged for easy installation via `cargo-dist` (curl script, prebuilt binaries for Windows, macOS, Linux) and `cargo install`.
* **MIT Licensed:** Open-source and privacy-first.
* **Secure by Design:** No runtime evaluations, no external network calls, immutable default rules, sandboxed YAML, and opt-in clipboard.
* **ANSI Escape Stripping Layer:** All input content is **sanitized for ANSI escape codes** prior to applying redaction rules, preventing evasion via terminal formatting.

### 📈 Recent Milestones (from Changelog):
* **v0.1.5 (2025-07-25 – Or earlier):** **Phase 1 Complete.** Includes refined default redaction rules, new CLI flags for enhanced control (`--no-redaction-summary`, `--enable-rules`, `--disable-rules`, `--quiet`), ANSI escape stripping, enhanced redaction summaries, extensive integration tests, and a default `WARN` logging level.
* **v0.1.2 (2025-07-12):** Focused on output stability and refinement, resolving critical formatting issues and ensuring correct summary message display.
* **v0.1.1 (2025-07-12):** Enhanced diff view accuracy.
* **v0.1.0 (2025-07-12):** Initial public release with core sanitization capabilities and foundational CLI features.

---

## 2. Strategic Roadmap & Future Enhancements (Post v0.1.5)

This section outlines the progression of `cleansh` beyond its current robust state, focusing on adding more value, improving user experience, and expanding its reach.

### Phase 1: Decoupling & Core Stabilization (Target: v0.1.6)

The `0.1.6` release of `cleansh` will primarily focus on an internal architectural refactoring: splitting the current monolithic project into a core library and a command-line interface. This strategic investment will establish a more modular, maintainable, and scalable foundation, aligning with our development philosophy of high-quality, reusable code.

**Expected Outcome for 0.1.6:** Users will continue to install and use `cleansh` with no change to external functionality or command-line experience. Internally, the project will gain a clean separation of core sanitization logic from CLI components, making the codebase significantly easier to maintain, test, and extend for future ambitious features planned for `cleansh` 1.0.0.

### Phase 2: Advanced Redaction Strategies & Performance (Target: v0.2.x - v0.3.x)
* **2A. More Sophisticated Redaction Options:**
    * **Limited Character Masking:** Provide an option to mask only parts of a sensitive string (e.g., `user@example.com` becomes `u*****@e*****m`), maintaining some original structure.
    * **Group-Based Redaction:** Allow specifying which parts of a matched pattern to redact versus which to keep (e.g., in `username:password`, only redact `password`), enhancing precision.
* **2B. Performance & Scalability:**
    * **Optimized Processing for Large Inputs:** Introduce efficient processing techniques to handle very large inputs by chunking and processing them concurrently, significantly improving performance.
    * **Performance Benchmarking:** Establish a dedicated benchmarking suite to consistently track and optimize performance across various scenarios.
* **2C. Refined Path Redaction:**
    * **User-Agnostic Path Masking:** Explore more generic masking (e.g., `~/<path-to-file>`) that maintains more of the path structure while redacting sensitive user components.
    * **Relative Path Awareness:** Consider patterns for relative paths (e.g., `../.ssh/id_rsa`) that might expose sensitive directory structures.

### Phase 3: "Smarter" & More Precise Text Redaction (Target: v0.4.x - v0.5.x)
* **3A. Context-Sensitive Pattern Matching:**
    * Explore adding support for more complex pattern matching capabilities that allow rules to define conditional matching (e.g., "redact X only if not preceded by Y"), moving towards a form of contextual awareness within patterns.
* **3B. Data Type Validation (Beyond Regex):**
    * Introduce a mechanism for rules to specify an optional validation function that performs a check-digit algorithm or other basic syntactic validation on a matched string *before* redaction. This would significantly reduce false positives for specific data types (e.g., validating a credit card number match).

### Phase 4: Architectural Flexibility & Ecosystem Expansion (Target: v1.0+)
* **4A. Core Plugin System:**
    * Design and implement a robust, secure, and stable plugin system. This system will allow external, dynamically loadable modules to extend `cleansh`'s functionality without altering its core, maintaining the core's "high-trust" security principles while enabling broad extensibility.
* **4B. Advanced Output Formats & UI:**
    * **Structured Output (JSON/YAML):** Introduce a flag (e.g., `--output-format json`) to output the sanitized content and a detailed redaction summary/log in a machine-readable format.
    * **Web-Based & Desktop Interfaces:** Explore building lightweight graphical user interfaces or web-based versions of `cleansh` for users who prefer visual interaction, leveraging the core logic.
* **4C. Specialized Input/Output & Redaction:**
    * Investigate providing extensible support for sanitizing content within structured document formats, such as extracting text for processing and then re-integrating redacted content back into the original document format where feasible. This would enable `cleansh` to extend its capabilities beyond plain text.
* **4D. Custom Git Hooks:**
    * Provide tools or guidance for integrating `cleansh` as a pre-commit or post-merge Git hook to automatically sanitize commit messages or patch diffs.

---

## 3. Core Technical Stack & Principles (Confirmed & Ongoing)

| Area | Stack/Choice | Principle |
| :--- | :--- | :--- |
| Language | Rust | Performance, memory safety, concurrency, robust CLI development. |
| Config Format | `.env` + optional YAML | Flexibility, user extensibility, clear separation of runtime vs. rules. |
| CLI Parsing | `clap` with derives | Ergonomic and powerful CLI interface. |
| Regex Engine | `regex` crate | Fast, safe, and robust regex processing. |
| Clipboard | `arboard` | Cross-platform clipboard integration. |
| Logging | `log` + `env_logger` | Flexible, environment-controlled logging for debugging and user feedback. |
| Error Handling | `anyhow` + `thiserror` | Consistent and robust error management. |
| Install Method | `cargo-dist` + curl script / `cargo install` | Broad accessibility for both Rust and non-Rust developers. |
| License | MIT | Encourages adoption and contribution. |
| Security Principles | No runtime evals, no external calls, immutable defaults, YAML sandboxing, opt-in clipboard. | **Secure by Default**; high-trust. |
| Modularity | Structured `src` (commands, tools, ui), future plugin system. | Clean architecture, ease of maintenance and future expansion. |
| Test Coverage | Comprehensive unit and integration tests. | Ensures reliability and prevents regressions. |