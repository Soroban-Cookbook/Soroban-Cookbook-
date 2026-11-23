# Data Aggregation Oracle

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
> An aggregation oracle that is not called for an extended period will
> have its data — including aggregated values, source records, and
> freshness timestamps — **silently deleted**.  Production deployments
> **must** extend instance (and any persistent) storage TTL on every
> submission or via an off-chain keeper.  Failure to do so will cause
> the oracle history to be lost and downstream consumers to read
> uninitialized or stale data.

Data aggregation with manipulation detection and outlier filtering.

## Role in Learning Path

This is the **second step** in the [oracle patterns learning path](../README.md#oracle-patterns--price-feeds). After learning single-source oracles, this example shows:
- Multiple data source aggregation
- Outlier detection and filtering
- Manipulation resistance
- Median and consensus strategies
- Robust data validation

**Prerequisites:** Start with [`03-oracle-pattern`](../03-oracle-pattern/) to understand basic oracle mechanics.

**Next steps:**
- **[`15-oracle-integration`](../15-oracle-integration/)** — Integration patterns for consumers
- **[`06-price-oracle`](../06-price-oracle/)** — Price oracle specifics
- **[`12-oracle-consumer`](../12-oracle-consumer/)** — Safe consumption patterns
- **[`../defi/11-amm-price-oracle`](../defi/11-amm-price-oracle/)** — AMM-coupled pricing

## Key Concepts

- Multi-source data collection
- Outlier detection (e.g., remove extremes)
- Median/consensus computation
- Manipulation resistance
- Threshold validation
- Feed quality scoring

## Pattern Progression

**Basic producer → Aggregation → Integration → Price-specific → Safe consumption → AMM-coupled**

See the [advanced examples README](../README.md) for the full oracle patterns learning path.
