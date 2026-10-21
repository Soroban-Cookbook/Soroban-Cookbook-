# Soroban Cookbook Grant Application Form

> **Instructions:** Copy this markdown template, fill out all required sections, and submit it either as a GitHub Issue labeled `grant-application` or as a new file in a Pull Request under `grants/proposals/your-project-name.md`.

---

## 1. Project & Applicant Information

### 1.1 Project Meta
- **Project Title:** [e.g., Soroban Decentralized Oracle Aggregator]
- **Target Grant Tier:** [ ] Tier 1: Micro-Grant ($2,500) | [ ] Tier 2: Standard Grant ($2,500–$10,000) | [ ] Tier 3: Strategic Grant ($10,000+)
- **Requested Funding Amount (USD / XLM equivalent):** [$X,XXX]
- **Estimated Project Duration:** [e.g., 6 weeks]
- **Primary Category:** [ ] DeFi | [ ] Security/Auditing | [ ] Developer Tooling | [ ] NFT/Gaming | [ ] Governance | [ ] Education

### 1.2 Applicant / Team Details
- **Primary Contact Name:** [Full Name or pseudonym]
- **GitHub Username(s):** [@handle]
- **Email Address:** [contact@example.com]
- **Discord / Telegram Handle:** [@handle]
- **Organization / Entity (if applicable):** [Company / DAO / Individual]
- **Applicant Country & Timezone:** [Country, UTC+/-X]
- **Team Size:** [Number of developers working on the grant]

---

## 2. Executive Summary & Problem Statement

### 2.1 Project Summary (2–3 sentences)
Provide a concise overview of what you are building, why it is necessary, and how it benefits developers using the Soroban Cookbook.

### 2.2 Problem Statement
- What specific gap, challenge, or developer pain point in the Soroban ecosystem does this project solve?
- Why is the existing documentation or tooling insufficient?

### 2.3 Proposed Solution
- Describe your solution and how it will be integrated into the Soroban Cookbook repository or ecosystem.
- List the key features and deliverables.

---

## 3. Technical Architecture & Implementation Plan

### 3.1 Smart Contract Architecture (if applicable)
- **Soroban SDK Version:** [e.g., 22.0.0+ / current stable]
- **Storage Strategy:** [Describe Persistent, Instance, or Temporary storage usage, including TTL extension strategy]
- **Authorization & Security Model:** [Detail how `require_auth`, role-based access control, or multi-sig patterns will be implemented]
- **Event Logging:** [List critical events and indexed topics]

```
[Insert ASCII Architecture Diagram or Mermaid Graph here]
```

### 3.2 Full-Stack & Tooling Architecture (if applicable)
- **Frontend / Client Tech Stack:** [e.g., TypeScript, Vite, React, Vanilla JS, @stellar/stellar-sdk, Freighter API]
- **CLI / Backend Tech Stack:** [e.g., Rust, Node.js, Python, Docker]

### 3.3 Security & Testing Strategy
- What is your testing plan? (Unit tests, cross-contract integration tests, fuzz testing)
- What is your target test coverage percentage? (Minimum 90% required)
- How will you handle edge cases, arithmetic overflow, and unexpected rollbacks?

---

## 4. Ecosystem Value & Open Source Commitment

### 4.1 Ecosystem Impact
- How will this grant project benefit the broader Stellar and Soroban developer community?
- Who is the intended user or consumer of this work?

### 4.2 Open Source Licensing
- [ ] We confirm all project code, documentation, and assets will be released under an open-source license (MIT, Apache 2.0, or dual license).
- **Target License:** [MIT / Apache 2.0 / Dual]

---

## 5. Milestone Breakdown & Budget Allocation

> **Note:** Funding is released in tranches upon verified completion of each milestone.

### Milestone 1: Core Design & Initial Implementation
- **Estimated Duration:** [e.g., 2 weeks]
- **Tranche Amount:** [$X,XXX]
- **Deliverables:**
  - [ ] Architecture design document and specification
  - [ ] Initial smart contract implementation / prototype
  - [ ] Basic unit tests passing locally and in CI
- **Acceptance Criteria:**
  - Code builds cleanly with zero warnings
  - Baseline tests pass with >85% coverage

### Milestone 2: Full Implementation, Integration & Comprehensive Tests
- **Estimated Duration:** [e.g., 2 weeks]
- **Tranche Amount:** [$X,XXX]
- **Deliverables:**
  - [ ] Complete contract features / frontend integration / CLI tooling
  - [ ] Comprehensive test suite (unit + integration + fuzz/security checks)
  - [ ] Automated CI workflow (lint, test, build)
- **Acceptance Criteria:**
  - 100% CI pass rate with >90% test coverage
  - Security checklist verification completed

### Milestone 3: Documentation, Video Walkthrough & Cookbook Integration
- **Estimated Duration:** [e.g., 2 weeks]
- **Tranche Amount:** [$X,XXX]
- **Deliverables:**
  - [ ] Comprehensive `README.md` and mdBook tutorial page
  - [ ] Interactive demo / deployed testnet contract instance
  - [ ] Video walkthrough or recorded demo (optional for micro-grants)
  - [ ] Pull request merged into Soroban Cookbook repository
- **Acceptance Criteria:**
  - mdBook builds with zero broken links or navigation errors
  - Reviewer sign-off from Grant Committee

### 5.4 Budget Breakdown Table
| Item / Category | Description | Allocation (USD) |
| :--- | :--- | :--- |
| Smart Contract Development | Core contract logic and optimization | $X,XXX |
| Testing & Security Review | Integration tests, edge cases, security checklist | $X,XXX |
| Frontend / Tooling Scaffolding | Web UI, CLI utilities, deployment scripts | $X,XXX |
| Documentation & Education | mdBook tutorials, walkthroughs, diagrams | $X,XXX |
| **Total Requested** | | **$X,XXX** |

---

## 6. Team Experience & Past Work

- **Team Member 1:**
  - Name / Handle:
  - Role on project:
  - Background & Experience:
  - Relevant GitHub Repositories / Contributions:
- **Team Member 2 (if applicable):**
  - Name / Handle:
  - Role on project:
  - Background & Experience:

---

## 7. Long-Term Maintenance & Future Vision

- How do you plan to maintain this recipe or tool as Soroban SDK and Stellar Protocol evolve?
- Are there future features or ecosystem integrations planned beyond this grant?

---

## 8. Applicant Declaration & Sign-off

By submitting this application, the applicant confirms that:
1. All information provided is accurate and complete.
2. The team has the technical competence and availability to execute the proposed milestones.
3. All code produced will be 100% open-source and free of proprietary restrictions.
4. The team agrees to adhere to the Soroban Cookbook Code of Conduct and Grant Committee review policies.

**Applicant Signature / GitHub Handle:** `@username`  
**Date Submitted:** `YYYY-MM-DD`
