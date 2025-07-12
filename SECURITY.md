# Security Policy for Cleansh

We take the security of `cleansh` very seriously. We are committed to protecting our users and ensuring the integrity of the tool. This policy outlines our approach to security, including supported versions and how to report vulnerabilities.

---

## Supported Versions

`cleansh` is currently in active development, and we aim to provide security updates for the **latest stable release**. As a command-line utility, `cleansh` does not have "versions" in the traditional sense of long-term support branches like larger software frameworks. Instead, we follow a rapid release cycle, with each new version building upon and enhancing the previous one.

**Therefore, we recommend all users update to the [latest available version on crates.io](https://crates.io/crates/cleansh) to ensure they receive all security patches and bug fixes.**

At this stage of development (pre-v1.0), only the **most recent published version** is actively supported with security fixes.

| Version | Supported          |
| :------ | :----------------- |
| **0.1.x** | :white_check_mark: |
| < 0.1.x | :x:                |

*Note: The table above reflects the current `v0.1.x` series. As `cleansh` matures and reaches `v1.0` and beyond, this policy will be updated to reflect a more structured long-term support model if applicable.*

---

## Reporting a Vulnerability

We deeply appreciate the efforts of security researchers and the open-source community. If you discover a security vulnerability in `cleansh`, we ask that you report it responsibly to give us an opportunity to address it before public disclosure.

**How to Report a Vulnerability:**

1.  **Direct Email:** Please report vulnerabilities by sending an email to `security@cleansh.dev` (assuming you will set up this email address).
2.  **Encryption (Optional but Recommended):** For sensitive disclosures, you may request our PGP key in your initial email for encrypted communication.
3.  **Provide Details:** In your report, please include as much detail as possible:
    * A clear and concise description of the vulnerability.
    * Steps to reproduce the vulnerability.
    * The version of `cleansh` affected (e.g., `v0.1.2`).
    * The operating system and Rust toolchain version you used.
    * Any potential impact or exploit scenario.

**Our Response Process:**

1.  **Acknowledgement:** You can expect an acknowledgment of your report within **2 business days**.
2.  **Assessment:** We will investigate the reported vulnerability promptly. Our team will assess the severity and potential impact.
3.  **Status Updates:** We aim to provide regular updates on the progress of our investigation, typically within **5 business days** of the initial acknowledgment and then as significant progress is made.
4.  **Resolution & Disclosure:**
    * If the vulnerability is confirmed, we will work to develop a fix as quickly as possible.
    * Once a fix is ready, we will coordinate with you on the disclosure timeline. We typically aim for a public disclosure after the fix has been released in a new `cleansh` version.
    * We believe in responsible disclosure and will credit you for your discovery in our release notes and/or security advisory, unless you prefer to remain anonymous.
    * If the vulnerability is declined (e.g., deemed not a security issue or out of scope), we will provide a clear explanation for our decision.

**Please do not disclose potential vulnerabilities publicly until we have had an opportunity to address them.** We are committed to addressing valid concerns promptly and openly.

---

This revised policy is more tailored to `cleansh`'s current stage and provides clear instructions for reporting. Once you've had a chance to review this, let me know if you'd like any adjustments!
