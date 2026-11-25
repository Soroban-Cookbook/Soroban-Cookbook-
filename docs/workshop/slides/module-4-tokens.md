---
marp: true
theme: default
paginate: true
---

# Module 4: Tokens & SEP-41 Standard

## Soroban Development Workshop

---

# What Is SEP-41?

SEP-41 is Stellar's fungible token standard (like ERC-20 for Ethereum).

Required interface:
- `name()`, `symbol()`, `decimals()` — metadata
- `total_supply()` — supply info
- `balance(owner)` — check balance
- `transfer(from, to, amount)` — move tokens
- `approve(owner, spender, amount)` — grant allowance
- `transfer_from(spender, owner, to, amount)` — delegated transfer
- `allowance(owner, spender)` — check allowance
- Optional: `mint()`, `burn()`

---

# SEP-41 Data Model

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,                       // Instance: who controls mint/burn
    TotalSupply,                 // Instance: total tokens in circulation
    Name,                        // Instance: token name
    Symbol,                      // Instance: ticker symbol
    Decimals,                    // Instance: decimal places
    Balance(Address),            // Persistent: user balances
    Allowance(Address, Address), // Persistent: owner → spender
}
```

Source: [`examples/tokens/01-sep41-token/`](../../examples/tokens/01-sep41-token/)

---

# Token Initialization

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: Symbol,
    decimals: u32,
    initial_supply: i128,
) -> Result<(), TokenError> {
    // Prevent re-initialization
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(TokenError::AlreadyInitialized);
    }
    if initial_supply < 0 {
        return Err(TokenError::InvalidAmount);
    }

    // Store metadata in instance storage
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Name, &name);
    env.storage().instance().set(&DataKey::Symbol, &symbol);
    env.storage().instance().set(&DataKey::Decimals, &decimals);
    env.storage().instance().set(&DataKey::TotalSupply, &initial_supply);

    // Credit admin with initial supply (persistent storage)
    env.storage()
        .persistent()
        .set(&DataKey::Balance(admin.clone()), &initial_supply);

    Ok(())
}
```

---

# Transfer Implementation

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
    from.require_auth();              // 1. Require caller authorization
    ensure_initialized(&env)?;       // 2. Check contract is initialized
    require_positive(amount)?;       // 3. Validate amount

    let from_balance = read_balance(&env, &from);
    if from_balance < amount {         // 4. Check sufficient balance
        return Err(TokenError::InsufficientBalance);
    }

    let to_balance = read_balance(&env, &to);
    let new_to_balance = to_balance
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;  // 5. Check overflow

    // 6. Update balances
    env.storage().persistent().set(&DataKey::Balance(from.clone()), &(from_balance - amount));
    env.storage().persistent().set(&DataKey::Balance(to.clone()), &new_to_balance);

    // 7. Emit event
    publish_transfer(&env, from, to, amount);
    Ok(())
}
```

---

# Approve & Transfer-From

Allow a third party (spender) to transfer on your behalf.

```rust
// Owner grants allowance
pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) -> Result<(), TokenError> {
    owner.require_auth();
    ensure_initialized(&env)?;
    if amount < 0 { return Err(TokenError::InvalidAmount); }

    env.storage().persistent()
        .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);
    publish_approval(&env, owner, spender, amount);
    Ok(())
}

// Spender uses allowance
pub fn transfer_from(env: Env, spender: Address, owner: Address, to: Address, amount: i128)
    -> Result<(), TokenError>
{
    spender.require_auth();
    // Check allowance, deduct it, move tokens...
    let allowance = read_allowance(&env, &owner, &spender);
    if allowance < amount { return Err(TokenError::AllowanceExceeded); }
    // ... deduct and transfer
}
```

---

# Mint & Burn

```rust
pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<i128, TokenError> {
    admin.require_auth();
    if admin != read_admin(&env)? { return Err(TokenError::Unauthorized); }
    require_positive(amount)?;

    let to_balance = read_balance(&env, &to);
    let new_balance = to_balance.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;
    let total_supply = read_total_supply(&env);
    let new_supply = total_supply.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;

    env.storage().persistent().set(&DataKey::Balance(to.clone()), &new_balance);
    env.storage().instance().set(&DataKey::TotalSupply, &new_supply);

    publish_transfer(&env, env.current_contract_address(), to, amount);
    Ok(new_balance)
}
```

---

# Token Metadata

```rust
pub fn name(env: Env) -> Result<String, TokenError> {
    env.storage().instance().get(&DataKey::Name).ok_or(TokenError::NotInitialized)
}

pub fn symbol(env: Env) -> Result<Symbol, TokenError> {
    env.storage().instance().get(&DataKey::Symbol).ok_or(TokenError::NotInitialized)
}

pub fn decimals(env: Env) -> Result<u32, TokenError> {
    env.storage().instance().get(&DataKey::Decimals).ok_or(TokenError::NotInitialized)
}

pub fn total_supply(env: Env) -> Result<i128, TokenError> {
    ensure_initialized(&env)?;
    Ok(read_total_supply(&env))
}

pub fn balance(env: Env, user: Address) -> i128 {
    read_balance(&env, &user)
}
```

---

# Token Events

Events are important for indexing and UI updates:

```rust
fn publish_transfer(env: &Env, from: Address, to: Address, amount: i128) {
    env.events().publish(
        (symbol_short!("events"), symbol_short!("transfer"), from, to),
        TransferEventData { amount },
    );
}

fn publish_approval(env: &Env, owner: Address, spender: Address, amount: i128) {
    env.events().publish(
        (symbol_short!("events"), symbol_short!("approve"), owner, spender),
        ApprovalEventData { amount },
    );
}
```

Topics are indexed and queryable. Use them for filtering.

---

# Token Variants in the Cookbook

| Example | Pattern | Source |
|---------|---------|--------|
| SEP-41 Token | Base standard | [`tokens/01-sep41-token/`](../../examples/tokens/01-sep41-token/) |
| SEP-41 Extensions | Batch ops, permits | [`tokens/02-sep41-extensions/`](../../examples/tokens/02-sep41-extensions/) |
| Pausable Token | Emergency stop | [`tokens/03-pausable-token/`](../../examples/tokens/03-pausable-token/) |
| Mint & Burn | Supply management | [`tokens/04-mint-burn/`](../../examples/tokens/04-mint-burn/) |
| Vesting | Time-locked release | [`tokens/05-vesting/`](../../examples/tokens/05-vesting/) |
| Token Wrapper | Wrapped asset | [`tokens/06-token-wrapper/`](../../examples/tokens/06-token-wrapper/) |
| Reward Token | Staking rewards | [`tokens/06-reward-token/`](../../examples/tokens/06-reward-token/) |

---

# Testing Token Contracts

```rust
#[test]
fn test_transfer_moves_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Sep41Token);
    let client = Sep41TokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Initialize
    env.mock_auths(&[MockAuth { address: &admin, invoke: &MockAuthInvoke {
        contract: &contract_id, fn_name: "initialize",
        args: (&admin, &String::from_str(&env, "Test"), &symbol_short!("TST"),
               &7u32, &1_000_000i128),
        sub_invokes: &[],
    }}]);
    client.initialize(&admin, &String::from_str(&env, "Test"),
                       &symbol_short!("TST"), &7u32, &1_000_000i128);

    // Transfer
    env.mock_auths(&[/* ... admin auth for transfer ... */]);
    client.transfer(&admin, &recipient, &500_000i128);

    assert_eq!(client.balance(&admin), 500_000);
    assert_eq!(client.balance(&recipient), 500_000);
}
```

---

# Module 4 Summary

- SEP-41 = Stellar's fungible token standard (transfer, approve, transfer_from)
- Metadata (name, symbol, decimals) in instance storage; balances in persistent
- `require_auth()` on every mutating function — `from` signs transfers, owner signs approvals
- `checked_add` / `checked_sub` for overflow protection on amounts
- Events for transfer and approval enable external indexing
- Cookbook has 12+ token variants to study

---

# Up Next

**Module 5: Advanced Patterns & Deployment**
- Proxy patterns, upgradeability
- Cross-contract calls
- Deployment to testnet

---

# Exercise Time!

See [Exercise 4](../exercises/module-4-exercise.md)

**Goal:** Implement a SEP-41 token with mint, burn, and pausable functionality.

⏱️ 60 minutes
