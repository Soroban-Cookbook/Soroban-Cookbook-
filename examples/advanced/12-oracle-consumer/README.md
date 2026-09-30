# Oracle Consumer

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
- [`03-data-aggregation-oracle`](../03-data-aggregation-oracle/) — Aggregation strategies
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
