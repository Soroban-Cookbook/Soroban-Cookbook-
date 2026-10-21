# Soroban Cookbook Grants Milestone Tracking & Verification

This guide outlines the policies, templates, verification criteria, and disbursement procedures for tracking milestones across all active projects funded by the **Soroban Cookbook Grants Program**.

---

## Table of Contents

- [Milestone Principles](#milestone-principles)
- [Milestone Definition Guidelines](#milestone-definition-guidelines)
- [Bi-Weekly Progress Reporting](#bi-weekly-progress-reporting)
- [Milestone Submission Report Template](#milestone-submission-report-template)
- [Verification & Acceptance Criteria](#verification--acceptance-criteria)
- [Payout & Disbursement Process](#payout--disbursement-process)
- [Handling Delays, Extensions & Scope Changes](#handling-delays-extensions--scope-changes)
- [Project Completion & Showcase Integration](#project-completion--showcase-integration)

---

## Milestone Principles

The Soroban Cookbook Grants Program links all funding disbursements to tangible, verifiable development milestones:

1. **Deliverable-Centric:** Payouts correspond to concrete deliverables (e.g. tested contracts, documentation chapters, UI scaffolding, tooling crates).
2. **Quality-Gated:** Every milestone deliverable must pass automated CI checks (formatting, Clippy, tests, coverage, docs build).
3. **Public Transparency:** All progress reports, demo recordings, PRs, and payment hashes are tracked openly in GitHub issues.

---

## Milestone Definition Guidelines

When defining milestones in the grant application:
- **Duration:** Individual milestones should span **2 to 4 weeks**.
- **Scope:** Each milestone should deliver a self-contained, demonstrable capability.
- **SMART Criteria:**
  - **Specific:** Exact list of crates, contracts, functions, or UI components.
  - **Measurable:** Target test coverage percentage, number of test cases, performance benchmark.
  - **Achievable:** Realistic development workload for the stated timeframe.
  - **Relevant:** Directly serves the goals of the Soroban Cookbook.
  - **Time-bound:** Definite target delivery date.

---

## Bi-Weekly Progress Reporting

Grant recipients must post a brief status update on their GitHub tracking issue every two weeks. The update should follow this structure:

```markdown
### 📢 Bi-Weekly Update — [YYYY-MM-DD]
- **Current Milestone:** [Milestone 1 / 2 / 3]
- **Target Completion Date:** [YYYY-MM-DD]
- **Progress Accomplished in Last 2 Weeks:**
  - Implemented X smart contract module
  - Wrote Y unit tests achieving Z% coverage
  - Drafted architecture diagram and mdBook chapter
- **Current Blockers / Dependencies:** [None / List blockers]
- **Plan for Next 2 Weeks:** [Upcoming tasks]
```

---

## Milestone Submission Report Template

When a milestone is completed, the grant recipient creates a formal milestone submission comment on their tracking issue:

```markdown
# 🎯 Milestone [Number] Completion Report

- **Grant Project:** [Project Title]
- **Milestone Number & Name:** [e.g., Milestone 2: AMM Router & Multi-Hop Execution]
- **Requested Tranche Amount:** [$X,XXX]
- **Recipient Stellar Public Address:** `G...`

---

## 1. Summary of Deliverables Completed

| Deliverable Description | Repository Link / PR | Status |
| :--- | :--- | :---: |
| Smart Contract Crate | [PR #XXX / commit](https://github.com/...) | ✅ Complete |
| Test Suite (>90% coverage) | [PR #XXX / commit](https://github.com/...) | ✅ Complete |
| mdBook Tutorial Page | [PR #XXX / commit](https://github.com/...) | ✅ Complete |
| Live Interactive Demo / Video | [Link to demo / video](https://...) | ✅ Complete |

---

## 2. Verification Instructions & Test Execution

Provide exact commands for reviewers to verify the build and tests locally:

```bash
# 1. Clone branch and navigate to crate
cd examples/defi/my-new-example

# 2. Run unit and integration tests
cargo test --all-targets

# 3. Check Clippy lints
cargo clippy --all-targets -- -D warnings

# 4. Verify mdBook builds cleanly
cd ../../../book && mdbook build
```

---

## 3. Security & Quality Checklist

- [x] All unit and integration tests pass with zero failures
- [x] Code passes Clippy with zero warnings
- [x] Code adheres to repository style guide and naming conventions
- [x] Authorization checks (`require_auth`) implemented on all sensitive endpoints
- [x] Storage TTL management properly configured
- [x] Documentation includes architecture diagram, error glossary, and code commentary

---

## 4. Demo Artifacts

- **Live Testnet Contract Address:** `C...`
- **Video Walkthrough Link:** [YouTube / Loom link]
- **Screenshots / GIFs:** [Attach UI or CLI execution screenshots]
```

---

## Verification & Acceptance Criteria

Reviewers verify milestone submissions against this rigorous acceptance protocol:

1. **Automated CI Validation:**
   - Cargo test passes on stable Rust toolchain.
   - Clippy and `cargo fmt` pass without warnings.
   - mdBook documentation builds with zero errors and no broken internal links.
2. **Code Quality & Architecture Review:**
   - Proper use of Soroban SDK conventions, custom error enums, and event logging.
   - Safe arithmetic (checked math, prevention of precision truncation).
   - Storage safety (TTL extensions, safe key construction).
3. **Documentation Quality:**
   - Clear explanations of the concept, code walkthrough, security implications, and testing instructions.

---

## Payout & Disbursement Process

1. **Approval Sign-off:** Two designated Grant Committee reviewers approve the milestone report on GitHub.
2. **Treasury Processing:** The Treasury Lead issues payment within **3 business days** of committee approval.
3. **Public Ledger Verification:** The Stellar transaction hash (TX ID) is posted as a public comment on the issue for complete auditing.

---

## Handling Delays, Extensions & Scope Changes

- **Requesting an Extension:** If unexpected delays arise, recipients should submit an extension request at least 5 business days prior to the milestone deadline. Extensions up to 14 days are routinely granted for legitimate technical roadblocks.
- **Scope Adjustments:** If unforeseen technical constraints require modifying deliverables, the recipient can submit a Scope Adjustment Proposal for committee review.
- **Non-Responsive Grants:** If a team fails to submit updates for >30 consecutive days without prior notice, the grant may be paused or cancelled, and remaining unspent funds returned to the grant pool.

---

## Project Completion & Showcase Integration

Upon approval of the final milestone:
1. The project code and documentation are merged into the main branch of the Soroban Cookbook.
2. The project is featured in [SHOWCASE.md](../../SHOWCASE.md).
3. The team is invited to present a live demo during the monthly [Community Call](../../GOVERNANCE/README.md).
4. Commemorative contributor badges are awarded in the [Recognition System](../../docs/recognition-system.md).
