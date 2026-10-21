# Soroban Cookbook Grants Review Process

This document outlines the evaluation criteria, scoring methodology, and governance rules used by the **Grant Committee** to assess applications submitted to the Soroban Cookbook Community Grants Program.

---

## Table of Contents

- [Review Workflow Overview](#review-workflow-overview)
- [Grant Committee Structure](#grant-committee-structure)
- [Multi-Stage Evaluation Process](#multi-stage-evaluation-process)
- [Scoring Rubric & Matrix](#scoring-rubric--matrix)
- [Decision Thresholds & Outcomes](#decision-thresholds--outcomes)
- [Conflict of Interest & Ethics Policy](#conflict-of-interest--ethics-policy)
- [Appeals & Resubmission Process](#appeals--resubmission-process)

---

## Review Workflow Overview

```
[Application Received] 
        │
        ▼
[Stage 1: Intake & Triage] ──── (Ineligible / Incomplete) ───► [Clarification Request / Rejection]
        │ (Pass)
        ▼
[Stage 2: Technical Due Diligence]
        │
        ▼
[Stage 3: Ecosystem Value & Impact]
        │
        ▼
[Stage 4: Committee Scoring & Consensus Meeting]
        │
        ▼
[Stage 5: Award Notification & Grant Agreement]
```

---

## Grant Committee Structure

The Grant Committee consists of:
- **Core Maintainers:** Ensure architectural alignment and cookbook standards.
- **Security & Ecosystem Reviewers:** Assess smart contract safety, threat models, and code quality.
- **Community Representatives:** Evaluate developer impact, tutorial clarity, and ecosystem demand.

Committee members rotate periodically to maintain diverse perspectives across the Stellar ecosystem.

---

## Multi-Stage Evaluation Process

### Stage 1: Intake & Triage (Days 1–3)
- **Completeness Check:** Verify that all required sections of the [Grant Application Form](./application-form.md) are filled out.
- **Eligibility Verification:** Ensure the project is 100% open-source, aligned with Soroban, and within requested funding caps.
- **Duplicate Prevention:** Check if similar recipes or tools are already under active development in the repository.

### Stage 2: Technical Due Diligence (Days 4–7)
- **Architecture Assessment:** Review proposed contract design, storage strategies (persistent vs instance vs temporary), TTL handling, and event structures.
- **Security & Threat Modeling:** Evaluate authorization models (`require_auth`), arithmetic safety, reentrancy guards, and upgrade mechanics.
- **Test Strategy:** Review planned unit, integration, and fuzz testing coverage.

### Stage 3: Ecosystem Value & Impact (Days 8–10)
- **Community Demand:** Evaluate how urgently the cookbook community needs this recipe or tool.
- **Tutorial & Documentation Quality:** Assess whether the applicant can produce clear, well-structured mdBook documentation.
- **Maintenance Viability:** Review team capacity for ongoing upkeep as Soroban SDK versions update.

### Stage 4: Committee Scoring & Consensus Meeting (Days 11–13)
- Committee members independently score the proposal according to the rubric.
- The committee convenes bi-weekly to discuss applications, calibrate scores, and reach consensus.

### Stage 5: Final Decision & Award Notification (Day 14)
- Formal decision letter sent to applicant via GitHub Issue and email.
- Approved projects receive onboarding materials and initial grant agreement.

---

## Scoring Rubric & Matrix

Applications are evaluated across five key dimensions on a scale from 1 (Unacceptable) to 5 (Exceptional). The weighted total score is calculated out of 100 points:

| Dimension | Weight | Evaluation Criteria |
| :--- | :---: | :--- |
| **1. Technical Merit & Architecture** | 30% | Architectural soundness, adherence to Soroban idioms, safe storage & TTL design, event hygiene. |
| **2. Security & Testing Rigor** | 25% | Comprehensive test coverage plan (>90%), authorization robustness, arithmetic safety, edge-case mitigation. |
| **3. Ecosystem Impact & Need** | 20% | Fills a known recipe gap, high utility for Stellar developers, educational clarity, showcase potential. |
| **4. Team Capability & Feasibility** | 15% | Relevant Rust/Soroban experience, past open-source track record, realistic milestone timelines. |
| **5. Budget & Milestone Clarity** | 10% | Clear deliverable definitions, SMART milestone criteria, reasonable cost breakdown per task. |

### Scoring Scale Definition
- **5 (Exceptional):** Exceeds all expectations; production-grade design; highly innovative; outstanding value for funds.
- **4 (Strong):** Meets all requirements with high quality; clear plan; minimal risks; solid team capabilities.
- **3 (Acceptable):** Meets basic requirements but has minor ambiguities or areas needing refinement.
- **2 (Weak):** Substantial gaps in technical plan, security considerations, or budget realism.
- **1 (Unacceptable):** Not viable, off-scope, closed-source, or deficient technical architecture.

---

## Decision Thresholds & Outcomes

Total weighted score out of 100 points determines the outcome:

| Score Range | Outcome | Action Required |
| :---: | :--- | :--- |
| **85 – 100** | **Approved (Unconditional)** | Award granted immediately; project advances to Grant Agreement and Milestone 1 kickoff. |
| **70 – 84** | **Approved (Conditional)** | Award approved subject to minor milestone adjustments, scope clarifications, or added test requirements. |
| **50 – 69** | **Request Revisions** | Application deferred; detailed feedback provided; applicant invited to revise and resubmit within 14 days. |
| **< 50** | **Rejected** | Application declined with constructive explanation; applicant may re-apply with a distinct proposal after 30 days. |

---

## Conflict of Interest & Ethics Policy

To ensure fairness, transparency, and impartiality:
1. **Recusal Requirement:** Any Grant Committee member who is affiliated with an applicant team, has a financial interest in the applicant project, or contributed to drafting the proposal must recuse themselves from scoring and deliberations.
2. **Transparency:** All committee scores, comments, and decision rationale are recorded and made available upon request.
3. **Anti-Collusion:** Reviewers must evaluate proposals independently before committee consensus discussions.

---

## Appeals & Resubmission Process

If an applicant believes an evaluation overlooked key technical aspects:
- The applicant may submit an appeal via GitHub Discussions within 7 business days of notification.
- The Grant Committee will assign two independent secondary reviewers to re-evaluate the proposal.
- A final unappealable determination will be provided within 10 business days.
