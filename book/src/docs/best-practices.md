# Soroban Smart Contract Best Practices

A comprehensive guide to writing secure, efficient, and maintainable Soroban smart contracts.

## 🔒 Security Best Practices

### 1. Always Use Authorization Checks

✅ **DO:** Always verify authorization before state changes

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: u64) {
    // Verify the sender has authorized this action
    from.require_auth();

    // Proceed with transfer logic
    let balance = read_balance(&env, &from);
    write_balance(&env, &from, balance - amount);
    write_balance(&env, &to, read_balance(&env, &to) + amount);
}
```

❌ **DON'T:** Skip authorization checks

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: u64) {
    // DANGER: No authorization check!
    let balance = read_balance(&env, &from);
    write_balance(&env, &from, balance - amount);
    write_balance(&env, &to, read_balance(&env, &to) + amount);
}
```

### 2. Validate All Inputs

✅ **DO:** Validate inputs before processing

```rust
pub fn set_limit(env: Env, admin: Address, limit: u64) {
    admin.require_auth();

    // Validate input
    if limit == 0 {
        panic!("Limit must be greater than zero");
    }

    if limit > MAX_LIMIT {
        panic!("Limit exceeds maximum allowed");
    }

    env.storage().persistent().set(&symbol_short!("limit"), &limit);
}
```

❌ **DON'T:** Trust inputs blindly

```rust
pub fn set_limit(env: Env, admin: Address, limit: u64) {
    admin.require_auth();
    // DANGER: No validation, could set to 0 or excessive value
    env.storage().persistent().set(&symbol_short!("limit"), &limit);
}
```

### 3. Use Safe Arithmetic

✅ **DO:** Use checked arithmetic to prevent overflows

```rust
pub fn add_balance(env: Env, user: Address, amount: u64) {
    let current = read_balance(&env, &user);

    // Safe addition with overflow check
    let new_balance = current.checked_add(amount)
        .expect("Balance overflow");

    write_balance(&env, &user, new_balance);
}
```

❌ **DON'T:** Use unchecked arithmetic

```rust
pub fn add_balance(env: Env, user: Address, amount: u64) {
    let current = read_balance(&env, &user);
    // DANGER: Could overflow
    write_balance(&env, &user, current + amount);
}
```

### 4. Protect Admin Functions

✅ **DO:** Restrict sensitive functions to authorized admins

```rust
const ADMIN_KEY: Symbol = symbol_short!("admin");

pub fn initialize(env: Env, admin: Address) {
    if env.storage().persistent().has(&ADMIN_KEY) {
        panic!("Already initialized");
    }
    env.storage().persistent().set(&ADMIN_KEY, &admin);
}

pub fn set_config(env: Env, config: Config) {
    // Only admin can change config
    let admin: Address = env.storage().persistent()
        .get(&ADMIN_KEY)
        .expect("Not initialized");
    admin.require_auth();

    env.storage().persistent().set(&symbol_short!("config"), &config);
}
```

❌ **DON'T:** Leave admin functions unprotected

```rust
pub fn set_config(env: Env, config: Config) {
    // DANGER: Anyone can change config!
    env.storage().persistent().set(&symbol_short!("config"), &config);
}
```

### 5. Handle Errors Properly

✅ **DO:** Use custom error types for clarity

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InsufficientBalance = 1,
    Unauthorized = 2,
    InvalidAmount = 3,
}

pub fn withdraw(env: Env, user: Address, amount: u64) -> Result<(), Error> {
    user.require_auth();

    let balance = read_balance(&env, &user);

    if amount == 0 {
        return Err(Error::InvalidAmount);
    }

    if balance < amount {
        return Err(Error::InsufficientBalance);
    }

    write_balance(&env, &user, balance - amount);
    Ok(())
}
```

❌ **DON'T:** Use generic panics without context

```rust
pub fn withdraw(env: Env, user: Address, amount: u64) {
    user.require_auth();
    let balance = read_balance(&env, &user);
    // POOR: No clear error message
    assert!(balance >= amount);
    write_balance(&env, &user, balance - amount);
}
```

## ⚡ Resource Optimization Best Practices

### 1. Choose the Right Storage Type

✅ **DO:** Use appropriate storage for data lifetime

```rust
// Persistent: Critical data that must survive
pub fn set_admin(env: Env, admin: Address) {
    env.storage().persistent().set(&symbol_short!("admin"), &admin);
}

// Instance: Contract configuration
pub fn initialize(env: Env, config: Config) {
    env.storage().instance().set(&symbol_short!("config"), &config);
}

// Temporary: Short-lived data (e.g., nonces, locks)
pub fn create_lock(env: Env, user: Address) {
    let ledger = env.ledger().sequence();
    env.storage().temporary().set(&user, &ledger);
}
```

❌ **DON'T:** Use persistent storage for everything

```rust
// INEFFICIENT: Using expensive persistent storage for temporary data
pub fn create_lock(env: Env, user: Address) {
    let ledger = env.ledger().sequence();
    env.storage().persistent().set(&user, &ledger); // Too expensive!
}
```

### 2. Minimize Storage Operations

✅ **DO:** Batch reads and writes

```rust
pub fn batch_transfer(env: Env, from: Address, transfers: Vec<(Address, u64)>) {
    from.require_auth();

    // Read once
    let mut from_balance = read_balance(&env, &from);

    // Calculate total
    let total: u64 = transfers.iter().map(|(_, amt)| amt).sum();
    from_balance = from_balance.checked_sub(total).expect("Insufficient balance");

    // Write once
    write_balance(&env, &from, from_balance);

    // Update recipients
    for (to, amount) in transfers.iter() {
        let balance = read_balance(&env, &to);
        write_balance(&env, &to, balance + amount);
    }
}
```

❌ **DON'T:** Make redundant storage calls

```rust
pub fn batch_transfer(env: Env, from: Address, transfers: Vec<(Address, u64)>) {
    from.require_auth();

    for (to, amount) in transfers.iter() {
        // INEFFICIENT: Reading/writing from_balance repeatedly
        let from_balance = read_balance(&env, &from);
        write_balance(&env, &from, from_balance - amount);

        let to_balance = read_balance(&env, &to);
        write_balance(&env, &to, to_balance + amount);
    }
}
```

### 3. Use Efficient Data Structures

✅ **DO:** Use appropriate collection types

```rust
// Use Map for key-value lookups
let mut balances: Map<Address, u64> = Map::new(&env);
balances.set(user, amount);

// Use Vec for ordered lists
let mut users: Vec<Address> = Vec::new(&env);
users.push_back(user);
```

❌ **DON'T:** Use inefficient structures

```rust
// INEFFICIENT: Storing list as separate keys
for i in 0..count {
    let key = (symbol_short!("user"), i);
    env.storage().persistent().set(&key, &users[i]);
}
```

### 4. Optimize WASM Size

✅ **DO:** Configure release profile for size

```toml
[profile.release]
opt-level = "z"          # Optimize for size
overflow-checks = true
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true
```

✅ **DO:** Avoid unnecessary dependencies

```toml
[dependencies]
soroban-sdk = "21.7.0"  # Only what you need

# Avoid heavy crates if possible
```

## 🧪 Testing Best Practices

### 1. Comprehensive Test Coverage

✅ **DO:** Test happy paths and edge cases

```rust
#[test]
fn test_transfer_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.mint(&user1, &1000);
    client.transfer(&user1, &user2, &500);

    assert_eq!(client.balance(&user1), 500);
    assert_eq!(client.balance(&user2), 500);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_transfer_insufficient_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &500); // Should panic
}

#[test]
fn test_transfer_zero_amount() {
    // Test edge case: zero amount
}

#[test]
fn test_transfer_to_self() {
    // Test edge case: transfer to same address
}
```

### 2. Use Test Utilities

✅ **DO:** Leverage Soroban test utilities

```rust
#[test]
fn test_with_auth() {
    let env = Env::default();
    env.mock_all_auths(); // Mock authorizations for testing

    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    client.protected_function(&user);

    // Verify authorization was required
    assert_eq!(
        env.auths(),
        std::vec![(
            user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    symbol_short!("protected"),
                    vec![&env, user.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}
```

### 3. Test Integration Scenarios

✅ **DO:** Test cross-contract interactions

```rust
#[test]
fn test_token_with_vault() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy token
    let token_id = env.register_contract(None, TokenContract);
    let token = TokenContractClient::new(&env, &token_id);

    // Deploy vault
    let vault_id = env.register_contract(None, VaultContract);
    let vault = VaultContractClient::new(&env, &vault_id);

    // Initialize vault with token
    vault.initialize(&token_id);

    // Test deposit flow
    let user = Address::generate(&env);
    token.mint(&user, &1000);
    vault.deposit(&user, &500);

    assert_eq!(token.balance(&user), 500);
    assert_eq!(vault.balance(&user), 500);
}
```

## 📝 Code Organization Best Practices

### 1. Structure Your Code

✅ **DO:** Organize code into logical modules

```rust
// lib.rs
mod storage;
mod validation;
mod events;

use storage::{read_balance, write_balance};
use validation::validate_amount;
use events::emit_transfer;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        validate_amount(amount);

        // Implementation
        emit_transfer(&env, from, to, amount);
    }
}
```

### 2. Use Meaningful Names

✅ **DO:** Use clear, descriptive names

```rust
pub fn calculate_interest_rate(
    total_borrowed: u64,
    total_supplied: u64,
    base_rate: u64,
) -> u64 {
    // Implementation
}
```

❌ **DON'T:** Use cryptic abbreviations

```rust
pub fn calc_ir(tb: u64, ts: u64, br: u64) -> u64 {
    // Hard to understand
}
```

### 3. Document Public Functions

✅ **DO:** Add doc comments to public functions

````rust
/// Transfers tokens from one account to another.
///
/// # Arguments
///
/// * `from` - The address to transfer from (must authorize)
/// * `to` - The address to transfer to
/// * `amount` - The amount of tokens to transfer
///
/// # Panics
///
/// Panics if:
/// - `from` hasn't authorized the transfer
/// - `from` has insufficient balance
/// - `amount` is negative
///
/// # Examples
///
/// ```
/// client.transfer(&from, &to, &1000);
/// ```
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    // Implementation
}
````

### 4. Keep Functions Focused

✅ **DO:** Write small, single-purpose functions

```rust
fn validate_transfer(from: &Address, amount: i128) {
    from.require_auth();
    validate_amount(amount);
}

fn execute_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    let from_balance = read_balance(env, from);
    let to_balance = read_balance(env, to);

    write_balance(env, from, from_balance - amount);
    write_balance(env, to, to_balance + amount);
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    validate_transfer(&from, amount);
    execute_transfer(&env, &from, &to, amount);
    emit_transfer(&env, from, to, amount);
}
```

## 📚 Documentation Best Practices

### 1. Write Clear READMEs

Each example should have a README with:

- Purpose and use case
- Key concepts demonstrated
- How to build and test
- How to deploy
- Example invocations
- Security considerations

### 2. Comment Complex Logic

✅ **DO:** Explain non-obvious code

```rust
pub fn calculate_reward(env: &Env, user: &Address) -> u64 {
    let stake_amount = read_stake(env, user);
    let stake_time = read_stake_time(env, user);
    let current_time = env.ledger().timestamp();

    // Calculate time-weighted reward
    // Formula: stake_amount * time_multiplier * base_rate
    // Where time_multiplier = sqrt(days_staked) / 10
    let days_staked = (current_time - stake_time) / 86400;
    let time_multiplier = (days_staked as f64).sqrt() / 10.0;
    let base_rate = 100; // 1% represented as basis points

    (stake_amount as f64 * time_multiplier * base_rate as f64) as u64
}
```

### 3. Include Usage Examples

Add examples in tests or documentation:

````rust
/// # Example
///
/// ```rust
/// let env = Env::default();
/// let contract_id = env.register_contract(None, MyContract);
/// let client = MyContractClient::new(&env, &contract_id);
///
/// // Initialize the contract
/// client.initialize(&admin);
///
/// // Make a deposit
/// client.deposit(&user, &1000);
/// ```
````

## 🔄 Upgrade & Migration Best Practices

### 1. Version Your Storage Keys

✅ **DO:** Include version in storage keys for upgradability

```rust
const VERSION: u32 = 1;

fn get_balance_key(user: &Address) -> (Symbol, u32, Address) {
    (symbol_short!("balance"), VERSION, user.clone())
}
```

### 2. Plan for Initialization

✅ **DO:** Use initialization pattern

```rust
pub fn initialize(env: Env, admin: Address, config: Config) {
    if has_admin(&env) {
        panic!("Already initialized");
    }

    set_admin(&env, &admin);
    set_config(&env, &config);
}

fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&symbol_short!("admin"))
}
```

## Summary Checklist

Before deploying a contract, verify:

- [ ] All state-changing functions require authorization
- [ ] All inputs are validated
- [ ] Arithmetic operations use checked math
- [ ] Admin functions are protected
- [ ] Errors are handled with custom types
- [ ] Storage types are appropriate for data lifetime
- [ ] Storage operations are minimized
- [ ] Test coverage is >80%
- [ ] Edge cases are tested
- [ ] Code is well-documented
- [ ] WASM size is optimized
- [ ] Security considerations are documented

---

For more detailed examples, see our [examples directory](../examples/) and specific guides.
