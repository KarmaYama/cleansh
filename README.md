# Cleansh Workspace – A Monorepo for Secure Terminal Output Sanitization

[![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![Dependabot Updates](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![Rust CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml) [![Star](https://img.shields.io/github/stars/KarmaYama/cleansh.svg?style=social)](https://github.com/KarmaYama/cleansh/stargazers)

**Stop relying on leaky regex. Cleansh is a high-trust, modular Rust utility designed to securely and programmatically sanitize sensitive data from your terminal output, logs, and text. Star this repo to follow our journey!**

---

## Overview

This repository (`cleansh-workspace`) is a **Rust monorepo** designed for the secure sanitization of terminal output. It houses a growing ecosystem of tools and libraries under a unified development environment, promoting modularity, reusability, and maintainability.

---

### Key Components

1.  **`cleansh` (CLI Application):** A command-line utility for redacting sensitive information. This is the main user-facing application, built for high-trust and reliability.
    * **Location:** [`/cleansh`](./cleansh/README.md)
    * **Purpose:** Provides a pre-configured solution for data sanitization with flexible options for custom rules and output formats via the CLI.

2.  **`cleansh-core` (Core Library):** A standalone, reusable Rust library that encapsulates the fundamental logic for data redaction and validation.
    * **Location:** [`/cleansh-core`](./cleansh-core/README.md)
    * **Purpose:** Designed to be highly reliable and independent, enabling seamless integration into other Rust projects. The `cleansh` CLI uses this core library.

---

### Getting Started

To explore or contribute to the `cleansh` project:

1.  **Clone the Repository:**
    ```bash
    git clone [https://github.com/KarmaYama/cleansh-workspace.git](https://github.com/KarmaYama/cleansh-workspace.git)
    cd cleansh-workspace 
    ```

2.  **Build the Workspace:**
    The project is a Rust workspace, so you can build all components from the root:
    ```bash
    cargo build --release --features "test-exposed clipboard"
    ```

3.  **Run Tests:**
    Ensure everything is functioning correctly by running the full test suite:
    ```bash
    cargo test --package cleansh --features "test-exposed clipboard"
    ```
---

### **Community and Support**

**We're building `cleansh` together with our users and contributors!** If you have questions, feedback, or want to discuss a new feature, don't hesitate to reach out.

* **Ask a Question or Share an Idea:** Our **[GitHub Discussions](https://github.com/KarmaYama/cleansh-workspace/discussions)** page is the best place to connect with us directly.
* **Report a Bug:** Please open an issue on the **[Issues page](https://github.com/KarmaYama/cleansh-workspace/issues)**. We appreciate detailed bug reports!

---

## License Notes

Please check our [`License Notes`](./cleansh/LICENSE_NOTES.md) for more information.

The overall `cleansh` project, including the `cleansh` CLI application, is licensed under the [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Please refer to the specific license notes within the `cleansh` directory for full details on commercial use.

---

**Cleansh Workspace: Modular design for secure and adaptable terminal output sanitization.**