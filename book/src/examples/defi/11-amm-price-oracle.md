# 11 · AMM Price Oracle

**Source:** [`examples/defi/11-amm-price-oracle/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/11-amm-price-oracle)

A Time-Weighted Average Price (TWAP) oracle built on top of an AMM's reserve history. Accumulates price-seconds each ledger; callers compute the TWAP over any window by snapshotting at two points in time.

## What You'll Learn

- The TWAP accumulator pattern (`price0_cumulative`, `price1_cumulative`)
- Why spot price is unsafe for on-chain use and TWAP is preferred
- Observation storage: recording reserve snapshots at regular intervals
- How lending protocols and derivatives use oracle prices safely

## Key Concepts

| Concept | Detail |
|---------|--------|
| `price_cumulative` | `sum(reserve_b / reserve_a * elapsed_seconds)` over all ledgers |
| TWAP window | `(cumulative_end - cumulative_start) / elapsed_seconds` |
| Manipulation resistance | Attacker must hold an inflated price for the full window |
| Observation TTL | Old observations expire; callers should use recent windows |

## Quick Code

```rust
let snapshot_a = client.current_cumulative_prices();
// ... wait N ledgers ...
let snapshot_b = client.current_cumulative_prices();
let twap = (snapshot_b.price0 - snapshot_a.price0) / elapsed;
```

## Run the Example

```bash
cd examples/defi/11-amm-price-oracle
cargo test
```

## Next: [12 · Farming Pool](./12-farming-pool.md)
