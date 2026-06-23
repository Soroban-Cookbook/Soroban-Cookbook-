# Vault Strategies

**Source:** [`examples/defi/01-vault-strategies`](../../examples/defi/01-vault-strategies/)

A yield-bearing vault that supports multiple pluggable strategies with on-chain risk management.

## What It Does

Users deposit tokens into the vault. The vault allocates those funds to an active **yield strategy**. The admin can switch strategies at any time, subject to risk-management guards that protect depositors.

## Strategy Interface

Every strategy is described by a `StrategyParams` struct returned by the `strategy_params()` factory:

```rust
pub struct StrategyParams {
    pub name: Symbol,
    pub max_allocation_bps: i128,  // max % of TVL (10 000 = 100 %)
    pub expected_apy_bps: i128,    // indicative annual yield
    pub risk_level: RiskLevel,     // Low | Medium | High
}
```

Adding a new strategy means adding a variant to `StrategyType` and a matching arm in `strategy_params()` — no other changes required.

## Strategy Implementations

| Strategy      | APY    | Allocation Cap | Risk   |
|---------------|--------|----------------|--------|
| Conservative  | ~3%    | 100% (no cap)  | Low    |
| Balanced      | ~8%    | 80% of TVL     | Medium |
| Aggressive    | ~20%   | 50% of TVL     | High   |

## Strategy Switching

```rust
// Admin switches from Conservative to Balanced
client.switch_strategy(&admin, &StrategyType::Balanced);

// Query the active strategy's parameters
let info = client.strategy_info();
// info.expected_apy_bps == 800
// info.risk_level == RiskLevel::Medium
```

## Risk Management

### Allocation Cap

Each strategy caps how much of the vault's TVL it may hold. A deposit that would exceed the cap is rejected:

```rust
// Balanced has an 80 % cap.
// Depositing 100 tokens as the only depositor (100 % of TVL) → panic
client.switch_strategy(&admin, &StrategyType::Balanced);
client.deposit(&user, &100); // panics: "Exceeds strategy allocation cap"
```

### Emergency Pause

```rust
client.pause(&admin);    // deposits blocked, withdrawals still open
client.unpause(&admin);  // deposits re-enabled
```

Withdrawals are always permitted so users can exit even during an emergency.

### TVL Guard for Aggressive Strategy

Switching to `Aggressive` is blocked when the vault holds more than 1 000 000 token units:

```rust
client.deposit(&user, &1_000_001);
client.switch_strategy(&admin, &StrategyType::Aggressive);
// panics: "TVL too high for aggressive strategy"
```

## Yield Estimation

```rust
// Estimate yield for 10 000 tokens over 365 days with the active strategy
let yield_amount = client.estimate_yield(&10_000, &365);
// Conservative (300 bps): yield_amount == 300
// Aggressive  (2000 bps): yield_amount == 2000
```

## Key Concepts

- **Strategy interface** — uniform `StrategyParams` struct for all strategies
- **Strategy switching** — admin-gated with TVL circuit-breaker
- **Allocation caps** — per-strategy deposit limits in basis points
- **Emergency pause** — deposits blocked, withdrawals always open
- **Yield estimation** — linear APY model for off-chain planning

## Prerequisites

- [Storage Patterns](./storage-patterns.md)
- [Error Handling](./error-handling.md)
- [Events](./events.md)

## Next

- [Advanced: Timelock](../advanced.md) — combine with a timelock for governance-controlled strategy switches
