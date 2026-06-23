# Security Audit Preparation

This directory collects the materials needed to take the Soroban Cookbook's
**intermediate examples** through an external security audit with minimal
back-and-forth. The goal is *audit readiness*: a clear scope, an honest record
of what is already known, and a checklist the maintainers can drive to "green"
before engaging an auditor.

> **Status:** Preparation. Nothing here asserts that the code is secure; these
> documents define *what will be reviewed* and *what is already known* so the
> audit itself can focus on finding new issues.

## Documents

| Document | Purpose |
| --- | --- |
| [`audit-scope.md`](./audit-scope.md) | Finalized scope: in/out of scope, objectives, threat model, methodology, severity rubric, and deliverables. |
| [`audit-prep-checklist.md`](./audit-prep-checklist.md) | Repo-wide and per-example readiness checklist to complete before the audit. |
| [`known-issues-log.md`](./known-issues-log.md) | Tracked, already-known issues so the audit does not re-report them. |

## Scope at a glance

The audit targets the eight intermediate examples under
[`examples/intermediate/`](../../examples/intermediate/):

- `02-role-based-access-control`
- `03-pause-unpause`
- `03-priority-queue`
- `ajo-factory`
- `event-history`
- `iterable-mappings`
- `multi-sig-patterns`
- `storage-migration`

See [`audit-scope.md`](./audit-scope.md) for the precise boundaries and
rationale.

## How to use these documents

1. Read [`audit-scope.md`](./audit-scope.md) to confirm the boundaries with the
   auditing party.
2. Work the [`audit-prep-checklist.md`](./audit-prep-checklist.md) until the
   repo-wide gates pass and each in-scope example reaches "ready".
3. Keep [`known-issues-log.md`](./known-issues-log.md) current; hand it to the
   auditor as the baseline of pre-existing issues.

## Related automation

A scheduled dependency advisory scan already exists at
[`.github/workflows/security-audit.yml`](../../.github/workflows/security-audit.yml),
which runs `cargo audit --deny warnings --deny unsound` against the RustSec
Advisory Database. This manual audit complements that automated check; it does
not replace it.
