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
| Clipboard       | `copypasta`                        |
| Logging         | `log` + `env_logger`               |
| Error handling  | `anyhow` + `thiserror`             |
| Install method  | `cargo-dist` + curl script or `cargo install` |
| License         | MIT                                |

```
```