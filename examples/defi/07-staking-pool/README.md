# Staking Pool

**Canonical Staking Pattern** — time-based reward distribution for Soroban smart contracts. This is the recommended starting point for learning staking in the cookbook: it is the `07 · Staking Pool` chapter of the book, and it is the staking example every DeFi index in this repository points at.

> **Note:** This is the canonical staking example in the cookbook. A second, unnumbered crate implements a different, lockup-based design and is kept as a comparison. See [Related Examples](#related-examples) below.

## Features

- Stake and unstake a fungible token
- Distribute rewards over time
- Time-based reward accrual
- Reward claims for stakers
- Pool share tracking via stake balances

## Related Examples

### Other staking designs

This example is the **canonical staking pattern**. One other staking design exists in this repository:

- **[`staking-pool` (legacy)](../staking-pool/)** — Fixed 30 / 90 / 180-day lockups with a tiered maturity boost (0% / 10% / 25%) and a 20% penalty for withdrawing early. Use when you need explicit lockup terms instead of continuous time-based accrual. It is published as `staking-pool-legacy`. See [how it differs](../staking-pool/#how-this-differs-from-the-canonical-example) for a line-by-line comparison.

## Build

```bash
cd examples/defi/07-staking-pool
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/defi/07-staking-pool
cargo test
```
