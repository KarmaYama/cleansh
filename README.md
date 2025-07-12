# 🧭 Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml)
[![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml)

> A high-trust, single-purpose CLI tool that sanitizes terminal output for safe sharing. Secure by default. Zero config required. Extendable when needed. It is still in active development; while the latest **`v0.1.2`** release addresses critical output formatting and stability, we value your feedback, so please do report any issues you encounter.

---

## 📚 Table of Contents

* [✨ Overview](#-overview)
* [✅ Core Capabilities – Current Version (v0.1.2)](#-core-capabilities--current-version-v012---stability--output-refinement)
    * [🎯 Primary Redaction Categories:](#-primary-redaction-categories)
    * [💡 Optional Features (with flags):](#-optional-features-with-flags)
* [🚀 Usage Examples](#-usage-examples)
    * [Basic Sanitization (Piping from `stdin`)](#basic-sanitization-piping-from-stdin)
    * [Copying to Clipboard (`-c` or `--clipboard`)](#copying-to-clipboard--c-or---clipboard)
    * [Showing a Diff View (`-d` or `--diff`)](#showing-a-diff-view--d-or---diff)
    * [Loading Custom Redaction Rules (`--config <path>`)](#loading-custom-redaction-rules---config-path)
    * [Outputting to a File (`-o <path>`)](#outputting-to-a-file--o-path)
    * [Reading Input from a File](#reading-input-from-a-file)
    * [Combining Flags](#combining-flags)
* [⚠️ Known Issues](#-known-issues)
    * [1. Windows Absolute Path Redaction](#1-windows-absolute-path-redaction)
    * [2. Partial Custom Rule Interaction](#2-partial-custom-rule-interaction)
* [🧱 Project Structure](#-project-structure)
* [⚙ Configuration Strategy](#-configuration-strategy)
    * [1. Runtime Settings (from `.env`)](#1-runtime-settings-from-env)
    * [2. User Rule Configuration (Optional, via `--config`)](#2-user-rule-configuration-optional-via---config)
* [📋 Clipboard Support](#-clipboard-support)
* [🧠 Sanitizer Engine Design](#-sanitizer-engine-design-in-srctoolssanitize_shellrs)
    * [Internal Pipeline:](#internal-pipeline)
    * [Engine Architecture:](#engine-architecture)
* [📊 Logging and Error Handling](#-logging-and-error-handling)
    * [Logging:](#logging)
    * [Error Handling:](#error-handling)
* [🧪 Testing and Validations](#-testing-and-validations)
    * [Unit Tests:](#unit-tests)
    * [Integration Tests:](#integration-tests)
* [🚀 Packaging & Distribution](#-packaging--distribution)
    * [Installation](#installation)
    * [Building from Source](#building-from-source)
    * [Distribution Automation](#distribution-automation)
* [📜 Metadata & License](#-metadata--license)
    * [Metadata (in `Cargo.toml`)](#metadata-in-cargotoml)
    * [License](#license)
* [🔐 Security by Default Principles](#-security-by-default-principles)
* [🛠 Future-Proofing (Post v1.0 Aspirations)](#-future-proofing-post-v10-aspirations)
* [🧵 Summary of Technology Stack](#-summary-of-technology-stack)

---

## ✨ Overview

`cleansh` is a powerful and reliable command-line utility designed to help you quickly and securely redact sensitive information from your terminal output. Whether you're debugging, collaborating, or sharing logs, `cleansh` ensures that confidential data like IP addresses, email addresses, and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `cleansh` provides a robust, pre-configured solution for data sanitization, with flexible options for custom rules and output formats.

---

## ✅ Core Capabilities – Current Version (**v0.1.2**) - Stability & Output Refinement

This version of `cleansh` focuses on providing essential sanitization features with a strong emphasis on security and ease of use, building upon the "Precision View" of `v0.1.1`. Based on our rigorously passing test suite, you can trust `cleansh` to accurately mask the following sensitive data types and handle output reliably:

### 🎯 Primary Redaction Categories:

* **Emails:** Common email address formats (e.g., `user@example.com`).
* **IP Addresses:** Both IPv4 addresses (e.g., `192.168.1.1`).
* **Tokens & Secrets:** Generic tokens, JSON Web Tokens (JWTs), AWS keys, GCP keys, SSH keys, and common hex secrets.
* **Absolute Paths:** Currently, `cleansh` redacts Linux paths (e.g., `/home/user/documents/report.pdf` are transformed to `~/home/user/...`) and macOS paths (e.g., `/Users/admin/logs/app.log` are transformed to `~/Users/admin/...`).

### 💡 Optional Features (with flags):

`cleansh` provides command-line flags to customize its behavior, all thoroughly tested:

* **Copy to Clipboard:** Use `--clipboard` (`-c`) to automatically copy the sanitized output to your system's clipboard.
* **Show Diff View:** Use `--diff` (`-d`) to display a clear, **line-by-line colored diff** between the original and sanitized content, highlighting all redactions, powered by the `diffy` crate.
* **Load Custom Config:** Use `--config <path/to/config.yaml>` to apply your own custom redaction rules, which can augment or override the powerful built-in defaults.
* **Output to File:** Use `--out <path/to/result.txt>` to write the sanitized output directly to a specified file.

---

## 🚀 Usage Examples

`cleansh` is designed to integrate seamlessly into your command-line workflow. Here are some common ways to use it:

### Basic Sanitization (Piping from `stdin`)

Pipe the output of any command directly into `cleansh`. The sanitized content will be printed to your terminal.

**Example: Sanitizing a sensitive echo message**

```powershell
# On Windows (PowerShell)
echo "My email is test@example.com and my IP is 192.168.1.1." | cleansh

# On Linux/macOS (Bash/Zsh)
echo "My email is test@example.com and my IP is 192.168.1.1." | cleansh
````

**Example: Cleaning `docker logs` before sharing**

```bash
docker logs my-sensitive-container | cleansh
```

**Example: Sanitizing `kubectl logs` output**

```bash
kubectl logs my-pod-with-secrets | cleansh
```

### Copying to Clipboard (`-c` or `--clipboard`)

Sanitize output and instantly copy the result to your system's clipboard.

```bash
git config --list | cleansh -c
```

### Showing a Diff View (`-d` or `--diff`)

See exactly what `cleansh` changed with a clear, colored diff.

```bash
cat /var/log/app/errors.log | cleansh -d
```

### Loading Custom Redaction Rules (`--config <path>`)

Apply your own specific patterns for redaction by providing a path to a custom YAML configuration file.

```bash
cat my_sensitive_data.txt | cleansh --config /path/to/my_custom_rules.yaml
```

### Outputting to a File (`-o <path>`)

Instead of printing to `stdout`, save the sanitized content directly to a file.

```bash
my-script-with-secrets.sh | cleansh -o safe_output.log
```

### Reading Input from a File

You can also provide a file as input using standard shell redirection.

```bash
cleansh < raw_log_file.txt
```

### Combining Flags

Flags can be combined for powerful workflows. For example, sanitize, show diff, and save to a file:

```bash
my-command-output | cleansh -d -o sanitized_output.txt
```

-----

## ⚠️ Known Issues

While `cleansh v0.1.2` significantly improves stability and output, we are aware of a few areas that are currently being tracked for future improvements:

### 1\. Windows Absolute Path Redaction

  * **Severity:** Medium (Affects cross-platform consistency for a specific redaction category).
  * **Description:** `cleansh`'s built-in absolute path redaction currently supports Linux and macOS path formats (e.g., `/home/user/documents`). However, it does **not** currently identify or redact Windows absolute paths (e.g., `C:\Users\User\Documents\file.txt`).
  * **Impact:** Users on Windows environments may experience unexpected leakage of local absolute paths if they rely solely on the built-in path redaction.
  * **Workaround:** For immediate needs, users can define their own custom regex rules in their `config.yaml` to specifically target and redact Windows path patterns.

### 2\. Partial Custom Rule Interaction

  * **Severity:** Low (Does not prevent redaction, but may alter the intended custom placeholder).
  * **Description:** In some scenarios, a custom redaction rule designed to apply a specific placeholder might be partially or fully overridden by a more general, built-in "generic token" rule. This can result in the `[GENERIC_TOKEN_REDACTED]` placeholder appearing instead of the custom one (e.g., `EMP-12345` becoming `[GENERIC_TOKEN_REDACTED]-12345` instead of `[EMPLOYEE_ID_REDACTED]`).
  * **Impact:** While the sensitive data is always redacted, the desired custom placeholder for the entire matched string might not be used, potentially reducing clarity or adherence to specific reporting formats.
  * **Workaround:** When defining custom rules that are intended to override or be very specific, ensure their patterns are as precise as possible and carefully review their interaction with existing default rules, especially those for generic tokens.

-----

## 🧱 Project Structure

The `cleansh` codebase is thoughtfully organized for clarity, modularity, and maintainability, adhering to best practices for Rust projects.

```
cleansh/
├── src/
│   ├── main.rs                 # CLI entrypoint, argument parsing, high-level orchestration
│   ├── commands/
│   │   └── cleansh.rs          # Main CLI logic, handles command execution, config loading, and flag processing
│   ├── tools/
│   │   └── sanitize_shell.rs   # Core sanitization engine: contains all regex definitions, redaction logic, and path normalization
│   ├── config/                 # (New: Consider this as a module for config handling)
│   │   └── mod.rs              # Logic for loading default and user-defined rules
│   ├── ui/                     # (New: Consider this as a module for UI handling)
│   │   ├── mod.rs              # Public UI functions
│   │   ├── output_format.rs    # Handles all terminal output formatting (summaries, diffs, messages)
│   │   └── theme.rs            # Manages color themes and styling
│   └── tests/                  # Unit tests for individual components
├── config/
│   └── default_rules.yaml      # Embedded immutable default redaction rules
├── .env                        # Runtime configuration settings (local development)
├── .gitignore
├── Cargo.toml                  # Rust project manifest
├── README.md                   # This file
├── LICENSE (MIT)               # MIT License file
```

-----

## ⚙ Configuration Strategy

`cleansh` employs a layered configuration approach, prioritizing security and ease of use.

### 1\. Runtime Settings (from `.env`)

These settings control `cleansh`'s operational behavior and are loaded using `dotenvy` for flexible overrides per deployment environment.

**Example `.env` keys:**

  * `LOG_LEVEL=info` (Controls verbosity of internal logging)
  * `CLIPBOARD_ENABLED=true` (Enables or disables clipboard functionality globally)
  * `DEFAULT_CONFIG=./config/default_rules.yaml` (Specifies the path to default rules)

> This strategy ensures secure, minimal configuration that is easily overridable for different use cases.

### 2\. User Rule Configuration (Optional, via `--config`)

For advanced users, `cleansh` supports loading custom redaction rules from a YAML file specified via the `--config` flag. These rules are parsed with `serde_yaml` and intelligently merged with the built-in default rules.

**Example `custom_rules.yaml`:**

```yaml
rules:
  - name: my_company_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    description: "Redacts company employee IDs."
    multiline: false
    dot_matches_new_line: false
  - name: email # Overrides the default email rule
    pattern: '([a-z]+@[a-z]+\.org)' # Only matches .org emails
    replace_with: '[ORG_EMAIL_REDACTED]'
    multiline: false
    dot_matches_new_line: false
```

> As confirmed by our integration tests, custom rules can selectively override default rules, providing granular control over the sanitization process.

-----

## 📋 Clipboard Support

When using the `-c` / `--clipboard` flag, `cleansh` will copy sanitized output to your system clipboard.

### ✔️ Supported by Default:

  * **macOS**
  * **Windows**

### ⚠️ Linux Users:

Clipboard support requires one of the following utilities to be installed:

  * `xclip`
  * `xsel`
  * `wl-clipboard`

Without these, clipboard functionality may fail silently or print a warning.

> If you're running `cleansh` in a headless server or container, clipboard features will be disabled automatically.

-----

## 🧠 Sanitizer Engine Design (in `src/tools/sanitize_shell.rs`)

The heart of `cleansh` is its robust sanitization engine, designed for efficiency and precision.

### Internal Pipeline:

1.  **Input Acquisition:** Reads content from `stdin` or a specified file.
2.  **Path Normalization:** Transforms absolute paths (e.g., `/Users/alex/...`) into user-friendly tilde-prefixed paths (e.g., `~/...`).
3.  **Built-in Rule Application:** Applies a comprehensive set of immutable default regex rules, embedded directly at compile time for security and performance.
4.  **User Rule Application:** If provided, user-defined YAML rules are dynamically merged and applied, allowing for flexible customization.
5.  **Output & Interaction:** The sanitized content is then routed based on flags: printed to `stdout`, copied to the clipboard, written to a file, or presented in a diff view.

### Engine Architecture:

  * **Efficient Matching:** Utilizes the `regex::RegexSet` for highly optimized, simultaneous matching of multiple regular expressions against the input.
  * **Immutable Defaults:** The core redaction rule-set is compiled directly into the binary, preventing runtime tampering.
  * **ANSI Stripping:** Employs the `strip-ansi-escapes` crate to ensure that sensitive data hidden within ANSI escape codes (common in terminal output) is also properly identified and redacted.

-----

## 📊 Logging and Error Handling

`cleansh` is built with a focus on clear operational visibility and resilient error management.

### Logging:

Leverages the `log` crate with `env_logger` to provide detailed insights into its operation.

  * **Levels:** Supports `trace`, `debug`, `info`, `warn`, and `error` levels.
  * **Control:** Log levels can be configured via the `.env` file (`LOG_LEVEL`) or a dedicated CLI flag (e.g., `--debug` for `debug` level).

### Error Handling:

Implements robust error management using `anyhow` for top-level error aggregation and `thiserror` for defining structured, custom error types.

  * All sanitization failures, I/O errors, or configuration issues are:
      * Logged cleanly with relevant context.
      * Designed to be non-fatal where possible, allowing the tool to continue processing if an error is not explicitly blocking.

-----

## 🧪 Testing and Validations

A comprehensive testing strategy ensures the reliability and correctness of `cleansh`. Our recent test runs confirm all critical functionalities are working as expected.

### Unit Tests:

  * **Regex Pattern Accuracy:** Thoroughly validate that individual regex patterns correctly identify and redact specific sensitive data types.
  * **Path Normalization Behavior:** Ensures that absolute paths are accurately converted to their tilde-prefixed equivalents.
  * **YAML Parsing Logic:** Confirms the correct loading and interpretation of custom rule YAML files.

### Integration Tests:

  * **Simulated Stdin Piping:** Tests the primary mode of operation by piping various inputs to `cleansh` via `stdin`.
  * **Assert Output Match:** Verifies that the resulting sanitized output precisely matches expected strings for different input scenarios.
  * **Clipboard Behavior (Mocked):** Confirms that the clipboard functionality is correctly invoked and handles data as expected.
  * **File Output Validation:** Tests the `--out` flag, ensuring content is accurately written to the specified file.
  * **Diff View Accuracy:** Asserts that the diff output correctly highlights redactions and **line changes as per the `diffy` crate's output.**
  * **Custom Configuration Application:** Validates that `--config` files are loaded, merged, and correctly apply custom and overridden rules.
  * **No Redaction Scenario:** Ensures `cleansh` behaves gracefully and provides appropriate messages when no sensitive data is found.

-----

## 🚀 Packaging & Distribution

`cleansh` is designed for seamless cross-platform deployment and ease of installation, catering to both Rust developers and general users.

### Installation

#### 📦 Recommended: Prebuilt Cross-Platform Binaries via `cargo-dist`

The easiest way to get `cleansh` for most users is by using our pre-built binaries. These are automatically generated for various operating systems (Windows, macOS, Linux) when a new release is tagged.

**One-line Install (Linux/macOS):**

```bash
curl -sSf [https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh](https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh) | sh
```

#### For Rust Developers: Install from Crates.io

If you have the Rust toolchain installed, you can quickly install `cleansh` directly from [crates.io](https://crates.io/crates/cleansh):

```bash
cargo install cleansh
```

To **update** to the latest version, simply run:

```bash
cargo install cleansh --force
```

This is the recommended and most secure way to update for Rust developers, leveraging Cargo's robust package management.

#### Building from Source

To build `cleansh` from its source code, you'll need the Rust toolchain installed.

1.  **Clone the repository:**
    ```bash
    git clone [https://github.com/KarmaYama/cleansh.git](https://github.com/KarmaYama/cleansh.git)
    cd cleansh
    ```
2.  **Build in release mode (recommended for performance):**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/cleansh` (or `cleansh.exe` on Windows).
3.  **Run tests (optional, but recommended):**
    ```bash
    cargo test
    ```

### Distribution Automation

We leverage `cargo-dist` to streamline the release process.
After setting up `cargo-dist` (using `cargo dist init`), you can build distribution archives with:

```bash
cargo dist build
```

This generates packages ready for release to platforms like GitHub Releases.

-----

## 📜 Metadata & License

### Metadata (in `Cargo.toml`)

```toml
[package]
name = "cleansh"
version = "0.1.2"
edition = "2024"
license = "MIT"
repository = "[https://github.com/KarmaYama/cleansh](https://github.com/KarmaYama/cleansh)"
homepage = "[https://github.com/KarmaYama/cleansh](https://github.com/KarmaYama/cleansh)"
authors = ["Cleansh Technologies LLC"]
readme = "README.md"
crates.io = "[https://crates.io/crates/cleansh](https://crates.io/crates/cleansh)" 
rust-version = "1.88.0"
categories = ["command-line-utilities"]
keywords = ["cli", "security", "redact", "sanitize", "clipboard"]
description = "Sanitize your terminal output. One tool. One purpose.
```

### License

`cleansh` is open-source software distributed under the permissive **MIT License**. A full copy of the license is included in the `LICENSE` file within the source repository.

-----

## 🔐 Security by Default Principles

`cleansh` is engineered with a "secure by default" mindset, embodying several key security principles:

| Feature                   | Security Principle                                                                                                          |
| :------------------------ | :-------------------------------------------------------------------------------------------------------------------------- |
| No runtime evaluations    | All redaction logic is static and regex-based, preventing arbitrary code execution from external inputs.                      |
| No external network calls | `cleansh` operates entirely locally, with no HTTP/cloud dependencies or telemetry. Your data stays private.                 |
| Immutable default rules   | The core redaction rule-set is embedded at compile time and cannot be altered without recompilation, ensuring integrity.    |
| Path redaction built-in   | Automatically prevents the unintentional leakage of personal filesystem details by normalizing paths.                       |
| YAML sandboxed parsing    | User-defined YAML config files are strictly parsed for declarative rules; no execution capabilities are allowed.              |
| Clipboard output opt-in   | Copying to clipboard is an explicit opt-in action (`-c` flag), not a default, to prevent silent data transfer.              |

-----

## 🛠 Future-Proofing (Post v1.0 Aspirations)

As `cleansh` evolves, we envision expanding its utility and integration capabilities:

  * **Plugin System:** Develop a modular plugin architecture to allow dynamic loading of external redaction logic (e.g., from `/tools/*.rs`).
  * **Integrated Solutions:** Explore creating VS Code extensions or a lightweight web GUI for broader accessibility.
  * **WebAssembly (Wasm) Version:** Compile `cleansh` to WebAssembly for client-side, browser-based log sanitization.
  * **Custom Git Hooks:** Implement pre-commit or post-merge Git hooks to automatically sanitize commit messages or patch diffs before sharing.
  * **Advanced Redaction Tiers:** Investigate features like auto-detection of security tokens from cloud providers and dynamic secrets for enterprise use cases.

-----

## 🧵 Summary of Technology Stack

| Area              | Stack/Choice                                 |
| :---------------- | :------------------------------------------- |
| Language          | Rust                                         |
| Config Format     | `.env` + Optional YAML                       |
| CLI Parsing       | `clap` with derives                          |
| Regex Engine      | `regex` crate                                |
| ANSI Stripping    | `strip-ansi-escapes`                         |
| Diff Generation   | `diffy`                                      |
| Clipboard         | `arboard`                                    |
| Logging           | `log` + `env_logger`                         |
| Error Handling    | `anyhow` + `thiserror`                       |
| Installation      | `cargo-dist` + curl script / `cargo install` |
| License           | MIT                                          |

-----
