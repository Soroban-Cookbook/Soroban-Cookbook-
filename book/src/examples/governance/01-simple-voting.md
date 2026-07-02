# 01 · Simple Voting

**Source:** [`examples/governance/01-simple-voting/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/01-simple-voting)

A foundational on-chain governance contract: admin creates proposals, authenticated users cast a single vote, and anyone can query the tally or trigger execution after the deadline.

## What You'll Learn

- Admin-gated proposal creation with `require_auth()`
- One-address-one-vote via persistent storage keyed on `(proposal_id, voter)`
- Time-based deadline enforcement with `env.ledger().timestamp()`
- Structured vote and execution events for governance indexers

## Vote Choices

```rust
#[contracttype]
pub enum VoteChoice { For, Against, Abstain }
```

## Quick Code

```rust
let pid = client.create_proposal(&admin, &title, &deadline_unix);
client.cast_vote(&alice, &pid, &VoteChoice::For);
client.cast_vote(&bob,   &pid, &VoteChoice::Against);
let tally = client.tally(&pid);
client.execute(&pid);   // callable after deadline
```

## Run the Example

```bash
cd examples/governance/01-simple-voting
cargo test
```

## Next: [02 · Voting Time Constraints](./02-voting-time-constraints.md)
