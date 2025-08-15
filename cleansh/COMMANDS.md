# CleanSH Command Handbook

Welcome to the **CleanSH Command Handbook**! This guide dives deeper into `CleanSH`, your indispensable command-line utility for sanitizing sensitive information from terminal output. Whether you're a developer, system administrator, or anyone who values privacy and security in their shared data, `CleanSH` empowers you to control what leaves your terminal.

We'll go beyond just listing commands, exploring scenarios, use cases, and how to harness `CleanSH`'s full potential, including building your own custom rules.

---

## 1. What is CleanSH?

At its core, `CleanSH` (pronounced "clean shell") is a **high-trust, single-purpose CLI tool** designed to redact sensitive data from text streams. Think of it as your digital bouncer, ensuring confidential information like **IP addresses, email addresses, API keys, and even personal identifiers** never accidentally leak when you're sharing logs, debugging output, or collaborating with others.

It operates **locally**, requires zero configuration to get started with its robust default rules, and offers extensive flexibility for custom needs. Whether you pipe output directly from other commands or feed it files, `CleanSH` is built for secure, efficient, and precise data sanitization.

---

## 2. Getting Started with CleanSH

Before diving into the commands, make sure you have `CleanSH` installed.

### Installation

The **recommended** way to get `CleanSH` is by downloading the latest prebuilt binaries for your platform from the [GitHub Releases](https://github.com/KarmaYama/cleansh-workspace/releases) page.

Alternatively, you can use the install script:

```bash
curl -sSf [https://github.com/KarmaYama/cleansh-workspace/releases/download/v0.1.8/cleansh-installer.sh](https://github.com/KarmaYama/cleansh-workspace/releases/download/v0.1.8/cleansh-installer.sh) | sh
````

If you have Rust's Cargo installed, you can also install it directly:

```bash
cargo install cleansh # Use `cargo install cleansh --force` to update
```

For the adventurous, you can build from source:

```bash
git clone [https://github.com/KarmaYama/cleansh-workspace.git](https://github.com/KarmaYama/cleansh-workspace.git)
cd cleansh
cargo build --release
cargo test
```

Once installed, you're ready to start cleaning your output\!

-----

## 3\. Cleansh's Command Architecture

`CleanSH` has adopted a modern, subcommand-based architecture. Instead of relying on a single top-level command for all functions, its capabilities are now organized into distinct commands for clarity and purpose.

| Command | Description | Use Case |
| :--- | :--- | :--- |
| **`cleansh sanitize`** | The primary command for redacting sensitive data. | Daily use, sanitizing logs or terminal output. |
| **`cleansh scan`** | Scans for sensitive data and provides a report without redacting. | Security auditing, pre-scan assessments. |
| **`cleansh profiles`** | Manages redaction profiles and rule sets. | Creating, signing, and verifying custom rules. |
| **`cleansh uninstall`** | Safely removes the `cleansh` CLI and its associated files. | System maintenance. |
| **`cleansh sync`** | (Pro Feature) Synchronizes redaction profiles with a central server. | Enterprise-grade policy management. |
| **`cleansh verify`** | (Pro Feature) Cryptographically verifies the integrity of a redaction artifact. | Auditable security and compliance workflows. | 

-----

## 4\. Core Capabilities: The Free Tier

These commands are available for personal, academic, and non-commercial use under the Polyform Noncommercial License.

### 4.1. `cleansh sanitize` – Redacting Sensitive Output

This is the most common command, handling the core redaction logic. It reads from standard input (`stdin`) or a file and writes the sanitized content to standard output (`stdout`) by default.

**Basic Usage: Piping Content**
The most common way to use `cleansh sanitize` is by piping the output of another command into it.

**Scenario:** You're debugging an application, and its logs contain an email address and an internal IP. You want to share these logs with a colleague but without revealing the sensitive information.

```powershell
"User login attempt from test@example.com at 192.168.1.1." | cleansh sanitize
```

**Output:**

```
User login attempt from [EMAIL_REDACTED] at [IPV4_REDACTED].
```

**Why this is useful:** Quick and effortless sanitization directly in your workflow. The original sensitive data never leaves your terminal window unmasked.

**Sanitizing File Content**
Instead of piping, you can also provide `cleansh sanitize` with a file path.

**Scenario:** You have an existing log file, `application.log`, that might contain sensitive data, and you want to create a sanitized version.

```powershell
cleansh sanitize ./application.log -o sanitized_application.log
```

**Explanation:**

  * `cleansh sanitize ./application.log`: Reads the content of `application.log`.
  * `-o sanitized_application.log`: Writes the sanitized output to a new file named `sanitized_application.log`.

### 4.2. `cleansh scan` – Auditing for Secrets

The `scan` command is designed for auditing. It identifies sensitive data based on your rules and provides a report without performing any redaction.

**Scenario:** Before sharing a large log file, you want to get a summary of all the sensitive data types found within it.

```powershell
cleansh scan my_logfile.txt
```

**Output (example to `stderr`):**

```
Redaction Statistics Summary:
  EmailAddress: 1 match
  IPv4Address: 1 match
  UsSsn: 1 match
```

This command supports all the advanced flags from `sanitize`, such as `--stats-json-file` for machine-readable output and `--sample-matches` for context.

### 4.3. `cleansh scan` – Enforcing Security in Your Pipeline

This command is a specialized version of `scan` designed for automated pipelines. It scans for secrets and exits with an error code if the total number of detections exceeds a specified threshold, which can be configured with `--fail-over-threshold`.

**Scenario:** Your build pipeline should fail if more than 2 secrets are detected in the build logs.

```bash
docker logs my-app-container | cleansh scan --fail-over-threshold 2
```

**Explanation:** If more than two secrets are detected, the command will exit with a non-zero status, causing a CI/CD job to fail. This enforces a "security by design" principle.

### 4.4. `cleansh profiles` – Managing Redaction Rules Locally

The `profiles` command is a suite of subcommands for managing and verifying your custom redaction rules and rule sets.

  * **`cleansh profiles list`:** Lists all the redaction profiles and rule sets that are available on your local system, including their names and descriptions. This helps you keep track of your configurations.
  * **`cleansh profiles sign`:** Signs a profile YAML file with a private key. This is the first step in creating a cryptographically verifiable rule set. This is a core component for the Pro features.
  * **`cleansh profiles verify`:** Verifies the signature of a profile YAML file using a public key. This ensures that the profile has not been tampered with and comes from a trusted source.

-----

## 5\. CleanSH Pro Features

These features are intended for commercial and enterprise use and require a commercial license. They are designed for teams that need to enforce consistent policies and ensure data integrity.

  * **`cleansh sync`:** Synchronize redaction profiles with a central server using an organization ID and API key. This ensures all team members are using the same, up-to-date rules. This is a crucial feature for enterprise-wide policy enforcement.- not yet available.
  * **`cleansh verify`:** Cryptographically verify the signature of a redaction artifact JSON file using a public key. This provides an auditable, non-repudiable proof that a file was processed correctly and has not been tampered with.

----- 

## 6\. Global Flags and Advanced Features

The following flags are available for most of the `cleansh` commands.

  * **Copy to Clipboard (`-c` / `--clipboard`):** Instantly copy sanitized output.
  * **Diff View (`-d` / `--diff`):** Show a colored, line-by-line diff of redactions.
  * **Custom Config (`--config <path>`):** Load and merge your YAML redaction rules with built-in defaults.
  * **Output File (`-o <path>`):** Write sanitized content to a file.
  * **Suppress Summary (`--no-redaction-summary`):** Suppress the display of the redaction summary at the end of the output.
  * **Enable Specific Rules (`--enable <names>`):** Explicitly activate opt-in redaction rules.
  * **Disable Specific Rules (`--disable <names>`):** Explicitly deactivate any redaction rules.
  * **Select Rule Set (`--rules <name>`):** Apply a predefined rule configuration (`default` or `strict`).
  * **Debug Logging (`--debug`):** Enable verbose debug output for troubleshooting.
  * **Quiet Output (`--quiet`):** Suppress all warnings and informational messages.
  * **Suppress Donation Prompts (`--disable-donation-prompts`):** Disable donation prompts for automated environments.

-----

## 7\. Configuration Strategy

### Custom Rules with `--config`

You can define your own rules in a YAML file, which will be merged with Cleansh's built-in defaults.

**Example:**
Create `my_custom_rules.yaml`:

```yaml
rules:
  - name: "emp_id"
    pattern: 'EMP-\d{5}'
    replace_with: '[EMPLOYEE_ID_REDACTED]'
```

Then use it with `cleansh sanitize`:

```bash
"Employee ID is EMP-12345, email is test@company.com." | cleansh sanitize --config ./my_custom_rules.yaml
```

### Enabling/Disabling Specific Rules

Use `--enable` and `--disable` for fine-grained control.

**Example:** Activating an opt-in rule for AWS keys.

```bash
"My AWS Secret Key is f8N/pD+gA5T7j2K1L0mXq9Y4c3b6a8s0d2f1e5i7h9j0k4l3m2n1o6p5q4r3s2t1u9v8w7x6y5z4a3b2c1d0e9f8g7h6i5j4k3l3n1o0p. Also a regular email@example.com." | cleansh sanitize --enable aws_secret_key
```

### Rule Configurations: `default` vs. `strict`

The `--rules` flag allows you to switch between predefined rule sets.

  * `--rules default`: The standard, balanced rule set.
  * `--rules strict`: Activates all rules, including those prone to false positives, for comprehensive auditing.

-----

## 8\. Docker & Log Sanitization Scenarios

`CleanSH` is perfect for operational security, especially with Docker.

### Sanitizing Past Docker Logs (Batch):

This is the recommended approach for persistent logging.

```bash
docker logs cleansh-test-logger | cleansh sanitize --output "sanitized_docker_logs.log"
```

### Real-time Sanitization of Docker Logs:

This pipes live logs through `cleansh`. Note: This can be unstable on Windows PowerShell.

```bash
docker logs -f cleansh-test-logger | cleansh sanitize
```

-----

## 9\. Security By Default Principles

`CleanSH` is built with a strong focus on security and trust:

  * **Local-Only Operation:** No network calls or telemetry.
  * **Immutable Defaults:** Built-in rules are embedded at compile time.
  * **No Runtime Evaluation:** All redaction is done via static regex.
  * **ANSI Stripping:** All input is pre-sanitized of escape codes.
  * **Programmatic Validation:** Critical PII rules have additional code-based checks.

-----

**Precision redaction. Local-only trust. Built for devs.**

*Copyright 2025 Obscura Tech.*