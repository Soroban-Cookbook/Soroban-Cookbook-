# Data Types

This example demonstrates Soroban's comprehensive type system and how to work effectively with each data type. Understanding when and how to use each type is essential for writing efficient, gas-optimized smart contracts.

## Project Structure

```text
examples/basics/10-data-types/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## Type System Overview

Soroban's type system is designed for the constraints of a blockchain environment: deterministic execution, gas metering, and cross-language ABI compatibility. All types must implement `soroban_sdk::Val` to be passed across the host/contract boundary.

| Category | Types | Notes |
|----------|-------|-------|
| Integers | `u32`, `u64`, `i128` | Native Rust integer types |
| Text | `Symbol`, `String` | `Symbol` is gas-optimized for short identifiers |
| Binary | `Bytes`, `BytesN<N>` | `BytesN` is fixed-size and more efficient |
| Identity | `Address` | Accounts and contracts share one type |
| Collections | `Vec<T>`, `Map<K, V>` | Host-managed, not Rust `std` collections |

All Soroban types live in the host environment, not in contract WASM memory. Operations on them cross the host/guest boundary and have associated gas costs.

## Usage Guide

### Primitive Types (Integers)

```rust
let count: u32  = 42;
let timestamp: u64 = 1_700_000_000;
let amount: i128 = 1_000_000_000; // 100 XLM in stroops
```

- `u32` — counters, IDs, small values
- `u64` — timestamps, large counters
- `i128` — financial amounts (standard for token balances and transfers)

Always use checked arithmetic to prevent panics on overflow:

```rust
let sum = a.checked_add(b).unwrap_or_else(|| panic!("overflow"));
```

### Symbol

Short, gas-efficient identifiers. Use `symbol_short!` for literals up to 9 characters; use `Symbol::new` for longer names.

```rust
let token  = symbol_short!("USDC");          // compile-time constant
let status = Symbol::new(&env, "active");    // runtime, any length
```

Use Symbol for: token symbols, enum-like values, storage keys, event topics.

### String

Variable-length Unicode text. More expensive than Symbol for short content.

```rust
let msg = String::from_str(&env, "Transaction completed successfully");
```

Use String for: user-provided descriptions, error messages, content longer than 9 characters.

### Bytes

Variable-length binary data.

```rust
let sig = Bytes::from_slice(&env, &signature_bytes);
```

Use Bytes for: signatures, serialized objects, data of unknown length.

### BytesN\<N\>

Fixed-length binary data. The size is a compile-time constant, making it more gas-efficient than `Bytes` for fixed-size data.

```rust
let hash: BytesN<32> = BytesN::from_array(&env, &sha256_result);
```

Common sizes: 32 bytes (SHA-256), 20 bytes (address hashes), 64 bytes (Ed25519 signatures).

### Address

Represents a Stellar account or a deployed contract. Used for authentication and cross-contract calls.

```rust
let user     = Address::generate(&env);          // test helper
let contract = env.current_contract_address();

// Require the address to have signed the transaction
user.require_auth();

// Equality comparison
assert!(user != contract);
```

### Vec

Ordered, dynamically-sized collection. All elements must share the same type.

```rust
let mut list: Vec<i128> = Vec::new(&env);
list.push_back(100);
list.push_back(200);

let first  = list.get(0).unwrap();
let length = list.len();
```

Use Vec for: ordered lists, batch operations, sequences.

### Map

Key-value store with fast lookups.

```rust
let mut settings: Map<Symbol, i128> = Map::new(&env);
settings.set(symbol_short!("fee"), 50);

let fee = settings.get(symbol_short!("fee")).unwrap();
```

Use Map for: metadata, user settings, dictionary-like data.

## Best Practices

1. **Symbol over String for short text** — Symbol is significantly cheaper for identifiers ≤9 characters.
2. **BytesN over Bytes for fixed-size data** — avoids the overhead of length encoding.
3. **i128 for all financial values** — matches the Stellar token standard and avoids precision loss.
4. **Map over Vec for lookups** — Vec lookup is O(n); Map lookup is O(1).
5. **Validate before storing** — check lengths, ranges, and non-zero values at the contract boundary.
6. **Use `checked_*` arithmetic** — `checked_add`, `checked_sub`, etc. prevent silent overflow panics.

## Performance Tips

| Comparison | Cheaper option | Why |
|------------|---------------|-----|
| `Symbol` vs `String` (≤9 chars) | `Symbol` | Encoded as a 64-bit integer, no heap allocation |
| `BytesN<32>` vs `Bytes` (32 bytes) | `BytesN<32>` | Fixed-size, no length field |
| `Map` lookup vs `Vec` scan | `Map` | O(1) vs O(n) |
| `u32` vs `i128` for small counters | `u32` | Smaller encoding |

Gas costs are approximate and subject to change with protocol upgrades. Always benchmark with `soroban contract invoke` on testnet for production-critical paths.

## Type Conversions

```rust
// BytesN → Bytes
let hash: BytesN<32> = BytesN::from_array(&env, &data);
let bytes: Bytes = Bytes::from_slice(&env, hash.to_array().as_slice());

// Bytes → BytesN<32> (panics if not exactly 32 bytes)
let fixed: BytesN<32> = BytesN::from_array(&env, &{
    let mut arr = [0u8; 32];
    for (i, b) in arr.iter_mut().enumerate() { *b = bytes.get(i as u32).unwrap(); }
    arr
});

// Symbol from a string literal
let sym = Symbol::new(&env, "my_key");

// String from a &str
let s = String::from_str(&env, "hello");
```

## Migration from Other Languages

### From Solidity

| Solidity | Soroban (Rust) | Notes |
|----------|---------------|-------|
| `uint256` | `i128` | Soroban uses `i128` for token amounts; no native 256-bit type |
| `uint32` | `u32` | Direct equivalent |
| `address` | `Address` | Covers both EOAs and contracts |
| `bytes32` | `BytesN<32>` | Fixed-size byte array |
| `bytes` | `Bytes` | Variable-length byte array |
| `string` | `String` | UTF-8, host-managed |
| `mapping(k => v)` | `Map<K, V>` | Not persistent by default — store in `env.storage()` |
| `uint[]` | `Vec<u64>` | Typed, host-managed |
| `bool` | `bool` | Direct equivalent |

Key differences:
- There is no `uint256`. Use `i128` for amounts; use two `i128` values or a custom struct for larger numbers.
- `address` in Solidity is a 20-byte value. Soroban's `Address` is an opaque type that covers both G-addresses (accounts) and contract IDs.
- Mappings in Solidity are implicitly persistent. In Soroban, you must explicitly write to `env.storage().persistent()`.

### From EVM (general)

- No `msg.sender` — pass the caller's `Address` explicitly and call `.require_auth()`.
- No `block.timestamp` — use `env.ledger().timestamp()`.
- No `payable` — token transfers are explicit cross-contract calls to a token contract.
- No `revert` with strings — use `panic!` or return a `Result<_, ContractError>`.

### From CosmWasm

| CosmWasm | Soroban |
|----------|---------|
| `Uint128` | `i128` |
| `Addr` | `Address` |
| `Binary` | `Bytes` |
| `Map<K, V>` (cw-storage-plus) | `env.storage().persistent()` with a typed key |
| `Item<T>` | `env.storage().instance().set(key, &value)` |

## Build

```bash
# From repository root
cargo build -p data-types

# WASM release build
cargo build -p data-types --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test -p data-types
```

## Related Examples

- [06-soroban-types](../06-soroban-types/) — `Address`, `Symbol`, `Bytes`, `Map`, `Vec` in depth
- [09-primitive-types](../09-primitive-types/) — integer types and arithmetic safety
- [08-custom-structs](../08-custom-structs/) — `#[contracttype]` structs and nested types
- [07-enum-types](../07-enum-types/) — `#[contracttype]` enums

## References

- [Soroban SDK docs](https://docs.rs/soroban-sdk/)
- [Soroban type system](https://developers.stellar.org/docs/smart-contracts/learn/storing-data)
- [Stellar token standard (SEP-41)](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
