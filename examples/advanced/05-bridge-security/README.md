# Bridge Security Example

This advanced example models the security controls around an inbound cross-chain bridge release flow.

It focuses on four controls that teams usually layer together instead of treating bridge security as a single check:

- rate limiting for capped value release per time window
- an emergency pause switch for incident response
- a challenge period before funds can be finalized
- fraud-proof submission that permanently blocks a fraudulent transfer

## Contract Surface

- `initialize(admin, rate_limit_amount, rate_limit_window, challenge_period)`
- `submit_transfer(operator, recipient, amount, source_chain, evidence_hash)`
- `challenge_transfer(challenger, transfer_id)`
- `submit_fraud_proof(reviewer, transfer_id, proof_hash)`
- `finalize_transfer(operator, transfer_id)`
- `pause(admin)` / `unpause(admin)`

## Security Model

1. Every inbound release is recorded as pending first.
2. Pending transfers count against a per-window rate limit.
3. Finalization is blocked until the challenge window expires.
4. Any challenged transfer stays blocked.
5. A submitted fraud proof marks the transfer as fraudulent and non-finalizable.
6. The admin can pause new submissions and finalization during incident response.

## Build

```bash
cd examples/advanced/05-bridge-security
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/advanced/05-bridge-security
cargo test
```