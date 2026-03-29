# Ajo Factory Pattern

The **Contract Factory** pattern is a powerful architectural design for Soroban smart contracts. It enables the creation of multiple isolated contract instances with identical logic but unique state.

## 📋 Overview

In traditional blockchain platforms like Ethereum, the "minimal proxy" (EIP-1167) pattern is used to save deployment costs by pointing to a single implementation. In Soroban, this is natively supported by separate storage and logic:

1. **Wasm Hash**: You upload the contract's binary once to the network.
2. **Deployer**: The factory contract uses the `env.deployer()` interface to instantiate new contracts from that hash.
3. **Isolation**: Each deployed instance has its own unique address and isolated storage.

## 🧠 Key Patterns

### 1. The Deployment Salt

Address derivation in Soroban is deterministic. To ensure each `Ajo` has its own address, the factory must provide a unique salt.

```rust
// Combine creator address and counter to generate a unique salt
let salt = env.crypto().sha256(&(&creator, ajos.len()).into_val(&env));

let deployed_address = env
    .deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);
```

### 2. Initialization & Re-entrancy Protection

Since Soroban contracts are immutable once deployed, they use an `initialize` function. The factory contract **must** call this immediately to prevent anyone else from taking control of the instance.

```rust
// Call initialize immediately after deployment
let ajo_client = AjoClient::new(&env, &deployed_address);
ajo_client.initialize(&amount, &max_members, &creator);
```

The `Ajo` contract itself should guard against multiple initializations:

```rust
if env.storage().instance().has(&AjoDataKey::Creator) {
    panic!("Already initialized");
}
```

## 🛠️ Usage Example

### Ajo Factory Logic

```rust
pub fn create_ajo(env: Env, amount: i128, max_members: u32, creator: Address) -> Address {
    creator.require_auth();

    let wasm_hash: BytesN<32> = env.storage().instance().get(&WasmHash).unwrap();

    // Deploy
    let deployed_address = env
        .deployer()
        .with_current_contract(salt)
        .deploy(wasm_hash);

    // Initialize
    let ajo_client = AjoClient::new(&env, &deployed_address);
    ajo_client.initialize(&amount, &max_members, &creator);

    deployed_address
}
```

## 🚀 Use Cases

- **Savings Groups (Ajos)**: Each group gets its own isolated ledger.
- **DAO Governance**: Spawning individual proposal contracts.
- **Marketplace Escrows**: Creating a fresh escrow contract for every trade.
- **Multi-tenant dApps**: Isolating user state for security and independent TTL management.

## 🧪 Testing

The factory pattern is best tested by simulating the Wasm upload in your unit tests:

```rust
#[test]
fn test_factory() {
    let env = Env::default();
    let wasm_hash = env.deployer().upload_contract_wasm(AjoWasm);
    // ... initialize factory and call create_ajo
}
```
