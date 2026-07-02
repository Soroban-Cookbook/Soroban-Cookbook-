# 02 — Token-Weighted Voting

A token-weighted governance example that demonstrates how token balances drive voting power, including delegation and vote snapshots.

## Concepts

- **Token-weighted voting**: voting power is derived from on-chain token balances.
- **Snapshot voting**: proposals record a ledger snapshot so voting power is locked at proposal submission.
- **Delegation**: voters can delegate their vote to another address.
- **Quorum enforcement**: proposals require a minimum total vote weight to pass.
- **Cross-contract execution**: passed proposals execute a target contract action.

## Features

- `initialize(admin, min_quorum)` — set the governance admin and quorum.
- `mint(admin, to, amount)` — issue voting tokens.
- `transfer(from, to, amount)` — move tokens between users.
- `delegate(delegator, delegatee)` — delegate voting power to another address.
- `create_proposal(proposer, ...)` — create a draft proposal.
- `submit_proposal(proposer, proposal_id, voting_duration, execution_duration)` — start voting and capture a snapshot.
- `vote(voter, proposal_id, approve)` — cast a token-weighted vote.
- `vote_for(delegatee, proposal_id, delegator, approve)` — vote on behalf of a delegator.
- `execute_proposal(executor, proposal_id)` — execute the proposal action after passing.

## Build

```bash
cargo build --target wasm32-unknown-unknown --release -p token-voting
```

## Test

```bash
cargo test -p token-voting
```
