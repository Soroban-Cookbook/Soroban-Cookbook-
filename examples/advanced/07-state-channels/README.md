# State Channels

A generic state channel framework for off-chain state updates with on-chain settlement and dispute resolution.

## Role in Learning Path

This is the **foundation example** in the [state channels learning path](../README.md#state-channels--payment-channels). Start here to understand:
- How two parties maintain shared off-chain state
- Signing and validating state updates
- On-chain settlement and finality
- Basic dispute resolution mechanics

After learning this foundation, proceed to:
- **[`08-payment-channels`](../08-payment-channels/)** — Specialize for payment transactions
- **[`34-state-channel-disputes`](../34-state-channel-disputes/)** — Add formal dispute resolution with challenges and proofs
- **[`13-virtual-channel`](../13-virtual-channel/)** — Route through intermediaries for hub-based networks

## Key Concepts

- Off-chain state updates signed by both parties
- Nonce tracking to prevent replay attacks
- On-chain settlement with agreed final state
- Dispute detection and resolution

## Pattern Progression

**Generic → Payment-specific → Dispute resolution → Virtual routing**

See the [advanced examples README](../README.md) for the full state channels learning path.
