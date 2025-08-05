# Contributing to CleanSH

First and foremost, thank you for considering contributing to `CleanSH`\! Your interest, whether through bug reports, feature requests, or code contributions, is incredibly valuable. `CleanSH` is a high-trust, single-purpose CLI tool, and we believe that community involvement is key to its security, robustness, and continued growth.

This document outlines the guidelines for contributing to the `CleanSH` project. Following these guidelines helps ensure a smooth and effective collaboration for everyone involved.

-----

## 💡 How Can I Contribute?

There are many ways to contribute to `CleanSH`, even if you're not a Rust expert:

  * **Reporting Bugs:** Found an issue? Let us know\!
  * **Suggesting Features:** Have an idea to make `CleanSH` even better? We'd love to hear it.
  * **Improving Documentation:** Clarity in documentation is crucial. If something is unclear or missing, consider suggesting improvements.
  * **Writing Code:** This includes bug fixes, new features, or performance enhancements.
  * **Providing Feedback:** General feedback on usability, design, or anything else is always welcome.

-----

## 🐞 Reporting Bugs

If you encounter a bug or unexpected behavior, please help us by reporting it. Before opening a new issue, please:

1.  **Search Existing Issues:** Check the [GitHub Issues](https://www.google.com/search?q=https://github.com/KarmaYama/cleansh/issues) to see if the bug has already been reported.
2.  **Provide Clear Information:** If it's a new bug, open a [new issue](https://www.google.com/search?q=https://github.com/KarmaYama/cleansh/issues/new/choose) and include:
      * A clear and concise **description** of the bug.
      * **Steps to reproduce** the behavior (e.g., specific command, input data, configuration).
      * The **`CleanSH` version** you are using (e.g., `v0.1.2`).
      * Your **operating system** and environment details (e.g., Windows 10, macOS Ventura, Ubuntu 22.04, PowerShell, Bash, Zsh).
      * Any **error messages** or stack traces.
      * Expected vs. Actual behavior.

-----

## ✨ Suggesting Enhancements

We're always looking for ways to improve `CleanSH`\! If you have an idea for a new feature or an enhancement to existing functionality:

1.  **Search Existing Issues:** Check for similar suggestions first.
2.  **Open a Feature Request:** Open a [new issue](https://www.google.com/search?q=https://github.com/KarmaYama/cleansh-workspace/issues/new/choose) and choose the "Feature Request" template.
      * Clearly describe the **proposed feature** and its **purpose**.
      * Explain the **problem it solves** or the **value it adds**.
      * Provide any **examples** of how it might be used.
      * Consider potential **impacts** (performance, security, usability).

-----

## 👩‍💻 Code Contributions

Ready to dive into the code? We appreciate well-crafted contributions that adhere to the project's standards.

### Setting Up Your Development Environment

1.  **Fork the Repository:** Start by forking the [CleanSH repository](https://github.com/KarmaYama/cleansh-workspace/cleansh) to your GitHub account.
2.  **Clone Your Fork:**
    ```bash
    git clone https://github.com/KarmYama/cleansh-workspace.git
    cd cleansh
    ```
3.  **Install Rust:** If you don't have Rust installed, follow the instructions on [rustup.rs](https://rustup.rs/). `CleanSH` uses the latest stable Rust version (currently `1.88.0` as per `Cargo.toml`).
4.  **Build and Test:**
    ```bash
    cargo build
    cargo test
    ```
    Ensure all tests pass before making changes.

### Making Changes

1.  **Create a New Branch:** Always work on a new branch for your changes. Use a descriptive name like `fix/windows-paths` or `feat/add-json-redaction`.
    ```bash
    git checkout -b your-feature-or-bugfix-branch
    ```
2.  **Code Style:** We follow standard Rust formatting. Please run `cargo fmt` before committing.
    ```bash
    cargo fmt
    ```
3.  **Tests:**
      * For bug fixes, add a test that reproduces the bug and then passes with your fix.
      * For new features, add comprehensive unit and integration tests covering the new functionality and edge cases. Refer to the `src/tests/` and the `Integration Tests` section in the `README.md` for examples.
4.  **Commit Messages:** Write clear, concise, and descriptive commit messages. A good commit message explains *what* was changed and *why*.
      * Use the present tense ("Add feature" instead of "Added feature").
      * Limit the first line to 72 characters.
      * Reference relevant issue numbers (e.g., `Fix #123`).

### Submitting Your Pull Request (PR)

1.  **Push Your Changes:** Push your local branch to your forked repository.
    ```bash
    git push origin your-feature-or-bugfix-branch
    ```
2.  **Open a Pull Request:** Go to the [cleansh GitHub repository](https://github.com/KarmaYama/cleansh-workspace) and open a new Pull Request.
      * Provide a clear **title** and detailed **description** of your changes.
      * Reference the issue your PR addresses (e.g., `Closes #123`, `Fixes #456`).
      * Explain your **solution** and any **design decisions**.
      * Mention that you've run `cargo fmt` and that tests pass.
      * Include any relevant `cleansh` command examples if your PR modifies behavior.
3.  **Code Review:** Your PR will be reviewed by maintainers. Be prepared for feedback and discussions. We may ask for changes or clarifications.
4.  **Address Feedback:** Make requested changes and push new commits to your branch. The PR will update automatically.
5.  **Merge:** Once approved, your changes will be merged into the `main` branch\!

-----

## 🛡️ Security Vulnerabilities

We prioritize the security of `cleansh`. If you believe you've found a security vulnerability, please **DO NOT** open a public GitHub issue. Instead, please follow our [Security Policy](SECURITY.md) for responsible disclosure.

-----

## ❓ Questions?

If you have any questions about contributing, don't hesitate to open a [discussion on GitHub](https://www.google.com/search?q=https://github.com/KarmaYama/cleansh-workspace/discussions) or reach out to the maintainers.

We're excited to see your contributions\!

-----
