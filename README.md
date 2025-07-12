# 🧭 Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml)
[![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/cleansh.svg)](https://crates.io/crates/cleansh)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml)
[![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql)

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

> **Sanitize your terminal output. One tool. One purpose.**

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

### Basic Sanitization (Piping from `stdin`)

```powershell
echo "My email is test@example.com and my IP is 192.168.1.1." | cleansh
````

### Cleaning `docker logs` before sharing

```bash
docker logs my-sensitive-container | cleansh
```

### Sanitizing `kubectl logs` output

```bash
kubectl logs my-pod-with-secrets | cleansh
```

### Copying to Clipboard (`-c` or `--clipboard`)

```bash
git config --list | cleansh -c
```

### Showing a Diff View (`-d` or `--diff`)

```bash
cat /var/log/app/errors.log | cleansh -d
```

### Loading Custom Redaction Rules (`--config <path>`)

```bash
cat my_sensitive_data.txt | cleansh --config ./custom_rules.yaml
```

### Outputting to a File (`-o <path>`)

```bash
my-script-with-secrets.sh | cleansh -o safe_output.log
```

### Reading Input from a File

```bash
cleansh < raw_log_file.txt
```

### Combining Flags

```bash
my-command-output | cleansh -d -o sanitized_output.txt
```

---

## ⚠️ Known Issues

### 1. Windows Absolute Path Redaction

* **Severity:** Medium
* **Impact:** Windows paths like `C:\Users\Alex\...` are not currently redacted.
* **Workaround:** Use a custom YAML rule.

### 2. Partial Custom Rule Interaction

* **Severity:** Low
* **Impact:** Custom rules may be partially overridden by built-in generic token rules.
* **Workaround:** Make custom patterns highly specific to avoid overlap.

---

## 🧱 Project Structure

```
cleansh/
├── src/
│   ├── main.rs
│   ├── commands/
│   ├── tools/
│   ├── config/
│   ├── ui/
│   └── tests/
├── config/
│   └── default_rules.yaml
├── .env
├── .gitignore
├── Cargo.toml
├── README.md
├── LICENSE (MIT)
```

---

## ⚙ Configuration Strategy

### 1. Runtime Settings (from `.env`)

**Example:**

```dotenv
LOG_LEVEL=info
CLIPBOARD_ENABLED=true
DEFAULT_CONFIG=./config/default_rules.yaml
```

### 2. User Rule Configuration (via `--config`)

```yaml
rules:
  - name: my_company_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    description: "Redacts company employee IDs."
    multiline: false
    dot_matches_new_line: false
```

---

## 📋 Clipboard Support

**macOS & Windows:** Fully supported.
**Linux:** Requires `xclip`, `xsel`, or `wl-clipboard`.

---

## 🧠 Sanitizer Engine Design (in `src/tools/sanitize_shell.rs`)

### Internal Pipeline:

1. Input Acquisition
2. Path Normalization
3. Built-in Rule Application
4. User Rule Application
5. Output (stdout, file, clipboard, or diff view)

### Engine Architecture:

* `regex::RegexSet`
* `strip-ansi-escapes`
* Compile-time embedded rules

---

## 📊 Logging and Error Handling

### Logging:

* Powered by `log` + `env_logger`
* Configurable via `.env` or `--debug`

### Error Handling:

* `anyhow` + `thiserror`
* Fail-safe, graceful recovery on non-blocking issues

---

## 🧪 Testing and Validations

### Unit Tests:

* Regex Accuracy
* Path Redaction
* YAML Parsing

### Integration Tests:

* Stdin Piping
* Clipboard Mocks
* File Output
* Diff Accuracy
* No Redaction Grace Case

---

## 🚀 Packaging & Distribution

### Installation

**Prebuilt Binaries:**
[GitHub Releases](https://github.com/KarmaYama/cleansh/releases)

**Recommended One-liner (Linux/macOS):**

```bash
curl -sSf https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh | sh
```

> ⚠️ Ensure the installer script adds `cleansh` to your `PATH`. This script currently assumes local install and does not require `sudo`.

**Rust Users (via [crates.io](https://crates.io/crates/cleansh)):**

```bash
cargo install cleansh
cargo install cleansh --force # to update
```

### Building from Source

```bash
git clone https://github.com/KarmaYama/cleansh.git
cd cleansh
cargo build --release
cargo test
```

### Distribution Automation

```bash
cargo dist build
```

---

## 📜 Metadata & License

### Metadata (in `Cargo.toml`)

```toml
[package]
name = "cleansh"
version = "0.1.2"
edition = "2024"
license = "MIT"
repository = "https://github.com/KarmaYama/cleansh"
homepage = "https://github.com/KarmaYama/cleansh"
authors = ["Cleansh Technologies LLC"]
readme = "README.md"
crates.io = "https://crates.io/crates/cleansh"
rust-version = "1.88.0"
categories = ["command-line-utilities"]
keywords = ["cli", "security", "redact", "sanitize", "clipboard"]
description = "Sanitize your terminal output. One tool. One purpose."
```

### License

This project is licensed under the [MIT License](LICENSE).

---

## 🔐 Security by Default Principles

| Feature            | Principle                                                |
| ------------------ | -------------------------------------------------------- |
| No runtime eval    | Regex-only logic — no execution of external inputs.      |
| Local-only         | No cloud or telemetry.                                   |
| Immutable defaults | Redaction rules are compiled in, not loaded dynamically. |
| Path redaction     | File system details are sanitized by default.            |
| YAML sandboxed     | Config is declarative only.                              |
| Clipboard opt-in   | Requires `-c` flag — avoids silent clipboard actions.    |

---

## 🛠 Future-Proofing (Post v1.0 Aspirations)

* Plugin system for redaction logic
* VS Code extension or web GUI
* WASM/browser version
* Git hook integration
* Advanced enterprise-focused redaction tiers

---

## 🧵 Summary of Technology Stack

| Area            | Stack/Choice                          |
| --------------- | ------------------------------------- |
| Language        | Rust                                  |
| Config Format   | `.env` + YAML                         |
| CLI Parsing     | `clap`                                |
| Regex Engine    | `regex`                               |
| ANSI Stripping  | `strip-ansi-escapes`                  |
| Diff Generation | `diffy`                               |
| Clipboard       | `arboard`                             |
| Logging         | `log`, `env_logger`                   |
| Error Handling  | `anyhow`, `thiserror`                 |
| Installation    | `cargo-dist`, `curl`, `cargo install` |
| License         | MIT                                   |

---

> 🚀 If you use `cleansh`, share your logs safely and confidently.
> 🎯 Precision redaction. Local-only trust. Built for devs.

```

