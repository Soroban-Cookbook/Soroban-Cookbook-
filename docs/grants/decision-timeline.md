# Soroban Cookbook Grants Decision Timeline & SLA

This document establishes the official Service Level Agreements (SLAs), review cycles, and decision timelines for applications submitted to the Soroban Cookbook Community Grants Program.

---

## Table of Contents

- [Standard Decision Timeline SLA](#standard-decision-timeline-sla)
- [Review Cadence & Meeting Schedules](#review-cadence--meeting-schedules)
- [Tier-Specific Timelines](#tier-specific-timelines)
- [Milestone Verification & Disbursement Schedule](#milestone-verification--disbursement-schedule)
- [Applicant Communication Schedule](#applicant-communication-schedule)
- [Delays & Holiday Exceptions](#delays--holiday-exceptions)

---

## Standard Decision Timeline SLA

The Grants Program operates under a strict **14-business-day decision SLA** for Standard and Micro-grant applications from the date of submission:

| Phase | Days | Lead Role | Deliverables / Outcomes |
| :--- | :---: | :--- | :--- |
| **1. Submission & Intake** | **Day 0** | System / Bot | Automated acknowledgment on GitHub Issue with tracking ID and intake receipt. |
| **2. Triage & Eligibility** | **Days 1–3** | Intake Lead | Completeness review, open-source verification, scope check. Assigned to reviewers. |
| **3. Technical Review** | **Days 4–7** | Tech Reviewers | Smart contract architecture analysis, security assessment, preliminary questions sent to applicant. |
| **4. Applicant Clarifications** | **Days 8–10** | Applicant | Applicant responds to technical questions or updates proposal if needed. |
| **5. Committee Meeting** | **Days 11–13** | Grant Committee | Committee deliberation, rubric scoring, consensus vote, award conditions formulated. |
| **6. Official Decision** | **Day 14** | Program Lead | Formal decision notice posted to Issue/PR; Grant Agreement sent for approved projects. |

```
[Day 0] Submission & Auto-Ack
   │
[Days 1-3] Intake Triage & Eligibility Check
   │
[Days 4-7] Technical & Security Deep Dive
   │
[Days 8-10] Applicant Q&A & Clarifications
   │
[Days 11-13] Committee Scoring & Vote
   │
[Day 14] Final Decision Notification & Onboarding
```

---

## Review Cadence & Meeting Schedules

- **Application Ingestion:** Rolling intake (applications accepted continuously 24/7).
- **Committee Review Sessions:** Bi-weekly every other Tuesday at 16:00 UTC.
- **Batch Notifications:** Formal decision announcements are issued within 2 business days following the committee review session.

---

## Tier-Specific Timelines

Depending on the size and complexity of the grant, review timelines vary:

### 1. Tier 1: Micro-Grants (< $2,500)
- **Review Pathway:** Fast-Track Review (evaluated asynchronously by 2 core maintainers).
- **Decision SLA:** **5 to 7 business days**.
- **Payout Structure:** Typically 50% upfront upon agreement signing, 50% upon final merged cookbook contribution.

### 2. Tier 2: Standard Grants ($2,500 – $10,000)
- **Review Pathway:** Full Committee Review with technical due diligence.
- **Decision SLA:** **14 business days**.
- **Payout Structure:** Multi-tranche (e.g. 30% Milestone 1, 40% Milestone 2, 30% Final Milestone & Cookbook merge).

### 3. Tier 3: Strategic Grants ($10,000+)
- **Review Pathway:** Comprehensive review including live video interview/presentation with the Grant Committee.
- **Decision SLA:** **21 to 28 business days**.
- **Payout Structure:** Strict milestone-based disbursement with dedicated code audit sign-off before final release.

---

## Milestone Verification & Disbursement Schedule

Once a project is approved and starts development, milestone payouts adhere to the following SLA:

| Stage | SLA Timeline | Responsible Party | Description |
| :--- | :---: | :--- | :--- |
| **Milestone Submission** | Day 0 | Applicant | Submits [Milestone Submission Report](./milestone-tracking.md) with PR and demo. |
| **Review & Test Execution** | Days 1–3 | Reviewer | Validates code, runs test suite, checks CI, reviews mdBook documentation. |
| **Committee Sign-off** | Days 4–5 | Grant Committee | Final approval recorded on tracking issue. |
| **Fund Disbursement** | Days 6–8 | Treasury Lead | Transaction executed on Stellar Mainnet/Testnet; TX hash recorded publicly. |

---

## Applicant Communication Schedule

We are committed to transparent, responsive communication throughout the entire grant lifecycle:

- **Day 0:** Instant GitHub automated reply confirming receipt and assigning tracking tag.
- **Day 3:** Status update confirming whether the application passed triage into technical review.
- **Day 7:** Consolidated technical questions or clarification requests posted on the proposal issue.
- **Day 14:** Final decision letter, detailed reviewer feedback, and next steps posted.
- **Active Grants:** Bi-weekly check-in on project progress and open blocker resolution.

---

## Delays & Holiday Exceptions

If application volume surges or during major public holiday periods (e.g. late December), the committee will post a notice on active issues outlining any temporary SLA extension (not to exceed +7 business days).
