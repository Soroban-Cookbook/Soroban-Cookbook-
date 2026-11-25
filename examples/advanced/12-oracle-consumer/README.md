# Oracle Consumer

> **⚠️ UNAUDITED EXAMPLE — NOT FOR PRODUCTION USE**
>
> The contract code and patterns shown on this page have **not been audited**.
> They are provided solely as a learning resource to illustrate Soroban
> development techniques.  **Do not deploy this contract with real funds
> or in a production environment without a professional security audit.**
>
> **Reentrancy:** Soroban's execution model does not support re-entrant
> cross-contract calls within the same transaction — re-entry is a
> protocol-level impossibility on Soroban.  Any reentrancy-style guards
> in this example are therefore illustrative rather than strictly
> necessary.
>
> **Storage TTL / data-expiry:** Soroban instance and persistent storage
> entries expire after a ledger-defined TTL (default ~30 days on Mainnet).
> Oracle consumer contracts that are not called for an extended period
> will have their cached data, quorum configuration, and circuit-breaker
> state **silently deleted**.  Production deployments **must** extend
> instance (and any persistent) storage TTL on every call or via an
> off-chain keeper.  Failure to do so will cause cached oracle values
> to vanish, safety thresholds to be lost, and settlement logic to
> operate against uninitialized state.

Three oracle consumer contracts: validated cache, quorum median consensus, and settlement circuit breaker.

## Role in Learning Path

This is the **fifth step** in the [oracle patterns learning path](../README.md#oracle-patterns--price-feeds). After learning oracle producers, this example shows:
- Consumer-side validation and caching
- Consensus via quorum/median algorithms
- Circuit breaker safety mechanisms
- Multi-oracle aggregation on consumption side
- Production-grade safety patterns

**Prerequisites:** Understand oracle producer patterns:
- [`03-oracle-pattern`](../03-oracle-pattern/) — Basic oracle mechanics
- [`26-data-aggregation-oracle`](../26-data-aggregation-oracle/) — Aggregation strategies
- [`15-oracle-integration`](../15-oracle-integration/) — Integration patterns
- [`06-price-oracle`](../06-price-oracle/) — Price oracle specifics

**Next step:**
- **[`../defi/11-amm-price-oracle`](../defi/11-amm-price-oracle/)** — AMM-coupled pricing alternative

## Key Concepts

- Consumer-side data validation
- Caching strategies
- Quorum consensus (majority rules)
- Median consensus (statistical robustness)
- Circuit breaker safety
- Fallback mechanisms

## Three Consumer Variants

1. **Validated Cache** — Fetch oracle data and cache locally with validation
2. **Quorum Median** — Consensus from multiple oracles using median
3. **Settlement Circuit Breaker** — Emergency pause on price anomalies

## Pattern Progression

**Basic producer → Aggregation → Integration → Price-specific → Safe consumption → AMM-coupled**

See the [advanced examples README](../README.md) for the full oracle patterns learning path.
