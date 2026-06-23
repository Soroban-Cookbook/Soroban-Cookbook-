# DeFi Examples

Decentralized finance on Soroban: AMMs, lending, yield protocols.

## Staking Pool
A working staking example demonstrates how to build lockup duration options, calculate boost rewards, and enforce early withdrawal penalties.

**Key Concepts:**
- Lockup tiers and term-based incentives
- Withdrawals before maturity incur a fixed penalty
- Longer lockups earn higher boost rewards
- State tracking per staker via durable storage

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
