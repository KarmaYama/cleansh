# `CleanSH-core` - Core Sanitization Library

[![CodeQL](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/github-code-scanning/codeql) [![CodeQL Advanced](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/codeql.yml) [![Dependabot Updates](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/dependabot/dependabot-updates) [![Release](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/release.yml) [![Rust CI](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml/badge.svg)](https://github.com/KarmaYama/cleansh/actions/workflows/rust.yml)

This crate provides the core logic for sensitive data redaction and validation used by the `CleanSH` CLI application. This library is not meant for consumers. It was specifically designed for cleansh cli. Though you can depend on it at your own risk as public api is unstable and we have no plans in releasing a readme explaining how this library works yet.

For details on how to use the `CleanSH` CLI, please refer to the [main CLI documentation](./../cleansh/README.md).

Further documentation and API details for `CleanSH-core` will be provided upon its independent release to `crates.io`.