# Soroban & Stellar Glossary

A comprehensive glossary of terms used in Soroban smart contract development and the Stellar ecosystem.

## A

### Account

A Stellar account identified by a public key (G address). Accounts hold balances, can invoke contracts, and authorize operations in Soroban.

**See also:** [Address](#address), [Identity](#identity)

### Address

In Soroban, an Address is a generic identifier that can represent an account, a contract, or other addressable entities. The `Address` type is used for authorization and identification.

**Official docs:** [Soroban Address](https://developers.stellar.org/docs/smart-contracts/guides/conventions/address)

### Asset

A representation of value on the Stellar network. Assets can be native (XLM) or issued by accounts (custom tokens).

**See also:** [Lumens](#lumens-xlm), [Token](#token)

### Authorization

The process of verifying that an entity has permission to perform an action. In Soroban, this is typically done using `require_auth()`.

**See also:** [require_auth](#require_auth)

## B

### Bytes / BytesN

Soroban SDK types for handling byte arrays. `Bytes` is dynamic-length, while `BytesN<N>` is fixed-length.

```rust
let data: Bytes = Bytes::from_slice(&env, &[1, 2, 3]);
let hash: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);
```

## C

### Contract

A smart contract deployed on the Stellar network using Soroban. Contracts are written in Rust and compiled to WebAssembly.

**See also:** [WASM](#wasm-webassembly), [Invocation](#invocation)

### Contract ID

A unique identifier for a deployed contract. Used to invoke contract functions and interact with the contract.

### contractimpl

Rust macro that marks an implementation block as containing contract functions that can be invoked externally.

```rust
#[contractimpl]
impl MyContract {
    pub fn my_function(env: Env) { }
}
```

### contracttype

Rust macro that marks a struct or enum as a contract type that can be stored or passed between contracts.

```rust
#[contracttype]
pub struct MyData {
    value: u64,
}
```

## D

### Deploy

The process of uploading and installing a contract on the Stellar network, making it available for invocation.

**See also:** [Soroban CLI](#soroban-cli)

## E

### Env

The Soroban environment object that provides access to host functions, storage, events, and other contract capabilities.

```rust
pub fn my_function(env: Env) {
    // Access environment features
}
```

### Events

Notifications emitted by contracts to log important state changes or actions. Events can be queried off-chain.

```rust
env.events().publish((symbol_short!("transfer"),), (from, to, amount));
```

**See also:** [Publish](#publish)

## F

### Footprint

The set of ledger entries (storage, contracts, etc.) that a transaction will read or write. Required for resource planning.

### Friendbot

A service that funds testnet accounts with test XLM for development purposes.

**Testnet Friendbot:** https://friendbot.stellar.org

## H

### Horizon

The REST API for the Stellar network. Provides access to accounts, transactions, and ledger data.

**Official docs:** [Horizon API](https://developers.stellar.org/api/horizon)

### Host Functions

Functions provided by the Soroban host environment that contracts can call to interact with the ledger, perform cryptographic operations, etc.

## I

### Identity

A key pair (public and private key) used to sign transactions and authorize operations on Stellar.

**See also:** [Account](#account), [Soroban CLI](#soroban-cli)

### Instance Storage

Contract storage that persists across invocations but is specific to a contract instance. More expensive than temporary but cheaper than persistent.

**See also:** [Persistent Storage](#persistent-storage), [Temporary Storage](#temporary-storage)

### Invocation

The act of calling a contract function. Can be done via transactions or from other contracts.

**See also:** [Contract](#contract)

## L

### Ledger

The Stellar ledger is the record of all accounts, balances, and contracts on the network. Each ledger close represents a new block.

### Ledger Entry

A piece of data stored on the Stellar ledger, such as an account, contract code, or contract data.

### Lumens (XLM)

The native cryptocurrency of the Stellar network. Used for paying transaction fees and maintaining minimum account balances.

**See also:** [Stroop](#stroop)

## M

### Mainnet

The production Stellar network where real-value transactions occur.

**Network Passphrase:** "Public Global Stellar Network ; September 2015"

**See also:** [Testnet](#testnet)

### Map

A Soroban SDK type for key-value storage. Similar to HashMap but optimized for the Soroban environment.

```rust
let mut map: Map<Address, u64> = Map::new(&env);
map.set(address, 100);
```

## N

### Network Passphrase

A string that uniquely identifies a Stellar network (mainnet, testnet, or custom). Required when signing transactions.

## P

### Panic

An unrecoverable error in Rust. In Soroban, panics cause the entire transaction to fail and revert.

**See also:** [Error Handling](#error-handling)

### Persistent Storage

Contract storage that persists indefinitely (with proper TTL extension). Most expensive storage type but guarantees data persistence.

**See also:** [Instance Storage](#instance-storage), [Temporary Storage](#temporary-storage), [TTL](#ttl-time-to-live)

### Publish

Function to emit events from a contract.

```rust
env.events().publish((symbol_short!("event"),), (data,));
```

**See also:** [Events](#events)

## R

### require_auth

A Soroban function that verifies an address has authorized the current operation. Causes transaction to fail if authorization is missing.

```rust
user.require_auth();
```

**See also:** [Authorization](#authorization)

### RPC (Remote Procedure Call)

The protocol used to communicate with Soroban nodes. Required for deploying contracts and submitting transactions.

**Testnet RPC:** https://soroban-testnet.stellar.org

## S

### Soroban

The smart contract platform built on Stellar, supporting contracts written in Rust and compiled to WebAssembly.

### Soroban CLI

Command-line tool for building, deploying, and interacting with Soroban contracts.

```bash
soroban contract build
soroban contract deploy
soroban contract invoke
```

### Soroban SDK

The Rust library (crate) that provides types, macros, and utilities for writing Soroban smart contracts.

**Crate:** `soroban-sdk`

### Stellar

The blockchain network that Soroban contracts run on. Known for fast, low-cost transactions and built-in asset support.

### Storage

Persistent data storage for contracts. Soroban provides three storage types: Persistent, Temporary, and Instance.

**See also:** [Persistent Storage](#persistent-storage), [Instance Storage](#instance-storage), [Temporary Storage](#temporary-storage)

### Stroop

The smallest unit of XLM. 1 XLM = 10,000,000 stroops.

**See also:** [Lumens](#lumens-xlm)

### Symbol

A Soroban type for short string identifiers, optimized for use as storage keys or event topics.

```rust
let key = symbol_short!("balance");
let long_key = Symbol::new(&env, "longer_identifier");
```

## T

### Temporary Storage

Contract storage that only persists for a short period (minimum 16 ledgers). Cheapest storage option, ideal for ephemeral data.

**See also:** [Persistent Storage](#persistent-storage), [Instance Storage](#instance-storage)

### Testnet

A test version of the Stellar network for development. Uses test XLM with no real-world value.

**Network Passphrase:** "Test SDF Network ; September 2015"

**See also:** [Mainnet](#mainnet), [Friendbot](#friendbot)

### Token

A fungible asset on Stellar or Soroban. Can be native (XLM) or custom tokens following standards like SEP-41.

**See also:** [Asset](#asset), [SEP-41](#sep-41)

### Transaction

A collection of operations signed by one or more accounts. In Soroban, transactions include contract invocations.

### Trustline

A Stellar concept where an account explicitly trusts and can hold a specific asset issued by another account.

### TTL (Time To Live)

The number of ledgers that contract data will persist before expiring. Must be extended to keep data alive.

**See also:** [Storage](#storage)

## U

### Unit Test

Tests that verify individual contract functions in isolation. Soroban provides test utilities for writing comprehensive tests.

```rust
#[test]
fn test_my_function() {
    let env = Env::default();
    // Test code
}
```

## V

### Val

The base value type in Soroban. Most Soroban types can be converted to/from `Val`.

### Vec

A Soroban SDK type for dynamic arrays. Similar to Rust's `Vec` but optimized for Soroban.

```rust
let mut vec: Vec<u64> = Vec::new(&env);
vec.push_back(42);
```

## W

### WASM (WebAssembly)

The binary format that Soroban contracts are compiled to. Provides portability and security.

**Target:** `wasm32-unknown-unknown`

**See also:** [Contract](#contract)

### wasm-opt

A tool for optimizing WASM binaries to reduce size and improve performance.

```bash
wasm-opt -Oz input.wasm -o output.wasm
```

## X

### XLM

Ticker symbol for Lumens, Stellar's native cryptocurrency.

**See also:** [Lumens](#lumens-xlm)

## Additional Standards

### SEP-41

Stellar Ecosystem Proposal 41 - The token interface standard for Soroban, similar to ERC-20 on Ethereum.

**Official spec:** [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)

---

## Need More Help?

- **Official Soroban Docs:** https://developers.stellar.org/docs/smart-contracts
- **Soroban Examples:** https://github.com/stellar/soroban-examples
- **Stellar Discord:** https://discord.gg/stellardev
- **This Cookbook:** Check our comprehensive guides and examples

## Contributing

Found a term that's missing or needs clarification? Please open an issue or submit a PR to help improve this glossary!
