# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.8] - 2025-08-08 — Core Engine Refactoring & CLI Improvements

This release is a major architectural overhaul, focusing on making `CleanSH` more **modular, maintainable, and extensible**. We've introduced an abstraction layer for the sanitization engine, streamlined the command-line interface, and separated core logic into reusable functions. This release also marks the introduction of our new **Pro tier** with license-gated features.

---

### Added

* **Engine Abstraction:** A new `SanitizationEngine` trait allows for different sanitization backends, starting with the primary `RegexEngine`. This lays the groundwork for future engines (e.g., entropy-based detection).
* **`--engine` Flag:** A new CLI flag to explicitly select the sanitization engine, making `CleanSH` more flexible and adaptable.
* **Pro Tier Features:** New license-gated features are now available to support commercial use. The application now checks for a valid license token before running specific commands.
* **New Subcommands:** A new, more intuitive subcommand structure has been introduced.
    * `cleansh sanitize`: The core redaction command.
    * `cleansh scan`: Audits for secrets without redacting. This command is now part of the **Pro Tier**.
    * `cleansh profiles`: Manages redaction profiles. This is also a **Pro Tier** feature.
* **Testability Improvements:** Added an environment variable override for the application state file path, allowing for isolated and non-intrusive integration tests.
* **Line-Buffered Mode Summary:** The line-buffered mode now aggregates match counts and prints a summary upon completion.

---

### Changed

* **Code Modularity:** All key logic, including input reading, engine creation, and command handling, has been centralized into dedicated helper functions. This significantly improves readability and reduces code duplication.
* **Simplified CLI Flags:** The `--no-clipboard` and `--no-diff` flags have been removed. The `--clipboard` and `--diff` flags now default to `false` and must be explicitly enabled.
* **State Management:** The logic for donation prompts and application state persistence has been moved to the main command loop to ensure usage counters are incremented and saved correctly after each command execution.
* **`cleansh_core` Integration:** The `cleansh` CLI now relies on the `cleansh_core` library for core engine and config types, improving dependency management.
* **Streamlined Functions:** The `run_cleansh_opts` and `run_stats_command` functions have been refactored to be more modular and accept the new `SanitizationEngine` trait.
* **UI Decoupling:** The UI modules have been refactored to accept an explicit `enable_colors` flag, making them more flexible and easier to test.
* **Command Restructure:** The previous single-command CLI has been replaced by subcommands for a more intuitive user experience.

---

### Fixed

* **Incompatibility Checks:** The `--line-buffered` flag now has a clearer and more robust single-statement check for incompatible flags.
* **App State Override:** The `--disable-donation-prompts` flag now correctly overrides the saved application state, prioritizing the user's immediate command-line intention.

---