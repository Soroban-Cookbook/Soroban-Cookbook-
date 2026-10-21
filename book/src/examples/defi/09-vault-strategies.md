# 09 · Vault Strategies

**Source:** [`examples/defi/09-vault-strategies/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/09-vault-strategies)

A yield-bearing vault that supports three pluggable strategies — Conservative (~3 % APY), Balanced (~8 % APY), and Aggressive (~20 % APY). The admin can switch the active strategy, subject to TVL circuit-breakers and allocation caps.

## What You'll Learn

- The pluggable strategy interface (`StrategyParams` + `StrategyType`)
- TVL circuit-breaker: strategy switch blocked if total value at risk exceeds a threshold
- Emergency pause: deposits blocked, withdrawals always open
- Basis-point allocation caps per strategy

## Key Concepts

| Concept | Detail |
|---------|--------|
| `StrategyType` enum | `Conservative`, `Balanced`, `Aggressive` |
| TVL check | Switch rejected if `tvl > max_switch_tvl` |
| Pause mode | Admin can pause deposits; withdrawals unaffected |
| `estimate_yield` | Off-chain helper to project returns given amount + days |

## Quick Code

```rust
client.deposit(&user, &10_000_i128);
client.switch_strategy(&admin, &StrategyType::Balanced);
let yield_est = client.estimate_yield(&10_000_i128, &365u64);
client.withdraw(&user, &shares);
```

## Run the Example

```bash
cd examples/defi/09-vault-strategies
cargo test
```

## Next: [10 · Swap Liquidity](./10-swap-liquidity.md)
