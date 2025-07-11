# 🧭 cleansh – Full Scope & Enterprise Architecture Plan

> A high-trust, single-purpose CLI tool that sanitizes terminal output for safe sharing.
Secure by default. Zero config required. Extendable when needed.

---

## 1. ✅ Core Capabilities – MVP

### 🎯 Primary Goal:
Sanitize shell output piped via stdin (or loaded from a file), masking:

* Emails
* IP addresses
* Tokens, JWTs, AWS keys, GCP keys
* SSH keys and hex secrets
* Absolute paths (e.g., `/Users/alex/...`) → `~/...`

### 💡 Optional (with flags):
* Copy result to clipboard (`--clipboard`)
* Show diff view of redactions (`--diff`)
* Load custom redaction config (`--config config.yaml`)
* Output to file (`--out result.txt`)

---

## 2. 🧱 File Structure

```

cleansh/
├── src/
│   ├── main.rs                 \# CLI entrypoint, arg parsing
│   ├── commands/
│   │   └── cleansh.rs          \# Main CLI logic + config/flags
│   └── tools/
│       └── sanitize\_shell.rs   \# All regex, redaction, path normalization
├── config/
│   └── default\_rules.yaml      \# Embedded default rules (immutable)
├── .env                        \# Runtime config (log level, debug mode, etc.)
├── .gitignore
├── Cargo.toml
├── README.md
├── LICENSE (MIT)

````

---

## 3. ⚙ Configuration Strategy

### 1. Runtime Settings (from `.env`)
Loaded using `dotenv` or `dotenvy`. Keys:

* `LOG_LEVEL=info`
* `CLIPBOARD_ENABLED=true`
* `DEFAULT_CONFIG=./config/default_rules.yaml`

> Secure, minimal, easily overridable per deployment.

### 2. User Rule Config (Optional)
Supports a user-defined YAML via `--config`. Parsed with `serde_yaml`.

**Example:**
```yaml
rules:
  - name: email
    pattern: '[\w.+-]+@[\w-]+\.[\w.-]+'
    replace_with: '[email]'
  - name: ip
    pattern: '\b\d{1,3}(\.\d{1,3}){3}\b'
    replace_with: '[ip]'
````

-----

## 4\. 🧠 Sanitizer Tool Design (in `sanitize_shell.rs`)

### Internal Pipeline:

```
[ stdin or file input ]
          ↓
[ normalize paths (~) ]
          ↓
[ apply built-in regex rules ]
          ↓
[ apply optional user rules from YAML ]
          ↓
[ optionally copy to clipboard or output to file ]
```

### Engine Design:

  * Uses `regex::RegexSet` for efficient multi-pattern matching.
  * Immutable default rule-set embedded at compile time.
  * Optional merge with YAML rules.
  * Strip ANSI with `strip-ansi-escapes`.

-----

## 5\. 📊 Logging and Error Handling

### Logging:

  * Use `log` + `env_logger`
  * Levels: `trace`, `debug`, `info`, `warn`, `error`
  * Controlled via `.env` or CLI flag (`--debug`)

### Error handling:

  * Use `anyhow` for robust top-level error aggregation, `thiserror` for custom errors.
  * All sanitization failures or I/O errors should be:
      * Logged cleanly
      * Not fatal unless explicitly blocking behavior

-----

## 6\. 🧪 Testing and Validations

### Unit tests:

  * Regex pattern accuracy
  * Path normalization behavior
  * YAML parsing logic

### Integration tests:

  * Simulate stdin piping
  * Assert output match
  * Clipboard behavior (mocked)

-----

## 7\. 🚀 Packaging & Distribution

### 📦 Preferred Method: Prebuilt Cross-Platform Binaries via `cargo-dist`

**One-line install:**

```bash
curl -sSf [https://cleansh.sh/install.sh](https://cleansh.sh/install.sh) | sh
```

**Build:**

```bash
cargo install cargo-dist
cargo dist init
cargo dist build
```

**Supports:**

  * Windows (.exe)
  * macOS (arm64 + x86)
  * Linux (deb, rpm, tarball)
  * Homebrew tap (optional)
  * GitHub Releases auto-publish

> Alternative: `cargo install cleansh` (via crates.io) for Rust devs

-----

## 8\. 📜 Metadata & License

### Metadata (in `Cargo.toml`)

```toml
[package]
name = "cleansh"
version = "0.1.0"
edition = "2021"
description = "Sanitize your terminal output. One tool. One purpose."
license = "MIT"
repository = "[https://github.com/yourname/cleansh](https://github.com/yourname/cleansh)" # Update this to your repo URL
readme = "README.md"
categories = ["command-line-utilities", "security", "productivity"]
keywords = ["cli", "security", "redact", "sanitize", "clipboard"]
```

### License

  * Use MIT. Include `LICENSE` file with proper headers in source.

-----

## 9\. 🔐 Security by Default

| Feature                  | Security Principle                                |
| :----------------------- | :------------------------------------------------ |
| No runtime evals         | Everything static / regex-based                   |
| No external calls        | No HTTP/cloud dependencies                        |
| Immutable default rules  | Cannot be edited without recompile                |
| Path redaction built-in  | Prevents leaking personal filesystem details      |
| YAML sandboxed           | No execution, only declarative parsing            |
| Clipboard output opt-in  | Disabled by default, not silent                   |

-----

## 10\. 🛠 Future-Proofing (Post v1.0)

  * Plugin system: Load `/tools/*.rs` redactors dynamically
  * VSCode extension or web GUI
  * WebAssembly version for browser-based logs
  * Custom Git hook to sanitize commit messages or patch diffs
  * Subscription tier: auto-detect security tokens & dynamic secrets

-----

## 🧵 Summary

| Area            | Stack/Choice                       |
| :-------------- | :--------------------------------- |
| Language        | Rust                               |
| Config format   | .env + optional YAML               |
| CLI parsing     | `clap` with derives                |
| Regex engine    | `regex` crate                      |
| Clipboard       | `arboard`                        |
| Logging         | `log` + `env_logger`               |
| Error handling  | `anyhow` + `thiserror`             |
| Install method  | `cargo-dist` + curl script or `cargo install` |
| License         | MIT                                |

```

future updates


---

## Phased Development: Plugin System + New File Type

Let's break down how we can build this out logically, tackling the core challenges step by step. This keeps the project manageable while aiming for that powerful long-term vision:

### Phase 1: Core Plugin Infrastructure & Rule Extension

This phase focuses on establishing the foundation for plugins and expanding the types of redaction you can perform. This avoids the complexity of file parsing initially, letting you get the plugin mechanism right.

* **1A. Design the Plugin Interface:** Define how plugins will interact with `cleansh`. This means deciding:
    * **What data flows in and out of a plugin?** (e.g., text content, redaction rules, metadata).
    * **How are plugins loaded?** (e.g., dynamically loaded libraries, separate executables communicating via IPC). Rust has excellent support for dynamic linking, which could be a strong candidate.
    * **What capabilities can a plugin have?** (e.g., adding/modifying regex rules, custom redaction logic, specific output formatting).
* **1B. Implement Basic Rule Plugins:** Create a simple example plugin that just adds a new custom redaction rule (perhaps using the existing `regex` crate or a slightly more complex Rust-native regex capability). This validates your plugin loading and rule merging mechanisms.
* **1C. Develop Initial "Sanitized Document Format" (SDF) Definition:**
    * **Start Simple:** Begin by defining a **text-based, structured format** (like a YAML or JSON schema) that `cleansh` can output.
    * **Core Elements:** At a minimum, it should include the **sanitized content** and a **redaction summary/log** (what was redacted, where).
    * **Future-Proofing:** Think about placeholders for future metadata (original filename, timestamps, sanitization parameters).
    * **`cleansh`'s New Output Mode:** Modify `cleansh` to output this SDF instead of just plain text when a specific flag is used (e.g., `--format sdf`).

### Phase 2: Input/Output Transformation Plugins (Document Handling)

Once your core plugin system is solid and `cleansh` can output your new SDF, you can introduce plugins that handle different document types.

* **2A. Input Plugin Interface for Document Parsing:** Define an interface for plugins that take a specific document type (e.g., PDF file path) and output **plain text** that `cleansh` can then sanitize. This decouples the parsing logic from `cleansh`'s core.
* **2B. Implement a "PDF Text Extraction" Plugin:** This would be your first big challenge here. The plugin would use a Rust PDF parsing library (like `pdf-extract` or bindings to `poppler`/`pdfium`) to extract raw text from a PDF. It would then output this text, perhaps along with some basic structural markers, in a format `cleansh` expects.
* **2C. Output Plugin Interface for Document Reconstruction/Transformation:** Define an interface for plugins that take `cleansh`'s **SDF output** and transform it into another format. This could be:
    * Generating a *new*, redacted PDF from the SDF.
    * Converting the SDF into a sanitized Word document.
    * Exporting to a specialized database format.
* **2D. Implement an "SDF to Redacted PDF" Plugin (Optional, Advanced):** This would be the most complex, as it involves not just text extraction but potentially re-rendering a PDF with redactions. A simpler initial approach might be an "SDF to Redacted Markdown" or "SDF to Redacted TXT with Markers" plugin.

### Phase 3: Advanced Plugin Capabilities & External Regex Engines

With the foundation in place, you can explore more sophisticated integrations.

* **3A. Custom Redaction Logic Plugins:** Allow plugins to provide entirely custom sanitization logic that goes beyond simple regex (e.g., context-aware redaction, AI-driven anonymization).
* **3B. External Regex Engine Plugin:** If a specific use case truly demands it, design a plugin that allows `cleansh` to offload complex pattern matching to an external, more specialized regex engine (e.g., one optimized for very large inputs, or specific pattern types). This keeps `cleansh`'s core simple, as you desired.

---

