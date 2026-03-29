# Ajo Factory Pattern

This example demonstrates how to implement a **Contract Factory** in Soroban. A factory contract allows you to dynamically deploy and initialize new contract instances from within another contract.

This is the Soroban equivalent of the Ethereum **EIP-1167 Minimal Proxy** pattern (often referred to as "clones").

## 📋 Features

- **Wasm Hash Deployment**: Spawns multiple instances of the same contract code using its unique Wasm hash.
- **Initialization Guard**: Automatically initializes the new instance immediately after deployment to prevent unauthorized setup.
- **Salted Addresses**: Uses unique salts (deterministic generation) to ensure each instance has its own address.
- **Tracking**: Maintains a registry of all deployed instances.
- **Gas Efficiency**: Uploading the Wasm code once and deploying multiple instances is significantly cheaper than uploading the same code multiple times.

## 🧠 Key Concepts

### 1. `env.deployer()`

The `deployer()` host function provides the interface for creating new contracts. In this example, we use `with_current_contract(salt)` to specify the salt for address derivation.

```rust
let deployed_address = env
    .deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);
```

### 2. Wasm Hash

In Soroban, code is separated from state. You upload the contract's Wasm binary once to the network, which returns a `BytesN<32>` Wasm hash. Any contract can then use this hash to create new instances.

### 3. Initialization Pattern

Because Soroban contracts don't have a traditional constructor that runs during deployment (the WASM is immutable), we use an `initialize` method. The factory calls this method immediately after deployment:

```rust
let ajo_client = AjoClient::new(&env, &deployed_address);
ajo_client.initialize(&amount, &max_members, &creator);
```

## 🛠️ Usage

### Build the Contract

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests

```bash
cargo test
```

## 🎯 When to use this?

- **SPV (Special Purpose Vehicles)**: Like the "Ajo" savings groups where each group needs its own isolated state and logic.
- **DAO Governance**: Deploying a new governance structure for each proposal or sub-DAO.
- **Marketplaces**: Creating individual escrow or listing contracts for each transaction.
- **Multi-tenant Applications**: Isolating user data into separate contracts for maximum security and independent TTL (Time To Live) management.
