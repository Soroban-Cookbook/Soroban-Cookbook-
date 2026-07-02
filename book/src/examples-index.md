# All Examples Index

Search-optimized index of all examples with formatted code snippets and live demos.

## Basics

### 01 Hello World
[View Source](../examples/basics/01-hello-world)

[Live Demo](https://soroban.stellar.org/docs)

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;          // (1) plain unit struct — no fields needed

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {   // (2) Env is always first
        vec![&env, symbol_short!("Hello"), to]            // (3) host-allocated Vec
    }
}
```

### 02 Storage Patterns
[View Source](../examples/basics/02-storage-patterns)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Persistent: Per-key TTL
env.storage().persistent().set(&key, &value);
env.storage().persistent().extend_ttl(&key, 100, 100);

// Instance: Shared TTL for all instance data
env.storage().instance().set(&key, &value);
env.storage().instance().extend_ttl(100, 100);

// Temporary: Ephemeral, no rent
env.storage().temporary().set(&key, &value);
```

### 03 Authentication
[View Source](../examples/basics/03-authentication)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Owner grants spender the right to move up to 500 tokens.
pub fn approve(env: Env, from: Address, spender: Address, amount: i128)
    -> Result<(), AuthError>
{
    from.require_auth();                                    // owner signs
    env.storage().persistent()
        .set(&DataKey::Allowance(from, spender), &amount);
    Ok(())
}

// Spender exercises the allowance.
pub fn transfer_from(
    env: Env, spender: Address, from: Address, to: Address, amount: i128,
) -> Result<(), AuthError> {
    spender.require_auth();                                 // spender signs
    let allowance: i128 = env.storage().persistent()
        .get(&DataKey::Allowance(from.clone(), spender.clone()))
        .unwrap_or(0);
    if allowance < amount { return Err(AuthError::Unauthorized); }
    // … update balances and reduce allowance …
    Ok(())
}
```

### 03 Custom Errors
[View Source](../examples/basics/03-custom-errors)

[Live Demo](https://soroban.stellar.org/docs)

```rust
use soroban_sdk::{contracterror, contractimpl, Env};

#[contracterror]
#[repr(u32)]
pub enum ContractError {
    InvalidInput = 1,
    Unauthorized = 2,
    NotFound = 3,
}

#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env, value: u64) -> Result<(), ContractError> {
        if value == 0 {
            return Err(ContractError::InvalidInput);
        }
        // ... function logic
        Ok(())
    }
}
```

### 04 Events
[View Source](../examples/basics/04-events)

[Live Demo](https://soroban.stellar.org/docs)

```rust
env.events().publish(
    (topic_0, topic_1, topic_2, topic_3),  // up to 4 indexed topics
    data_payload,                           // arbitrary SCVal; not indexed
);
```

### 05 Auth Context
[View Source](../examples/basics/05-auth-context)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 05 Error Handling
[View Source](../examples/basics/05-error-handling)

[Live Demo](https://soroban.stellar.org/docs)

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount       = 1,
    InsufficientBalance = 2,
    Unauthorized        = 3,
}
```

### 06 Validation Patterns
[View Source](../examples/basics/06-validation-patterns)

[Live Demo](https://soroban.stellar.org/docs)

```rust
ValidationError::InvalidAmount = 100,
ValidationError::AmountTooSmall = 101,
ValidationError::AmountTooLarge = 102,
ValidationError::InvalidAddress = 103,
ValidationError::InvalidString = 104,
ValidationError::StringTooShort = 105,
ValidationError::StringTooLong = 106,
```

### 07 Type Conversions
[View Source](../examples/basics/07-type-conversions)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// native → Val
let val: Val = 42u32.into_val(&env);

// Val → native (safe, returns Result)
let n: u32 = u32::try_from_val(&env, &val).unwrap_or(0);
```

### 08 Soroban Types
[View Source](../examples/basics/08-soroban-types)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// ✅ Use Symbol for short identifiers
let token_symbol = symbol_short!("USDC");
let action = Symbol::from_str(&env, "transfer");

// ✅ Use String for longer text
let message = String::from_str(&env, "Transaction completed successfully");
let username = String::from_str(&env, "alice_blockchain_dev");

// ✅ Use Bytes for variable binary data
let signature = Bytes::from_slice(&env, &signature_data);
let serialized_object = Bytes::from_slice(&env, &encoded_data);

// ✅ Use BytesN for fixed-size data
let hash = BytesN::<32>::from_array(&env, &sha256_result);
let address_hash = BytesN::<20>::from_array(&env, &address_bytes);

// ✅ Use Address for accounts and contracts
let user = Address::generate(&env);
let contract_address = env.current_contract_address();

// ✅ Use Vec for ordered collections
let mut numbers = Vec::new(&env);
numbers.push_back(1);

// ✅ Use Map for key-value associations
let mut settings = Map::new(&env);
settings.set(symbol_short!("theme"), 1);
```

### 09 Enum Types
[View Source](../examples/basics/09-enum-types)

[Live Demo](https://soroban.stellar.org/docs)

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UserRole {
    None = 0,
    User = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractState {
    Uninitialized = 0,
    Active = 1,
    Paused = 2,
    Frozen = 3,
    Shutdown = 4,
}
```

### 10 Custom Structs
[View Source](../examples/basics/10-custom-structs)

[Live Demo](https://soroban.stellar.org/docs)

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub name: String,
    pub email: Option<String>,
    pub reputation: u32,
    pub verified: bool,
    pub created_at: u64,
}
```

### 11 Primitive Types
[View Source](../examples/basics/11-primitive-types)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Use u32 for small counters, flags, and indices
let counter: u32 = 100;
let flags: u32 = 0b1010;

// Use u64 for timestamps, large counters, and IDs
let timestamp: u64 = env.ledger().timestamp();
let user_id: u64 = 1234567890;

// Use i128 for financial calculations (balances, amounts)
let balance: i128 = 1000000; // $1M in smallest unit
let amount: i128 = -500; // Negative amount for refunds
```

### 12 Data Types
[View Source](../examples/basics/12-data-types)

[Live Demo](https://soroban.stellar.org/docs)

```rust
let count: u32  = 42;
let timestamp: u64 = 1_700_000_000;
let amount: i128 = 1_000_000_000; // 100 XLM in stroops
```

### 13 Collection Types
[View Source](../examples/basics/13-collection-types)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Create and populate a Vec
let mut items: Vec<i128> = Vec::new(&env);
items.push_back(10);
items.push_back(20);
items.push_back(30);

// Random access — O(1)
let first = items.get(0); // Some(10)
let len   = items.len();  // 3

// Remove the last element
let last = items.pop_back(); // Some(30)
```

### 13 Compressed Storage
[View Source](../examples/basics/13-compressed-storage)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 13 Queue Variants
[View Source](../examples/basics/13-queue-variants)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 14 Event Filtering
[View Source](../examples/basics/14-event-filtering)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 14 Fifo Queue
[View Source](../examples/basics/14-fifo-queue)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Lazy Cache
[View Source](../examples/basics/lazy-cache)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

## Intermediate

### 02 Role Based Access Control
[View Source](../examples/intermediate/02-role-based-access-control)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 03 Pause Unpause
[View Source](../examples/intermediate/03-pause-unpause)

[Live Demo](https://soroban.stellar.org/docs)

```rust
fn require_not_paused(env: &Env) -> Result<(), PauseError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused { return Err(PauseError::ContractPaused); }
    Ok(())
}
```

### 03 Priority Queue
[View Source](../examples/intermediate/03-priority-queue)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Ajo Factory
[View Source](../examples/intermediate/ajo-factory)

[Live Demo](https://soroban.stellar.org/docs)

```rust
let wasm_hash = env.deployer().upload_contract_wasm(template_wasm);

factory_client.register_template(
    &admin,
    &symbol_short!("savings"),
    &symbol_short!("v1"),
    &wasm_hash,
);
```

### Event History
[View Source](../examples/intermediate/event-history)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Iterable Mappings
[View Source](../examples/intermediate/iterable-mappings)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Multi Sig Patterns
[View Source](../examples/intermediate/multi-sig-patterns)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn initialize(env: Env, threshold: u32, signers: Vec<Address>) -> Result<(), AuthError>
pub fn create_proposal(env: Env, proposer: Address) -> Result<u32, AuthError>
pub fn approve(env: Env, proposal_id: u32, signer: Address) -> Result<(), AuthError>
pub fn cancel(env: Env, proposal_id: u32, signer: Address) -> Result<(), AuthError>
pub fn execute(env: Env, proposal_id: u32, executor: Address) -> Result<bool, AuthError>
pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal>
```

### Storage Migration
[View Source](../examples/intermediate/storage-migration)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Storage Pagination
[View Source](../examples/intermediate/storage-pagination)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

## Advanced

### 01 Multi Party Auth
[View Source](../examples/advanced/01-multi-party-auth)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn multi_sig_transfer(env: Env, signers: Vec<Address>, to: Address, amount: i128)
```

### 02 Timelock
[View Source](../examples/advanced/02-timelock)

[Live Demo](https://soroban.stellar.org/docs)

```rust
client.initialize(&admin_address);
```

### 03 Cross Contract Optimization
[View Source](../examples/advanced/03-cross-contract-optimization)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 03 Oracle Pattern
[View Source](../examples/advanced/03-oracle-pattern)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn submit(env: Env, updater: Address, value: i128) -> Result<(), OracleError> {
    updater.require_auth();
    env.storage().instance().set(&DataKey::Value, &value);
    env.storage().instance().set(&DataKey::Timestamp, &env.ledger().timestamp());
    Ok(())
}
```

### 03 Proxy Admin
[View Source](../examples/advanced/03-proxy-admin)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    require_unpaused(&env)?;   // blocks when paused
    user.require_auth();
    // ... rest of logic
}
```

### 03 Rbac Modifiers
[View Source](../examples/advanced/03-rbac-modifiers)

[Live Demo](https://soroban.stellar.org/docs)

```rust
client.grant_role(&admin, &ROLE_MINTER, &alice);
```

### 03 Registry Access Controls
[View Source](../examples/advanced/03-registry-access-controls)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 04 Upgradeable Proxy
[View Source](../examples/advanced/04-upgradeable-proxy)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 05 Batch Transfer
[View Source](../examples/advanced/05-batch-transfer)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub struct Transfer {
    pub to: Address,   // recipient
    pub amount: i128,  // must be > 0
}
```

### 05 Bridge Security
[View Source](../examples/advanced/05-bridge-security)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 05 Diamond Facets
[View Source](../examples/advanced/05-diamond-facets)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Router atomically calls TokenFacet then RegistryFacet.
// If either fails, the whole transaction reverts.
router.mint_and_register(&admin, &recipient, &750, &key, &"metadata");
```

### Contract Registry
[View Source](../examples/advanced/contract-registry)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

## Defi

### 01 Simple Swap
[View Source](../examples/defi/01-simple-swap)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 01 Vault Strategies
[View Source](../examples/defi/01-vault-strategies)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub struct StrategyParams {
    pub name: Symbol,
    pub max_allocation_bps: i128,
    pub expected_apy_bps: i128,
    pub risk_level: RiskLevel,
}
```

### 02 Constant Product Amm
[View Source](../examples/defi/02-constant-product-amm)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 02 Swap Liquidity
[View Source](../examples/defi/02-swap-liquidity)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 03 Amm Price Oracle
[View Source](../examples/defi/03-amm-price-oracle)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 03 Farming Pool
[View Source](../examples/defi/03-farming-pool)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn add_pool(env: Env, admin: Address, ...) {
    admin.require_auth();
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    if admin != stored_admin { panic!("Unauthorized"); }
    // ...
}
```

### 03 Lending Pool
[View Source](../examples/defi/03-lending-pool)

[Live Demo](https://soroban.stellar.org/docs)

```rust
client.initialize(&base_rate, &kink_rate, &kink_utilization);
```

### 04 Collateralized Lending
[View Source](../examples/defi/04-collateralized-lending)

[Live Demo](https://soroban.stellar.org/docs)

```rust
client.initialize(&ltv_ratio, &liquidation_threshold, &liquidation_incentive, &partial_liquidation_ratio);
```

### 05 Amm Router
[View Source](../examples/defi/05-amm-router)

[Live Demo](https://soroban.stellar.org/docs)

```rust
client.initialize();
```

### 05 Flash Loans
[View Source](../examples/defi/05-flash-loans)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub trait FlashLoanReceiver {
    fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128);
}
```

### 06 Flash Loan Use Cases
[View Source](../examples/defi/06-flash-loan-use-cases)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 07 Staking Pool
[View Source](../examples/defi/07-staking-pool)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 08 Liquidity Mining
[View Source](../examples/defi/08-liquidity-mining)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// One-time setup
fn initialize(env: Env, admin: Address);

// Create a new pool
fn add_pool(env: Env, pool_id: u32, lp_token: Address, reward_token: Address, reward_rate: i128);

// Change the reward rate (settles existing rewards first)
fn set_reward_rate(env: Env, pool_id: u32, new_rate: i128);

// Pause or resume a pool
fn set_pool_active(env: Env, pool_id: u32, active: bool);
```

## Nfts

### 01 Basic Nft
[View Source](../examples/nfts/01-basic-nft)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 01 Nft Metadata
[View Source](../examples/nfts/01-nft-metadata)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    base_uri: String,   // pass "" to use per-token image URIs
) -> Result<(), NftError>
```

### 02 Nft Marketplace
[View Source](../examples/nfts/02-nft-marketplace)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 02 Nft Metadata
[View Source](../examples/nfts/02-nft-metadata)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

## Governance

### 01 Voting Time Constraints
[View Source](../examples/governance/01-voting-time-constraints)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 04 Proposal Lifecycle
[View Source](../examples/governance/04-proposal-lifecycle)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

## Tokens

### 01 Sep41 Token
[View Source](../examples/tokens/01-sep41-token)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### 02 Sep41 Extensions
[View Source](../examples/tokens/02-sep41-extensions)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub struct Transfer {
    pub to: Address,
    pub amount: i128,  // must be > 0
}
```

### 03 Optimized Operations
[View Source](../examples/tokens/03-optimized-operations)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn batch_transfer(
    env: Env,
    user: Address,
    recipients: Vec<BatchTransfer>,
) -> Result<(), OptimizedError> {
    // Process all transfers in ONE contract call
    for recipient_data in recipients.iter() {
        // Update balances
    }
    Ok(())
}
```

### Allowance Pattern
[View Source](../examples/tokens/allowance-pattern)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Mint Burn
[View Source](../examples/tokens/mint-burn)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Multi Token Balance Manager
[View Source](../examples/tokens/multi-token-balance-manager)

[Live Demo](https://soroban.stellar.org/docs)

```rust
manager.register_token(
    &admin,
    &token_address,
    &TokenMetadata {
        name,
        symbol,
        decimals: 7,
        standard: MetadataStandard::Sep41,
    },
);
```

### Optimized Token Ops
[View Source](../examples/tokens/optimized-token-ops)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Code snippet coming soon
```

### Token Metadata
[View Source](../examples/tokens/token-metadata)

[Live Demo](https://soroban.stellar.org/docs)

```rust
// Decimals are set once and never exposed through an update path.
// Changing decimals after tokens are in circulation would mean that
// a balance of 1_000_0000000 (7 decimals = 1000.0) silently becomes
// 1_000_0000000 (6 decimals = 10000.0) — a 10× reinterpretation with
// no on-chain record of the change.
pub fn decimals(env: Env) -> Result<u32, MetadataError> {
    read_decimals(&env)
}
```

### Token Wrapper
[View Source](../examples/tokens/token-wrapper)

[Live Demo](https://soroban.stellar.org/docs)

```rust
pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, WrapperError> {
    user.require_auth();

    let wrapper = env.current_contract_address();
    TokenClient::new(&env, &underlying).transfer(&user, &wrapper, &amount);

    // Mint exactly `amount` wrapped shares after validating arithmetic.
    // The full contract stores per-user balances and total supply.
    Ok(new_balance)
}
```

## Storage

## Hello-world

