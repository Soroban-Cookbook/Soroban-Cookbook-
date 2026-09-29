# 01 — Voting Time Constraints

Demonstrates how to build time-bounded governance voting on Soroban.

## Key Concepts

| Feature | Description |
|---------|-------------|
| **Voting period** | Proposals are open for a configurable window (default 3 days) |
| **Proposal deadline** | Votes submitted after the deadline are rejected |
| **Grace period** | A quiet window after voting closes before execution is allowed (default 1 day) |
| **Early closure** | Admin can close a proposal early once quorum is reached |

## Contract Functions

| Function | Description |
|----------|-------------|
| `initialize(admin, voting_period, grace_period)` | Set up the contract |
| `create_proposal(creator, proposal_id, quorum_threshold)` | Open a new vote |
| `vote(voter, proposal_id, support)` | Cast a for/against vote |
| `close_early(admin, proposal_id)` | Close early if quorum met |
| `finalize(proposal_id)` | Transition state after deadline/grace period |
| `execute(admin, proposal_id)` | Execute a proposal in `Executable` state |
| `get_proposal(proposal_id)` | Read full proposal state |
| `has_voted(proposal_id, voter)` | Check if an address has voted |

## Proposal Lifecycle

```
create_proposal
      │
      ▼
  [Active] ─── vote() ─── ...
      │
   deadline passes
      │
      ▼
[GracePeriod]
      │
   grace period elapses
      │
      ▼
[Executable] ──── execute() ───▶ [Executed]
```

Early closure shortcut (admin only, quorum required):

```
[Active] ──── close_early() ───▶ [GracePeriod] ──▶ [Executable] ──▶ [Executed]
```

## How to Run

```bash
# Unit tests
cargo test -p voting-time-constraints

# WASM release build
cargo build -p voting-time-constraints --target wasm32-unknown-unknown --release
```

## Storage

- **Instance storage**: admin address, voting period, grace period config
- **Persistent storage**: per-proposal state (`Proposal` struct), per-voter vote record
