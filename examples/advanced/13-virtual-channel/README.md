# Virtual Payment Channel Pattern

A virtual channel lets two endpoints (Alice, Bob) pay each other through an
intermediary (Ingrid) without putting every balance update on-chain. Only two
on-chain objects exist: a ledger channel between Alice and Ingrid, and a ledger
channel between Ingrid and Bob. The virtual channel between Alice and Bob lives
entirely off-chain.

## Why virtual channels?

With plain ledger channels, paying a stranger requires opening a new on-chain
channel per counterparty. The virtual channel pattern **routes payments through
an intermediary** who already has ledger channels with both endpoints: opening
and updating the virtual channel costs one on-chain transaction total, and
settlement splits the collateral back to the endpoints.

## Lifecycle

| Step | On-chain? | What happens |
|------|-----------|--------------|
| 1. `open_ledger` ×2 | yes | Alice↔Ingrid and Bob↔Ingrid deposit-backed ledger channels |
| 2. `open_virtual` | yes | Ingrid's collateral guarantees the virtual channel `{A, I, B, amount}` |
| 3. updates | **no** | Endpoints exchange signed states `(seq, bal_a, bal_b)` off-chain |
| 4. `materialize` | yes | Latest signed state rebalances both ledger channels (settlement) |
| 5. `close_ledger` | yes | Cooperative close, collateral released |

Only steps 1, 2, 4, 5 touch the chain — step 3 is unlimited and free.

## Routing through intermediaries

- `open_virtual` validates the **topology**: both ledger channels must connect
  the same intermediary to the correct endpoints (`alice↔I`, `I↔bob`).
- Ingrid's collateral in each ledger channel guarantees the virtual channel;
  `open_virtual` checks collateral ≥ `amount` on both legs.
- On `materialize`, the ledger channels are **rebalanced** to the virtual
  balances: Ingrid's exposure drops to `amount − balance` on each leg, and the
  endpoints' collateral equals their virtual balances.

## Settlement

`materialize(channel_id, seq, bal_a, bal_b)` is the dispute path: either
endpoint can force the chain to adopt the latest agreed state. The contract:

- requires **both endpoint signatures** (Soroban `require_auth` — in a real
  deployment, off-chain Ed25519 signatures verified on-chain),
- enforces **monotonic sequence numbers** (stale states cannot be replayed),
- enforces **collateral conservation** (`bal_a + bal_b == amount`, both ≥ 0),
- rebalances both backing ledger channels and stamps `settled_seq`.

## Security properties

- **Replay protection** — sequence numbers are strictly monotonic.
- **Collateral conservation** — balances must sum to the channel capacity.
- **Topology integrity** — ledger channels must match the virtual topology.
- **Authorization** — opening, updating, and closing all require the involved
  parties' `require_auth` (models off-chain signature possession).
- **One-shot materialization** — a virtual state can be materialized once;
  further updates require a fresh channel (keeps the example minimal; a
  production design would allow successive materializations).

## Tests

Run the test suite:

```bash
cargo test -p virtual-channel
```

Coverage: creation (topology checks, collateral checks), routing updates with
sequence monotonicity, settlement rebalancing with collateral conservation,
and cooperative close.

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo build --target wasm32-unknown-unknown --release -p virtual-channel
```
