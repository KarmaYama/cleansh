<!-- cargo-rdme -->

# Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml)  
[![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml)  
[![crates.io](https://img.shields.io/crates/v/cleansh.svg)](https://crates.io/crates/cleansh)  
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)  
[![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml)  
[![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql)

> Cleansh is a high‑trust, single‑purpose CLI tool designed to sanitize terminal output for safe sharing.  
> It prioritizes security by default, requires zero configuration to get started, and offers extendability when needed.  
> The project is in active development; while the latest **`v0.1.2`** release addresses critical output formatting and stability,  
> your feedback is valued. Please report any issues you encounter.

---

## Table of Contents

| Section                                                                 |
| :---------------------------------------------------------------------- |
| [1. Overview](#1-overview)                                              |
| [2. Core Capabilities – Current Version (v0.1.2)](#2-core-capabilities--current-version-v012) |
| &nbsp;&nbsp;&nbsp;&nbsp;2.1. Primary Redaction Categories               |
| &nbsp;&nbsp;&nbsp;&nbsp;2.2. Optional Features (with flags)             |
| [3. Usage Examples](#3-usage-examples)                                  |
| [4. Known Issues](#4-known-issues)                                      |
| [5. Project Structure](#5-project-structure)                            |
| [6. Configuration Strategy](#6-configuration-strategy)                  |
| [7. Clipboard Support](#7-clipboard-support)                            |
| [8. Sanitizer Engine Design](#8-sanitizer-engine-design)                |
| [9. Logging and Error Handling](#9-logging-and-error-handling)          |
| [10. Testing and Validations](#10-testing-and-validations)              |
| [11. Packaging & Distribution](#11-packaging--distribution)             |
| [12. Metadata & License](#12-metadata--license)                         |
| [13. Security by Default Principles](#13-security-by-default-principles)|
| [14. Future‑Proofing (Post v1.0 Aspirations)](#14-future-proofing-post-v10-aspirations) |
| [15. Summary of Technology Stack](#15-summary-of-technology-stack)      |

---

## 1. Overview

`cleansh` is a powerful and reliable command‑line utility designed to help you quickly and securely redact sensitive information from your terminal output.  
Whether you're debugging, collaborating, or sharing logs, `cleansh` ensures that confidential data like IP addresses, email addresses,  
and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `cleansh` provides  
a robust, pre‑configured solution for data sanitization, with flexible options for custom rules and output formats.

**Sanitize your terminal output. One tool. One purpose.**

---

## 2. Core Capabilities – Current Version (**v0.1.2**)

This version of `cleansh` focuses on providing essential sanitization features with a strong emphasis on security and ease of use,  
building upon the "Precision View" introduced in v0.1.1. Based on our rigorously passing test suite, `cleansh` accurately masks:

### 2.1. Primary Redaction Categories:

* **Emails:** Common email formats (e.g., `user@example.com`).  
* **IP Addresses:** IPv4 addresses (e.g., `192.168.1.1`).  
* **Tokens & Secrets:** JWTs, AWS/GCP keys, SSH keys, hex secrets, and generic tokens.  
* **Absolute Paths:** Linux (`/home/user/...` → `~/home/user/...`) and macOS paths (`/Users/admin/...` → `~/Users/admin/...`).  

### 2.2. Optional Features (with flags):

`cleansh` provides command‑line flags to customize its behavior, all thoroughly tested:

* **Copy to Clipboard (`-c` / `--clipboard`):** Instantly copy sanitized output.  
* **Diff View (`-d` / `--diff`):** Show a colored, line‑by‑line diff of redactions.  
* **Custom Config (`--config <path>`):** Merge in your YAML redaction rules.  
* **Output File (`-o <path>`):** Write sanitized content to a file.

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
cleansh  ./raw_log_file.txt
```

**Combined:**

```bash
mycmd | cleansh -d -o sanitized.txt
```

---

## 4. Known Issues

### 4.1. Windows Path Redaction

* **Severity:** Medium — Windows paths (`C:\Users\…`) aren’t auto‑redacted yet.
* **Workaround:** Add a custom YAML rule targeting `^[A-Z]:\\.*`.

### 4.2. Custom‑Rule Overrides

* **Severity:** Low — Broad “generic token” rules can override specific placeholders.
* **Workaround:** Make custom patterns more precise.

---

## 5. Project Structure

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

---

## 6. Configuration Strategy

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

---

## 7. Clipboard Support

* **macOS & Windows:** Built‑in.
* **Linux:** Requires `xclip`, `xsel` or `wl-clipboard`.

---

## 8. Sanitizer Engine Design

*(in `src/tools/sanitize_shell.rs`)*

1. Strip ANSI escapes
2. Apply built‑in regex rules
3. Merge & apply custom YAML rules
4. Output via stdout, clipboard, file, or diff

Performance via `regex::RegexSet`, security via compile‑time defaults.

---

## 9. Logging & Error Handling

* **Logging:** `log` + `env_logger`, configurable via `.env` or `--debug`.
* **Errors:** `anyhow` + `thiserror`, non‑fatal by default.

---

## 10. Testing & Validation

* **Unit Tests:** Regex accuracy, path normalization, YAML parsing.
* **Integration Tests:** stdin piping, clipboard mocks, file output, diff, no‑redaction case.

---

## 11. Packaging & Distribution

* **Prebuilt Binaries:** GitHub Releases with `cargo-dist`.
* **Install Script:**

  ```bash
  curl -sSf https://github.com/KarmaYama/cleansh/releases/download/v0.1.2/cleansh-installer.sh | sh
  ```
* **crates.io:** `cargo install cleansh` (+ `--force` to update)
* **From Source:**

  ```bash
  git clone https://github.com/KarmaYama/cleansh.git
  cd cleansh
  cargo build --release
  cargo test
  ```

---

## 12. Metadata & License

### 12.1. Cargo.toml metadata

```toml
[package]
name = "cleansh"
version = "0.1.2"
edition = "2024"
license = "MIT"
repository = "https://github.com/KarmaYama/cleansh"
readme = "README.md"
```

### 12.2. License

This project is licensed under the [MIT License](./LICENSE).

---

## 13. Security by Default Principles

| Feature            | Principle                                         |
| :----------------- | :------------------------------------------------ |
| No runtime eval    | All redaction via static regex, no code execution |
| Local‑only         | No network calls or telemetry                     |
| Immutable defaults | Built‑in rules embedded at compile time           |
| Path redaction     | Filesystem paths normalized to `~`                |
| YAML sandboxed     | Declarative custom rules only, no execution       |
| Clipboard opt‑in   | `-c` flag required for clipboard copy             |

---

## 14. Future‑Proofing (Post v1.0 Aspirations)

* Plugin architecture for external redactors
* VS Code extension / Web GUI
* WebAssembly build for browser sanitization
* Git hooks integration (pre‑commit, pre‑push)
* Enterprise tier: compliance‑focused redaction

---

## 15. Summary of Technology Stack

| Area            | Stack/Choice                      |
| :-------------- | :-------------------------------- |
| Language        | Rust                              |
| Config Format   | `.env` + YAML                     |
| CLI Parser      | `clap`                            |
| Regex Engine    | `regex`                           |
| ANSI Stripping  | `strip-ansi-escapes`              |
| Diff Generation | `diffy`                           |
| Clipboard       | `arboard`                         |
| Logging         | `log`, `env_logger`               |
| Error Handling  | `anyhow`, `thiserror`             |
| Packaging       | `cargo-dist`, curl install, cargo |
| License         | MIT                               |

---

**Precision redaction. Local‑only trust. Built for devs.**

<!-- cargo-rdme -->

