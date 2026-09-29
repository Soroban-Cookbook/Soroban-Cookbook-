# DeFi Examples

This category contains examples related to Decentralized Finance (DeFi) protocols. These contracts demonstrate common financial primitives and systems built on Soroban.

## What's Inside?

- **Automated Market Makers (AMMs)**: Examples of different AMM designs.
- **Lending & Borrowing**: Implementations of lending pools and collateralized debt positions.
- **Vaults & Yield Farming**: Contracts for yield aggregation and automated strategies.
- **Farming Pool**: Multi-pool reward distribution system with admin controls.
- **Escrow**: Trustless escrow contracts for secure value exchange.

## Planned Examples

### Vaults & Yield
Automated yield aggregation and reward systems.
- **Planned:** Yield Vaults, Staking Pools, Liquidity Mining incentives.

### Derivatives & Advanced Financials
Complex financial instruments and stablecoin models.
- **Planned:** Options protocols, Perpetual Swaps, Synthetic Assets, Collateralized Stablecoins.

## ≡ƒôï Available Examples

- **Staking Pool** - Rewards for locking up balances with lockup duration options, early withdrawal penalties, and boost incentives for longer locks.
  - [Staking Pool example](./staking-pool/)
- **Automatic Snapshot Triggers** - Time-based & event-based snapshot triggers with pruning.
  - [Snapshot Triggers example](./14-automatic-snapshot-triggers/)
  - **Cross-domain pattern:** The same snapshot-trigger pattern is implemented across domains: [`tokens/10-automatic-snapshot-triggers`](../tokens/10-automatic-snapshot-triggers/), [`governance/11-automatic-snapshot-triggers`](../governance/11-automatic-snapshot-triggers/), [`nfts/05-automatic-snapshot-triggers`](../nfts/05-automatic-snapshot-triggers/). All implement the same time-based and event-based snapshot mechanics; only the domain-specific logic differs.
- **Reward Strategies** - Linear, exponential decay, and performance-based reward distribution.
  - [Reward Strategies example](./15-reward-strategies/)

## ≡ƒôï Planned Examples

- **Constant Product AMM** - Core liquidity pool mechanics (x * y = k).
- **Simple Lending** - Basic lending and borrowing with interest.
- **Yield Vault** - Automated yield harvesting and compounding.
- **Flash Loans** - Uncollateralized borrowing within a single transaction.
- **Stablecoin** - Collateral-backed stable asset.

## ⚠️ Security First

DeFi protocols are high-stakes. Before deploying:
1. **Audit your code** - Have it reviewed by security experts.
2. **Test extensively** - Simulate extreme market conditions and edge cases.
3. **Oracle Safety** - Ensure price feeds are secure and resistant to manipulation.
4. **Safety Checks** - Implement slippage protection and circuit breakers.

## 🎯 Prerequisites

Before diving into DeFi examples, ensure you understand:
- [Basic Examples](../basics/) - Core concepts.
- [Token Examples](../tokens/) - Fungible token standards.
- [Intermediate Patterns](../intermediate/) - Security and access control.
- [Advanced Patterns](../advanced/) - Complex architectural designs.
- `01-simple-amm`: A basic constant-product AMM.
- `02-constant-product-amm`: Uniswap V2-style AMM with liquidity provider tokens.
- `03-lending-pool`: A contract for depositing assets and borrowing against them.
- `04-yield-vault`: A simple vault that implements a basic yield strategy.
- `04-escrow`: A multi-party escrow contract.
- `14-automatic-snapshot-triggers`: Time-based & event-based snapshots with pruning.
