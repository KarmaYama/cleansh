# Cleansh Workspace – A Monorepo for Secure Terminal Output Sanitization

[![Downloads from crates.io](https://img.shields.io/crates/d/cleansh.svg?style=for-the-badge&labelColor=334155&color=4FC3F7)](https://crates.io/crates/cleansh) [![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![Dependabot Updates](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![Rust CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml) [![Star](https://img.shields.io/github/stars/KarmaYama/cleansh.svg?style=social)](https://github.com/KarmaYama/cleansh/stargazers)


## Overview

This repository (`cleansh-workspace`) is a **Rust monorepo** designed for the secure sanitization of terminal output. It houses a growing ecosystem of tools and libraries under a unified development environment, promoting modularity, reusability, and maintainability.

* **license Notes Changes And Updates:** 

Please check our  [`License Notes`](./cleansh/LICENSE_NOTES.md) for more information.

The primary components currently include:

1.  **`cleansh` (CLI Application):** A high-trust, single-purpose command-line interface tool for redacting sensitive information from terminal output. This is the main user-facing application.
    * **Location:** [`/cleansh`](./cleansh/README.md)
    * **Purpose:** Provides a robust, pre-configured solution for data sanitization with flexible options for custom rules and output formats via the CLI.

2.  **`cleansh-core` (Core Library):** A standalone, reusable Rust library that encapsulates all the fundamental logic for data redaction, configuration management, and sensitive data validation.
    * **Location:** [`/cleansh-core`](./cleansh-core/README.md)
    * **Purpose:** Designed to be highly reliable and independent, enabling its potential future integration into other Rust projects and separate distribution on `crates.io`. The `cleansh` CLI seamlessly integrates with and utilizes this core library.

---

## Getting Started

To explore or contribute to the `cleansh` project:

1.  **Clone the Repository:**
    ```bash
    git clone [https://github.com/KarmaYama/cleansh.git](https://github.com/KarmaYama/cleansh.git)
    cd cleansh 
    ```

2.  **Build the Workspace:**
    The project is a Rust workspace, so you can build all components from the root:
    ```bash
    cargo build --workspace --release
    ```

3.  **Run Tests:**
    Ensure everything is functioning correctly by running the full test suite:
    ```bash
    cargo test --package cleansh --features "test-exposed clipboard"
    ```
    ```bash
    cargo test --package cleansh 
    ```

---

## Navigating the Workspace

* **For `cleansh` CLI details, usage, and installation:**
    Please refer to the dedicated README: **[`cleansh/README.md`](./cleansh/README.md)**

* **For `cleansh-core` library details and API documentation:**
    Please refer to its dedicated README (to be created): **[`cleansh-core/README.md`](./cleansh-core/README.md)**
    *(Note: This README will be developed further when `cleansh-core` is prepared for public consumption/crates.io release.)*

---

## Contributing

We welcome contributions to the `cleansh` workspace! Please see our:

* **[Contributing Guidelines](/cleansh/CONTRIBUTING.md)**
* **[Code of Conduct](/cleansh/CODE_OF_CONDUCT.md)**
* **[Changelog](/cleansh/CHANGELOG.md)**

---

## Licensing

The overall `cleansh` project, including the `cleansh` CLI application, is licensed under the [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Please refer to the specific license notes within the `cleansh` directory for full details on commercial use.

The `cleansh-core` library's licensing will be clearly defined upon its separate release to `crates.io`.

---

**Cleansh Workspace: Modular design for secure and adaptable terminal output sanitization.**
