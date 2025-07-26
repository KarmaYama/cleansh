````markdown
# Cleansh – Sanitize Your Terminal Output, Securely.

[![CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![crates.io](https://img.shields.io/crates/v/cleansh.svg)](https://crates.io/crates/cleansh) [![License](https://img.shields.io/badge/license-PNL-blue.svg)](LICENSE) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![Netlify Status](https://api.netlify.com/api/v1/badges/2586fe1f-e613-4516-9dd8-6e4f06e58935/deploy-status)](https://app.netlify.com/projects/cleansh/deploys)

**[Contributing Guidelines](CONTRIBUTING.md)** | **[Code of Conduct](CODE_OF_CONDUCT.md)** | **[Changelog](CHANGELOG.md)** | **[Security Policy](SECURITY.md)** | **[Trademark Policy](TRADEMARK.md)** | **[Command Handbook](COMMANDS.md)**

> Cleansh is a high‑trust, single‑purpose CLI tool designed to sanitize terminal output for safe sharing.  
> It prioritizes security by default, requires zero configuration to get started, and offers extendability when needed.  
> The project is in active development, with **`v0.1.5`** bringing significant enhancements to redaction accuracy, security, and user control.  
> We value your feedback. Please report any issues you encounter.

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
| [5. Configuration Strategy](#5-configuration-strategy) |
| [6. Clipboard Support](#6-clipboard-support) |
| [7. Security by Default Principles](#7-security-by-default-principles) |
| [8. Future Vision & Roadmap](#8-future-vision--roadmap) |
| [9. Installation & Getting Started](#9-installation--getting-started) |
| [10. License](#10-license) |

---

## 1. Overview

`cleansh` is a powerful and reliable command‑line utility designed to help you quickly and securely redact sensitive information from your terminal output.  
Whether you're debugging, collaborating, or sharing logs, `cleansh` ensures that confidential data like IP addresses, email addresses, and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `cleansh` provides a robust, pre‑configured solution for data sanitization, with flexible options for custom rules and output formats.

**Sanitize your terminal output. One tool. One purpose.**

---

## 2. Core Capabilities – Current Version (**v0.1.5**)

This release represents a significant leap forward in `cleansh`'s accuracy, security, and testability. Based on our rigorously passing test suite, `cleansh` accurately masks:

### 2.1. Enhanced Redaction Categories:

`cleansh` offers broad and precise detection across a wide range of sensitive data types, complemented by robust programmatic validation for key PII:

* **Emails:** Common email formats (e.g., `user@example.com`).  
* **IP Addresses:** Both **IPv4** (e.g., `192.168.1.1`) and **IPv6** addresses (full uncompressed form, e.g., `2001:0db8:85a3:0000:0000:8a2e:0370:7334`).  
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
  * **Linux/macOS Absolute Paths** (`/home/user/...` → `~/home/user/...`).  
  * **Windows Absolute Paths** (`C:\\Users\\…`, `\\Server\\Share\\…`).  
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
* **Select Rule Set (`--rules <name>`):** Apply a predefined rule configuration (e.g., `default` for standard non-opt-in rules, `strict` to enable all rules including opt-in ones).  
* **Statistics Mode (`--stats-only`):** Analyze input for sensitive data and provide a summary without performing redaction.  
* **Export Stats to JSON File (`--stats-json-file <path>`):** When in statistics mode, write the detailed redaction summary to a JSON file.  
* **Export Stats to Stdout (`--export-json-to-stdout`):** When in statistics mode, print the JSON summary directly to `stdout`, suppressing other output.  
* **Sample Matches in Stats (`--sample-matches <count>`):** Include a specified number of unique original match examples for each rule in JSON statistics.  
* **Fail on Threshold (`--fail-over-threshold <count>`):** In statistics mode, exit with an error code if the total number of detections exceeds this count.  
* **Debug Logging (`--debug`):** Enable verbose debug output for troubleshooting.  
* **Suppress Debugging (`--no-debug`):** Disable debug logging.  
* **Quiet Output (`--quiet`):** Suppress all warnings and informational messages, showing only errors.

---

## 3. Usage Examples

**Basic Sanitization (stdin):**
```bash
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

---

## 4. Known Issues

### 4.1. Custom‑Rule Overrides

* **Severity:** Low — Broad “generic token” rules can potentially override more specific custom placeholders if not carefully defined.
* **Workaround:** Make your custom patterns more precise or use the `--disable-rules` flag to control which rules are active.

---

## 5. Configuration Strategy

### 5.1. Custom Rules (`--config`)

You can define your own custom redaction rules in a YAML file and merge them with Cleansh's built-in defaults:

```yaml
rules:
  - name: emp_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    multiline: false
    dot_matches_new_line: false
```

### 5.2. Rule Enable/Disable (`--enable-rules`, `--disable-rules`)

You can activate `opt_in` rules or deactivate any rule by name using CLI flags:

```bash
cleansh --enable-rules "uk_nino,aws_secret_key"
cleansh --disable-rules "email,ipv4_address"
```

---

## 6. Clipboard Support

* **macOS & Windows:** Built‑in.
* **Linux:** Requires `xclip`, `xsel` or `wl-clipboard`.

---

## 7. Security by Default Principles

| Feature                     | Principle                                                                        |
| :-------------------------- | :------------------------------------------------------------------------------- |
| No runtime eval             | All redaction via static regex, no code execution                                |
| Local‑only                  | No network calls or telemetry                                                    |
| Immutable defaults          | Built‑in rules embedded at compile time                                          |
| Path redaction              | Filesystem paths normalized to `~` or Windows equivalents                        |
| YAML sandboxed              | Declarative custom rules only, no arbitrary code execution                       |
| Clipboard opt‑in            | `-c` flag explicitly required for clipboard copy                                 |
| **ANSI Stripping**          | **Input content is pre-sanitized of escape codes to prevent evasion**            |
| **Programmatic Validation** | **Numerical PII rules have built-in validation for added accuracy and security** |

---

## 8. Future Vision & Roadmap

Cleansh is charting a course toward adaptive, user-driven enhancements, transforming it into an intelligent, trainable security assistant. Planned areas of exploration include:

* **Interactive Feedback Loop:** Enable users to provide feedback on specific matches (e.g., redact, ignore once, always ignore), allowing the tool to refine future detections.
* **Heuristic Tuning:** Adjustable detection thresholds (entropy levels, pattern sensitivity) for fine-grained control over candidate selection.
* **Enhanced CI/CD Modes:** Non-interactive audit outputs (JSON/exit codes) for automated pipelines, plus optional machine-readable reports.
* **Ecosystem Extensions:** Additional integrations (e.g., pre-commit hooks, GitHub Actions, GitLab CI templates) and a WASM core for broader compatibility.
* **Marketplace Concepts:** Explore the potential for a curated repository of community-maintained rule sets.
* **Enterprise Features:** Namespaced rule collections, role-based workflows, and centralized policy management.

These explorations will inform future releases, helping us build the most robust, flexible, and trustworthy sanitization tool for developers and organizations.

---

## 9. Installation & Getting Started

### Prebuilt Binaries (Recommended):

Download the latest prebuilt binaries for your platform from [GitHub](https://github.com/KarmaYama/cleansh/releases).

### Install Script:

```bash
curl -sSf https://github.com/KarmaYama/cleansh/releases/download/v0.1.5/cleansh-installer.sh | sh
```

### From crates.io:

```bash
cargo install cleansh # Use `cargo install cleansh --force` to update
```

### From Source:

```bash
git clone https://github.com/KarmaYama/cleansh.git
cd cleansh
cargo build --release
cargo test
```

---

## 10. License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/).

---
**Precision redaction. Local‑only trust. Built for devs.**
