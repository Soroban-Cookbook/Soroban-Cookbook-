# 03 · Proposal Lifecycle

**Source:** [`examples/governance/03-proposal-lifecycle/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/03-proposal-lifecycle)

A full proposal state machine modelled after OpenZeppelin Governor: proposals progress through **Draft → Active → Queued → Executed** (or **Defeated / Expired**). Each transition is guarded by time and quorum checks.

## What You'll Learn

- Encoding proposal state as a `#[contracttype]` enum
- State transition guards: who can advance a proposal and when
- The queue-then-execute two-step for timelocked execution
- Cancellation and veto flows

## State Machine

```
Draft ──(activate)──► Active ──(pass + queue)──► Queued ──(execute)──► Executed
                          │                                               
                          └──(fail/quorum miss)──► Defeated              
                          └──(expire)───────────► Expired                
```

## Quick Code

```rust
let pid = client.create_proposal(&admin, &title, &description);
// Wait for vote_start...
client.cast_vote(&alice, &pid, &VoteChoice::For);
// Wait for vote_end...
client.queue(&admin, &pid);      // moves Defeated → Queued (only if passed)
// Wait for timelock...
client.execute(&admin, &pid);    // moves Queued → Executed
```

## Run the Example

```bash
cd examples/governance/03-proposal-lifecycle
cargo test
```

## See Also

- [Governance Overview](../governance.md)
- [01 · Simple Voting](./01-simple-voting.md)
