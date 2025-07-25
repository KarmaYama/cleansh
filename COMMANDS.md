# Cleansh Command Handbook

Welcome to the **Cleansh Command Handbook**\! This guide dives deeper into `cleansh`, your indispensable command-line utility for sanitizing sensitive information from terminal output. Whether you're a developer, system administrator, or anyone who values privacy and security in their shared data, `cleansh` empowers you to control what leaves your terminal.

We'll go beyond just listing commands, exploring scenarios, use cases, and how to harness `cleansh`'s full potential, including building your own custom rules.

-----

## 1\. What is Cleansh?

At its core, `cleansh` (pronounced "clean shell") is a high-trust, single-purpose CLI tool designed to redact sensitive data from text streams. Think of it as your digital bouncer, ensuring confidential information like IP addresses, email addresses, API keys, and even personal identifiers never accidentally leak when you're sharing logs, debugging output, or collaborating with others.

It operates locally, requires zero configuration to get started with its robust default rules, and offers extensive flexibility for custom needs. Whether you pipe output directly from other commands or feed it files, `cleansh` is built for secure, efficient, and precise data sanitization.

-----

## 2\. Getting Started with Cleansh

Before diving into the commands, make sure you have `cleansh` installed.

### Installation

The **recommended** way to get `cleansh` is by downloading the latest prebuilt binaries for your platform from the [GitHub Releases](https://www.google.com/search?q=https://github.com/KarmaYama/cleansh/releases) page.

Alternatively, you can use the install script:

```bash
curl -sSf https://github.com/KarmaYama/cleansh/releases/download/v0.1.5/cleansh-installer.sh | sh
```

If you have Rust's Cargo installed, you can also install it directly:

```bash
cargo install cleansh # Use `cargo install cleansh --force` to update
```

For the adventurous, you can build from source:

```bash
git clone https://github.com/KarmaYama/cleansh.git
cd cleansh
cargo build --release
cargo test
```

Once installed, you're ready to start cleaning your output\!

-----

## 3\. Cleansh in Action: Core Sanitization

The primary purpose of `cleansh` is to redact sensitive data. By default, it reads from standard input (`stdin`) and writes the sanitized content to standard output (`stdout`).

### Basic Usage: Piping Content

The most common way to use `cleansh` is by piping the output of another command into it.

**Scenario:** You're debugging an application, and its logs contain an email address and an internal IP. You want to share these logs with a colleague but without revealing the sensitive information.

```bash
echo "User login attempt from test@example.com at 192.168.1.1." | cleansh
```

**Output:**

```
User login attempt from [EMAIL_REDACTED] at [IPV4_REDACTED].
```

**Why this is useful:** Quick and effortless sanitization directly in your workflow. The original sensitive data never leaves your terminal window unmasked.

### Sanitizing File Content

Instead of piping, you can also provide `cleansh` with a file path.

**Scenario:** You have an existing log file, `application.log`, that might contain sensitive data, and you want to create a sanitized version.

```bash
cleansh ./application.log -o sanitized_application.log
```

**Explanation:**

  * `cleansh ./application.log`: Reads the content of `application.log`.
  * `-o sanitized_application.log`: Writes the sanitized output to a new file named `sanitized_application.log`.

**Why this is useful:** Ideal for batch processing or when you need a clean version of a persistent file without modifying the original.

### Revealing Changes with `  --diff `

Sometimes you want to see exactly what `cleansh` redacted. The `--diff` (or `-d`) flag provides a colored, line-by-line comparison of the original and sanitized output.

**Scenario:** You've run `cleansh` on some output, and you want to visually confirm which parts were identified and changed.

```bash
echo "My Stripe key is sk_test_xyz123abc456 and my email is user@domain.com." | cleansh -d
```

**Output (example with color):**

```diff
--- Original
+++ Sanitized
@@ -1,1 +1,1 @@
-My Stripe key is sk_test_xyz123abc456 and my email is user@domain.com.
+My Stripe key is [STRIPE_SECRET_REDACTED] and my email is [EMAIL_REDACTED].
```

**Why this is useful:** Excellent for auditing, debugging your redaction rules, or simply gaining confidence that `cleansh` is working as expected.

### Copying to Clipboard with `--clipboard`

For quick sharing, `cleansh` can directly copy the sanitized output to your system's clipboard using the `--clipboard` (or `-c`) flag.

**Scenario:** You've just run a command that spits out some sensitive configuration, and you want to paste a clean version directly into a chat or document.

```bash
git config --list | cleansh -c
```

**Explanation:** The sanitized output of `git config --list` will be copied to your clipboard, ready for pasting.

**Why this is useful:** Streamlines your workflow by eliminating the need to redirect to a temporary file or manually copy from the terminal.

**Note on Clipboard Support:**

  * **macOS & Windows:** Built-in.
  * **Linux:** Requires `xclip`, `xsel`, or `wl-clipboard` to be installed on your system.

### Suppressing the Redaction Summary (`--no-redaction-summary`)

By default, `cleansh` provides a summary of all redacted items. In some automated scripts or when piping output to another tool, you might want to suppress this summary.

**Scenario:** You're integrating `cleansh` into a script where only the sanitized text is needed, and any extra output to `stderr` would interfere.

```bash
myscript_with_secrets.sh 2>&1 | cleansh --no-redaction-summary | grep "IMPORTANT"
```

**Explanation:** The `grep` command will only receive the sanitized output, free from the redaction summary.

**Why this is useful:** Ensures clean, predictable output for further processing by other tools or scripts.

-----

## 4\. Understanding Cleansh's Rule System

`cleansh` comes with a robust set of default rules for common sensitive data types. However, its true power lies in its flexibility to manage and customize these rules.

### Default Redaction Categories

`cleansh` accurately masks:

  * **Emails** (e.g., `user@example.com`)
  * **IP Addresses** (IPv4 and uncompressed IPv6)
  * **Various Tokens & Secrets**: JWTs, GitHub PATs (classic and fine-grained), Stripe keys, AWS Access/Secret Keys, GCP API Keys, Google OAuth tokens, SSH private keys, and generic hex or alphanumeric tokens.
  * **Personal Identifiable Information (PII)**: Credit Card Numbers, US Social Security Numbers (SSN), UK National Insurance Numbers (NINO), and South African ID Numbers. Some of these, like SSN and NINO, include **programmatic validation** for enhanced accuracy and reduced false positives.
  * **Paths & URLs**: Linux/macOS Absolute Paths (rewritten to `~` equivalents for brevity and privacy), Windows Absolute Paths, and Slack Webhook URLs.
  * **Authentication Headers**: HTTP Basic Auth Headers.

### Custom Rule Management

`cleansh` provides powerful ways to tailor its behavior:

#### Loading Custom Rules with `--config`

You can define your own redaction rules in a YAML file and tell `cleansh` to use them. These custom rules will be **merged with Cleansh's built-in defaults**, and any custom rule with the same `name` as a default rule will override it.

**Scenario:** Your company uses a specific internal employee ID format (`EMP-XXXXX`) that `cleansh` doesn't know about by default. You want to redact these.

1.  **Create a YAML file** (e.g., `my_custom_rules.yaml`):

    ```yaml
    rules:
      - name: "emp_id"
        pattern: 'EMP-\d{5}'
        replace_with: '[EMPLOYEE_ID_REDACTED]'
        description: "Company-specific employee ID"
        multiline: false
        dot_matches_new_line: false
    ```

2.  **Use it with `cleansh`:**

    ```bash
    echo "Employee ID is EMP-12345, email is test@company.com." | cleansh --config ./my_custom_rules.yaml
    ```

    **Output:**

    ```
    Employee ID is [EMPLOYEE_ID_REDACTED], email is [EMAIL_REDACTED].
    ```

**Why this is useful:** Essential for extending `cleansh` to protect company-specific sensitive data or to customize how existing categories are redacted.

#### Enabling/Disabling Specific Rules

You have granular control over which rules are active using `--enable-rules` and `--disable-rules`. These flags accept a comma-separated list of rule names.

  * **`--enable-rules <names>`:** Explicitly activates `opt_in` redaction rules that are otherwise inactive by default.
  * **`--disable-rules <names>`:** Explicitly deactivates *any* redaction rules by name, overriding defaults or rules enabled via `--config`. This is useful for reducing false positives for specific patterns in your environment.

**Scenario 1: Activating an `opt_in` rule.**
Some rules, like `aws_secret_key` or generic hex secrets, are `opt_in` by default because they have a higher false positive risk. You know your data structure, and you want to specifically look for AWS Secret Keys.

```bash
echo "My AWS secret key is aB1c2D3e4F5g6H7i8J9k0L1m2N3o4P5q6R7s8T9u." | cleansh --enable-rules aws_secret_key
```

**Scenario 2: Disabling a rule that causes false positives.**
If the `ipv4_address` rule is causing false positives by redacting values that *look* like IPs but aren't (e.g., version numbers or identifiers), you might want to disable it entirely for certain operations.

```bash
echo "Firmware version 1.2.3.4, system IP 192.168.1.100." | cleansh --disable-rules ipv4_address
```

**Output:**

```
Firmware version 1.2.3.4, system IP 192.168.1.100.
```

**Why this is useful:** This precise control helps `cleansh` adapt to your specific data environment, minimizing unintended redactions while ensuring sensitive data is still caught by other active rules.

#### Rule Configurations: `default` vs. `strict`

The `--rules <name>` flag allows you to select a predefined set of rules.

  * **`--rules default`**: This is the default behavior if `--rules` isn't specified. It loads all non-opt-in rules from the default configuration and any custom rules you've loaded with `--config`. Opt-in rules are *not* active unless explicitly enabled with `--enable-rules`.
  * **`--rules strict`**: This configuration activates *all* rules, including those marked as `opt_in` in your default or custom configurations. Use this when you need the most comprehensive redaction, even if it might lead to more false positives.

**Scenario:** You're performing a deep security audit and want `cleansh` to be as aggressive as possible in finding potential secrets, including generic hex strings that might be secret keys.

```bash
echo "My SHA256 hash: 01ba4719c80b6fe911b091a7051e4881ad0cfb939f3d906109968a867776263a" | cleansh --rules strict --enable-rules generic_hex_secret_64
```

**Why this is useful:** Provides a quick way to switch between a balanced, low-false-positive rule set (`default`) and a highly aggressive, comprehensive rule set (`strict`).

-----

## 5\. Building Your Own Custom Rules

Creating effective custom rules for `cleansh` involves understanding regular expressions and the `RedactionRule` structure.

### Anatomy of a `RedactionRule`

Each rule in your YAML file has several key fields:

  * **`name` (string, required):** A unique identifier for your rule (e.g., `"emp_id"`). Use snake\_case for consistency.
  * **`pattern` (string, required):** The regular expression that `cleansh` will use to find sensitive data. This is the core of your rule.
  * **`replace_with` (string, required):** The string that will replace any matches found by the `pattern` (e.g., `"[EMPLOYEE_ID_REDACTED]"`).
  * **`description` (string, optional):** A brief explanation of what the rule matches. Good for documentation.
  * **`multiline` (boolean, default: `false`):** If `true`, the `^` and `$` anchors in your pattern will match the start/end of *lines* within the input, not just the start/end of the entire input string. Useful for patterns that span multiple lines or need to anchor to line beginnings/ends.
  * **`dot_matches_new_line` (boolean, default: `false`):** If `true`, the `.` (dot) character in your pattern will also match newline characters (`\n`). This is crucial for patterns that might span multiple lines, like SSH keys.
  * **`opt_in` (boolean, default: `false`):** If `true`, this rule will only be active if explicitly enabled via `--enable-rules` or if the `--rules strict` configuration is used. Use this for patterns that are prone to false positives.
  * **`programmatic_validation` (boolean, default: `false`):** If `true`, `cleansh` will attempt to apply additional, code-based validation after a regex match (e.g., for US SSNs or UK NINOs). This significantly increases accuracy for specific structured data but requires an internal validator function to exist. **You cannot define new programmatic validators in custom rules**, but you can use this flag if you're overriding an existing default rule that *already* has programmatic validation, and you want to maintain that behavior.

### Regular Expression Best Practices

  * **Be Specific:** The more specific your regex, the fewer false positives. Use word boundaries (`\b`) where appropriate.
  * **Test Thoroughly:** Use online regex testers (e.g., regex101.com) to validate your patterns against various test cases, including edge cases and non-matches.
  * **Escaping:** Remember to escape special regex characters (like `.`, `*`, `+`, `?`, `(`, `)`, `[`, `]`, `{`, `}`, `|`, `^`, `$`, `\`) with a backslash if you want to match them literally.
  * **YAML Multi-line Strings:** For complex or multi-line patterns, use YAML's literal block scalar `|-` or folded block scalar `>`- syntax to define your pattern cleanly. This is particularly useful for patterns that contain backslashes, as it avoids extra escaping.

**Example: A Custom API Key Rule**

Let's say your internal API keys always start with `APIKEY_` followed by 20 alphanumeric characters.

```yaml
# my_api_keys.yaml
rules:
  - name: "internal_api_key"
    pattern: '\bAPIKEY_[A-Za-z0-9]{20}\b'
    replace_with: '[INTERNAL_API_KEY_REDACTED]'
    description: "Company-specific internal API key format."
    multiline: false
    dot_matches_new_line: false
    opt_in: false # This rule will be active by default if loaded
    programmatic_validation: false
```

Then use it:

```bash
echo "My internal system uses APIKEY_aBcDeFgHiJkLmNoPqRsT. Access granted." | cleansh --config my_api_keys.yaml
```

-----

## 6\. Advanced Usage & Scenarios

### Combined Flags for Complex Workflows

`cleansh` flags are designed to be composable, allowing you to build powerful, tailored commands.

**Scenario:** You need to sanitize Docker logs, save the cleaned version to a file, and also see a diff of what changed, while only looking for specific `github_pat` tokens.

```bash
docker logs my-app-container | cleansh -o sanitized_docker.log -d --enable-rules github_pat
```

**Explanation:**

  * `docker logs my-app-container`: Streams logs from your Docker container.
  * `| cleansh`: Pipes these logs into `cleansh`.
  * `-o sanitized_docker.log`: Saves the sanitized output to `sanitized_docker.log`.
  * `-d`: Displays a live diff of changes to your terminal's `stderr`.
  * `--enable-rules github_pat`: Ensures that only GitHub PATs are targeted for redaction (along with any other non-opt-in default rules).

### Debugging Your Rules with Logging

`cleansh` uses `env_logger` for its internal logging. You can control the verbosity using the `RUST_LOG` environment variable or by using `cleansh`'s dedicated flags. This is invaluable when building or troubleshooting custom rules.

  * **`--debug`:** Sets the log level to `debug` for `cleansh`'s internal operations. This is highly verbose and will print detailed information about rule compilation, matching, and decisions to `stderr`.
  * **`--no-debug`:** Explicitly disables debug logging (equivalent to `RUST_LOG=warn`).
  * **`--quiet`:** Sets the log level to `error`, suppressing warnings and info messages.
  * **`CLEANSH_ALLOW_DEBUG_PII=1`:** (Environment Variable) **Use with extreme caution\!** If set, `cleansh` will log the *original content* of matched sensitive strings during debug mode. This is strictly for development and testing in isolated, secure environments and should **never** be used in production or on real sensitive data.

**Scenario:** Your new custom rule isn't matching as expected. You want to see `cleansh`'s internal processing.

```bash
RUST_LOG=debug cleansh --config my_custom_rules.yaml < your_input.txt
# OR
cleansh --debug --config my_custom_rules.yaml < your_input.txt
```

**Why this is useful:** Provides deep insight into how `cleansh` is interpreting your rules and processing content, helping you pinpoint issues with regex patterns or rule configurations.

-----

## 7\. Statistics-Only Mode (`--stats-only`)

Beyond simple redaction, `cleansh` offers a powerful `--stats-only` mode for auditing and analysis. This mode identifies potential sensitive data matches without performing any redaction, providing a comprehensive report. It's perfect for pre-scan assessments or integrating into CI/CD pipelines for security checks.

### Basic Statistics

The most straightforward use of `--stats-only` is to get a human-readable summary of detected secrets.

**Scenario:** You want to see what sensitive data `cleansh` would find in a log file without actually changing the file.

```bash
cleansh --stats-only ./audit_log.txt
```

**Output (example to `stderr`):**

```
Redaction Statistics Summary:
  EmailAddress: 2 matches
  IPv4Address: 5 matches
  JWTToken: 1 match
```

**Why this is useful:** Provides a quick overview of sensitive data types present, helping you prioritize remediation efforts.

### Exporting Statistics to JSON (`--stats-json-file`, `--export-json-to-stdout`)

For machine-readable output, especially for integration with other tools or dashboards, `cleansh` can export the statistics as JSON.

  * **`--stats-json-file <path>`:** Writes the JSON summary to a specified file.
  * **`--export-json-to-stdout`:** Prints the JSON summary directly to `stdout`. **Important:** When this flag is used, `cleansh` will suppress all other human-readable output (like the summary to `stderr`) to ensure pure JSON output.

**Scenario 1: Saving statistics to a report file.**

```bash
my_pipeline_output | cleansh --stats-only --stats-json-file pipeline_report.json
```

**Scenario 2: Piping statistics to another tool (e.g., `jq`).**

```bash
docker logs my-app-with-secrets | cleansh --stats-only --export-json-to-stdout | jq .redaction_summary.EmailAddress.count
```

**Output (example to `stdout` from `jq`):**

```json
2
```

**Why this is useful:** Enables programmatic analysis and integration of `cleansh`'s detection capabilities into larger security or data governance workflows.

### Sampling Matches (`--sample-matches <count>`)

When generating JSON statistics, you might want to include samples of the actual matched strings for context, but not *all* of them if there are thousands. The `--sample-matches` flag allows you to specify how many unique samples to include per rule.

**Scenario:** You need to see a few examples of detected emails and IP addresses in your JSON report.

```bash
echo "a@b.com, c@d.com, e@f.com, 1.1.1.1, 2.2.2.2, 3.3.3.3" | cleansh --stats-only --export-json-to-stdout --sample-matches 2
```

**Output (example simplified):**

```json
{
  "redaction_summary": {
    "EmailAddress": {
      "count": 3,
      "samples": [
        "a@b.com",
        "c@d.com"
      ]
    },
    "IPv4Address": {
      "count": 3,
      "samples": [
        "1.1.1.1",
        "2.2.2.2"
      ]
    }
  }
}
```

**Why this is useful:** Provides concrete examples for validation or further investigation without overwhelming the report with excessive data.

### Fail-Over Threshold (`--fail-over-threshold <count>`)

This is a critical feature for CI/CD pipelines or automated security gates. If the total number of detected secrets exceeds a specified threshold, `cleansh` will exit with a non-zero status code, signaling a failure.

**Scenario:** Your build pipeline should fail if more than 5 secrets are detected in the build logs.

```bash
./build_and_test.sh 2>&1 | cleansh --stats-only --fail-over-threshold 5
```

**Explanation:**

  * If total secrets found are 5 or less, `cleansh` exits with code 0 (success).
  * If total secrets found are 6 or more, `cleansh` prints an error message to `stderr` and exits with code 1 (failure), causing the pipeline to fail.

**Why this is useful:** Enforces a "security by design" principle by automatically flagging builds or deployments that might introduce too much sensitive data.

### Suppressing Donation Prompts (`--cli-disable-donation-prompts`)

`cleansh` might occasionally prompt for donations to support development. For automated environments, you can disable these prompts.

**Scenario:** Running `cleansh` in a CI/CD pipeline where interactive prompts are not desired.

```bash
cleansh --stats-only --cli-disable-donation-prompts < input_file
```

**Why this is useful:** Ensures uninterrupted execution in non-interactive environments.

-----

## 8\. Logging and Verbosity

`cleansh` uses a flexible logging system to provide feedback on its operations. You can control how verbose `cleansh` is.

  * **Default:** `cleansh` operates relatively quietly, primarily outputting info messages to `stderr` for general operations and errors.
  * **`--debug`:** Activates detailed debug logging. This is invaluable for troubleshooting rule matching, understanding internal processes, and seeing more about how `cleansh` identifies and processes data.
  * **`--no-debug`:** Explicitly mutes debug logging, ensuring only warnings and errors are shown.
  * **`--quiet`:** Suppresses all informational messages and warnings, only displaying critical errors.

**Example:** Seeing verbose debug output.

```bash
echo "Hello World" | cleansh --debug
```

**Important Note on PII Debugging:**
There is an environment variable, `CLEANSH_ALLOW_DEBUG_PII`, which, if set to any value (e.g., `CLEANSH_ALLOW_DEBUG_PII=1`), will allow `cleansh` to log the *original, unredacted content* of matched sensitive strings in debug output. **Only use this in highly controlled, secure, and isolated development or testing environments. Never enable this on production systems or with real sensitive data.**

-----

## 9\. Security By Default Principles

`cleansh` is built with a strong focus on security and trust:

  * **No Runtime Evaluation:** All redaction occurs via static regular expressions. `cleansh` does not execute arbitrary code from patterns.
  * **Local-Only Operation:** No network calls, telemetry, or data transmission. Your sensitive data stays on your machine.
  * **Immutable Defaults:** Built-in rules are embedded at compile time, preventing external tampering.
  * **Path Redaction:** Filesystem paths are intelligently normalized or redacted to prevent revealing system structure.
  * **YAML Sandboxed:** Custom rules are declarative; they define patterns and replacements, not executable code.
  * **Clipboard Opt-in:** Copying to the clipboard requires an explicit flag (`-c`).
  * **ANSI Stripping:** All input content is pre-sanitized of ANSI escape codes to prevent clever evasion techniques.
  * **Programmatic Validation:** Critical PII rules (like US SSN and UK NINO) incorporate additional, code-based validation checks beyond just regex matching, significantly increasing accuracy and reducing false positives.

-----

## 10\. The Road Ahead: Cleansh 1.0.0

While `cleansh` `v0.1.5` offers powerful capabilities, we're actively working towards a transformative `1.0.0` release. This major version aims to introduce **adaptive intelligence** and **user-trainable features**, making `cleansh` even more precise and empowering.

Planned key features include:

  * **Pro Feature: Programmed Memory / Adaptive Redaction:** `cleansh` will learn from explicit user feedback (e.g., marking false positives or negatives) to reduce noise and increase accuracy tailored to your specific environment.
  * **Pro Feature: Simple Interactive Mode:** An interactive capability to pause `cleansh` when a potential secret is detected, prompting you for a decision (redact, ignore once, always ignore this pattern). This will be crucial for training `cleansh`'s memory.
  * **New CLI Flags for Adaptive Control:** Flags like `--interactive` and commands to manage learned patterns (`--forget-learned-pattern`, `--list-learned-patterns`).
  * **Dedicated User Feedback Data Storage:** A secure system to persistently store learned exceptions.

This evolution, coupled with a planned shift to the **PolyForm Noncommercial License 1.0.0** for `v1.0.0` and beyond (previous versions remain MIT licensed), is designed to ensure `cleansh`'s sustainable development and continued innovation.

-----

**Precision redaction. Local-only trust. Built for devs.**

Have you tried out the `--stats-only` command yet? It's a game-changer for auditing\!