# Oracle Pattern

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
> A single-source oracle that is not called for an extended period will
> have its data — including submitted values, authorization state, and
> freshness timestamps — **silently deleted**.  Production deployments
> **must** extend instance (and any persistent) storage TTL on every
> submission or via an off-chain keeper.  Failure to do so will cause
> the oracle state to be lost, authorized submitter roles to vanish,
> and downstream consumers to read uninitialized data.

Basic oracle with authorized submission and freshness checks.

## Role in Learning Path

This is the **foundation example** in the [oracle patterns learning path](../README.md#oracle-patterns--price-feeds). Start here to understand:
- Oracle architecture and flow
- Authorized data submission
- Freshness validation
- Data availability queries
- Basic consumer interactions

After learning this foundation, proceed to:
- **[`26-data-aggregation-oracle`](../26-data-aggregation-oracle/)** — Aggregate multiple data sources
- **[`15-oracle-integration`](../15-oracle-integration/)** — Integration patterns for consumers
- **[`06-price-oracle`](../06-price-oracle/)** — Price oracle specifics
- **[`12-oracle-consumer`](../12-oracle-consumer/)** — Safe consumption patterns
- **[`../defi/11-amm-price-oracle`](../defi/11-amm-price-oracle/)** — AMM-coupled pricing

## Key Concepts

- Data submission authorization
- Freshness timestamp tracking
- Data staleness detection
- Authorized oracle role
- Single-source pattern

## Pattern Progression

**Basic producer → Aggregation → Integration → Price-specific → Safe consumption → AMM-coupled**

## Use Cases

- Asset price feeds
- Data anchoring
- Timestamp authority
- Single-source reference data

See the [advanced examples README](../README.md) for the full oracle patterns learning path.
