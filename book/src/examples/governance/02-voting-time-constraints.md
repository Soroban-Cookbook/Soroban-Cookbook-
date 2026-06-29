# 02 · Voting Time Constraints

**Source:** [`examples/governance/02-voting-time-constraints/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/02-voting-time-constraints)

Adds configurable voting periods, a post-deadline grace period before execution, quorum thresholds, and early closure when a super-majority is reached.

## What You'll Learn

- Separate `vote_start` and `vote_end` timestamps per proposal
- Grace period: `execute` is blocked until `vote_end + grace_period`
- Quorum: minimum participation required before a result is binding
- Early closure: admin can close voting early if a configured threshold is met

## Time Windows

```
vote_start ────────── vote_end ──── grace_period ──── executable
              voting window             buffer
```

## Key Concepts

| Concept | Detail |
|---------|--------|
| `vote_start` | Votes rejected before this timestamp |
| `vote_end` | Votes rejected after this timestamp |
| `grace_period` | Execution blocked until `vote_end + grace_period` |
| Quorum | `(for + against + abstain) >= min_participation` |
| Early close | Admin can end voting before `vote_end` if threshold met |

## Run the Example

```bash
cd examples/governance/02-voting-time-constraints
cargo test
```

## Next: [03 · Proposal Lifecycle](./03-proposal-lifecycle.md)
