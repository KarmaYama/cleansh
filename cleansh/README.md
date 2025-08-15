# CleanSH – Sanitize Your Terminal Output, Securely.

[![Downloads from crates.io](https://img.shields.io/crates/d/cleansh.svg?style=for-the-badge&labelColor=334155&color=4FC3F7)](https://crates.io/crates/cleansh) [![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![Dependabot Updates](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![Rust CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml) [![Star](https://img.shields.io/github/stars/KarmaYama/cleansh.svg?style=social)](https://github.com/KarmaYama/cleansh/stargazers)

**[Contributing Guidelines](CONTRIBUTING.md)** | **[Code of Conduct](CODE_OF_CONDUCT.md)** | **[Changelog](CHANGELOG.md)** | **[Security Policy](SECURITY.md)** | **[Trademark Policy](TRADEMARK.md)** | **[Command Handbook](COMMANDS.md)**

> CleanSH (clean shell) is a high‑trust, single‑purpose CLI tool designed to sanitize terminal output for safe sharing.
> It prioritizes security by default, requires zero configuration to get started, and offers extendability when needed.
> The project is in active development, with **`v0.1.8`** bringing significant enhancements to redaction accuracy, security, and user control.
> We value your feedback. Please report any issues you encounter. Star the repository if you like it!

---

## Table of Contents

| Section |
| :---------------------------------------------------------------------- |
| [1. Overview](#1-overview) |
| [2. Important Note on Licensing](#2-important-note-on-licensing) |
| [3. Core Capabilities – Free Tier](#3-core-capabilities--free-tier) |
| [4. Cleansh Pro Features](#4-cleansh-pro-features) |
| [5. Usage Examples](#5-usage-examples) |
| [6. Known Issues](#6-known-issues) |
| [7. Configuration Strategy](#7-configuration-strategy) |
| [8. Clipboard Support](#8-clipboard-support) |
| [9. Security by Default Principles](#9-security-by-default-principles) |
| [10. Future Vision & Roadmap](#10-future-vision--roadmap) |
| [11. Installation & Getting Started](#11-installation--getting-started) |
| [12. License](#12-license) |

---

## 1. Overview

`CleanSH` is a powerful and reliable command‑line utility designed to help you quickly and securely redact sensitive information from your terminal output.
Whether you're debugging, collaborating, or sharing logs, `CleanSH` ensures that confidential data like IP addresses, email addresses, and access tokens never leave your local environment unmasked. Piped directly from `stdin` or loaded from files, `CleanSH` provides a robust, pre‑configured solution for data sanitization, with flexible options for custom rules and output formats.

**Sanitize your terminal output. One tool. One purpose.**

---

## 2. Important Note on Licensing

As part of `CleanSH`'s commitment to sustainable development and continued innovation, we have shifted our licensing model to ensure the project's long-term viability.

| Aspect | Versions (`< v0.1.5`) | Versions (`v0.1.5` and up to v0.x.x) |
| :---------------- | :----------------------------- | :---------------------------------------------------- |
| **Primary License** | **MIT License** | **PolyForm Noncommercial License 1.0.0** |
| **Noncommercial Use** | Free to use | **Free to use** (for personal, academic, research, etc.) |
| **Commercial Use** | Free to use | **Trial & Evaluation Period (see below)** |
| **Enforcement** | N/A | **Enforcement starts with `v1.0.0`** |

Effective with `v0.1.5`, `CleanSH` adopts the **PolyForm Noncommercial License 1.0.0**. For versions `v0.1.5` up to `v0.x.x`, commercial use is permitted for evaluation and trial purposes.

### Free Tier vs. Pro Tier
CleanSH provides a clear distinction between its free and pro tiers. The **free tier** includes all core sanitization functionality, available to all users under the PolyForm Noncommercial License 1.0.0. The **pro tier**, which includes advanced features for enterprise and commercial use, requires a valid commercial license key. The application will check for a valid license before executing these pro features and will exit with an error if one is not found.

**📢 For Commercial Licenses:**
Please email us at [licenses@obscuratech.tech](mailto:licenses@obscuratech.tech) for pricing and terms.

For a detailed breakdown of which features are in each tier, please refer to our dedicated **[License Notes](LICENSE_NOTES.md)**.

---

## 3. Core Capabilities – Free Tier

The core `cleansh` functionality is available to all users for free. This release represents a significant leap forward in `Cleansh`'s accuracy, security, and testability. Based on our rigorously passing test suite, `Cleansh` accurately masks:

### 3.1. Enhanced Redaction Categories:

`Cleansh` offers broad and precise detection across a wide range of sensitive data types, complemented by robust programmatic validation for key PII:

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
    * **Windows Absolute Paths** (`C:\Users\…`, `\\Server\Share\…`).
    * **Slack Webhook URLs** (`https://hooks.slack.com/services/T...`)
* **Authentication Headers:**
    * **HTTP Basic Auth Headers** (`Authorization: Basic ...`)

### 3.2. Primary Commands & Options:

The core `cleansh` CLI is organized into powerful subcommands, each with a specific purpose.

* **`cleansh sanitize`:** The primary command for redacting content. It can read from stdin or a file and write to stdout, a file, or the clipboard.
* **`cleansh uninstall`:** A utility command to safely remove `cleansh` and its associated files from your system.
* **`cleansh profiles list`:** Lists all available local redaction profiles.

### 3.3. Advanced Flags (within the free tier):

`Cleansh` provides command‑line flags to customize its behavior, all thoroughly tested:

* **Copy to Clipboard (`-c` / `--clipboard`):** Instantly copy sanitized output.
* **Diff View (`-d` / `--diff`):** Show a colored, line‑by‑line diff of redactions.
* **Custom Config (`--config <path>`):** Load and merge your YAML redaction rules with built-in defaults.
* **Output File (`-o <path>`):** Write sanitized content to a file.
* **Suppress Summary (`--no-redaction-summary`):** Suppress the display of the redaction summary at the end of the output.
* **Enable Specific Rules (`--enable <names>`):** Explicitly activate opt-in redaction rules by name (comma-separated).
* **Disable Specific Rules (`--disable <names>`):** Explicitly deactivate any redaction rules by name (comma-separated), overriding defaults or custom enabled rules.
* **Select Rule Set (`--rules <name>`):** Apply a predefined rule configuration (e.g., `default` for standard non-opt-in rules, `strict` to enable all rules including opt-in ones).
* **Debug Logging (`--debug`):** Enable verbose debug output for troubleshooting.
* **Suppress Debugging (`--no-debug`):** Disable debug logging.
* **Quiet Output (`--quiet`):** Suppress all warnings and informational messages, showing only errors.

---

## 4. Cleansh Pro Features

Cleansh offers powerful features designed for commercial and enterprise use cases, which require a commercial license. These features are intended for team collaboration, policy enforcement, and cryptographic verification of data handling.

**Pro Commands Included:**
* **`cleansh scan`:** Scans input and provides a detailed redaction summary without altering content. This is ideal for security audits and CI/CD pipelines.
* **`cleansh profiles sync`:** Synchronize redaction profiles with a central server using an organization ID and API key. This ensures all team members are using the same, up-to-date rules.
* **`cleansh profiles sign`:** Cryptographically sign a redaction profile for integrity and authenticity.
* **`cleansh verify-artifact`:** Cryptographically verify the signature of a redaction artifact JSON file using a public key. This provides an auditable, non-repudiable proof that a file was processed correctly and has not been tampered with.

---

## 5. Usage Examples

**Basic Sanitization (stdin):**

```bash
echo "My email is test@example.com and my IP is 192.168.1.1." | cleansh sanitize
````

**CI/CD Scan (using a fail threshold):**

```bash
cat build.log | cleansh scan --fail-over-threshold 0
```

**Docker Logs:**

```bash
docker logs my-sensitive-container | cleansh sanitize
```

**Kubectl Logs:**

```bash
kubectl logs my-pod-with-secrets | cleansh sanitize
```

**Clipboard:**

```bash
git config --list | cleansh sanitize -c
```

**Diff:**

```bash
cat /var/log/app/errors.log | cleansh sanitize -d
```

**Custom Rules:**

```bash
cat secrets.txt | cleansh sanitize --config ./custom_rules.yaml
```

**Save to File:**

```bash
myscript.sh | cleansh sanitize -o safe.log
```

**File Input:**

```bash
cleansh sanitize ./raw_log_file.txt
```

**Combined:**

```bash
mycmd | cleansh sanitize -d -o sanitized.txt
```

**Enable/Disable Specific Rules:**

```bash
echo "My Stripe key is sk_live_abc123. Email: user@example.com" | cleansh sanitize --enable stripe_secret --disable email
```

-----

## 6\. Known Issues

### 6.1. Custom‑Rule Overrides

  * **Severity:** Low — Broad “generic token” rules can potentially override more specific custom placeholders if not carefully defined.
  * **Workaround:** Make your custom patterns more precise or use the `--disable` flag to control which rules are active.

-----

## 7\. Configuration Strategy

### 7.1. Custom Rules (`--config`)

You can define your own custom redaction rules in a YAML file and merge them with Cleansh's built-in defaults:

```yaml
rules:
  - name: emp_id
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
    pattern_type: "regex"
    version: "0.1.8"
    author: "Obscura Team"
    created_at: "2025-06-12T00:00:00Z"
    updated_at: "2025-08-11T00:00:00Z"
    dot_matches_new_line: false
    programmatic_validation: false
    multiline: false
```

### 7.2. Rule Enable/Disable (`--enable`, `--disable`)

You can activate `opt_in` rules or deactivate any rule by name using CLI flags:

```bash
cleansh sanitize --enable "uk_nino,aws_secret_key" 
cleansh sanitize --disable "email,ipv4_address"
```

-----

## 8\. Clipboard Support

  * **macOS & Windows:** Built‑in.
  * **Linux:** Requires `xclip`, `xsel` or `wl-clipboard`.

-----

## 9\. Security by Default Principles

| Feature | Principle |
| :-------------------------- | :------------------------------------------------------------------------------- |
| No runtime eval | All redaction via static regex, no code execution |
| Local‑only | No network calls or telemetry |
| Immutable defaults | Built‑in rules embedded at compile time |
| Path redaction | Filesystem paths normalized to `~` or Windows equivalents |
| YAML sandboxed | Declarative custom rules only, no arbitrary code execution |
| Clipboard opt‑in | `-c` flag explicitly required for clipboard copy |
| **ANSI Stripping** | **Input content is pre-sanitized of escape codes to prevent evasion** |
| **Programmatic Validation** | **Numerical PII rules have built-in validation for added accuracy and security** |

-----

## 10\. Future Vision & Roadmap

CleanSH is charting a course toward adaptive, user-driven enhancements, transforming it into an intelligent, trainable security assistant. Planned areas of exploration include:

  * **Pluggable Detection Architecture:** Introduce a modular architecture that allows for multiple, independent detection engines to run simultaneously, including an **entropy-based engine** for finding high-randomness secrets and a future **Adaptive Interactive Learning (AIL)** engine. This will significantly reduce false negatives without adding complexity to the user's workflow.
  * **Interactive Feedback Loop:** Enable users to provide feedback on specific matches (e.g., redact, ignore once, always ignore), allowing the tool to refine future detections.
  * **Heuristic Tuning:** Adjustable detection thresholds (entropy levels, pattern sensitivity) for fine-grained control over candidate selection.
  * **Enhanced CI/CD Modes:** Non-interactive audit outputs (JSON/exit codes) for automated pipelines, plus optional machine-readable reports.
  * **Ecosystem Extensions:** Additional integrations (e.g., pre-commit hooks, GitHub Actions, GitLab CI templates) and a WASM core for broader compatibility.
  * **Marketplace Concepts:** Explore the potential for a curated repository of community-maintained rule sets.
  * **Enterprise Features:** Namespaced rule collections, role-based workflows, and centralized policy management.

These explorations will inform future releases, helping us build the most robust, flexible, and trustworthy sanitization tool for developers and organizations.

-----

## 11\. Installation & Getting Started

### Prebuilt Binaries (Recommended):

Download the latest prebuilt binaries for your platform from [GitHub](https://github.com/KarmaYama/cleansh-workspace/releases).

### Install Script:

```bash
curl -sSf [https://github.com/KarmaYama/cleansh/releases/download/v0.1.8/cleansh-installer.sh](https://github.com/KarmaYama/cleansh/releases/download/v0.1.8/cleansh-installer.sh) | sh
```

### From crates.io:

```bash
cargo install cleansh # Use `cargo install cleansh --force` to update
```

### From Source:

```bash
git clone [https://github.com/KarmaYama/cleansh-workspace.git](https://github.com/KarmaYama/cleansh-workspace.git)
cd cleansh
cargo build --release
cargo test --package cleansh
```

-----

## 12\. License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/).

-----

**Precision redaction. Local‑only trust. Built for devs.**

*Copyright 2025 Obscura Tech.*
