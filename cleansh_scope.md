# 🧭 cleansh – Project Scope & Strategic Roadmap

> A high-trust, single-purpose CLI tool that securely sanitizes terminal output for safe sharing. Secure by default. Zero config required. Extendable when needed.


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

This section outlines the progression of `cleansh` beyond its current robust state, focusing on adding more value, improving user experience, and expanding its reach. These align with your `Future-Proofing` section and `Phased Development` ideas.

### Phase 1: Enhanced Core Value & UX (Target: v0.2.x - v0.5.x)
* **1A. Expanded Built-in Redaction Rules:**
    * **New Sensitive Data Types:** Integrate default rules for common sensitive data not yet covered: credit card numbers, phone numbers, GUIDs, full URLs, and specific API keys (e.g., GitHub, Stripe).
    * **Advanced Regex Patterns:** Research and integrate more sophisticated regex patterns for higher accuracy and fewer false positives, potentially leveraging known patterns from security standards.
* **1B. Interactive & "Dry Run" Mode (`--dry-run`):**
    * Implement a flag that displays the sanitized output and summary *without* affecting files or clipboard, allowing users to safely preview redactions. This would be a crucial feature for trust and verification.
* **1C. Configurable Redaction Placeholders:**
    * Allow users to define custom replacement strings per rule (e.g., `[EMAIL_ADDRESS]`, `[AWS_TOKEN]`) in their YAML config, providing more context to redacted output.
* **1D. Structured Output (JSON/YAML):**
    * Introduce a flag (e.g., `--output-format json`) to output the sanitized content and a detailed redaction summary/log in a machine-readable format. This is crucial for pipeline integration and aligns with the "Sanitized Document Format" (SDF) idea.

### Phase 2: Foundation for Extensibility & Scalability (Target: v0.6.x - v0.9.x)
* **2A. Core Plugin System for Rules (Initial Architecture):**
    * **Design Plugin Interface:** Define clear, secure interfaces for external modules to contribute or modify redaction rules. Focus on static, compile-time loadable plugins initially to maintain security and performance.
    * **Proof-of-Concept Plugin:** Develop a simple example plugin (e.g., for a very specific, niche token type) to validate the loading and integration mechanism.
* **2B. Performance Benchmarking Suite:**
    * Establish and integrate a dedicated benchmarking suite to consistently track and optimize performance, especially for large inputs and complex rule sets. This is vital for **optimizing performance**.
* **2C. Modularity Refinement:**
    * Continuously refactor internal components (e.g., `commands`, `tools`) to enhance **modularity** and maintain strict **separation of concerns**, preparing for more complex integrations.

### Phase 3: Advanced Integrations & Ecosystem Expansion (Target: v1.0+)
* **3A. WebAssembly (WASM) Version & Browser Demo:**
    * Compile `cleansh` to WASM to enable client-side, browser-based log sanitization, powering a robust online demo that runs the *actual* `cleansh` logic. This directly supports the vision mentioned in the roadmap.
* **3B. External Regex Engine/Custom Redaction Logic Plugins:**
    * Explore allowing plugins to utilize alternative regex engines or custom, more complex (potentially AI-driven or context-aware) redaction logic for advanced use cases, while still safeguarding `cleansh`'s core security principles. This moves towards "Advanced Redaction Tiers."
* **3C. Document Transformation & Input/Output Plugins:**
    * **Input Plugins:** Develop interfaces and example plugins for parsing specific document types (e.g., PDF text extraction, Word document text extraction) into plain text for `cleansh` to process.
    * **Output Plugins:** Design interfaces for plugins that can take `cleansh`'s structured output (SDF) and transform it back into a redacted document (e.g., a new redacted PDF, Markdown, or specialized database format). This aligns with your "Sanitized Document Format" idea.
* **3D. Custom Git Hooks:**
    * Provide tools or guidance for integrating `cleansh` as a pre-commit or post-merge Git hook to automatically sanitize commit messages or patch diffs.
* **3E. Desktop GUI / VS Code Extension:**
    * Explore building a lightweight graphical user interface or a VS Code extension for users who prefer a visual interaction, potentially leveraging the WASM core or `tauri`/`electron` for a desktop app.

---

## 3. Core Technical Stack & Principles (Confirmed & Ongoing)

| Area                 | Stack/Choice                                  | Principle                                                                 |
| :------------------- | :-------------------------------------------- | :------------------------------------------------------------------------ |
| Language             | Rust                                          | Performance, memory safety, concurrency, robust CLI development.          |
| Config Format        | `.env` + optional YAML                        | Flexibility, user extensibility, clear separation of runtime vs. rules.   |
| CLI Parsing          | `clap` with derives                           | Ergonomic and powerful CLI interface.                                     |
| Regex Engine         | `regex` crate                                 | Fast, safe, and robust regex processing.                                  |
| Clipboard            | `arboard`                                     | Cross-platform clipboard integration.                                     |
| Logging              | `log` + `env_logger`                          | Flexible, environment-controlled logging for debugging and user feedback. |
| Error Handling       | `anyhow` + `thiserror`                        | Consistent and robust error management.                                   |
| Install Method       | `cargo-dist` + curl script / `cargo install`  | Broad accessibility for both Rust and non-Rust developers.                |
| License              | MIT                                           | Encourages adoption and contribution.                                     |
| Security Principles  | No runtime evals, no external calls, immutable defaults, YAML sandboxing, opt-in clipboard. | **Secure by Default**; high-trust.                                        |
| Modularity           | Structured `src` (commands, tools, ui), future plugin system. | Clean architecture, ease of maintenance and future expansion.             |
| Test Coverage        | Comprehensive unit and integration tests.     | Ensures reliability and prevents regressions.                             |

