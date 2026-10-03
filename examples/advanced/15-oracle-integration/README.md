# Oracle Integration

Integration patterns for consuming oracle data in other contracts.

## Role in Learning Path

This is the **third step** in the [oracle patterns learning path](../README.md#oracle-patterns--price-feeds). After learning oracle producers and aggregators, this example shows:
- Cross-contract oracle calls
- Data freshness validation in consumers
- Error handling for stale data
- Integration testing patterns
- Safe consumption workflows

**Prerequisites:**
- Start with [`03-oracle-pattern`](../03-oracle-pattern/) for basic oracle mechanics
- Then [`26-data-aggregation-oracle`](../26-data-aggregation-oracle/) for aggregation strategies

**Next steps:**
- **[`06-price-oracle`](../06-price-oracle/)** — Price oracle specifics
- **[`12-oracle-consumer`](../12-oracle-consumer/)** — Advanced consumption patterns
- **[`../defi/11-amm-price-oracle`](../defi/11-amm-price-oracle/)** — AMM-coupled pricing

## Key Concepts

- Cross-contract calls to oracles
- Data freshness checks
- Stale data handling
- Timeout management
- Consumer-side validation
- Error recovery patterns

## Pattern Progression

**Basic producer → Aggregation → Integration → Price-specific → Safe consumption → AMM-coupled**

See the [advanced examples README](../README.md) for the full oracle patterns learning path.
