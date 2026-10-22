# Interactive Playground — Intermediate Examples

Run, edit, and explore the **intermediate** Soroban examples locally. Every
snippet on this page is a trimmed version of a real crate in
`examples/intermediate/`, and every snippet is covered by the integration
suite (`cargo test -p integration-tests`), so what you read here is what the
repository actually runs.

| You will learn | Examples |
|----------------|----------|
| Token flows | [Token examples](#1-token-examples) |
| Cross-contract calls | [Cross-contract examples](#2-cross-contract-examples) |
| Access control | [Access control examples](#3-access-control-examples) |

---

## How to Use This Playground

### Prerequisites

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add the Wasm target
rustup target add wasm32-unknown-unknown

# 3. Clone the cookbook
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-
```

### Run Any Example

```bash
# Pattern: cd examples/intermediate/<example-name>
cd examples/intermediate/02-role-based-access-control

# Run the example's own tests
cargo test

# Run the cross-example integration suite
cargo test -p integration-tests
```

### Edit and Experiment

Each intermediate example is an isolated crate. Open its `src/lib.rs`, make a
change, and re-run `cargo test` — the example's own unit tests plus the
cross-contract integration tests catch regressions immediately.

---

## 1. Token Examples

### 1.1 SEP-41 token — the base layer

> **Example:** [`examples/tokens/01-sep41-token`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/01-sep41-token)
> **Book page:** [Token Examples →](./tokens.md)

A fungible token implements the SEP-41 interface: `balance`, `transfer`,
`allowance`, `approve`, `transfer_from`, `burn` and `burn_from`.

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
    Allowance(Address, Address),
}

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Moves `amount` from `from` to `to`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        // The owner of the funds must sign for the transfer.
        from.require_auth();

        let mut from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        from_balance -= amount;
        let to_balance = to_balance.checked_add(amount).unwrap();

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &from_balance);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &to_balance);
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }
}
```

**Try it:**

```bash
cd examples/tokens/01-sep41-token
cargo test
```

**What to edit:** remove `from.require_auth()` and re-run the tests — the
contract no longer enforces ownership of the funds.

### 1.2 Allowance pattern — "spending" someone else's balance

> **Example:** [`examples/tokens/05-allowance-pattern`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/05-allowance-pattern)
> **Book page:** [Token Examples →](./tokens.md)

An allowance lets a *spender* move funds on behalf of an *owner*, with an
expiry ledger so stale approvals die automatically.

```rust
/// The owner approves `spender` to move up to `amount`.
pub fn approve(
    env: Env,
    owner: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) -> Result<(), AllowanceError> {
    owner.require_auth();
    if amount < 0 {
        return Err(AllowanceError::InvalidAmount);
    }
    if expiration_ledger < env.ledger().sequence() {
        return Err(AllowanceError::InvalidExpiration);
    }
    env.storage().persistent().set(
        &DataKey::Allowance(owner, spender),
        &AllowanceValue { amount, expiration_ledger },
    );
    Ok(())
}

/// The spender moves `amount` out of the owner's balance.
pub fn transfer_from(
    env: Env,
    spender: Address,
    owner: Address,
    to: Address,
    amount: i128,
) -> Result<(), AllowanceError> {
    spender.require_auth();

    let allowance = read_allowance(&env, &owner, &spender);
    // An expired allowance is worth nothing.
    if allowance.amount == 0
        || (allowance.expiration_ledger < env.ledger().sequence())
    {
        return Err(AllowanceError::InsufficientAllowance);
    }
    if allowance.amount < amount {
        return Err(AllowanceError::InsufficientAllowance);
    }
    // ... move the funds and decrement the allowance
    Ok(())
}
```

**Try it:**

```bash
cd examples/tokens/05-allowance-pattern
cargo test
```

**What to edit:** drop the expiry check in `read_allowance` and see the
"expired allowance" integration test fail — approvals must expire.

### 1.3 Token wrapper — composing two tokens

> **Example:** [`examples/tokens/06-token-wrapper`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/06-token-wrapper)
> **Book page:** [Token Examples →](./tokens.md)

A wrapper contract holds an underlying token and exposes it under a new
address — the classic building block for vault shares and receipt tokens.

```rust
/// Wrap `amount` of the underlying token into wrapper shares.
pub fn deposit(env: Env, user: Address, amount: i128) -> Result<i128, TokenWrapperError> {
    user.require_auth();

    // Pull the underlying tokens into this contract first ...
    self::TokenClient::new(&env, &underlying).transfer(
        &user,
        &env.current_contract_address(),
        &amount,
    );

    // ... then credit the wrapper shares.
    let shares = deposit_internal(&env, &user, amount);
    Ok(shares)
}
```

**Try it:**

```bash
cd examples/tokens/06-token-wrapper
cargo test
```

---

## 2. Cross-Contract Examples

### 2.1 Ajo factory — deploying contracts from a contract

> **Example:** [`examples/intermediate/ajo-factory`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/ajo-factory)
> **Book page:** [Ajo Factory →](./ajo-factory.md)

A factory stores a Wasm hash and derives a unique address per deployment, so
every instance is isolated but reproducible.

```rust
/// Deploy a new instance of the template contract.
pub fn deploy(env: Env, salt: BytesN<32>) -> Result<Address, FactoryError> {
    let wasm_hash: BytesN<32> = env
        .storage()
        .instance()
        .get(&DataKey::TemplateWasm)
        .ok_or(FactoryError::NotInitialized)?;

    // `with_current_contract` derives a deterministic address from the salt.
    let address = env
        .deployer()
        .with_current_contract(salt)
        .deploy(wasm_hash);

    env.storage()
        .instance()
        .set(&DataKey::DeployedAjo(address.clone()), &true);
    Ok(address)
}
```

**Try it:**

```bash
cd examples/intermediate/ajo-factory
cargo test
```

### 2.2 Contract registry — one address to rule them all

> **Example:** [`examples/advanced/contract-registry`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/contract-registry)
> **Book page:** [Advanced Examples →](./advanced.md)

A registry maps a human-friendly name to a contract address and lets callers
reach other contracts through a single entry point.

```rust
/// Register `implementation` under `name`.
pub fn register(env: Env, name: Symbol, implementation: Address) -> Result<(), RegistryError> {
    admin(&env)?.require_auth();
    env.storage().persistent().set(&DataKey::Contract(name), &implementation);
    Ok(())
}

/// Forward a call to the contract registered under `name`.
pub fn call(env: Env, name: Symbol, fn_name: Symbol, args: Vec<Val>) -> Result<Val, RegistryError> {
    let target: Address = env
        .storage()
        .persistent()
        .get(&DataKey::Contract(name))
        .ok_or(RegistryError::NotFound)?;
    Ok(env.invoke_contract::<Val>(&target, &fn_name, args))
}
```

**Try it:**

```bash
cd examples/advanced/contract-registry
cargo test
```

### 2.3 Oracle pattern — reading external data

> **Example:** [`examples/advanced/03-oracle-pattern`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/03-oracle-pattern)
> **Book page:** [Oracle Pattern →](./oracle-pattern.md)

Contracts cannot fetch prices on their own; an oracle contract is updated by
privileged sources and read by everyone else.

```rust
/// Publish a new price.  Only trusted oracles may do this.
pub fn update_price(env: Env, oracle: Address, asset: Symbol, price: i128) -> Result<(), OracleError> {
    if !env.storage().persistent().has(&DataKey::Oracle(oracle.clone())) {
        return Err(OracleError::Unauthorized);
    }
    oracle.require_auth();
    env.storage()
        .persistent()
        .set(&DataKey::Price(asset.clone(), price), &price);
    Ok(())
}

/// Read the last published price.
pub fn get_price(env: Env, asset: Symbol) -> Result<i128, OracleError> {
    env.storage()
        .persistent()
        .get(&DataKey::Price(asset))
        .ok_or(OracleError::NotFound)
}
```

**Try it:**

```bash
cd examples/advanced/03-oracle-pattern
cargo test
```

---

## 3. Access Control Examples

### 3.1 Role-based access control

> **Example:** [`examples/intermediate/02-role-based-access-control`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/02-role-based-access-control)
> **Book page:** [Intermediate Examples →](./intermediate.md)

Roles are stored per address. Every guarded function checks the role of the
caller before it touches state.

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Role(Address, Role),
    Admin,
}

/// True when `account` holds `role`.
pub fn has_role(env: &Env, account: &Address, role: Role) -> bool {
    env.storage().persistent().has(&DataKey::Role(account.clone(), role))
}

/// Guard used by the privileged entry points.
fn require_role(env: &Env, account: &Address, role: Role) {
    assert!(has_role(env, account, role), "unauthorized");
}
```

**Try it:**

```bash
cd examples/intermediate/02-role-based-access-control
cargo test
```

### 3.2 Hierarchical access control — roles that inherit

> **Example:** [`examples/advanced/16-hierarchical-access-control`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/16-hierarchical-access-control)
> **Book page:** [Advanced Examples →](./advanced.md)

`ADMIN > MANAGER > OPERATOR`: a higher role inherits the permissions of all
lower roles, so an admin never has to be granted permissions one by one.

```rust
pub const ROLE_ADMIN: Symbol = symbol_short!("ADMIN");
pub const ROLE_MANAGER: Symbol = symbol_short!("MANAGER");
pub const ROLE_OPERATOR: Symbol = symbol_short!("OPERATOR");
pub const PERM_MANAGE_ROLES: Symbol = symbol_short!("MNG_ROLES");
pub const PERM_USE_RESOURCES: Symbol = symbol_short!("USE_RES");

/// Grant `role` to `account`.  Only callers that may manage `role` succeed.
pub fn grant_role(env: Env, caller: Address, role: Symbol, account: Address) {
    caller.require_auth();
    require_permission(&env, &caller, PERM_MANAGE_ROLES);
    assert!(can_manage_role(&env, &caller, &role), "Caller cannot manage target role");
    env.storage()
        .persistent()
        .set(&DataKey::RoleMembers(role), &account);
}
```

**Try it:**

```bash
cd examples/advanced/16-hierarchical-access-control
cargo test
```

### 3.3 Pause / unpause — the emergency brake

> **Example:** [`examples/intermediate/03-pause-unpause`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/03-pause-unpause)
> **Book page:** [Pause / Unpause →](./pause-unpause.md)

A single flag halts every state-changing entry point while reads keep
working.

```rust
/// Reject the call when the contract is paused.
fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        Err(ContractError::ContractPaused)
    } else {
        Ok(())
    }
}

/// Emergency switch used by the admin.
pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
    read_admin(&env)?.require_auth();
    env.storage().instance().set(&DataKey::Paused, &true);
    Ok(())
}
```

**Try it:**

```bash
cd examples/intermediate/03-pause-unpause
cargo test
```

---

## 4. Instructions and Guides

Work through the intermediate material in this order:

1. [Playground Tutorial 1 — Hello World](../guides/playground-tutorial-01-hello-world.md)
2. [Playground Tutorial 2 — Contract Storage](../guides/playground-tutorial-02-storage.md)
3. [Playground Tutorial 3 — Error Handling](../guides/playground-tutorial-03-error-handling.md)
4. [Playground Tutorial 4 — Authentication](../guides/playground-tutorial-04-authentication.md)
5. [Playground Tutorial 5 — Multi-Contract Interactions](../guides/playground-tutorial-05-multi-contract.md)
6. [Testing Guide](../guides/testing.md) — mocking auth, time travel, event assertions
7. [Contract Development Guide](../guides/contract-development.md) — project layout and conventions
8. [Deployment Guide](../guides/deployment.md) — ship a contract to testnet
9. [Security Checklist](../guides/security-checklist.md) — review your own contract
10. [Common Patterns](../docs/common-patterns.md) and
    [Factory, Proxy, and Registry Patterns](../docs/cross-contract-patterns.md)

Reference material:

- [Code Examples (all topics)](../docs/code-examples.md) — one tested snippet per topic
- [Token Security Checklist](../docs/token-security-checklist.md)
- [Best Practices](../docs/best-practices.md)
- [Common Pitfalls](../docs/common-pitfalls.md)

---

## 5. Running Everything at Once

```bash
# The intermediate examples' own tests
for dir in examples/intermediate/*/; do
    echo "Testing $dir..."
    (cd "$dir" && cargo test) || exit 1
done

# The cross-contract integration suite that covers the intermediate examples
cargo test -p integration-tests
```

Building the Wasm artifacts:

```bash
for dir in examples/intermediate/*/; do
    cargo build --manifest-path "$dir/Cargo.toml" \
                --target wasm32-unknown-unknown --release
done
```

On Windows (PowerShell):

```powershell
Get-ChildItem examples\intermediate -Directory | ForEach-Object {
    cargo build --manifest-path "$($_.FullName)\Cargo.toml" `
                --target wasm32-unknown-unknown --release
}
```

---

## 6. Next Steps

- [Advanced Examples →](./advanced.md) — proxies, oracles, batch calls
- [Use-Case Examples →](./defi.md) — DeFi, NFTs, governance, tokens
- [Testing Best Practices →](../docs/testing-best-practices.md)
- [Playground Infrastructure →](../playground.md)
