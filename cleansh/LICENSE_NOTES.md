# CleanSH License Notes

This document provides detailed information regarding the licensing of CleanSH, particularly concerning the transition to the PolyForm Noncommercial License 1.0.0, which became effective with `v0.1.5`.

## License Summary
* **Versions < v0.1.5:** Remain under the permissive **MIT License**. These versions are not actively maintained and do not receive security patches or new features.
* **Versions >= v0.1.5:** Are licensed under the **PolyForm Noncommercial License 1.0.0**. This license permits noncommercial use and an evaluation period for commercial entities, as detailed below.

## Commercial Use Defined
"Commercial Use" includes, but is not limited to:
* Any use by for-profit entities or organizations.
* Use in a production environment for a commercial product or service.
* Use that directly or indirectly contributes to commercial advantage or monetary compensation.
* Use by government agencies, unless covered by a separate agreement.

---

## CleanSH Tiers & Feature Breakdown

We have categorized CleanSH's features into two tiers to provide a clear understanding of what is available for noncommercial use and what is reserved for commercial licenses.

### Free Tier (PolyForm Noncommercial License 1.0.0)
This tier is free for all noncommercial use cases, including personal, academic, research, and hobby projects. It includes all core sanitization functionality and is not subject to in-app license key validation, regardless of commercial use.

**Core Commands Included:**
* `cleansh sanitize`: The primary command for redacting content from stdin or a file.
* `cleansh uninstall`: A utility command to safely remove `cleansh` and its associated files.
* `cleansh profiles list`: Lists all available redaction profiles.

### Pro Tier (Requires Commercial License)
The Pro tier includes all features necessary for enterprise integration, team collaboration, and automated workflows. The use of these features requires a valid commercial license key, and they are subject to an in-app license check.

**Pro Commands & Features Included:**
* `cleansh scan`: Scans input and provides a detailed redaction summary without altering content. This is designed for security audits and CI/CD pipelines.
* `cleansh profiles sign`: Cryptographically signs a redaction profile for integrity and authenticity.
* `cleansh profiles verify`: Verifies the signature of a signed redaction profile.
* `cleansh profiles sync`: Synchronizes profiles with a central server for team-wide policy enforcement.

---

## Long-Term Plan: CleanSH v1.0.0 and Beyond
With the release of CleanSH version 1.0.0 (expected Q4 2025 or Q1 2026), all commercial use and Pro features will strictly require a valid commercial license key. We understand that commercial use is a broad term. Any feature not tagged as a Pro feature does not fall under the commercial use umbrella regardless of how it is used. This effectively means there will always be a free tier available.

* **Noncommercial Use:** The core CleanSH CLI will remain available for noncommercial use under the PolyForm Noncommercial License 1.0.0.
* **Commercial Use:** Version `1.0.0` and all subsequent versions will incorporate an in-app license key validation mechanism. Commercial entities will be required to purchase a license key to continue using these versions.
* **Version Support:** After version 1.0.0 is released, we will no longer be supporting 0.x.x versions of CleanSH with security patches or feature updates.

## How to Obtain a Commercial License
To obtain a commercial license for CleanSH, please visit our official website (URL to be provided upon v1.0.0 release) or contact us directly at [licenses@obscuratech.tech](mailto:licenses@obscuratech.tech).

## Policy Enforcement
Any commercial use of CleanSH from version 1.0.0 onwards without a valid license key is a violation of the Commercial Use Policy. This policy is in place to ensure the project's sustainability and continued development for all users.