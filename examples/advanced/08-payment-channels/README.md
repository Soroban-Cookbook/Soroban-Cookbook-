# Payment Channels

Bidirectional payment channel example with off-chain state updates and final settlement.

## Use Cases
- Instant micro-payments between two parties
- Recurring payments without per-payment tx fees
- Scalable payment hub via off-chain balance updates

## Functions
- `init` - Set up channel with token, participants, expiry
- `deposit` - Fund the channel
- `submit_state` - Update balances with both parties' signatures
- `close` - Finalize and pay out

## Tests
`cargo test`
