# Quality Dashboard

Live indicators and historical data for the Soroban Cookbook workspace. This page is published with the [public documentation site](../README.md).

## Current Status

| Area | Latest signal | History |
|---|---|---|
| Tests and lint | [![Test and Lint](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/test.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/test.yml) | [Recent test runs](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/test.yml) |
| Coverage | [![codecov](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook-/branch/main/graph/badge.svg)](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook-/branch/main) | [Coverage trends](https://codecov.io/gh/Soroban-Cookbook/Soroban-Cookbook-/branch/main/graphs) |
| Performance | [Benchmark workflow](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/benchmarks.yml) | [Benchmark runs and downloadable reports](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/benchmarks.yml) |
| Security | [![Security Audit](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/security-audit.yml/badge.svg)](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/security-audit.yml) | [Security audit runs](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/actions/workflows/security-audit.yml) |

The test and security badges show the latest workflow status. Open each workflow's run history to compare pass/fail results over time. Codecov provides coverage history and file-level detail. Each benchmark run stores raw output as a downloadable artifact for 90 days, allowing results to be compared across commits.

## Measurement Sources
## Weekly Snapshot

The pass rate below is the success share among the latest 20 completed `test.yml` runs on `main`. Security and benchmark columns show the latest completed run for each workflow.

| Week | Tests passed | Security audit | Benchmarks |
|---|---:|---|---|
<!-- QUALITY-SNAPSHOT-START -->
| 2026-10-19 | 0.0% (0/20) | action_required | action_required |
<!-- QUALITY-SNAPSHOT-END -->

## Measurement Sources

- **Test pass rates:** The `test.yml` workflow runs the test, lint, and build gates. Its run history is the source of truth for successful and failed runs.
- **Coverage trends:** `test.yml` generates Cobertura coverage and uploads it to Codecov. The graph above tracks changes on `main`.
- **Performance trends:** `benchmarks.yml` runs `scripts/benchmark.sh` and uploads the raw benchmark logs for each run. Results depend on the runner and toolchain, so compare like-for-like commits and environments.
- **Security status:** `security-audit.yml` runs the dependency audit; its badge and run history show the latest and historical outcomes.

For benchmark methodology and existing baselines, see [Gas Benchmarks](./benchmarks.md). For community health metrics, see the [Community Dashboard](../../../docs/community-dashboard.md).
