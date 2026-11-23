# Documentation Metrics Dashboard

**Soroban Cookbook · Phase 8 · Issue #927**

This dashboard tracks documentation usage across two surfaces:

1. **mdBook** — the primary documentation site (page views, search, user paths)
2. **Webapp** — the Next.js showcase app (page views, search)

It is updated automatically every Monday by the
`.github/workflows/documentation-metrics.yml` workflow.

For full metric definitions, targets, and alert thresholds see
[`docs/community-metrics.md`](./community-metrics.md) § 3.6.

---

## Current Status (Latest Snapshot)

> Updated automatically every Monday at 09:00 UTC.

### Page Views

| Source | Metric | Latest Value | Target | Status |
|---|---|---|---|---|
| mdBook | Total Page Views (7 d) | — | Upward trend | — |
| mdBook | Unique Page Views (7 d) | — | — | — |
| mdBook | Avg Pages / Session | — | ≥ 3 | — |
| Webapp | Total Page Views (7 d) | — | Upward trend | — |
| Webapp | Unique Page Views (7 d) | — | — | — |

### Search Queries

| Source | Metric | Latest Value | Target | Status |
|---|---|---|---|---|
| mdBook | Total Searches (7 d) | — | — | — |
| mdBook | Unique Search Terms (7 d) | — | — | — |
| Webapp | Total Searches (7 d) | — | — | — |
| Webapp | Top Search Term | — | — | — |
| Combined | Zero-Result Searches (%) | — | < 10% | — |

### Popular Content (Top 10 Pages — Last 7 Days)

<!-- AUTO-GENERATED: documentation-metrics workflow updates this table -->

| Rank | Page | Views | Source |
|---|---|---|---|
| _Pending first run_ | — | — | — |

### User Path Analysis

<!-- AUTO-GENERATED: documentation-metrics workflow updates this section -->

| Metric | Value | Description |
|---|---|---|
| Most Common Entry Page | — | First page in a session |
| Most Common Exit Page | — | Last page before session ends |
| Avg Path Length | — | Pages viewed per session |
| Top 3-Page Flow | — | Most frequent 3-page sequence |

**Top User Paths (Last 7 Days)**

| Rank | Path Sequence | Sessions |
|---|---|---|
| _Pending first run_ | — | — |

---

## Rolling Weekly Data

<!-- AUTO-GENERATED: the documentation-metrics workflow appends rows here -->
<!-- Do not edit the table below manually -->

| Week Ending | mdBook Views | mdBook Searches | mdBook Avg Path | Webapp Views | Webapp Searches | Top Page | Top Search Term |
|---|---|---|---|---|---|---|---|
| _Pending first run_ | — | — | — | — | — | — | — |

---

## Popular Content Trends (Monthly)

### Top 20 Pages (Last 30 Days)

| Rank | Page | Views | Δ vs Prev 30d | Category |
|---|---|---|---|---|
| _Pending first run_ | — | — | — | — |

### Zero-Result Search Terms

Search queries that returned no results — candidates for new content:

| Search Term | Count | Suggested Content |
|---|---|---|
| _Pending first run_ | — | — |

---

## mdBook vs Webapp Comparison

| Metric | mdBook | Webapp | Notes |
|---|---|---|---|
| Page Views (7 d) | — | — | mdBook is primary docs surface |
| Search Activity (7 d) | — | — | mdBook has full-text search |
| Avg Session Length | — | — | Pages per session |
| DNT Respected | ✅ | ✅ | No tracking if Do-Not-Track is enabled |

---

## Privacy & Data Handling

All analytics are **privacy-respecting by design**:

- **No cookies** — session data is stored in `sessionStorage` (per-tab, cleared on close).
- **No PII** — the session id is a random 8-hex string per tab; it cannot be correlated across sessions or devices.
- **Do-Not-Track** — both the mdBook and webapp scripts check `navigator.doNotTrack` and fully no-op when DNT is enabled.
- **No fingerprinting** — no user agent, IP, or device data is collected.
- **Aggregated reporting** — the dashboard only shows aggregate counts, never individual session data.
- **Opt-out** — users with DNT enabled are not tracked at all.

See the analytics source:
- mdBook: [`book/analytics.js`](../book/analytics.js)
- Webapp: [`webapp/src/lib/analytics.ts`](../webapp/src/lib/analytics.ts)

---

## Dashboard Maintenance

This file is co-maintained by:

1. **GitHub Actions** (`documentation-metrics.yml`) — appends weekly rows and updates popular content / search / path tables automatically.
2. **On-call maintainer** — reviews trends, identifies content gaps from zero-result searches, and opens issues for missing documentation.

To suggest a new documentation metric, open a GitHub Discussion in the `Ideas` category.

---

*Dashboard last manually reviewed: October 2026 · Next review: November 2026*
