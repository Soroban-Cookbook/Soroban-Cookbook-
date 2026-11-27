# Virtual Payment Channels

Virtual payment channels routed through an intermediary: ledger channels, off-chain updates, and on-chain settlement.

## Role in Learning Path

This is the **fourth step** in the [state channels learning path](../README.md#state-channels--payment-channels). After learning the foundations, this example adds:
- Hub-based routing through intermediaries
- Ledger channel connections to the hub
- Off-chain payment routing without direct channels
- Network effects and scalability

**Prerequisites:** Understand all three previous patterns:
- [`07-state-channels`](../07-state-channels/) — Foundation
- [`08-payment-channels`](../08-payment-channels/) — Payment-specific logic
- [`34-state-channel-disputes`](../34-state-channel-disputes/) — Dispute resolution

## Key Concepts

- Ledger channels (hub-to-participant)
- Virtual channels (participant-to-participant routed through hub)
- Off-chain routing with hub guarantees
- Trustless settlement via hub
- Network scalability through indirect connections

## Settlement and the Challenge Window

Closing a virtual channel has two paths:

| Path | Calls | Finality |
|---|---|---|
| Cooperative | `materialize(channel_id, seq, bal_a, bal_b)` with a state both endpoints signed as *final* | Immediate |
| Unilateral | `initiate_close` -> optional `challenge_close` -> `finalize_close` | After the challenge window |

The challenge window stops a party from closing with an outdated state while
the counterparty is offline:

```rust
client.initialize(&admin, &17_280);                  // ~24h at 5s ledgers

// Alice submits an old state (seq 1) that favours her.
let deadline = client.initiate_close(&vc, &1, &90, &10);

// Bob comes back before `deadline` with the latest co-signed state.
client.challenge_close(&vc, &5, &40, &60);

// After the deadline anyone can settle the highest-seq state.
client.finalize_close(&vc);
```

Rules enforced on-chain:

- The window is set once in `initialize` (`1..=MAX_CHALLENGE_WINDOW` ledgers)
  and copied into each virtual channel when it opens.
- `challenge_close` is accepted up to and including `close_deadline`, only with
  a strictly higher `seq`. It does not extend the deadline, so every close
  finishes in bounded time.
- `finalize_close` is accepted only after `close_deadline`.
- Every submitted state must be co-signed by both endpoints
  (`require_auth_for_args`). Regular updates use the `state` tag and
  cooperative final states use the `final` tag, so an old update cannot be
  replayed through `materialize` to skip the window.
- A ledger channel cannot be closed while a virtual channel routed through it
  is still open or pending.

## Pattern Progression

**Generic state channels → Payment-specific → Dispute resolution → Virtual routing**

## Run the Tests

```bash
cargo test -p virtual-channel
```

## Use Cases

- Multi-party payment networks
- High-throughput micropayment systems
- Hub-based layer 2 solutions
- Scalable sidechains

See the [advanced examples README](../README.md) for the full state channels learning path.
