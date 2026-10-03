# DeFi Examples

Decentralized finance on Soroban: AMMs, lending, yield protocols.

## Staking Pool

**[07 · Staking Pool](./defi/07-staking-pool.md)** — the canonical staking example. A single-asset pool that distributes reward tokens proportionally to stakers over time.

**Key Concepts:**
- Time-based reward accrual through a global `reward_per_share` accumulator
- Per-user `reward_debt` snapshots, so pending rewards resolve in O(1) instead of iterating over every staker
- Claiming rewards without unstaking
- Stake, unstake, and claim move real SEP-41 tokens via `token::Client::transfer`

> **Also in this category:** [`staking-pool`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/staking-pool) implements a *different* design — fixed 30 / 90 / 180-day lockups, a tiered maturity boost (0% / 10% / 25%), and a 20% penalty for withdrawing early. It is **not** the canonical example. See [how the two differ](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/blob/main/examples/defi/staking-pool/README.md#how-this-differs-from-the-canonical-example).

## 📋 Coming Soon
## 📋 Examples (1 currently)

### [01-vault-strategies](../examples/vault-strategies/)
**Multi-strategy yield vault** with pluggable strategies and risk management.

**Key Concepts:**
- Strategy interface (`StrategyParams` + `StrategyType`)
- Three strategy implementations: Conservative, Balanced, Aggressive
- Admin-gated strategy switching with TVL circuit-breaker
- Emergency pause (deposits blocked, withdrawals always open)
- Allocation caps per strategy in basis points

**Quick Code:**
```rust
// Switch to a higher-yield strategy
client.switch_strategy(&admin, &StrategyType::Balanced);

// Estimate yield for planning
let yield_amount = client.estimate_yield(&10_000, &365);
```

---

## 📋 Coming Soon

### Automated Market Maker (AMM)
**Constant product pools** (x*y=k).

**Key Concepts:**
- Price curves & liquidity
- Swap math with slippage
- LP token mint/burn

### Lending Protocol
**Over-collateralized loans**.

**Key Concepts:**
- Oracle price feeds
- Liquidation thresholds
- Interest accrual

### Yield Vault
**Automated yield optimization**.

**Key Concepts:**
- Strategy rotation
- Performance fees
- Emergency withdrawal

## Prerequisites
- [Basics](../basics.md), [Tokens](../tokens.md)

## Prerequisites
- [Basics](../basics.md), [Tokens](../tokens.md)

## Resources
- [Uniswap V2 Math](https://uniswap.org/whitepaper.pdf)
- Soroban token standards

## Next: [NFTs](../nfts.md)
