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

## Pattern Progression

**Generic state channels → Payment-specific → Dispute resolution → Virtual routing**

## Use Cases

- Multi-party payment networks
- High-throughput micropayment systems
- Hub-based layer 2 solutions
- Scalable sidechains

See the [advanced examples README](../README.md) for the full state channels learning path.
