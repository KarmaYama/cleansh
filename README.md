<!-- cargo-rdme -->

# Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml)
[![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/cleansh.svg)](https://crates.io/crates/cleansh)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml)
[![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql)

> Cleansh is a high-trust, single-purpose CLI tool designed to sanitize terminal output for safe sharing. It prioritizes security by default, requires zero configuration to get started, and offers extendability when needed. The project is in active development; while the latest **`v0.1.2`** release addresses critical output formatting and stability, your feedback is valued. Please report any issues you encounter.

---

## Table of Contents

| Section                                     |
| :------------------------------------------ |
| [1. Overview](#1-overview)                  |
| [2. Core Capabilities – Current Version (v0.1.2)](#2-core-capabilities--current-version-v012) |
| &nbsp;&nbsp;&nbsp;&nbsp; 2.1. Primary Redaction Categories |
| &nbsp;&nbsp;&nbsp;&nbsp; 2.2. Optional Features (with flags) |
| [3. Usage Examples](#3-usage-examples)      |
| [4. Known Issues](#4-known-issues)          |
| [5. Project Structure](#5-project-structure) |
| [6. Configuration Strategy](#6-configuration-strategy) |
| [7. Clipboard Support](#7-clipboard-support) |
| [8. Sanitizer Engine Design](#8-sanitizer-engine-design) |
| [9. Logging and Error Handling](#9-logging-and-error-handling) |
| [10. Testing and Validations](#10-testing-and-validations) |
| [11. Packaging & Distribution](#11-packaging--distribution) |
| [12. Metadata & License](#12-metadata--license) |
| [13. Security by Default Principles](#13-security-by-default-principles) |
| [14. Future-Proofing (Post v1.0 Aspirations)](#14-future-proofing-post-v10-aspirations) |
| [15. Summary of Technology Stack](#15-summary-of-technology-stack) |

---

## 1. Overview

`cleansh` is a powerful and reliable command-line utility designed to help you quickly and securely redact sensitive information from your terminal output. Whether you're debugging, collaborating, or sharing logs, `cleansh` ensures that confidential data like IP addresses, email addresses, and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `cleansh` provides a robust, pre-configured solution for data sanitization, with flexible options for custom rules and output formats.

**Sanitize your terminal output. One tool. One purpose.**

---

## 2. Core Capabilities – Current Version (**v0.1.2**)

This version of `cleansh` focuses on providing essential sanitization features with a strong emphasis on security and ease of use, building upon the "Precision View" of `v0.1.1`. Based on our rigorously passing test suite, `cleansh` accurately masks the following sensitive data types and handles output reliably:

### 2.1. Primary Redaction Categories:

* **Emails:** Common email address formats (e.g., `user@example.com`).
* **IP Addresses:** Both IPv4 addresses (e.g., `192.168.1.1`).
* **Tokens & Secrets:** Generic tokens, JSON Web Tokens (JWTs), AWS keys, GCP keys, SSH keys, and common hex secrets.
* **Absolute Paths:** `cleansh` redacts Linux paths (e.g., `/home/user/documents/report.pdf` are transformed to `~/home/user/...`) and macOS paths (e.g., `/Users/admin/logs/app.log` are transformed to `~/Users/admin/...`).

### 2.2. Optional Features (with flags):

`cleansh` provides command-line flags to customize its behavior, all thoroughly tested:

* **Copy to Clipboard:** Use `--clipboard` (`-c`) to automatically copy the sanitized output to your system's clipboard.
* **Show Diff View:** Use `--diff` (`-d`) to display a clear, **line-by-line colored diff** between the original and sanitized content, highlighting all redactions, powered by the `diffy` crate.
* **Load Custom Config:** Use `--config <path/to/config.yaml>` to apply your own custom redaction rules, which can augment or override the powerful built-in defaults.
* **Output to File:** Use `--out <path/to/result.txt>` to write the sanitized output directly to a specified file.

---

## 3. Usage Examples

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

-----

## 4\. Known Issues

### 4.1. Windows Absolute Path Redaction

  * **Severity:** Medium
  * **Impact:** Windows paths like `C:\Users\Alex\...` are not currently redacted.
  * **Workaround:** Implement a custom YAML rule for Windows paths.

### 4.2. Partial Custom Rule Interaction

  * **Severity:** Low
  * **Impact:** Custom rules may be partially overridden by built-in generic token rules due to broad pattern matching.
  * **Workaround:** Ensure custom patterns are highly specific to avoid unintended overlaps with default rules.

-----

## 5\. Project Structure

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
└── LICENSE (MIT)
```

-----

## 6\. Configuration Strategy

### 6.1. Runtime Settings (from `.env`)

**Example:**

```dotenv
LOG_LEVEL=info
CLIPBOARD_ENABLED=true
DEFAULT_CONFIG=./config/default_rules.yaml
```

### 6.2. User Rule Configuration (Optional, via `--config`)

```yaml
rules:
  - name: my_company_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    description: "Redacts company employee IDs."
    multiline: false
    dot_matches_new_line: false
```

-----

## 7\. Clipboard Support

  * **macOS & Windows:** Fully supported.
  * **Linux:** Requires `xclip`, `xsel`, or `wl-clipboard` to be installed on your system.

-----

## 8\. Sanitizer Engine Design

(Located in `src/tools/sanitize_shell.rs`)

### 8.1. Internal Pipeline:

1.  Input Acquisition
2.  Path Normalization
3.  Built-in Rule Application
4.  User Rule Application
5.  Output (stdout, file, clipboard, or diff view)

### 8.2. Engine Architecture:

  * `regex::RegexSet` for efficient pattern matching.
  * `strip-ansi-escapes` for clean text processing.
  * Compile-time embedded rules for performance and immutability.

-----

## 9\. Logging and Error Handling

### 9.1. Logging:

  * Powered by the `log` and `env_logger` crates.
  * Configurable via `.env` file or the `--debug` command-line flag.

### 9.2. Error Handling:

  * Utilizes `anyhow` and `thiserror` for robust error management.
  * Designed for fail-safe operation and graceful recovery on non-blocking issues, ensuring the tool's reliability.

-----

## 10\. Testing and Validations

### 10.1. Unit Tests:

  * Comprehensive validation of Regex Accuracy.
  * Thorough Path Redaction testing.
  * Rigorous YAML Parsing checks.

### 10.2. Integration Tests:

  * End-to-end scenarios for Stdin Piping.
  * Clipboard Mocks for reliable testing across platforms.
  * Verification of File Output.
  * Accuracy checks for Diff Output.
  * Validation of No Redaction Grace Cases, ensuring correct behavior when no sensitive data is present.

-----

## 11\. Packaging & Distribution

### 11.1. Installation

**Prebuilt Binaries:**
Available for download on [GitHub Releases](https://github.com/KarmaYama/cleansh/releases).

**Recommended One-liner (Linux/macOS):**

```bash
curl -sSf [https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh](https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh) | sh
```

> **Note:** Ensure the installer script adds `cleansh` to your system's `PATH`. This script is designed for local installation and does not require `sudo` privileges.

**Rust Users (via [crates.io](https://crates.io/crates/cleansh)):**

```bash
cargo install cleansh
cargo install cleansh --force # Use --force to update an existing installation
```

### 11.2. Building from Source

```bash
git clone [https://github.com/KarmaYama/cleansh.git](https://github.com/KarmaYama/cleansh.git)
cd cleansh
cargo build --release
cargo test
```

### 11.3. Distribution Automation

```bash
cargo dist build
```

-----

## 12\. Metadata & License

### 12.1. Metadata (in `Cargo.toml`)

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
description = "Sanitize your terminal output. One tool. One purpose."
```

### 12.2. License

This project is licensed under the [MIT License](./LICENSE).

-----

## 13\. Security by Default Principles

| Feature            | Principle                                                      |
| :----------------- | :------------------------------------------------------------- |
| No runtime eval    | The tool relies solely on regex-based logic; no external inputs are executed. |
| Local-only         | `cleansh` operates entirely locally, with no cloud interaction or telemetry. |
| Immutable defaults | Built-in redaction rules are compiled directly into the binary, ensuring consistency and immutability. |
| Path redaction     | File system details within output are sanitized by default to prevent accidental exposure. |
| YAML sandboxed     | Custom configuration via YAML is strictly declarative, preventing injection of malicious logic. |
| Clipboard opt-in   | Clipboard actions require an explicit `-c` flag, preventing silent or unexpected data copying. |

-----

## 14\. Future-Proofing (Post v1.0 Aspirations)

  * **Plugin System:** Develop a robust plugin architecture for custom redaction logic.
  * **Extended Interfaces:** Explore a VS Code extension or a web-based graphical user interface (GUI).
  * **WASM/Browser Version:** Adapt `cleansh` for WebAssembly to enable browser-based sanitization.
  * **Git Hook Integration:** Provide options for integrating `cleansh` directly into Git workflows (e.g., pre-commit hooks).
  * **Advanced Enterprise Tiers:** Develop specialized redaction features for enterprise environments, focusing on compliance and large-scale data handling.

-----

## 15\. Summary of Technology Stack

| Area              | Stack/Choice                                  |
| :---------------- | :-------------------------------------------- |
| **Language** | Rust                                          |
| **Config Format** | `.env` files + YAML                           |
| **CLI Parsing** | `clap`                                        |
| **Regex Engine** | `regex`                                       |
| **ANSI Stripping**| `strip-ansi-escapes`                          |
| **Diff Generation**| `diffy`                                       |
| **Clipboard** | `arboard`                                     |
| **Logging** | `log`, `env_logger`                           |
| **Error Handling**| `anyhow`, `thiserror`                         |
| **Installation** | `cargo-dist`, `curl`, `cargo install`         |
| **License** | MIT                                           |

-----

**Precision redaction. Local-only trust. Built for devs.**

-----
<!-- cargo-rdme -->