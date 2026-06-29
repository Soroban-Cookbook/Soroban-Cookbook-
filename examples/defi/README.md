# DeFi Examples

Production-ready Soroban smart contract examples covering the most common Decentralized Finance primitives — from basic token swaps to liquidity mining and yield vaults.

All examples compile against the latest Soroban SDK, include comprehensive tests, and emit structured events for off-chain indexers.

---

## Examples

| # | Directory | Pattern | Difficulty |
|---|-----------|---------|------------|
| 01 | [simple-swap](./01-simple-swap/) | Fixed-rate token swap with slippage protection | Beginner |
| 02 | [constant-product-amm](./02-constant-product-amm/) | Uniswap V2-style x·y=k AMM with LP tokens | Intermediate |
| 03 | [lending-pool](./03-lending-pool/) | Deposit/borrow pool with interest accrual | Intermediate |
| 04 | [collateralized-lending](./04-collateralized-lending/) | CDP with collateral ratio and liquidation | Advanced |
| 05 | [flash-loans](./05-flash-loans/) | Atomic, fee-bearing flash loan primitive | Advanced |
| 06 | [flash-loan-use-cases](./06-flash-loan-use-cases/) | Arbitrage, refinancing, and security patterns built on flash loans | Advanced |
| 07 | [staking-pool](./07-staking-pool/) | Staking contract with time-weighted rewards | Intermediate |
| 08 | [liquidity-mining](./08-liquidity-mining/) | Multi-pool liquidity mining with reward epochs | Advanced |
| 09 | [vault-strategies](./09-vault-strategies/) | Yield vault with pluggable Conservative/Balanced/Aggressive strategies | Advanced |
| 10 | [swap-liquidity](./10-swap-liquidity/) | Liquidity provision and removal for an AMM pair | Intermediate |
| 11 | [amm-price-oracle](./11-amm-price-oracle/) | TWAP price oracle derived from AMM reserves | Advanced |
| 12 | [farming-pool](./12-farming-pool/) | Multi-pool reward farming with admin controls | Intermediate |
| 13 | [amm-router](./13-amm-router/) | Multi-hop swap router across AMM pairs | Advanced |

---

## Key Patterns

### Authentication (`require_auth`)

Every state-mutating function that acts on behalf of a user calls `require_auth()` on that user's address, following the pattern established in `examples/basics/03-authentication`.

```rust
pub fn deposit(env: Env, user: Address, amount: i128) {
    user.require_auth();
    // ...
}
```

### Structured Events

All contracts emit events with a consistent `(contract_namespace, action, primary_key)` topic layout, enabling off-chain indexers to filter efficiently (see `examples/basics/04-events`):

```rust
env.events().publish(
    (symbol_short!("defi"), symbol_short!("swap"), from.clone()),
    (amount_in, amount_out),
);
```

### Fixed-Point Arithmetic

Soroban has no floating-point in contracts. All price and ratio calculations use scaled integer arithmetic (typically `i128` with 7-decimal or 18-decimal precision). See `examples/basics/11-primitive-types` for overflow-safe patterns.

### Storage Tier Selection

| Data type | Storage tier | Reason |
|-----------|-------------|--------|
| User balances | `Persistent` | Must survive ledger gaps |
| Pool state / reserves | `Instance` | Scoped to the contract lifetime |
| Temporary locks / nonces | `Temporary` | Cheap, auto-expiry |

See `examples/basics/02-storage-patterns` for the full storage reference.

---

## Quick Start

```bash
# Run a single example
cd examples/defi/01-simple-swap
cargo test

# Build as WASM
cargo build --target wasm32-unknown-unknown --release

# Run the entire DeFi suite
cargo test -p simple-token-swap -p constant-product-amm -p lending-pool
```

---

## Security Notes

- **Reentrancy**: Soroban's single-invocation execution model prevents classic reentrancy, but cross-contract call order still matters. Always update state before making outbound calls.
- **Oracle manipulation**: Examples `11-amm-price-oracle` use TWAPs; never use spot price as an oracle in production collateral systems.
- **Flash loan atomicity**: `05-flash-loans` enforces repayment within the same transaction frame using balance-difference checks.
- **Integer overflow**: All arithmetic uses `checked_add` / `checked_mul` or relies on the `overflow-checks = true` compile profile.

For a full security guide, see [docs/security-best-practices.md](../../docs/security-best-practices.md).
