# Price Oracle

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
> A price oracle that is not called for an extended period will have its
> data — including asset prices, pair metadata, and freshness timestamps
> — **silently deleted**.  Production deployments **must** extend
> instance (and any persistent) storage TTL on every submission or via
> an off-chain keeper.  Failure to do so will cause the price feed to
> vanish, authorized roles to be lost, and downstream DeFi consumers to
> operate against uninitialized or stale prices.

Price oracle with specific focus on financial data. Specialized producer for asset prices.

## Role in Learning Path

This is the **fourth step** in the [oracle patterns learning path](../README.md#oracle-patterns--price-feeds). After learning general oracle patterns, this example specializes for:
- Asset price feeds
- Precision and decimal handling
- Financial data quirks (minimum prices, decimals, pairs)
- Price feed standards
- Typical DeFi use cases

**Prerequisites:** Understand general oracle patterns first:
- [`03-oracle-pattern`](../03-oracle-pattern/) — Basic oracle mechanics
- [`26-data-aggregation-oracle`](../26-data-aggregation-oracle/) — Aggregation strategies
- [`15-oracle-integration`](../15-oracle-integration/) — Integration patterns

**Next steps:**
- **[`12-oracle-consumer`](../12-oracle-consumer/)** — Advanced consumption patterns
- **[`../defi/11-amm-price-oracle`](../defi/11-amm-price-oracle/)** — AMM-coupled pricing

## Key Concepts

- Asset pair pricing
- Precision and decimal normalization
- Price stability checks
- Financial standard compliance
- Real-world asset prices

## Pattern Progression

**Basic producer → Aggregation → Integration → Price-specific → Safe consumption → AMM-coupled**

See the [advanced examples README](../README.md) for the full oracle patterns learning path.
