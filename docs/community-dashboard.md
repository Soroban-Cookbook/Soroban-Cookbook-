# Community Dashboard

**Soroban Cookbook · Phase 8 Community Metrics**

This dashboard tracks community health at a glance. It is updated automatically
every Monday by the `.github/workflows/community-metrics.yml` workflow. Maintainers
also add the monthly narrative summary in the [Reports](#monthly-reports) section.

For full metric definitions, targets, and alert thresholds see
[`docs/community-metrics.md`](./community-metrics.md).

---

## Current Status (Latest Snapshot)

> Updated automatically every Monday at 09:00 UTC.

| Category | Metric | Latest Value | Target | Status |
|---|---|---|---|---|
| Growth | Stars | — | Upward trend | — |
| Growth | Forks | — | Upward trend | — |
| Growth | Unique Cloners (14 d) | — | ≥ 100 | — |
| Growth | New Contributors (month) | — | ≥ 2 | — |
| Engagement | Open Issues | — | — | — |
| Engagement | Issue Response Time (median) | — | < 48 h | — |
| Engagement | PR Review Time (median) | — | < 72 h | — |
| Engagement | Issues Close Rate | — | ≥ 70% | — |
| Engagement | PRs Merged (week) | — | — | — |
| Quality | CI Pass Rate | ✅ Tracked by badges | 100% | See badges |
| Quality | Test Coverage | ✅ Tracked by Codecov | ≥ 90% | See badge |
| Quality | Clippy Warnings | ✅ Enforced by CI | 0 | See CI |
| Health | Bus Factor | — | ≥ 3 | Quarterly |
| Health | CoC Incidents | — | 0 | Monthly |
| Docs | Satisfaction Score | — | ≥ 4.0/5 | Quarterly |

---

## CI & Quality Badges (Live)

[![Test and Lint](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/test.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/test.yml)
[![Security Audit](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/security-audit.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/security-audit.yml)
[![codecov](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook-/branch/main/graph/badge.svg)](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook-)

---

## Rolling Weekly Data

<!-- AUTO-GENERATED: the community-metrics workflow appends rows here -->
<!-- Do not edit the table below manually -->

| Week Ending | Stars | Forks | Unique Cloners | Issues Opened | Issues Closed | PRs Opened | PRs Merged | New Contributors | Median Response Time |
|---|---|---|---|---|---|---|---|---|---|
| _Pending first run_ | — | — | — | — | — | — | — | — | — |

---

## Quarterly Health Reports

### Q3 2026 (Jul – Sep)

> Report due: **1 October 2026**. Survey distributed in September.

| Metric | Q3 2026 | Q2 2026 | Δ |
|---|---|---|---|
| Bus Factor | — | — | — |
| Contributor Retention | — | — | — |
| First-timer Success Rate | — | — | — |
| Community Satisfaction | — | — | — |
| Docs Clarity Score | — | — | — |

---

### Q2 2026 (Apr – Jun)

> Report due: **1 July 2026**.

| Metric | Q2 2026 | Q1 2026 | Δ |
|---|---|---|---|
| Bus Factor | — | — | — |
| Contributor Retention | — | — | — |
| First-timer Success Rate | — | — | — |
| Community Satisfaction | — | — | — |
| Docs Clarity Score | — | — | — |

---

## Monthly Reports

### July 2026

> Posted to GitHub Discussions by: _[maintainer name]_ on _[date]_.

**Highlights**

- Community metrics tracking system established (Phase 8, Issue #426).
- Dashboard, automated workflow, and metric definitions all published.
- Tracking tools activated; first automated snapshot expected Monday 28 July 2026.

**Growth**

| Metric | Jul 2026 | Jun 2026 | Δ |
|---|---|---|---|
| New Contributors | — | — | — |
| Stars | — | — | — |
| Unique Cloners | — | — | — |

**Engagement**

| Metric | Jul 2026 | Target | Status |
|---|---|---|---|
| Issue Response Time (median) | — | < 48 h | — |
| PR Review Time (median) | — | < 72 h | — |
| Issues Close Rate | — | ≥ 70% | — |

**Content Quality**

- CI Pass Rate: Live (see badges above)
- Test Coverage: Live (see Codecov badge)
- New Examples Added: —

**Action Items for August**

- [ ] Fill in first automated data row after Monday 28 July run.
- [ ] Distribute Q3 satisfaction survey in Discord `#soroban`.
- [ ] Review bus factor and update quarterly health table.

---

## Active Alerts

> Alerts are opened automatically as GitHub issues tagged `metrics-alert`.
> Resolved alerts are closed and archived below.

| Alert | Opened | Status | Resolution |
|---|---|---|---|
| _None_ | — | — | — |

---

## Dashboard Maintenance

This file is co-maintained by:

1. **GitHub Actions** (`community-metrics.yml`) — appends weekly rows automatically.
2. **On-call maintainer** — writes the monthly narrative section and resolves alerts.
3. **Core team** — publishes quarterly health report and satisfaction survey results.

To suggest a new metric or report format, open a GitHub Discussion in the `Ideas` category.

---

*Dashboard last manually reviewed: July 2026 · Next review: August 2026*
