# Development Workflow

## Property-Based Testing

The cookbook uses [`proptest`](https://crates.io/crates/proptest) to verify invariants that must hold across a wide range of inputs, not just the hand-picked cases covered by unit tests.

### When to add property tests

Add a `proptest` module whenever a contract has a **mathematical invariant** — a statement that must be true for all valid inputs rather than a specific example:

- Arithmetic invariants (token balances, reserve ratios, LP supply)
- Idempotency / symmetry (stake then unstake, deposit then withdraw)
- Monotonicity (reward accumulator never decreases over time)
- Double-action rejection (voting, approvals)

### Where to put them

| Package | Location |
|---------|----------|
| `examples/governance/01-simple-voting` | `src/test.rs` → `mod proptest_voting` |
| `examples/defi/02-constant-product-amm` | `src/prop_test.rs` |
| `examples/defi/07-staking-pool` | `src/test.rs` → proptest functions |
| new examples | `src/prop_test.rs` or a `proptest_*` submodule of `src/test.rs` |

### Conventions

1. **Add `proptest = "1"` to `[dev-dependencies]`** in the crate's `Cargo.toml`.
2. **Gate with `#[cfg(test)]`** — proptest is a dev-only dependency and must not reach production.
3. **Keep case counts low in CI** — use `proptest::test_runner::Config::with_cases(32)` or fewer so the full test suite remains fast. Local runs can override with the `PROPTEST_CASES` env var.
4. **Name the property clearly** — the test name must describe the invariant, not the scenario (e.g., `reserves_positive_after_swap`, not `test_swap`).
5. **Document the property** — a one-line comment above the `#[test]` explaining what invariant is being verified helps reviewers and future contributors.
6. **Use `prop_assert!` / `prop_assert_eq!`** instead of `assert!` so proptest can shrink failing inputs.

### Example

```rust
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    #[test]
    fn deposit_withdraw_symmetric(amount in 1i128..=100_000i128) {
        // Property: staking then unstaking the same amount is a no-op on balance.
        let f = setup(0);
        let staker = Address::generate(&f.env);
        f.stake_tok.init(&staker, &(amount + 1));
        let before = f.stake_tok.balance(&staker);
        f.pool.stake(&staker, &amount);
        f.pool.unstake(&staker, &amount);
        prop_assert_eq!(f.stake_tok.balance(&staker), before);
    }
}
```

### Running property tests

```bash
# Run the full suite (uses with_cases(32) limits)
cargo test -p simple-voting
cargo test -p constant-product-amm
cargo test -p staking-pool

# Run with more cases locally
PROPTEST_CASES=1000 cargo test -p simple-voting proptest
```
