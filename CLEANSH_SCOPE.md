# 🧭 cleansh – Project Scope & Strategic Roadmap

> A high-trust, single-purpose CLI tool that securely sanitizes terminal output for safe sharing. Secure by default. Zero config required. Extendable when needed.

---

## 1. Current State (Achieved as of v0.1.2)

### 🎯 Core Capabilities:
`cleansh` currently provides robust sanitization of shell output (piped via stdin or loaded from a file), intelligently masking:

* **Emails**
* **IPv4 addresses**
* **Generic tokens, JWTs, AWS/GCP keys, SSH keys, and common hex secrets**
* **Absolute paths** (e.g., `/Users/alex/...`) which are normalized to `~/...`

### ✨ Key Features Implemented:
* **Clipboard Integration (`--clipboard` / `-c`):** Automatically copies sanitized output to the system clipboard.
* **Diff View (`--diff` / `-d`):** Displays a colored diff between original and redacted content, now with improved accuracy thanks to `diffy` (`0.1.1`).
* **Custom Redaction Config (`--config config.yaml`):** Allows loading user-defined YAML rules for extended pattern matching. These rules intelligently merge with built-in defaults.
* **Output to File (`--out result.txt` / `-o`):** Directs sanitized output to a specified file.
* **Runtime Configuration:** Utilizes `.env` for settings like `LOG_LEVEL` and `CLIPBOARD_ENABLED`.
* **Robust Error Handling & Logging:** Implemented with `anyhow`, `thiserror`, `log`, and `env_logger`.
* **Comprehensive Testing:** Includes unit tests for regex accuracy, path normalization, and YAML parsing, as well as integration tests for I/O and flag behavior (with recent fixes for output consistency in `0.1.2`).
* **Cross-Platform Distribution:** Packaged for easy installation via `cargo-dist` (curl script, prebuilt binaries for Windows, macOS, Linux) and `cargo install`.
* **MIT Licensed:** Open-source and privacy-first.
* **Secure by Design:** No runtime evaluations, no external network calls, immutable default rules, sandboxed YAML, and opt-in clipboard.

### 📈 Recent Milestones (from Changelog):
* **v0.1.2 (2025-07-12):** Focused on output stability and refinement, resolving critical formatting issues and ensuring correct summary message display.
* **v0.1.1 (2025-07-12):** Enhanced diff view accuracy by upgrading to `diffy`.
* **v0.1.0 (2025-07-12):** Initial public release with core sanitization capabilities and foundational CLI features.

---

## 2. Strategic Roadmap & Future Enhancements (Post v0.1.2)

This section outlines the progression of `cleansh` beyond its current robust state, focusing on adding more value, improving user experience, and expanding its reach.

### Phase 1: Enhanced User Control & Minor Enhancements (Target: v0.1.5 - v0.2.0)
* **1A. Refined Default Redaction Rules:**
    * (Under research) Improve existing regex patterns for accuracy and to reduce false positives.
    * **Basic Windows Path Redaction:** Add a default rule for `C:\Users\...` paths to `default_rules.yaml` to improve cross-platform utility.
* **1B. Improved Rule Management & Prioritization:**
    * **Explicit Rule Ordering/Prioritization:** Add a `priority` field to the `Rule` struct (e.g., `priority: 100`, where a lower number indicates higher priority). Rules will be sorted and applied by priority during the `compile_rules` step, ensuring more specific rules can run before broader ones.
    * **Rule Disabling:** Introduce an `enabled: bool` field to the `Rule` struct (e.g., `enabled: false` in YAML) to allow users to disable specific default or custom rules.
* **1C. Configurable Redaction Placeholders:**
    * Implement support for using `$` (capture group) variables within `replace_with` strings in custom YAML rules (e.g., `[EMAIL_DOMAIN:$2]`). This provides more context in the redacted output by allowing parts of the original match to be preserved or referenced.

### Phase 2: Advanced Redaction Strategies & Performance (Target: v0.2.x - v0.3.x)
* **2A. More Sophisticated Redaction Options:**
    * **Limited Character Masking:** Provide an option to mask only parts of a sensitive string (e.g., `user@example.com` becomes `u*****@e*****.com`), maintaining some original structure. This will require re-architecting `sanitize_content` to allow for custom replacement logic per rule.
    * **Group-Based Redaction:** If a rule matches multiple capture groups, allow specifying which groups to redact versus which to keep (e.g., in `username:password`, only redact `password`). This enhances precision and will also require custom replacement logic.
* **2B. Performance & Scalability:**
    * **Multi-step Async Processing for Large Inputs:** Introduce asynchronous processing (leveraging `tokio` for async I/O and `rayon` for parallel CPU-bound operations) to handle very large inputs by chunking and processing them concurrently, significantly improving performance.
    * **Performance Benchmarking Suite:** Establish and integrate a dedicated benchmarking suite to consistently track and optimize performance, especially for large inputs and complex rule sets.
* **2C. Refined Path Redaction:**
    * **User-Agnostic Path Masking:** Instead of only `~/...`, explore more generic masking like `~/<path-to-file>` that maintains more of the path structure while redacting sensitive user components.
    * **Relative Path Awareness:** Consider patterns for relative paths (e.g., `../.ssh/id_rsa`) that might expose sensitive directory structures.

### Phase 3: "Smarter" & More Precise Text Redaction (Target: v0.4.x - v0.5.x)
* **3A. Context-Sensitive Pattern Matching:**
    * Explore adding support for more complex `regex` crate features (e.g., advanced lookaheads/lookbehinds) that allow rules to define conditional matching (e.g., "redact X only if not preceded by Y"), moving towards a form of contextual awareness within patterns.
* **3B. Data Type Validation (Beyond Regex):**
    * Introduce a mechanism for a `Rule` to specify an optional, compiled-in Rust function that performs a check-digit algorithm or other basic syntactic validation on a matched string *before* redaction. This would significantly reduce false positives for specific data types (e.g., validating a credit card number match with Luhn algorithm).

### Phase 4: Architectural Flexibility & Ecosystem Expansion (Target: v1.0+)
* **4A. Core Plugin System:**
    * Design and implement a robust, secure, and stable plugin system (e.g., via a clear API/ABI). This system will allow external, dynamically loadable modules to extend `cleansh`'s functionality without altering its core. The plugin architecture is critical for maintaining the core's "high-trust" security principles while enabling broad extensibility.
* **4B. Advanced Output Formats & UI:**
    * **Structured Output (JSON/YAML):** Introduce a flag (e.g., `--output-format json`) to output the sanitized content and a detailed redaction summary/log in a machine-readable format.
    * **WebAssembly (WASM) Version & Browser Demo:** Compile `cleansh` to WASM to enable client-side, browser-based log sanitization, powering an online demo that runs the *actual* `cleansh` logic.
    * **Desktop GUI / VS Code Extension:** Explore building a lightweight graphical user interface or a VS Code extension for users who prefer visual interaction, potentially leveraging the WASM core or `tauri`/`electron`.
* **4C. Specialized Input/Output & Redaction Plugins (Demand-Driven):**
    * **Document Transformation & I/O Plugins:** Design interfaces for plugins that can handle specific document types.
        * **Input Plugins:** For parsing structured documents (e.g., PDF text extraction, Word document text extraction) into plain text or an intermediate format for `cleansh` to process.
        * **Output Plugins:** To take `cleansh`'s structured output and transform it back into a redacted document (e.g., a new redacted PDF, Markdown, or specialized database format).
    * **Structured Document Format (SDF) Redaction Plugin (DOCX & PDF):** This would be one or more highly specialized plugins built on the new architecture. It would handle the deep parsing, in-place modification, metadata sanitization, and preservation of integrity (including re-applying password mechanics, and handling digital signatures gracefully by invalidating/removing them) for formats like `.docx` and `.pdf`. **This is a massive undertaking, requiring substantial external libraries and would only be pursued with heavy user demand**, ensuring `cleansh`'s core remains focused and high-trust.
    * **External Regex Engine/Custom Redaction Logic Plugins:** Allow plugins to utilize alternative regex engines or custom, more complex (potentially AI-driven or context-aware) redaction logic for highly advanced use cases, operating within the plugin's defined trust scope.
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

---


---

### TODO: Add `--quiet` Flag Full Support and Tests

**Description:**
Implement a `--quiet` CLI flag in `cleansh` to suppress all logs, diff output, and redaction summaries for cleaner UX and script-friendly output.

**Current Status:**

* Basic `--quiet` flag added and hooked into main CLI parser and logging setup.
* Passed `quiet` flag into `run_cleansh` to conditionally suppress some output.

**Next Steps:**

* Update `run_cleansh` and related UI modules (`diff_viewer`, `redaction_summary`, `output_format`) to fully respect `quiet` mode.
* Add integration and unit tests covering both quiet and verbose modes to ensure consistent behavior and no regressions.
* Document the `--quiet` flag usage in README and help messages.

---

