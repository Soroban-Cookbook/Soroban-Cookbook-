# DeFi Best Practices

This page collects DeFi-specific best practices for security, robustness, and maintainability on Soroban.

## Economic Security

- Align incentives: ensure users, oracles, and liquidators are incentivized to behave correctly.
- Avoid under-collateralized positions by setting conservative collateral ratios and stress-testing price movements.
- Use time-weighted oracles, multi-source pricing, and buffers to protect against flash crashes and price manipulation.
- Clearly separate accounting from execution logic to reduce the impact of contract upgrades or bug fixes.
- Include liquidation penalties, reserve cushions, and mechanism parameters that remain safe under extreme conditions.

## Oracle Usage

- Use trusted, decentralized oracle sources rather than a single price feed.
- Validate oracle data on-chain and reject stale or inconsistent updates.
- Protect against oracle failure modes by setting safe default values, maximum update intervals, and fallback behavior.
- If using automated oracles, restrict who can update feeds and require multi-party confirmation where appropriate.
- Prefer aggregated pricing over single-node or single-source values for economic-critical operations.

## Price Oracle Security

Protocols that price assets with oracles are the most frequently attacked class of DeFi contract. Treat oracle inputs as untrusted unless you have defended against the failure modes below.

### AMM spot price manipulation

Reading the current spot price of an AMM pool (the ratio of `reserve_a / reserve_b`) and using it as a price oracle is unsafe. A single attacker transaction can skew the spot price:

1. Swap a large amount in to push the pool to an extreme ratio.
2. Interact with the vulnerable protocol, which reads the manipulated spot price (e.g., borrows against inflated collateral, or liquidates a position on deflated collateral).
3. Swap back in the same transaction, restoring the pool and paying only fees.

Because the manipulate-and-restore loop happens atomically, the attack costs little more than swap fees whenever the extracted value exceeds them. Defenses:

- Never use the current spot price of a low-liquidity pool as a collateral price.
- Query prices only at the start of a transaction (snapshot `env.ledger().timestamp()`-driven feeds) rather than mid-flow, and re-derive value from externally committed prices.
- Prefer pools with deep liquidity, or better, price assets off an aggregated or time-weighted feed instead of any single pool.

### Time-Weighted Average Prices (TWAP)

A TWAP averages the spot price over a time window, so a one-block manipulation has bounded impact: the attacker must keep the pool skewed for the entire window, holding capital at risk and paying fees the whole time.

- Maintain a cumulative accumulator (`price * elapsed`) plus a timestamp checkpoint each update; the TWAP over a window is the difference of accumulators divided by the elapsed time.
- Choose the window deliberately: longer windows are more manipulation-resistant but staler. Common choices are 30 minutes to several hours for collateral pricing.
- Store the accumulator in persistent storage and update it on a cadence (per user call or via a keeper) so the window is always available; document the window length in the contract interface so integrators know what they are getting.
- For tokens with rebases or fees-on-transfer, weight by liquidity (`price * elapsed * reserve`) rather than time alone.

### Multi-source aggregation

No single feed is trustworthy. Combine multiple independent sources and use their agreement as a health check:

- Read from several oracles (AMM pools, an on-chain median of reporters, and an external feed) and take the median rather than the mean — the median discards a single wildly-deviating source.
- Reject a price if sources disagree beyond a configurable tolerance percentage; disagreement itself is a signal of manipulation or outage.
- For a complete implementation of weighted aggregation, outlier rejection, and staleness handling across multiple feeds, see the [Data Aggregation Oracle example](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/26-data-aggregation-oracle) (`examples/advanced/26-data-aggregation-oracle`).
- See also the [Oracle Pattern example](../examples/oracle-pattern.md) for the basic publisher/consumer structure.

### Staleness and circuit breakers

Even a well-sourced price can stop updating. Protect consumers on both axes:

- Record the ledger timestamp of every price update and **reject prices older than a maximum staleness** (e.g., a few minutes for volatile assets, longer for stablecoins) instead of silently using the last known value.
- Add sanity bounds per asset: a maximum single-update deviation (e.g., ±20%) and absolute min/max band. If a new price violates them, pause the affected market rather than accepting the price.
- Implement a circuit breaker: after N rejected updates or one hard breach, mark the price "invalid" and gate deposits, borrows, and liquidations behind a pause until an authorized party resets it. See the [Circuit Breaker example](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/04-circuit-breaker) (`examples/advanced/04-circuit-breaker`) and the [Oracle Consumer](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/12-oracle-consumer) (`examples/advanced/12-oracle-consumer`, guarded/aggregating/settlement variants) examples.
- Restrict who can publish prices, require auth on every update, and make the admin key a multi-sig — a compromised single-key oracle publisher is an instant exploit of every dependent protocol.

### Oracle failure mode checklist

| Failure | Symptom | Defense |
| --- | --- | --- |
| Flash-loan spot manipulation | Price spikes within one transaction | TWAP or multi-source aggregation; never trust raw AMM spot price |
| Stale feed | No updates during high volatility | Timestamp checks + staleness rejection |
| Broken/compromised source | Wild outlier values | Median aggregation, deviation bounds |
| Total feed outage | No usable price at all | Circuit breaker + market pause |
| Malicious publisher | Arbitrary prices | Multi-sig / multi-publisher, auth on updates |

## Liquidation Design

- Design liquidation logic to preserve solvency and avoid cascading failures.
- Keep auctions or liquidator incentives predictable, with clear rules for when, how, and at what costs positions can be closed.
- Use gradual penalties and slippage limits to reduce market impact during deleveraging events.
- Allow sufficient time for borrowers to react to margin or collateral calls before forced closure.
- Test liquidation workflows across edge cases such as rapid price moves, low liquidity, and partial fills.

## Testing Strategies

- Write unit tests for contract invariants, pricing logic, collateral accounting, and role-based access controls.
- Build integration tests that simulate real-world DeFi flows, including deposits, borrows, rate changes, oracle updates, and liquidations.
- Use fuzz testing and property-based tests for arithmetic safety, bounds checking, and overflow/underflow protection.
- Test failure cases explicitly: stale oracle data, invalid user input, unauthorized access, and emergency pause behavior.
- Consider gas costs and performance in the test suite, verifying that common operations remain within expected limits.

## Related Reading

- [Best Practices](./best-practices.md)
- [Common Pitfalls](./common-pitfalls.md)
- [DeFi Security Checklist](./defi-security-checklist.md)
- [Oracle Pattern Example](../examples/oracle-pattern.md)
