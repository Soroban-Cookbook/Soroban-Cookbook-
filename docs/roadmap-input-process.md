# Roadmap Input Process

This document describes how the Soroban Cookbook community can contribute ideas to the project roadmap, how those ideas are reviewed, and how decisions are communicated back.

---

## Overview

The roadmap is a living document maintained in [`ROADMAP.md`](../ROADMAP.md). It is reviewed and updated quarterly. Anyone — contributor, user, or ecosystem partner — can propose additions or changes by following this process.

---

## 1. Input Mechanism

### GitHub Discussions (preferred)

Open a discussion in the [Ideas category](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions) with the label `roadmap-input`. Use this format:

```
Title: [Roadmap] <short description>

## Problem or Opportunity
What problem are you solving, or what opportunity would this unlock?

## Proposed Approach
What should the project do? Be as specific or as high-level as you like.

## Who Benefits
Which developer personas or use cases does this help?

## Priority Suggestion
High / Medium / Low — and why.
```

### GitHub Issues

For well-defined, scoped improvements, open an issue and apply the `roadmap` and `good first issue` labels where applicable. Link the issue to an existing roadmap phase or suggest a new one.

### Community Calls

Roadmap items can be proposed live during the [monthly community call](../GOVERNANCE/README.md). Proposals discussed on the call are recorded in the follow-up notes and converted to a GitHub Discussion or issue within 48 hours by the call facilitator.

### Direct Pull Request

If you want to propose an update to `ROADMAP.md` directly, open a PR. The PR description should explain the context and priority in the same format used for GitHub Discussions.

---

## 2. Review Process

| Step | Who | When |
|------|-----|------|
| Acknowledge input | Any maintainer | Within 48 hours of submission |
| Initial triage | Core team | During the next weekly triage session |
| Community comment period | Open | 2 weeks after acknowledgment |
| Decision | Core team | End of comment period |
| Roadmap update (if accepted) | Maintainer | Within one week of decision |
| Communicating rejection | Maintainer | Within one week of decision, with reasoning |

### Triage criteria

Input is evaluated against these factors:

1. **Alignment** — does it fit the project's mission (practical Soroban smart-contract education)?
2. **Effort vs impact** — is the scope reasonable for the expected benefit?
3. **Community demand** — how many community members have expressed the same need?
4. **Dependencies** — does it depend on unreleased Soroban SDK features or external projects?
5. **Maintainer capacity** — is there a volunteer or assigned maintainer willing to own it?

---

## 3. Public Roadmap

The canonical roadmap lives in [`ROADMAP.md`](../ROADMAP.md) and is published to the documentation site. It shows:

- Completed phases with their outcomes
- Active phases with in-progress milestones
- Planned phases with high-level goals
- A backlog section for accepted-but-unscheduled items

The roadmap does **not** include tentative or proposed items that have not yet been accepted; those remain in GitHub Discussions until a decision is made.

---

## 4. Regular Updates

| Cadence | Activity |
|---------|----------|
| Weekly | Maintainer triage of new input |
| Monthly | Community call agenda includes a roadmap progress segment |
| Quarterly | Full roadmap review: phase completion check, milestone updates, backlog prioritization |
| Annually | Strategic review: retire old phases, introduce new multi-quarter goals |

Quarterly review notes are published as a comment in the relevant GitHub Discussion or as a pinned issue.

---

## 5. Community Votes

For significant decisions (e.g., adopting a new platform, restructuring phases), the core team will:

1. Post a GitHub Discussion summarizing the options
2. Run a 👍 / 👎 / 🤔 emoji poll for at least 7 days
3. Publish the vote outcome and the final decision with reasoning

Votes are advisory — the core team makes the final call — but community input carries meaningful weight and is documented in the decision record.

---

## 6. Linking Input to Roadmap Items

When a community-proposed input is accepted and added to the roadmap:

- The GitHub Discussion or issue is linked from `ROADMAP.md`
- The contributor is acknowledged in the milestone notes
- The contributor's GitHub handle is added to `CONTRIBUTORS.md` if not already present

---

## 7. External Resources

- [ROADMAP.md](../ROADMAP.md) — canonical roadmap file
- [GitHub Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions) — where to propose ideas
- [Monthly Community Call Governance](../GOVERNANCE/README.md) — call format and follow-up process
- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute code and documentation

---

*Document version: 1.0 (2026-10-19). Update this file and open a PR when the process changes.*
