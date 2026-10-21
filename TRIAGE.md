# Bug Report Triage Process

This document outlines the operational process, response Service Level Agreements (SLAs), severity assessment matrix, bounty reward guidelines, and communication templates for triaging bug reports in the Soroban Cookbook repository.

---

## 👥 Triage Team Roles

To ensure all community contributions and vulnerability reports are processed efficiently, we define the following roles:

*   **Lead Triager:** First responder to incoming GitHub issues and security advisories. Responsible for initial screening, reproducing bugs, and assigning severity.
*   **Security Reviewers:** Core developers who evaluate high/critical security reports, review code patches, and audit proposed fixes.
*   **Community Manager:** Coordinates public communication, updates the changelog, and manages reward payouts with sponsors.

---

## ⏱️ Response SLAs

We commit to the following response and resolution targets based on issue severity:

| Severity | First Response SLA | Resolution / Patch SLA | Public Disclosure |
| :--- | :--- | :--- | :--- |
| **Critical** | < 12 Hours | < 36 Hours | Immediate after patch is deployed |
| **High** | < 24 Hours | < 72 Hours | Scheduled release disclosure |
| **Medium** | < 48 Hours | < 5 Days | Listed in standard release notes |
| **Low** | < 72 Hours | Next Release | Standard issue tracker closure |

---

## 📊 Severity Assessment Matrix

We align bug impact using the following four-tier classification:

### 1. Critical Severity
*   **Definition:** Flaws that cause immediate, catastrophic risk to the platform, smart contracts, or user funds.
*   **Examples:**
    *   Direct drainage of locked assets in DeFi examples.
    *   Authentication bypass allowing unauthorized admin/governance actions.
    *   Reentrancy or compiler-specific bugs causing permanent contract bricking.

### 2. High Severity
*   **Definition:** Vulnerabilities or defects that compromise core functionalities under specific but realistic scenarios.
*   **Examples:**
    *   Temporary locking of funds or denial of service (DoS) of a contract.
    *   Bypassing fee structures or minor math errors leading to leakage of value.
    *   Broken state transitions in governance/voting logic.

### 3. Medium Severity
*   **Definition:** Operational bugs or validation failures that do not put funds at direct risk but degrade usability.
*   **Examples:**
    *   Lack of TTL extension on storage keys leading to premature key expiration.
    *   Missing input validations that cause transactions to fail gas-insufficiently.
    *   Inconsistent contract behavior across different simulation endpoints.

### 4. Low Severity
*   **Definition:** Non-breaking anomalies, documentation errors, or minor style discrepancies.
*   **Examples:**
    *   Typos, broken links, or outdated command flags in READMEs.
    *   Inefficient gas patterns (missing optimization) without functional impact.
    *   Compiler warnings or clippy suggestions.

---

## 💰 Reward & Bounty Determination

To incentivize responsible disclosure, the Cookbook program offers rewards based on severity:

*   **Critical:** $2,500 – $10,000 USD (paid in XLM/USDC) + Hall of Fame listing.
*   **High:** $1,000 – $2,500 USD (paid in XLM/USDC).
*   **Medium:** $250 – $1,000 USD.
*   **Low:** $50 – $250 USD or exclusive Soroban Cookbook Swag.

*Note: Rewards are only paid for unique, reproducible issues affecting codebase correctness or security. Code style recommendations or duplicates do not qualify.*

---

## 💬 Communication Templates

### 1. Initial Receipt & Triage
```markdown
Hi @{username},

Thank you for reporting this issue! 

Our triage team has received your report. We are currently reviewing the details and attempting to reproduce the behavior locally. 

- **Assigned Severity:** [Pending / Low / Medium / High / Critical]
- **Target Response SLA:** We will provide a status update within [12/24/48] hours.

We appreciate your contribution to keeping the Soroban Cookbook secure and reliable.
```

### 2. Request for Reproduction
```markdown
Hi @{username},

We are attempting to reproduce the reported issue but require additional context. Could you please provide:
1. The exact version of the `soroban-sdk` and `stellar-cli` (or `soroban-cli`) you are using.
2. A minimal Rust test case or a script showing the failed assertion/transaction.
3. The specific network/sandbox parameters (standalone, testnet, or local simulation).

Once we have this information, we will proceed with the assessment.
```

### 3. Bug Confirmed & Patch Pending
```markdown
Hi @{username},

We have successfully reproduced the bug and confirmed the issue. 

- **Final Severity:** [Low/Medium/High/Critical]
- **Resolution Plan:** We are preparing a patch under PR #[PR_NUMBER].
- **Reward Status:** This report is eligible for a **[Level]** bounty. We will contact you shortly to coordinate payment.

Thank you again for your valuable report!
```

### 4. Issue Closed as Duplicate / Out of Scope
```markdown
Hi @{username},

Thank you for taking the time to write this report. 

Upon review, we found that this issue has already been reported in #{original_issue_number} and is currently being addressed by the team. We will consolidate discussions in that thread.

[OR]

Upon review, we have determined that this behavior is by design/out of scope because [Reason]. 

We appreciate your interest in the Soroban Cookbook and hope to see more contributions from you in the future.
```
