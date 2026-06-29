# Governance Examples

On-chain governance contracts for Soroban — covering one-address-one-vote systems, time-constrained voting windows, and a full proposal state machine.

---

## Examples

| # | Directory | Pattern | Difficulty |
|---|-----------|---------|------------|
| 01 | [simple-voting](./01-simple-voting/) | Proposal creation, ballot casting, tally, and execution | Beginner |
| 02 | [voting-time-constraints](./02-voting-time-constraints/) | Configurable voting periods, grace periods, quorum, early closure | Intermediate |
| 03 | [proposal-lifecycle](./03-proposal-lifecycle/) | Full state machine: Draft → Active → Queued → Executed / Defeated | Advanced |

---

## Key Patterns

### Admin-Gated Proposal Creation

Only an authorized admin can create proposals. This uses the same `require_auth()` pattern as `examples/basics/03-authentication`:

```rust
pub fn create_proposal(env: Env, admin: Address, title: String, deadline: u64) -> u32 {
    admin.require_auth();
    // ...
}
```

### One-Address-One-Vote

Votes are stored as `Persistent` data keyed on `(proposal_id, voter_address)`. Before accepting a vote, the contract checks for an existing entry:

```rust
let key = DataKey::Vote(proposal_id, voter.clone());
if env.storage().persistent().has(&key) {
    return Err(VotingError::AlreadyVoted);
}
env.storage().persistent().set(&key, &choice);
```

### Time-Based Deadline Enforcement

Voting windows use `env.ledger().timestamp()` (Unix seconds) for deadline checks. The `02-voting-time-constraints` example additionally enforces a grace period after the deadline before execution is allowed:

```rust
let now = env.ledger().timestamp();
assert!(now >= proposal.vote_end + GRACE_PERIOD, "grace period not elapsed");
```

### Vote Choice Enum

All examples use a `#[contracttype]` enum for type-safe ballot choices:

```rust
#[contracttype]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}
```

### Event Emission

Every vote and execution emits a structured event following the `04-events` topic convention, enabling governance indexers to reconstruct full audit trails:

```rust
env.events().publish(
    (symbol_short!("gov"), symbol_short!("vote"), proposal_id),
    (voter.clone(), choice),
);
```

---

## Quick Start

```bash
# Run the beginner voting example
cd examples/governance/01-simple-voting
cargo test

# Run the full governance suite
cd examples/governance/03-proposal-lifecycle
cargo test
```

---

## Security Notes

- **Double-vote prevention**: All examples use persistent storage checks before recording a vote. Tests verify that a second `cast_vote` call from the same address returns `AlreadyVoted`.
- **Premature execution**: `02-voting-time-constraints` and `03-proposal-lifecycle` both enforce that execution cannot happen before the deadline (and optionally, a grace period).
- **Quorum**: `02-voting-time-constraints` enforces a minimum participation threshold before a proposal can be marked as passed. Without quorum enforcement, a single vote can pass a proposal on a low-turnout system.
- **Admin key management**: The admin address is stored at initialization. Use a multi-sig or timelock as the admin in production systems; see `examples/advanced/01-multi-party-auth`.

For a full security guide, see [docs/security-best-practices.md](../../docs/security-best-practices.md).
