# Governance Examples

On-chain governance contracts for Soroban — from one-address-one-vote systems through time-constrained voting to a full proposal state machine.

## Examples

| # | Example | Pattern | Difficulty |
|---|---------|---------|------------|
| 01 | [Simple Voting](./governance/01-simple-voting.md) | Proposal creation, ballot casting, tally, and execution | Beginner |
| 02 | [Voting Time Constraints](./governance/02-voting-time-constraints.md) | Voting periods, grace periods, quorum, early closure | Intermediate |
| 03 | [Proposal Lifecycle](./governance/03-proposal-lifecycle.md) | Full state machine: Draft → Active → Queued → Executed | Advanced |

## Key Concepts

- **Admin-gated proposals** — only an authorized admin can create proposals (`require_auth()`)
- **One-address-one-vote** — persistent storage keyed on `(proposal_id, voter)` prevents double voting
- **Timestamp deadlines** — `env.ledger().timestamp()` gates voting windows
- **Typed vote choices** — `#[contracttype]` enum: `For`, `Against`, `Abstain`
- **Structured events** — every vote and execution emits indexer-friendly topics

## Prerequisites

- [Basics](./basics.md)
- [03 · Authentication](./basics.md) — `require_auth()` patterns
- [04 · Events](./basics.md) — structured event emission

## Next: [Tokens](./tokens.md)
