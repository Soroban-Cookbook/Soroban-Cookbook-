# Code Examples

Every guide in this book points at a runnable example. This page is the
single index of **tested code snippets**: one short, self-contained snippet
per topic, the crate it comes from, and the command that runs its tests.

Everything here is exercised by the repository test-suites:

```bash
# The example's own unit tests
cd examples/<category>/<example> && cargo test

# The cross-example integration suite
cargo test -p integration-tests
```

---

## Table of Contents

1. [Storage](#1-storage)
2. [Authentication](#2-authentication)
3. [Events](#3-events)
4. [Errors and validation](#4-errors-and-validation)
5. [Tokens](#5-tokens)
6. [Cross-contract calls](#6-cross-contract-calls)
7. [Access control](#7-access-control)
8. [DeFi](#8-defi)
9. [NFTs](#9-nfts)
10. [Governance](#10-governance)
11. [Testing](#11-testing)

---

## 1. Storage

> **Full example:** [`examples/basics/02-storage-patterns`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/02-storage-patterns)

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Persistent(Symbol),
    Temporary(Symbol),
    Instance(Symbol),
}

// Persistent survives upgrades and needs explicit TTL management.
pub fn set_persistent(env: Env, key: Symbol, value: u64) {
    let storage_key = DataKey::Persistent(key);
    env.storage().persistent().set(&storage_key, &value);
    env.storage()
        .persistent()
        .extend_ttl(&storage_key, 1000, 10000);
}

pub fn get_persistent(env: Env, key: Symbol) -> Option<u64> {
    env.storage().persistent().get(&DataKey::Persistent(key))
}
```

| Storage | Lifetime | Use it for |
|---------|----------|------------|
| `persistent()` | until TTL expires | balances, ownership |
| `instance()` | contract instance | admin config, fees |
| `temporary()` | current ledger | reentrancy guards, caches |

**Run:** `cd examples/basics/02-storage-patterns && cargo test`

---

## 2. Authentication

> **Full example:** [`examples/basics/03-authentication`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/03-authentication)

```rust
// The owner of the funds signs for the transfer.
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
    from.require_auth();

    let from_bal: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Balance(from.clone()))
        .unwrap_or(0);
    if from_bal < amount {
        return Err(AuthError::InsufficientBalance);
    }
    // ... update both balances
    Ok(())
}

// Authorize a call for *specific* arguments only.
pub fn withdraw_exact(env: Env, user: Address, amount: i128) {
    user.require_auth_for_args((&user, &amount).into_val(&env));
}
```

**Run:** `cd examples/basics/03-authentication && cargo test`

---

## 3. Events

> **Full example:** [`examples/basics/04-events`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/04-events)

```rust
#[contracttype]
pub struct TransferEventData {
    pub amount: i128,
    pub memo: u64,
}

// Topics are indexed (max 4), the payload carries the details.
env.events().publish(
    (
        symbol_short!("events"),
        symbol_short!("transfer"),
        sender,
        recipient,
    ),
    TransferEventData { amount, memo },
);
```

Off-chain query order: `"events"` → `"events"+"transfer"` → `+sender` → `+recipient`.

**Run:** `cd examples/basics/04-events && cargo test`

---

## 4. Errors and validation

> **Full example:** [`examples/basics/03-custom-errors`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/basics/03-custom-errors)

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    InvalidInput        = 1,
    Unauthorized        = 2,
    NotFound            = 3,
    InsufficientBalance = 4,
}

pub fn validate_input(_env: Env, value: i64) -> Result<(), ContractError> {
    if value <= 0 {
        Err(ContractError::InvalidInput)
    } else {
        Ok(())
    }
}

// Propagate errors with `?` instead of matching everywhere.
pub fn complex_operation(env: Env, amount: u64) -> Result<u64, ContractError> {
    validate_amount(amount)?;
    let balance = load_balance(&env)?;
    Ok(balance - amount)
}
```

**Run:** `cd examples/basics/03-custom-errors && cargo test`

---

## 5. Tokens

> **Full examples:** [`examples/tokens/01-sep41-token`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/01-sep41-token),
> [`examples/tokens/05-allowance-pattern`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/05-allowance-pattern)

```rust
// SEP-41 surface: balance, transfer, allowance, approve, transfer_from, burn.
pub fn approve(
    env: Env,
    owner: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) -> Result<(), AllowanceError> {
    owner.require_auth();
    if expiration_ledger < env.ledger().sequence() {
        return Err(AllowanceError::InvalidExpiration);
    }
    env.storage().persistent().set(
        &DataKey::Allowance(owner, spender),
        &AllowanceValue { amount, expiration_ledger },
    );
    Ok(())
}
```

Token checklist: authorize the owner, use checked arithmetic, cap the supply,
make the decimals immutable, and expire allowances.

**Run:** `cd examples/tokens/01-sep41-token && cargo test`

---

## 6. Cross-contract calls

> **Full example:** [`examples/advanced/contract-registry`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/advanced/contract-registry)

```rust
// Invoke another contract by address.
let value: u32 = env.invoke_contract(&target, &symbol_short!("get_value"), args);

// Deploy a contract from a contract.
let address = env
    .deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);
```

Cross-contract checklist: the caller must be authorized for the *nested*
call too, use `try_invoke_contract` when a failing callee should not abort
the whole transaction, and never trust a returned value without validating it.

**Run:** `cd examples/advanced/contract-registry && cargo test`

---

## 7. Access control

> **Full example:** [`examples/intermediate/02-role-based-access-control`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/intermediate/02-role-based-access-control)

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}

pub fn has_role(env: &Env, account: &Address, role: Role) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Role(account.clone(), role))
}

fn require_role(env: &Env, account: &Address, role: Role) {
    assert!(has_role(env, account, role), "unauthorized");
}
```

**Run:** `cd examples/intermediate/02-role-based-access-control && cargo test`

---

## 8. DeFi

> **Full example:** [`examples/defi/01-simple-swap`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/defi/01-simple-swap)

```rust
// Constant-product swap with a 0.3% pool fee.
let amount_in_with_fee = (sell_amount * 997) / 1_000;
let amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee);

if amount_out < min_buy_amount {
    return Err(AmmError::InsufficientOutputAmount);
}
```

**Run:** `cd examples/defi/01-simple-swap && cargo test`

---

## 9. NFTs

> **Full example:** [`examples/nfts/01-basic-nft`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/01-basic-nft)

```rust
// Minting is admin-only and each token id may be minted once.
pub fn mint(env: Env, to: Address, token_id: u64) -> Result<(), NFTError> {
    admin(&env)?.require_auth();
    if env.storage().persistent().has(&DataKey::Owner(token_id)) {
        return Err(NFTError::TokenAlreadyMinted);
    }
    env.storage().persistent().set(&DataKey::Owner(token_id), &to);
    Ok(())
}
```

**Run:** `cd examples/nfts/01-basic-nft && cargo test`

---

## 10. Governance

> **Full example:** [`examples/governance/01-simple-voting`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/governance/01-simple-voting)

```rust
// Weighted votes, one vote per address, deadline enforced by the ledger.
pub fn vote(env: Env, voter: Address, proposal_id: u32, approve: bool, weight: i128) {
    voter.require_auth();
    if env.ledger().sequence() > proposal.voting_end_ledger {
        return Err(GovernanceError::VotingEnded);
    }
    // ... record the vote, update the tally
}
```

**Run:** `cd examples/governance/01-simple-voting && cargo test`

---

## 11. Testing

> **Full guide:** [Testing Guide →](../guides/testing.md)

```rust
#[test]
fn transfer_moves_funds() {
    let env = Env::default();
    // Accept every require_auth() in the test.
    env.mock_all_auths();

    let id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&alice, &1_000);
    client.transfer(&alice, &bob, &250);

    assert_eq!(client.balance(&alice), 750);
    assert_eq!(client.balance(&bob), 250);

    // Typed errors are matched exactly.
    assert_eq!(
        client.try_transfer(&alice, &bob, &10_000),
        Err(Ok(TokenError::InsufficientBalance))
    );
}
```

The same suite runs in CI:

```bash
cargo test -p integration-tests
```

---

## Related pages

- [Testing Best Practices](./testing-best-practices.md)
- [Common Patterns](./common-patterns.md)
- [Factory, Proxy, and Registry Patterns](./cross-contract-patterns.md)
- [Token Security Checklist](./token-security-checklist.md)
- [DeFi Security Checklist](./defi-security-checklist.md)
- [Governance Security Checklist](./governance-security-checklist.md)
- [Interactive Playground — Basic Examples](../examples/playground.md)
- [Interactive Playground — Intermediate Examples](../examples/playground-intermediate.md)
