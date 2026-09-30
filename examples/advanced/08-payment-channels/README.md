# Payment Channels

Specialized payment channels for transacting between two parties with immediate settlement.

## Role in Learning Path

## Functions
- `init(token, participant_a, participant_b, pubkey_a, pubkey_b, expiry)` - Set up the channel. Both participants must authorize. `pubkey_*` are the ed25519 keys each participant uses to sign off-chain state.
- `deposit(from, amount)` - Fund the channel (participant auth required)
- `submit_state(from, balance_a, balance_b, sequence, sig_a, sig_b)` - Update balances with both parties' signatures (participant auth required)
- `close(from)` - Pay out both balances and close the channel (participant auth required)
- `get_info()` - Read the current channel state

## Signed state format
Both participants sign the same bytes:
`contract_address.to_xdr() || balance_a (i128 BE) || balance_b (i128 BE) || sequence (u32 BE)`.
Including the contract address prevents a state from being replayed on another channel. The sequence must strictly increase, and the balances must add up to the deposited total.

## Tests
The unit tests in `src/test.rs` are wired from `lib.rs` via `#[cfg(test)] mod test;`:

```bash
cargo test -p payment-channels
```
This is the **second step** in the [state channels learning path](../README.md#state-channels--payment-channels). After learning generic state channels, this example shows:
- How to apply state channels specifically to payments
- Efficient payment settlement patterns
- Balance and nonce tracking for payment flows
- Fast, low-cost two-party transactions

**Prerequisites:** Start with [`07-state-channels`](../07-state-channels/) to understand the foundation.

**Next steps:**
- **[`03-state-channel-disputes`](../03-state-channel-disputes/)** — Handle disputes when participants disagree
- **[`13-virtual-channel`](../13-virtual-channel/)** — Route through intermediaries for network effects

## Key Concepts

- Specialized payment settlement logic
- Efficient balance updates
- Two-party trust model
- Immediate finality on settlement

## Pattern Progression

**Generic state channels → Payment-specific → Dispute resolution → Virtual routing**

See the [advanced examples README](../README.md) for the full state channels learning path.
