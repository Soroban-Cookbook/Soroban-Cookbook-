# Simple Voting

A foundational on-chain governance contract for Soroban that demonstrates the core voting lifecycle: proposal creation, authenticated ballot casting, vote tallying, and result execution.

## What This Example Shows

- **Admin-gated proposal creation** using `require_auth()` (from `03-authentication`)
- **One-address-one-vote enforcement** via persistent storage keyed on `(proposal_id, voter)`
- **Vote options** as a `#[contracttype]` enum: `For`, `Against`, `Abstain`
- **Time-based deadline enforcement** with `env.ledger().timestamp()`
- **Event emission** for every vote and execution (indexer-friendly, from `04-events` pattern)
- **Typed custom errors** with `#[contracterror]`

## Key Concepts

| Concept | Where to look |
|---------|--------------|
| `require_auth()` | `cast_vote`, `create_proposal` |
| Persistent vote storage | `DataKey::Vote(proposal_id, voter)` |
| Timestamp-gated logic | `close_voting`, `execute` |
| Event topics layout | `(symbol!("vote"), proposal_id, voter)` |
| Error enum | `VotingError` |

## Contract Interface

```rust
fn create_proposal(env, admin, title, deadline_unix) -> u32
fn cast_vote(env, voter, proposal_id, choice: VoteChoice) -> ()
fn tally(env, proposal_id) -> VoteTally
fn execute(env, proposal_id) -> VoteResult
fn get_proposal(env, proposal_id) -> Proposal
```

## How to Run

```bash
cd examples/governance/01-simple-voting

# Run all tests
cargo test

# Build the WASM contract
cargo build --target wasm32-unknown-unknown --release
```

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Contract implementation, types, and inline documentation |
| `src/test.rs` | Unit tests covering full voting lifecycle and edge cases |
| `Cargo.toml` | Crate metadata and workspace dependencies |

## Next Steps

- **[02-voting-time-constraints](../02-voting-time-constraints/)** — Adds configurable voting periods, grace periods, quorum thresholds, and early closure
- **[03-proposal-lifecycle](../03-proposal-lifecycle/)** — Full proposal state machine: Draft → Active → Queued → Executed/Defeated
