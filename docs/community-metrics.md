# Community Metrics — Definition & Tracking Guide

**Phase 8 · Issue #426 · Priority: High · Scope: M**

This document defines the community health metrics that the Soroban Cookbook tracks,
explains how each metric is collected, and sets the targets that determine whether the
community is growing in a healthy and sustainable direction.

---

## Table of Contents

1. [Why We Measure](#1-why-we-measure)
2. [Metric Categories](#2-metric-categories)
3. [Metric Definitions](#3-metric-definitions)
4. [Tracking Tools & Collection](#4-tracking-tools--collection)
5. [Reporting Cadence](#5-reporting-cadence)
6. [Alert Thresholds](#6-alert-thresholds)
7. [How to Contribute to Metrics](#7-how-to-contribute-to-metrics)

---

## 1. Why We Measure

Open-source communities thrive when growth, quality, and engagement are visible.
Tracking metrics allows us to:

- **Identify friction early** — a spike in unanswered issues signals overload before contributors burn out.
- **Prove value to funders and partners** — concrete numbers justify continued investment from the Stellar Development Foundation.
- **Reward contributors fairly** — contribution counts feed directly into the [recognition system](./recognition-system.md).
- **Guide the roadmap** — high-traffic documentation areas indicate where new examples are most needed.

> All metrics are collected in aggregate. No personally identifiable information beyond
> GitHub usernames (which are already public) is stored or reported.

---

## 2. Metric Categories

| Category | Description | Tracking Source |
|---|---|---|
| **Growth** | Contributor activity, stars, forks, clone traffic | GitHub Insights / Actions |
| **Engagement** | Issues opened/closed, PR activity, discussion threads | GitHub API |
| **Content Quality** | CI pass rate, test coverage, clippy warnings | GitHub Actions badges |
| **Community Health** | Response time, PR review velocity, Code of Conduct incidents | GitHub API / manual |
| **Documentation** | Page views, search queries, feedback submissions | GitHub Pages / Feedback System |

---

## 3. Metric Definitions

### 3.1 Growth Metrics

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **Contributors** | Unique GitHub accounts with at least one merged PR in the period | ≥ 2 / month | Monthly |
| **Repository Stars** | Cumulative star count at the end of the period | Sustained upward trend | Monthly |
| **Forks** | Cumulative fork count | Sustained upward trend | Monthly |
| **Unique Cloners** | 14-day unique clone traffic (from GitHub Traffic API) | ≥ 100 / 14 days | Bi-weekly |
| **Unique Views** | 14-day unique repository page view count | ≥ 500 / 14 days | Bi-weekly |

### 3.2 Engagement Metrics

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **Issues Opened** | New issues created in the period | Healthy demand (not a raw target) | Weekly |
| **Issues Closed** | Issues closed in the period | Close rate ≥ 70% of opened | Weekly |
| **Issue Response Time (median)** | Time from issue creation to first maintainer comment | < 48 hours | Weekly |
| **PRs Opened** | Pull requests opened in the period | — | Weekly |
| **PRs Merged** | Pull requests merged in the period | Merge rate ≥ 60% of opened | Weekly |
| **PR Review Time (median)** | Time from PR open to first substantive review comment | < 72 hours | Weekly |
| **Discussion Threads** | New GitHub Discussions threads created in the period | Growth trend | Monthly |
| **Discussion Replies** | Total replies on open discussions | Avg ≥ 2 replies / thread | Monthly |

### 3.3 Content Quality Metrics

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **CI Pass Rate** | % of workflow runs that complete green on `main` | 100% | Continuous |
| **Test Coverage** | Workspace-wide line coverage (tarpaulin) | ≥ 90% | On every PR |
| **Clippy Warnings** | Number of active clippy warnings on `main` | 0 | Continuous |
| **WASM Build Success** | Whether `wasm32-unknown-unknown` target builds cleanly | 100% | Continuous |
| **New Examples per Quarter** | Net new example contracts added | ≥ 4 / quarter | Quarterly |
| **Stale Docs** | Documentation pages not updated in > 6 months | 0 pages stale | Quarterly |

### 3.4 Community Health Metrics

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **Bus Factor** | Minimum contributors needed to halt project (≥ 3 = healthy) | ≥ 3 | Quarterly |
| **Contributor Retention** | % of contributors active in 2+ consecutive months | ≥ 40% | Quarterly |
| **First-timer Success Rate** | % of first-time contributors whose PR is ultimately merged | ≥ 60% | Quarterly |
| **CoC Incidents** | Confirmed Code of Conduct violations in the period | 0 | Monthly |
| **Unresolved Blocking Issues** | Open issues tagged `blocking` older than 2 weeks | 0 | Weekly |

### 3.5 Documentation & Feedback Metrics

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **Feedback Form Submissions** | Forms submitted via [feedback system](./feedback-system/README.md) | Tracking trend | Monthly |
| **Feedback Resolved** | % of feedback items that result in a closed action | ≥ 80% | Monthly |
| **Community Satisfaction Score** | Average rating from periodic surveys (1–5 scale) | ≥ 4.0 / 5 | Quarterly |
| **Docs Clarity Score** | Self-reported clarity rating on feedback forms | ≥ 4.5 / 5 | Quarterly |

### 3.6 Documentation Usage Metrics

These metrics track how developers interact with the documentation surfaces — the
mdBook site and the Next.js webapp. They are collected by privacy-respecting
analytics scripts and aggregated weekly by the
[`documentation-metrics.yml`](../.github/workflows/documentation-metrics.yml)
workflow. See the [Documentation Metrics Dashboard](./documentation-metrics.md)
for live data.

**Privacy guarantees:** No cookies, no PII, no fingerprinting. Session data is
stored in `sessionStorage` (per-tab, cleared on close). Do-Not-Track is fully
respected. All reporting is aggregate-only.

| Metric | Definition | Target | Frequency |
|---|---|---|---|
| **Page Views** | Total page load events across mdBook and webapp | Upward trend | Weekly |
| **Unique Page Views** | Distinct paths viewed (deduplicated by path) | Upward trend | Weekly |
| **Pages per Session** | Average number of pages viewed in a single session | ≥ 3 | Weekly |
| **Search Queries** | Total search events fired on mdBook or webapp search bars | Tracking trend | Weekly |
| **Unique Search Terms** | Distinct search query strings in the period | Tracking trend | Weekly |
| **Top Search Term** | Most frequently searched term in the period | — | Weekly |
| **Zero-Result Searches** | Search queries that returned no results | < 10% of searches | Weekly |
| **Popular Content (Top 10)** | Most-viewed pages by view count | — | Weekly |
| **Popular Content (Top 20)** | Most-viewed pages over 30-day window | — | Monthly |
| **Entry Page** | Most common first page in a user session | — | Weekly |
| **Exit Page** | Most common last page before session ends | — | Weekly |
| **Avg Path Length** | Mean pages viewed per session across all sessions | ≥ 3 | Weekly |
| **Top 3-Page Flow** | Most frequent 3-page navigation sequence | — | Weekly |
| **mdBook vs Webapp Split** | Page views and searches by surface | — | Weekly |

**Data sources:**
- **mdBook** — `book/analytics.js` (page views, search, user path via `sessionStorage`)
- **Webapp** — `webapp/src/lib/analytics.ts` (page views, search, user path)
- **GitHub Traffic API** — repository-level page views and popular paths (fallback when no analytics endpoint configured)

---

## 4. Tracking Tools & Collection

### 4.1 GitHub Native Insights

GitHub provides built-in analytics at no cost:

| Tool | Data Provided | Access |
|---|---|---|
| **Repository Insights → Traffic** | Clones, unique visitors, referring sites | Maintainers only (Settings → Insights → Traffic) |
| **Repository Insights → Contributors** | Commit activity per contributor | Public |
| **GitHub Actions Summary** | CI pass/fail history | Public (`.github/workflows/`) |
| **Dependabot Alerts** | Dependency security status | Maintainers only |

### 4.2 Automated Collection via GitHub Actions

A scheduled workflow (`.github/workflows/community-metrics.yml`) runs every Monday
at 09:00 UTC and:

1. Calls the **GitHub REST API** to collect the weekly snapshot of issues, PRs, and discussion counts.
2. Appends a row to [`docs/community-dashboard.md`](./community-dashboard.md) inside the rolling data table.
3. Posts a **GitHub Discussions summary** for community visibility.
4. Opens a labeled issue (`metrics-alert`) if any alert threshold is breached.

See [§ 6 Alert Thresholds](#6-alert-thresholds) for the list of conditions that trigger an alert.

A second scheduled workflow (`.github/workflows/documentation-metrics.yml`) runs
every Monday at 09:00 UTC and:

1. Collects repository page views and popular paths via the **GitHub Traffic API**.
2. Parses analytics event logs (from `book/analytics.js` and `webapp/src/lib/analytics.ts`)
   for search queries, popular content, and user path analysis.
3. Updates [`docs/documentation-metrics.md`](./documentation-metrics.md) with
   weekly page view counts, top search terms, popular content, entry/exit pages,
   average path length, and top user flows.
4. Identifies zero-result search terms as candidates for new documentation.

Both workflows commit their updates directly to `main` with `[skip ci]` to avoid
triggering downstream CI.

### 4.3 External / Optional Tools

The following tools can be adopted if maintainer capacity allows:

| Tool | Purpose | Cost |
|---|---|---|
| [Orbit.love](https://orbit.love) | Multi-channel community activity aggregation | Free tier available |
| [Stargazers](https://github.com/nicedoc/stargazers) | Star growth analytics | Free, self-hosted |
| [Codecov](https://codecov.io) | Coverage trend visualization (already integrated) | Free for public repos |
| [Open Collective Insights](https://opencollective.com) | Financial health if a fund is established | Free |

> Start with native GitHub Insights and the automated workflow. Add external tools
> only when the native data is insufficient for a specific decision.

### 4.4 Survey Infrastructure

Periodic community surveys are managed through the
[feedback system](./feedback-system/README.md). Surveys are:

- Distributed via GitHub Discussions and the Stellar Community Discord `#soroban` channel.
- Run quarterly using the template in `docs/feedback-system/surveys/`.
- Analyzed by a maintainer, with results published in the monthly report.

---

## 5. Reporting Cadence

| Cadence | Report | Owner | Audience |
|---|---|---|---|
| **Weekly** (every Monday) | Automated snapshot appended to dashboard | GitHub Actions bot | Maintainers (GitHub issue) |
| **Monthly** (1st of month) | Narrative summary + trend analysis | Rotating maintainer | Community (GitHub Discussions) |
| **Quarterly** (Jan / Apr / Jul / Oct) | Full health report with satisfaction survey | Core team | Public (README + Discussions) |

### Monthly Report Template

```markdown
## Community Report — [Month Year]

### Highlights
- [positive trend or achievement]
- [milestone reached]

### Growth
| Metric | This Month | Last Month | Δ |
|---|---|---|---|
| Contributors | | | |
| Stars | | | |
| Unique Cloners | | | |

### Engagement
| Metric | This Month | Target | Status |
|---|---|---|---|
| Issue Response Time (median) | | < 48 h | |
| PR Review Time (median) | | < 72 h | |
| Issues Close Rate | | ≥ 70% | |

### Content Quality
- CI Pass Rate: %
- Test Coverage: %
- New Examples Added:

### Action Items
- [ ] [action driven by data]
```

---

## 6. Alert Thresholds

When any of the following conditions are detected by the automated workflow, a
GitHub issue is automatically opened with label `metrics-alert` and assigned to
the on-call maintainer:

| Condition | Threshold | Severity |
|---|---|---|
| CI Pass Rate drops | < 95% on `main` over 7 days | Critical |
| Test Coverage drops | < 85% on latest run | Critical |
| Clippy warnings appear | > 0 on `main` | Critical |
| Median issue response time | > 72 hours in rolling 7 days | Warning |
| Median PR review time | > 5 days in rolling 7 days | Warning |
| Issues close rate | < 50% for the month | Warning |
| No contributor PR activity | 0 contributors with merged PRs in 30 days | Warning |
| Community satisfaction drops | Survey score < 3.5 / 5 | High |
| CoC incident reported | Any confirmed incident | Critical |

---

## 7. How to Contribute to Metrics

Community members can help improve metric quality:

- **Submit feedback** — use the [feedback form](./feedback-system/forms/feedback-form-template.md)
  to report friction points that might not appear in raw numbers.
- **Participate in surveys** — quarterly survey links are posted in GitHub Discussions
  and the Stellar Community Discord.
- **Flag anomalies** — if you notice a metric that seems wrong (e.g., a bot inflating
  counts), open an issue tagged `metrics-anomaly`.
- **Propose new metrics** — open a Discussion thread in the `Ideas` category.

---

## Related Documents

- [Community Dashboard](./community-dashboard.md) — live rolling data table
- [Documentation Metrics Dashboard](./documentation-metrics.md) — page views, search queries, popular content, user paths
- [Recognition System](./recognition-system.md) — how contribution counts are used for tiers
- [Feedback System](./feedback-system/README.md) — survey and feedback infrastructure
- [Roadmap](../ROADMAP.md) — how metrics inform phase planning
- [Contributing Guide](../CONTRIBUTING.md) — how to start contributing

---

*Last Updated: July 2026 · Maintained by: Soroban Cookbook Core Team*
