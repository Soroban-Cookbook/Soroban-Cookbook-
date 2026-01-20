# Hello World Contract

Your first Soroban smart contract! This example demonstrates the basic structure of a Soroban contract and how to interact with it.

## 📖 What You'll Learn

- Basic contract structure using `#[contract]` and `#[contractimpl]`
- How to define public contract functions
- Working with Soroban's type system (Symbol, Vec)
- Writing and running tests
- Building and deploying contracts

## 🔍 Contract Overview

This contract has a single function `hello()` that:

1. Takes a `Symbol` as input (a name to greet)
2. Returns a `Vec<Symbol>` containing ["Hello", name]

```rust
pub fn hello(env: Env, to: Symbol) -> Vec<Symbol>
```

## 🏗️ Key Concepts

### Contract Macro

```rust
#[contract]
pub struct HelloContract;
```

Defines your contract struct. Can be empty or contain state.

### Contract Implementation

```rust
#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        // Function implementation
    }
}
```

All public functions become callable contract methods.

### Environment

```rust
env: Env
```

The `Env` parameter provides access to blockchain context, storage, events, and more.

## 🧪 Testing

Run the tests:

```bash
cargo test
```

The test demonstrates:

- Creating a test environment
- Registering the contract
- Calling contract functions
- Asserting results

## 🚀 Building

Build the WASM binary:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The output will be in:

```
target/wasm32-unknown-unknown/release/hello_world.wasm
```

## 📦 Deployment

### Deploy to Testnet

1. Configure Soroban CLI for testnet:

```bash
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

2. Create an identity:

```bash
soroban keys generate alice --network testnet
```

3. Fund your account:

```bash
soroban keys fund alice --network testnet
```

4. Deploy the contract:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source alice \
  --network testnet
```

5. Invoke the contract:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- hello \
  --to World
```

## 🎓 Next Steps

Once you understand this example, explore:

- [Storage Patterns](../02-storage-patterns/) - Learn to persist data
- [Authentication](../03-authentication/) - Add security to your contracts
- [Events](../04-events/) - Emit events for off-chain tracking

## 📚 References

- [Soroban SDK Documentation](https://docs.rs/soroban-sdk)
- [Getting Started Guide](https://developers.stellar.org/docs/smart-contracts/getting-started)
- [Soroban CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
