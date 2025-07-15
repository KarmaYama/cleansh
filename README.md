# Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![crates.io](https://img.shields.io/crates/v/cleansh.svg)](https://crates.io/crates/cleansh) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql) [![Netlify Status](https://api.netlify.com/api/v1/badges/2586fe1f-e613-4516-9dd8-6e4f06e58935/deploy-status)](https://app.netlify.com/projects/cleansh/deploys)

**[Contributing Guidelines](CONTRIBUTING.md)** | **[Code of Conduct](CODE_OF_CONDUCT.md)** | **[Changelog](CHANGELOG.md)** | **[Project Scope](CLEANSH_SCOPE.md)** | **[Security Policy](SECURITY.md)** | **[Trademark Policy](TRADEMARK.md)**

> Cleansh is a high‑trust, single‑purpose CLI tool designed to sanitize terminal output for safe sharing.
> It prioritizes security by default, requires zero configuration to get started, and offers extendability when needed.
> The project is in active development, with **`v0.1.5`** bringing significant enhancements to redaction accuracy, security, and user control.
> Your feedback is valued. Please report any issues you encounter.

---

## Table of Contents

| Section |
| :---------------------------------------------------------------------- |
| [1. Overview](#1-overview) |
| [2. Core Capabilities – Current Version (v0.1.5)](#2-core-capabilities--current-version-v015) |
| &nbsp;&nbsp;&nbsp;&nbsp;2.1. Enhanced Redaction Categories |
| &nbsp;&nbsp;&nbsp;&nbsp;2.2. Advanced Features (with flags) |
| [3. Usage Examples](#3-usage-examples) |
| [4. Known Issues](#4-known-issues) |
| [5. Project Structure](#5-project-structure) |
| [6. Configuration Strategy](#6-configuration-strategy) |
| [7. Clipboard Support](#7-clipboard-support) |
| [8. Sanitizer Engine Design](#8-sanitizer-engine-design) |
| [9. Logging and Error Handling](#9-logging-and-error-handling) |
| [10. Testing and Validations](#10-testing-and-validations) |
| [11. Packaging & Distribution](#11-packaging--distribution) |
| [12. Metadata & License](#12-metadata--license) |
| [13. Security by Default Principles](#13-security-by-default-principles) |
| [14. Future‑Proofing (Post v1.0 Aspirations)](#14-future-proofing-post-v10-aspirations) |
| [15. Summary of Technology Stack](#15-summary-of-technology-stack) |

---

## 1. Overview

`cleansh` is a powerful and reliable command‑line utility designed to help you quickly and securely redact sensitive information from your terminal output.
Whether you're debugging, collaborating, or sharing logs, `cleansh` ensures that confidential data like IP addresses, email addresses,
and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `cleansh` provides
a robust, pre‑configured solution for data sanitization, with flexible options for custom rules and output formats.

**Sanitize your terminal output. One tool. One purpose.**

---
### Development Environment Setup

For the best Rust Analyzer experience in VS Code, the project includes a `.vscode/settings.json` that enables necessary Cargo features like `test-exposed` and `clipboard`.

## 2. Core Capabilities – Current Version (**v0.1.5**)

This release represents a significant leap forward in `cleansh`'s accuracy, security, and testability. Based on our rigorously passing test suite, `cleansh` accurately masks:

### 2.1. Enhanced Redaction Categories:

* **Emails:** Common email formats (e.g., `user@example.com`).
* **IP Addresses:** Both **IPv4** (e.g., `192.168.1.1`) and **IPv6 addresses** (full uncompressed form, e.g., `2001:0db8:85a3:0000:0000:8a2e:0370:7334`).
* **Tokens & Secrets:**
    * **JWTs**
    * **GitHub PATs** (`ghp_…`)
    * **GitHub fine‑grained PATs** (`github_pat_…`, 72 characters)
    * **Stripe keys** (`sk_live_…`, `sk_test_…`, `rk_live_…`)
    * **AWS Access/Secret Keys**
    * **GCP API Keys**
    * **Google OAuth tokens** (`ya29.…`, 20–120 characters)
    * **SSH private keys**
    * **Generic Hex Secrets** (32 and 64 characters)
    * **Generic Tokens**
* **Personal Identifiable Information (PII):**
    * **Credit Card Numbers**
    * **US Social Security Numbers (SSN)** (with programmatic validation against invalid patterns like `000-XX-XXXX`, `666-XX-XXXX`, or `9XX-XX-XXXX`).
    * **UK National Insurance Numbers (NINO)** (with programmatic validation against invalid prefixes and structural rules).
    * **South African ID Numbers**
* **Paths & URLs:**
    * **Linux/macOS Absolute Paths** (`/home/user/...` → `~/home/user/...`, refocused to user directories).
    * **Windows Absolute Paths** (`C:\Users\…`, `\\Server\Share\…`).
    * **Slack Webhook URLs** (`https://hooks.slack.com/services/T...`)
* **Authentication Headers:**
    * **HTTP Basic Auth Headers** (`Authorization: Basic ...`)

### 2.2. Advanced Features (with flags):

`cleansh` provides command‑line flags to customize its behavior, all thoroughly tested:

* **Copy to Clipboard (`-c` / `--clipboard`):** Instantly copy sanitized output.
* **Diff View (`-d` / `--diff`):** Show a colored, line‑by‑line diff of redactions.
* **Custom Config (`--config <path>`):** Load and merge your YAML redaction rules with built-in defaults.
* **Output File (`-o <path>`):** Write sanitized content to a file.
* **Suppress Summary (`--no-redaction-summary`):** Suppress the display of the redaction summary at the end of the output.
* **Enable Specific Rules (`--enable-rules <names>`):** Explicitly activate opt-in redaction rules by name (comma-separated).
* **Disable Specific Rules (`--disable-rules <names>`):** Explicitly deactivate any redaction rules by name (comma-separated), overriding defaults or custom enabled rules.

---

## 3. Usage Examples

**Basic Sanitization (stdin):**
```powershell
echo "My email is test@example.com and my IP is 192.168.1.1." | cleansh
````

**Docker Logs:**

```bash
docker logs my-sensitive-container | cleansh
```

**Kubectl Logs:**

```bash
kubectl logs my-pod-with-secrets | cleansh
```

**Clipboard:**

```bash
git config --list | cleansh -c
```

**Diff:**

```bash
cat /var/log/app/errors.log | cleansh -d
```

**Custom Rules:**

```bash
cat secrets.txt | cleansh --config ./custom_rules.yaml
```

**Save to File:**

```bash
myscript.sh | cleansh -o safe.log
```

**File Input:**

```bash
cleansh ./raw_log_file.txt
```

**Combined:**

```bash
mycmd | cleansh -d -o sanitized.txt
```

**Enable/Disable Specific Rules:**

```bash
echo "My Stripe key is sk_live_abc123. Email: user@example.com" | cleansh --enable-rules stripe_secret --disable-rules email
```

-----

## 4\. Known Issues

### 4.1. Custom‑Rule Overrides

  * **Severity:** Low — Broad “generic token” rules can potentially override more specific custom placeholders if not carefully defined.
  * **Workaround:** Make your custom patterns more precise or use the `--disable-rules` flag to control which rules are active.

-----

## 5\. Project Structure

```text
cleansh/
|-- src/
|   |-- main.rs
|   |-- commands/
|   |-- tools/
|   |-- config/
|   |-- ui/
|   `-- tests/
|-- config/
|   `-- default_rules.yaml
|-- .env
|-- .gitignore
|-- Cargo.toml
|-- README.md
`-- LICENSE (MIT)
```

-----

## 6\. Configuration Strategy

### 6.1. Runtime Settings (`.env`)

```dotenv
LOG_LEVEL=info
CLIPBOARD_ENABLED=true
DEFAULT_CONFIG=./config/default_rules.yaml
```

### 6.2. Custom Rules (`--config`)

```yaml
rules:
  - name: emp_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    multiline: false
    dot_matches_new_line: false
```

### 6.3. Rule Enable/Disable (`--enable-rules`, `--disable-rules`)

You can activate `opt_in` rules or deactivate any rule by name using CLI flags:

```bash
cleansh --enable-rules "uk_nino,aws_secret_key"
cleansh --disable-rules "email,ipv4_address"
```

-----

## 7\. Clipboard Support

  * **macOS & Windows:** Built‑in.
  * **Linux:** Requires `xclip`, `xsel` or `wl-clipboard`.

-----

## 8\. Sanitizer Engine Design

*(in `src/tools/sanitize_shell.rs`)*

1.  **Strip ANSI escapes:** All input content is now **sanitized for ANSI escape codes** prior to applying redaction rules, eliminating evasion via terminal formatting.
2.  **Apply built‑in regex rules:** Default rules are embedded at compile time.
3.  **Merge & apply custom YAML rules:** User-defined rules are loaded and merged, with granular control via `--enable-rules` and `--disable-rules`.
4.  **Output via stdout, clipboard, file, or diff:** Flexible output options, with enhanced redaction summary display (unique original/sanitized values) or suppression via `--no-redaction-summary`.

Performance via `regex::RegexSet`, security via compile‑time defaults and runtime validation.

-----

## 9\. Logging & Error Handling

  * **Logging:** `log` + `env_logger`, configurable via `.env` or `--debug`. Enhanced diagnostics now include detailed information about rule matches and programmatic validation results.
  * **Errors:** `anyhow` + `thiserror`, non‑fatal by default.
  * **Secure Logging:** Refined internal logging setup for tests using `test-log` crate to ensure debug messages are **conditionally logged only when explicitly enabled** via `CLEANSH_ALLOW_DEBUG_PII` environment variable, preventing accidental PII leakage.

-----

## 10\. Testing & Validation

  * **Unit Tests:** Regex accuracy, path normalization, YAML parsing.
  * **Integration Tests:** Extensive new tests validate:
      * ANSI-stripping effectiveness (`test_sanitize_content_with_ansi_escapes`)
      * Clipboard output (`test_run_cleansh_clipboard_copy`)
      * Rule opt-in and opt-out behaviors
      * Redaction summary toggling
      * Edge cases like overlapping rules and invalid formats.
  * **Programmatic Validation:** Rules like US SSN and UK NINO now include in-depth programmatic validation to reject invalid patterns.

-----

## 11\. Packaging & Distribution

  * **Prebuilt Binaries:** GitHub Releases with `cargo-dist`.

  * **Install Script:**

    ```bash
    curl -sSf [https://github.com/KarmaYama/cleansh/releases/download/v0.1.5/cleansh-installer.sh](https://github.com/KarmaYama/cleansh/releases/download/v0.1.5/cleansh-installer.sh) | sh
    ```

  * **crates.io:** `cargo install cleansh` (+ `--force` to update)

  * **From Source:**

    ```bash
    git clone [https://github.com/KarmaYama/cleansh.git](https://github.com/KarmaYama/cleansh.git)
    cd cleansh
    cargo build --release
    cargo test
    ```

-----

## 12\. Metadata & License

### 12.1. Cargo.toml metadata

```toml
[package]
name = "cleansh"
version = "0.1.5"
edition = "2024"
license = "MIT"
repository = "[https://github.com/KarmaYama/cleansh](https://github.com/KarmaYama/cleansh)"
readme = "README.md"
```

### 12.2. License

This project is licensed under the [MIT License](https://www.google.com/search?q=./LICENSE).

-----

## 13\. Security by Default Principles

| Feature | Principle |
| :----------------- | :------------------------------------------------ |
| No runtime eval | All redaction via static regex, no code execution |
| Local‑only | No network calls or telemetry |
| Immutable defaults | Built‑in rules embedded at compile time |
| Path redaction | Filesystem paths normalized to `~` or Windows equivalents |
| YAML sandboxed | Declarative custom rules only, no execution |
| Clipboard opt‑in | `-c` flag required for clipboard copy |
| **ANSI Stripping** | **Input content is pre-sanitized of escape codes to prevent evasion** |
| **Programmatic Validation** | **Numerical PII rules have built-in validation for added accuracy and security** |

-----

## 14\. Future‑Proofing (Post v1.0 Aspirations)

  * Plugin architecture for external redactors
  * VS Code extension / Web GUI
  * WebAssembly build for browser sanitization
  * Git hooks integration (pre‑commit, pre‑push)
  * Enterprise tier: compliance‑focused redaction

-----

## 15\. Summary of Technology Stack

| Area | Stack/Choice |
| :-------------- | :-------------------------------- |
| Language | Rust |
| Config Format | `.env` + YAML |
| CLI Parser | `clap` |
| Regex Engine | `regex` |
| ANSI Stripping | `strip-ansi-escapes` |
| Diff Generation | `diffy` |
| Clipboard | `arboard` |
| Logging | `log`, `env_logger`, `test-log` |
| Error Handling | `anyhow`, `thiserror` |
| Packaging | `cargo-dist`, curl install, cargo |
| License | MIT |

-----

**Precision redaction. Local‑only trust. Built for devs.**

