# Grants Application Process

The **Soroban Cookbook Community Grants Program** provides funding and technical mentorship to developers building high-impact recipes, developer tooling, security analysis frameworks, and educational guides for the Stellar and Soroban ecosystem.

---

## Overview

The grants program supports contributions that enhance developer experience, expand practical smart contract examples, and strengthen the security posture of Soroban applications.

### Key Highlights
- **Funding Tiers:** Micro-grants (up to $2,500), Standard grants ($2,500 – $10,000), and Strategic grants ($10,000+).
- **Fast Decision SLAs:** 5–7 days for Micro-Grants, 14 days for Standard Grants.
- **Tranche-Based Payouts:** Funds released upon verified milestone delivery.
- **100% Open Source:** All code and documentation published under open-source licenses.

---

## Program Components

The grants system is divided into four key operational areas:

```
┌─────────────────────────────────────────────────────────────┐
│             Soroban Cookbook Grants Program                 │
└─────────────────────────────────────────────────────────────┘
          │                 │                │
          ▼                 ▼                ▼
┌──────────────────┐┌───────────────┐┌────────────────────────┐
│ Application Form ││Review Process ││ Decision Timeline & SLA│
└──────────────────┘└───────────────┘└────────────────────────┘
          │                 │                │
          └─────────────────┼────────────────┘
                            ▼
           ┌─────────────────────────────────┐
           │  Milestone Tracking & Payouts   │
           └─────────────────────────────────┘
```

### 1. Application Form
Applicants provide structured project information, technical architecture, security considerations, testing strategies, milestone plans, and budget breakdowns.
- [View Application Form Guide & Template](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/application-form.md)

### 2. Review Process & Scoring Rubric
Proposals are evaluated by the Grant Committee across five weighted criteria:
- **Technical Merit & Architecture (30%):** Adherence to Soroban best practices, storage safety, TTL management.
- **Security & Testing Rigor (25%):** Comprehensive test coverage (>90%), authorization validation, arithmetic checks.
- **Ecosystem Impact (20%):** Value to developers, filling cookbook gaps.
- **Team Capability (15%):** Track record and feasibility of delivery.
- **Budget & Milestones (10%):** Clarity of deliverables and realistic cost breakdown.

- [View Review Process & Rubric](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/review-process.md)

### 3. Decision Timeline & SLAs
- **Day 0:** Submission & automated confirmation.
- **Days 1–3:** Intake triage & eligibility check.
- **Days 4–7:** Technical due diligence & applicant Q&A.
- **Days 11–13:** Committee scoring & consensus vote.
- **Day 14:** Formal decision notification & onboarding.

- [View Decision Timeline & SLAs](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/decision-timeline.md)

### 4. Milestone Tracking & Verification
Active grants report progress bi-weekly and submit milestone reports with code PRs, automated test results, documentation builds, and live demo links.
- Reviewers verify builds, test suites, and security practices before authorizing tranche disbursements.
- Stellar transaction hashes are posted publicly on tracking issues.

- [View Milestone Tracking Guide](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/milestone-tracking.md)

---

## How to Apply

1. Review the focus areas and eligibility criteria in the [Grants Overview](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/README.md).
2. Copy the [Application Form Template](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/docs/grants/application-form.md).
3. Open a GitHub Issue labeled `grant-application` or submit a PR under `grants/proposals/`.
4. The Grant Committee will acknowledge your submission within 24 hours and begin the review cycle.

---

## Related Resources

- [Contributing Guide](../CONTRIBUTING.md)
- [Project Showcase](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/SHOWCASE.md)
- [Community Guidelines](../community-guidelines.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Security Best Practices](./security-best-practices.md)
