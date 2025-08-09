# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.8] - 2025-08-08 — Core Engine Refactoring, Engine Abstraction & CLI Improvements

This release introduces a major architectural refactoring of the core sanitization engine. It abstracts the redaction logic behind a trait, enabling multiple backends and improving the application's extensibility. This version also streamlines CLI flags and centralizes key logic into dedicated helper functions, making the codebase more modular, maintainable, and robust.

---

### Added

* **Engine Abstraction (`SanitizationEngine` Trait):** Introduced a new trait to enable different sanitization backends. The primary `RegexEngine` is now a concrete implementation of this trait, setting the stage for future engines like the placeholder `Entropy` engine.
* **New `--engine` CLI Flag:** A new flag that allows users to explicitly select a sanitization engine (e.g., `--engine regex` or `--engine entropy`). This makes the application extensible and ready for new redaction methods.
* **Helper Functions:** Centralized input reading (`read_input`), engine creation (`create_sanitization_engine`), and main command handling (`handle_main_command`) into dedicated helper functions. This reduces code duplication and improves the modularity of the `main.rs` file.
* **Line-Buffered Mode Refinement:** The `run_line_buffered_mode` function has been refactored to be more robust, now aggregating match counts and printing a summary at the end of the stream. It also uses the new engine abstraction.
* **Testability Improvements:** Added an environment variable override (`CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS`) to allow a specific path for the application's state file. This is crucial for running isolated and non-intrusive integration tests.

---

### Changed

* **Simplified CLI Flags:** The redundant `--no-clipboard` and `--no-diff` flags have been removed. The `--clipboard` and `--diff` flags now default to `false` and can be toggled on explicitly. This aligns with standard CLI design principles and simplifies argument parsing.
* **Unified Input Handling:** All input-reading logic for the main command and the `stats` subcommand now uses the centralized `read_input` helper function, removing redundant code.
* **Refactored Command Execution:** The main command logic is now encapsulated within the `handle_main_command` function. It now accepts the `SanitizationEngine` as a parameter, clearly separating the concern of engine creation from command execution.
* **App State Management:** The logic for checking donation prompts and saving the application state is now handled within the specific command `match` arms. This ensures the state is saved only after a command has been fully executed, and usage counters are incremented at the appropriate time.
* **Dependency Updates:** `cleansh_core` is now used to import core engine and config types, improving dependency management and code clarity.
* **Streamlined `cleansh` Command:** The `run_cleansh_opts` function has been simplified to accept the new `SanitizationEngine` trait and a `CleanshOptions` struct. Its internal logic has been broken down into smaller, single-purpose helper functions for handling primary output, clipboard output, and the redaction summary, significantly improving readability and maintainability. The old, backward-compatible `run_cleansh` function has been removed.
* **Refactored `stats` Command:** The `run_stats_command` function now uses the `SanitizationEngine` trait, removing the need for it to handle rule loading and compilation. Application state management and donation prompt logic have been moved out of this module and into the main application flow. The function signature has been simplified to accept a `StatsCommand` struct, consolidating all related CLI options.
* **Decoupled UI Logic:** The UI modules (`diff_viewer`, `output_format`, and `redaction_summary`) have been refactored for improved modularity. The logic for determining whether to use ANSI colors has been moved out of these modules and into the higher-level command functions. UI functions now accept an explicit `enable_colors` boolean flag, making them more flexible, predictable, and easier to test.

---

### Fixed

* **Improved Incompatibility Checks:** The compatibility checks for the `--line-buffered` flag are now combined into a single `if` statement, which is clearer and less error-prone.
* **Correct App State Persistence:** The `--disable-donation-prompts` flag now correctly overrides the saved application state, ensuring that the user's intent from the command line is always prioritized.

---

---

*This release represents a major leap forward in `CleanSH`'s accuracy, testability, and secure-by-default foundation. It lays the groundwork for future enhancements such as entropy-based token detection, contextual redaction.*

---

