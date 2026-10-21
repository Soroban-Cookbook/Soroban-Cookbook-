# Vault Strategies

A yield-bearing vault that supports multiple pluggable strategies with built-in risk management.

## Overview

Users deposit tokens into the vault. The vault allocates those funds to an active **yield strategy**. The admin can switch strategies at any time, subject to risk-management guards that protect depositors.

```
User ──deposit/withdraw──► VaultContract
                                │
                    ┌───────────┼───────────┐
                    ▼           ▼           ▼
              Conservative  Balanced  Aggressive
              (~3% APY)    (~8% APY)  (~20% APY)
```

## Strategy Interface

Every strategy exposes a `StrategyParams` struct:

| Field                | Type        | Description                                      |
|----------------------|-------------|--------------------------------------------------|
| `name`               | `Symbol`    | Human-readable identifier                        |
| `max_allocation_bps` | `i128`      | Max % of vault TVL (basis points, 10 000 = 100%) |
| `expected_apy_bps`   | `i128`      | Indicative annual yield in basis points          |
| `risk_level`         | `RiskLevel` | `Low` / `Medium` / `High`                        |

```rust
pub struct StrategyParams {
    pub name: Symbol,
    pub max_allocation_bps: i128,
    pub expected_apy_bps: i128,
    pub risk_level: RiskLevel,
}
```

## Strategy Implementations

### 1. Conservative (`StrategyType::Conservative`)

- **Target**: Money-market / stable lending
- **APY**: ~3% (300 bps)
- **Allocation cap**: 100% (no cap)
- **Risk**: Low — principal protection focus

### 2. Balanced (`StrategyType::Balanced`)

- **Target**: Diversified LP positions
- **APY**: ~8% (800 bps)
- **Allocation cap**: 80% of TVL
- **Risk**: Medium — moderate impermanent-loss exposure

### 3. Aggressive (`StrategyType::Aggressive`)

- **Target**: Leveraged yield farming
- **APY**: ~20% (2 000 bps)
- **Allocation cap**: 50% of TVL
- **Risk**: High — liquidation risk; blocked for large vaults

## Strategy Switching

Only the admin can switch strategies:

```rust
client.switch_strategy(&admin, &StrategyType::Balanced);
```

The switch is subject to the TVL guard (see Risk Management below).

## Risk Management

Three layers of protection are enforced on-chain:

### 1. Allocation Cap

Each strategy declares a `max_allocation_bps`. A deposit is rejected if it would
cause the deposited amount to exceed that fraction of the new total TVL.

```
deposit_amount * 10_000 > new_total * max_allocation_bps  →  panic
```

### 2. Emergency Pause

The admin can pause the vault at any time. While paused:
- **Deposits are blocked** — no new capital enters the strategy.
- **Withdrawals remain open** — users can always exit.

```rust
client.pause(&admin);   // block deposits
client.unpause(&admin); // re-enable deposits
```

### 3. TVL Guard for Aggressive Strategy

Switching to `Aggressive` is blocked when the vault's TVL exceeds
`MAX_TVL_FOR_AGGRESSIVE` (1 000 000 token units). Large vaults must stay in
lower-risk strategies.

```rust
// Panics if total_deposits > 1_000_000
client.switch_strategy(&admin, &StrategyType::Aggressive);
```

## Key Concepts

- **Strategy interface** — `StrategyParams` struct + `strategy_params()` factory
- **Strategy switching** — admin-gated with TVL circuit-breaker
- **Allocation caps** — per-strategy deposit limits in basis points
- **Emergency pause** — deposits blocked, withdrawals always open
- **Yield estimation** — `estimate_yield(amount, periods)` for off-chain planning

## Running the Tests

```bash
cargo test -p vault-strategies
```

## Adding a New Strategy

1. Add a variant to `StrategyType`.
2. Add a matching arm in `strategy_params()` with appropriate `StrategyParams`.
3. Add risk checks in `switch_strategy()` if needed.
4. Write tests covering the new strategy's params and any new guards.

## Security Considerations

- This example uses a **simulated yield model**. A production vault would call
  an external protocol (lending pool, AMM) via cross-contract calls.
- The allocation cap check uses integer arithmetic — overflow is guarded with
  `checked_mul` / `checked_add`.
- The emergency pause only blocks deposits; withdrawals are always available to
  prevent a "rug by pause" attack vector.
- Admin key management is out of scope here; in production use a multisig or
  timelock (see [`02-timelock`](../../advanced/02-timelock/)).
