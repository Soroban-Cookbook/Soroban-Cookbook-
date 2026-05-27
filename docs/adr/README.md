# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Soroban Cookbook project.

ADRs document significant architectural and design decisions — the context that led to them, the decision itself, and the consequences. They live in version control alongside the code so the reasoning is never lost.

## What is an ADR?

An ADR is a short document that captures a single architectural decision. Each ADR describes:

- **Context** — the situation and constraints that made a decision necessary
- **Decision** — what was decided and why
- **Consequences** — what becomes easier, harder, or different as a result

ADRs are immutable once accepted. If a decision changes, a new ADR is written to supersede the old one. This preserves the full history of how the project evolved.

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [ADR-001](./001-record-architecture-decisions.md) | Record Architecture Decisions | Accepted |

## Creating a New ADR

1. Copy [`template.md`](./template.md) to a new file named `NNN-short-title.md` (e.g. `002-workspace-structure.md`)
2. Fill in all sections — context, decision, and consequences
3. Set the status to `Proposed`
4. Open a PR; once merged and agreed upon, update the status to `Accepted`
5. Add the new ADR to the index table above

## ADR Statuses

| Status | Meaning |
| ------ | ------- |
| `Proposed` | Under discussion, not yet decided |
| `Accepted` | Decision made and in effect |
| `Deprecated` | No longer relevant but kept for history |
| `Superseded by ADR-NNN` | Replaced by a newer decision |

## Further Reading

- [Michael Nygard's original ADR article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub organization](https://adr.github.io/)
