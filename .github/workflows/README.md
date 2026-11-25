# Community Metrics Workflow — Usage Guide

This directory contains GitHub Actions workflows that automate CI, CD, and community
operations for the Soroban Cookbook.

## Workflow Inventory

| Workflow | File | Trigger | Purpose |
|---|---|---|---|
| Test & Lint | `test.yml` | `push`, `pull_request` | Rust fmt, Clippy, tests, WASM build |
| Security Audit | `security-audit.yml` | Schedule (weekly) | `cargo audit` for known CVEs |
| Deploy Docs | `deploy-docs.yml` | Push to `main` | mdBook → GitHub Pages |
| Dependabot Auto-merge | `dependabot-auto-merge.yml` | Dependabot PRs | Auto-merge patch updates |
| Fuzz | `fuzz.yml` | Schedule | Fuzz testing of contract inputs |
| **Community Metrics** | `community-metrics.yml` | Schedule (weekly, Mon 09:00 UTC) | Collect engagement/health data |
| **Documentation Metrics** | `documentation-metrics.yml` | Schedule (weekly, Mon 09:00 UTC) | Collect page views, search queries, user paths |

## Community Metrics Workflow

The `community-metrics.yml` workflow queries the GitHub API each week and:

1. Records a snapshot of issues, PRs, and discussion activity.
2. Appends a row to `docs/community-dashboard.md`.
3. Opens a `metrics-alert` issue if any threshold is breached.

### Required Secrets

| Secret | Purpose |
|---|---|
| `GITHUB_TOKEN` | Built-in; grants read access to the repository API |
| `METRICS_GH_TOKEN` | Optional PAT with `discussions:write` scope for posting summaries |

### Manual Trigger

```bash
# Trigger via the GitHub CLI
gh workflow run community-metrics.yml
```

### Alert Labels

Ensure the following labels exist in the repository before the first run:

```bash
gh label create "metrics-alert" --color "d93f0b" --description "Automated metrics threshold alert"
gh label create "metrics-anomaly" --color "e4e669" --description "Suspected metric data anomaly"
```

## Documentation Metrics Workflow

The `documentation-metrics.yml` workflow runs each Monday at 09:00 UTC and:

1. Collects repository page views and popular paths via the GitHub Traffic API.
2. Parses analytics event logs (if available) for search queries and user path analysis.
3. Updates `docs/documentation-metrics.md` with page view counts, top search terms,
   popular content, entry/exit pages, average path length, and top user flows.
4. Identifies zero-result search terms as candidates for new documentation.

### Analytics Instrumentation

Two privacy-respecting scripts collect the raw data:

| Surface | Script | What it tracks |
|---|---|---|
| mdBook | `book/analytics.js` | Page views, search queries, user paths (sessionStorage) |
| Webapp | `webapp/src/lib/analytics.ts` | Page views, search queries, user paths (sessionStorage) |

Both scripts respect Do-Not-Track, use no cookies, collect no PII, and send data
via `navigator.sendBeacon`. To enable beacon collection, set
`window.SOROBAN_ANALYTICS_ENDPOINT` (mdBook) or
`NEXT_PUBLIC_ANALYTICS_ENDPOINT` (webapp) to your collection endpoint.

### Manual Trigger

```bash
gh workflow run documentation-metrics.yml
```
