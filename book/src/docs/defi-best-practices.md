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
