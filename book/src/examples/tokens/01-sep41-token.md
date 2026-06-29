# 01 · SEP-41 Token

**Source:** [`examples/tokens/01-sep41-token/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/01-sep41-token)

The canonical SEP-41 fungible token reference implementation. Covers the complete interface: transfers, allowances, burn, metadata, and structured events.

## SEP-41 Interface

```rust
fn allowance(env, from, spender) -> i128
fn approve(env, from, spender, amount, expiration_ledger)
fn balance(env, id) -> i128
fn transfer(env, from, to, amount)
fn transfer_from(env, spender, from, to, amount)
fn burn(env, from, amount)
fn burn_from(env, spender, from, amount)
fn decimals(env) -> u32
fn name(env) -> String
fn symbol(env) -> String
```

## Run the Example

```bash
cd examples/tokens/01-sep41-token
cargo test
```

## Next: [02 · SEP-41 Extensions](./02-sep41-extensions.md)
