---
name: Bug report
about: Create a report to help us improve
title: "[BUG]"
labels: ''
assignees: KarmaYama

---

### **Bug Report: Cleansh**

Thank you for reporting an issue\! To help us understand and resolve the bug efficiently, please provide the following details:

-----

**Describe the Bug**
A clear and concise description of what the bug is and its observed impact.

-----

**To Reproduce**
Please provide a **minimal, reproducible example (MRE)** that reliably demonstrates the bug. This is critical for us to investigate. Your steps should include:

1.  **Cleansh Version:**
    Paste the output of `cleansh --version` here.

    ```bash
    # Example:
    # cleansh 0.1.2
    ```

2.  **Input Data:**
    Provide the exact text or file content that you piped into `cleansh` or read from a file.

      * *If piping from `echo` or a string:*
        ```bash
        # Example:
        echo "My email is user@example.com and my IP is 192.168.1.1." | cleansh
        ```
      * *If reading from a file:*
        ```
        # Content of your_input_file.txt:
        # My sensitive log entry: email@domain.com, token=eyJhbGciOiJIUzI1NiJ9...
        ```
        (Then specify the command used to read from it in step 3, e.g., `cleansh < your_input_file.txt`)

3.  **Command Used:**
    The exact `cleansh` command you executed, including any flags (e.g., `-c`, `-d`, `--config`, `--out`).

    ```bash
    # Example:
    # cleansh -d --config ./custom_rules.yaml < input.log
    ```

4.  **Steps to Reproduce:**
    Provide a clear, numbered list of actions, starting from running the command:

    1.  Execute: `[Your exact cleansh command here]`
    2.  Observe: `[Describe what happens in the terminal/output]`
    3.  Result: `[State the bug or unexpected behavior]`

-----

**Expected Behavior**
A clear and concise description of what you expected to happen when you ran the command (e.g., "I expected the email address to be redacted to `[EMAIL_REDACTED]` and no diff to be shown.").

-----

**Observed Behavior / Terminal Output**
Please copy-paste the *full* terminal output you received, including the command you ran, any error messages, and the resulting `cleansh` output. If applicable, screenshots of the terminal (especially for diff views or colored output issues) can also be helpful.

```
# Paste your terminal output here
```

-----

**Environment:**
Please complete the following information about your operating environment:

  * **Operating System:** [e.g., `Windows 10 Pro (Build 19045)`, `Ubuntu 22.04 LTS (x86_64)`, `macOS Sonoma 14.4 (Apple Silicon)`]
  * **Rust Toolchain Version (if applicable):**
    Paste the output of `rustc --version` here if you built `cleansh` from source or are a Rust developer.
    ```bash
    # Example:
    # rustc 1.79.0 (171ae91b3 2024-06-04) (built from a source tarball)
    ```
  * **Terminal Emulator:** [e.g., `Windows Terminal`, `iTerm2`, `GNOME Terminal`, `cmd.exe`, `PowerShell`, `Alacritty`]
  * **Shell:** [e.g., `bash`, `zsh`, `fish`, `PowerShell`]

-----

**Additional Context**
Add any other context about the problem here that might be relevant. This could include:

  * Whether the bug is consistent or intermittent.
  * Any recent changes to your system or `cleansh` configuration.
  * Details of any custom configuration files (`--config`) used.
  * If you're building `cleansh` from source, please include the Git commit hash: `git rev-parse HEAD`.

-----
