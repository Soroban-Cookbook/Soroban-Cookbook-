# Proposal Validation

This governance example demonstrates defensive proposal validation before a proposal can be accepted for voting.

## What It Covers

- Validation rules for voting window and duration
- Parameter checks for quorum bounds and metadata payloads
- Conflict detection for overlapping active proposals in the same governance topic
- Proposal lifecycle update for close and re-submit flows

## Best Practices

- Enforce lead-time so voters and operators can review proposals before voting starts.
- Keep quorum checks strict and explicit in basis points.
- Use deterministic conflict detection to prevent simultaneous proposals for the same topic.
- Close stale proposals before opening replacements.
- Emit events for create and close transitions to support auditability.

## Run

```bash
cargo test -p proposal-validation
cargo build --target wasm32-unknown-unknown --release -p proposal-validation
```
