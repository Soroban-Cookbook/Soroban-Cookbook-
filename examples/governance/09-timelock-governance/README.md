# Timelock Governance

This example demonstrates how to implement a timelock governance contract on Soroban. A timelock introduces a mandatory delay between when a proposal is queued and when it can be executed. This delay gives users time to react to governance decisions (e.g., withdrawing funds) before they take effect.

## Concepts Covered

- Proposal Queuing: Registering a target contract, function, and arguments.
- Mandatory Delays: Ensuring a minimum amount of time passes before execution.
- Veto / Cancellation: Allowing admins or guardians to cancel a malicious or erroneous proposal.
- Emergency Execution: Allowing an immediate override in case of critical bugs.

## Usage

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## Structure

- `init`: Initializes the contract with an admin and a minimum delay.
- `queue`: Queues a new proposal with a specific delay (must be >= min_delay).
- `execute`: Executes a queued proposal if the delay has passed.
- `cancel`: Cancels a queued proposal (veto).
- `emergency_execute`: Bypasses the delay to execute a queued proposal immediately (admin only).
