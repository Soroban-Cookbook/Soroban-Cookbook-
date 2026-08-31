# State Channel Disputes

A Soroban Cookbook advanced example demonstrating **off-chain payment/state channel dispute resolution**, including challenge submission, response mechanisms, timeout handling, and fraud proofs.

## Overview

State channels allow parties to execute off-chain transactions rapidly and cheaply, settling on-chain only when closing the channel or resolving a dispute. If one party attempts to close the channel with an outdated or fraudulent state, the on-chain dispute resolution contract provides a mechanism to challenge, respond, slash fraud, and finalize the true state.

---

## Dispute Resolution Architecture

```text
               Alice and Bob open channel on-chain
                                 │
                                 ▼
                      Off-chain transactions
                     (increasing sequence #)
                                 │
        ┌────────────────────────┴────────────────────────┐
        ▼                                                 ▼
Honest Channel Closure                           Dispute Initiated
(both sign latest state)                   (challenger submits state `seq=N`)
                                                          │
                                                          ▼
                                                  Challenge Window
                                               (dispute_deadline set)
                                                          │
                             ┌────────────────────────────┼────────────────────────────┐
                             ▼                            ▼                            ▼
                      Response Submitted           Fraud Proof Submitted           Timeout Expired
                    (counterparty `seq > N`)       (invalid state target)       (no higher seq submitted)
                             │                            │                            │
                             ▼                            ▼                            ▼
                      Updated State                Challenger Slashed            Dispute Finalized
                   (waiting for deadline)       (victim receives deposit)      (state `seq=N` settled)
                             │                                                         │
                             └────────────────────────────┬────────────────────────────┘
                                                          │
                                                          ▼
                                                    Channel Closed
```

---

## Acceptance Criteria & Key Features

1. **Challenge Submission (`open_dispute`)**:
   - Any participant can initiate a challenge by submitting an off-chain signed state (`sequence`, `proposed_balance_a`, `proposed_balance_b`).
   - Transitions channel status to `Disputed` and starts the `dispute_window` timer (`deadline = now + dispute_window`).

2. **Response Mechanism (`respond_dispute`)**:
   - The counterparty can submit a newer valid state (`sequence > current_dispute_sequence`) before the deadline expires.
   - Updates the recorded highest sequence number and balance distribution for final settlement.

3. **Timeout Handling (`finalize_dispute`)**:
   - Ensures disputes cannot be finalized before `env.ledger().timestamp() >= deadline`.
   - Once the timeout expires, any caller can trigger finalization, setting the stored channel balances to the highest submitted state and closing the channel.

4. **Fraud Proofs (`submit_fraud_proof`)**:
   - Demonstrates slashing for invalid/fraudulent state claims.
   - If a party submits a fraudulent challenge (e.g. invalid sequence, invariant breach), the honest party can submit a fraud proof to immediately slash the offender and award 100% of the channel deposit to the victim.

---

## Contract Public API

- `open_channel(env, participant_a, participant_b, balance_a, balance_b, dispute_window)`
- `open_dispute(env, challenger, sequence, proposed_balance_a, proposed_balance_b)`
- `respond_dispute(env, responder, sequence, proposed_balance_a, proposed_balance_b)`
- `submit_fraud_proof(env, submitter, invalid_sequence)`
- `finalize_dispute(env)`
- `get_channel(env)`
- `get_dispute(env)`

---

## Testing & Verification

Run the crate tests:

```bash
cargo test -p state-channel-disputes
```

Build WASM release artifact:

```bash
cargo build --target wasm32v1-none --release -p state-channel-disputes
```
