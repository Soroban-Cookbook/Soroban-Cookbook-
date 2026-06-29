# Token Examples

Soroban fungible token contracts from a minimal SEP-41 implementation to advanced extensions, mint/burn flows, allowances, wrappers, and optimized batch operations.

---

## Examples

| # | Directory | Pattern | Difficulty |
|---|-----------|---------|------------|
| 01 | [sep41-token](./01-sep41-token/) | Complete SEP-41 fungible token with metadata, approvals, and events | Beginner |
| 02 | [sep41-extensions](./02-sep41-extensions/) | Optional extensions: permit (EIP-2612 equivalent), batch transfer, batch approve | Intermediate |
| 03 | [optimized-operations](./03-optimized-operations/) | Storage-layout optimizations and batch operation patterns with benchmarks | Advanced |
| 04 | [mint-burn](./04-mint-burn/) | Admin-controlled minting and user burn flows with supply cap enforcement | Beginner |
| 05 | [allowance-pattern](./05-allowance-pattern/) | Delegated spending with `approve`/`transfer_from`, expiration ledgers, and revocation | Intermediate |
| 06 | [token-wrapper](./06-token-wrapper/) | 1:1 wrapper around an existing token with deposit, withdraw, and invariant tests | Intermediate |
| 07 | [token-metadata](./07-token-metadata/) | Mutable and immutable on-chain metadata with update authorization | Beginner |
| 08 | [multi-token-balance-manager](./08-multi-token-balance-manager/) | Single contract managing balances across multiple token addresses | Intermediate |
| 09 | [optimized-token-ops](./09-optimized-token-ops/) | Batched transfers and storage-layout micro-optimizations | Intermediate |

---

## Key Patterns

### SEP-41 Interface

Soroban's fungible token standard (SEP-41) defines a minimal interface that all compliant tokens must implement. `01-sep41-token` provides the canonical reference implementation:

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

### Mint / Burn Controls

The `04-mint-burn` example demonstrates the standard admin-controlled supply management pattern using `require_auth()` (from `03-authentication`):

```rust
pub fn mint(env: Env, admin: Address, to: Address, amount: i128) {
    admin.require_auth();
    let supply: i128 = get_supply(&env);
    assert!(supply.checked_add(amount).unwrap() <= MAX_SUPPLY, "supply cap exceeded");
    write_balance(&env, &to, get_balance(&env, &to) + amount);
    write_supply(&env, supply + amount);
}
```

### Allowance with Expiration

`05-allowance-pattern` stores allowances alongside an `expiration_ledger` to prevent stale approvals from being exercised indefinitely:

```rust
#[contracttype]
pub struct AllowanceData {
    pub amount: i128,
    pub expiration_ledger: u32,
}
```

### Token Wrapper Invariant

`06-token-wrapper` maintains the invariant that wrapped supply always equals the contract's underlying balance. Tests verify this after every deposit and withdrawal:

```
wrapped_supply == underlying.balance(wrapper_contract)
```

### Storage Layout for Balances

For large token contracts, `03-optimized-operations` demonstrates key packing strategies that reduce per-entry ledger costs:

```rust
// Less efficient: two separate keys
DataKey::Balance(address)
DataKey::Allowance(from, spender)

// More efficient: packed into instance storage for low-cardinality data
```

### Event Topics

Token events follow a consistent `(symbol!("token"), action, address)` layout:

```rust
env.events().publish(
    (symbol_short!("token"), symbol_short!("transfer"), from.clone()),
    (to.clone(), amount),
);
```

---

## Quick Start

```bash
# Run the SEP-41 reference implementation
cd examples/tokens/01-sep41-token
cargo test

# Run benchmarks for optimized operations
cd examples/tokens/03-optimized-operations
cargo bench
```

---

## Security Notes

- **Integer overflow**: Use `checked_add` / `checked_sub` for all balance arithmetic, or rely on the workspace's `overflow-checks = true` release profile.
- **Allowance race conditions**: An allowance change from a non-zero value to a different non-zero value is susceptible to a front-running double-spend. `05-allowance-pattern` documents the mitigation (set to zero first, or use `increase_allowance`/`decrease_allowance`).
- **Supply cap enforcement**: Always check the cap *before* minting, not after. An overflow before the check can bypass the cap with `overflow-checks = false`.
- **Wrapper peg risk**: `06-token-wrapper` is only as sound as the underlying token. A pausable or blacklistable underlying can lock wrapper holders.

For a full security guide, see [docs/security-best-practices.md](../../docs/security-best-practices.md).
