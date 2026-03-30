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

## What This Example Shows

### Primitive Types (Integers)

Soroban provides several integer types optimized for different use cases:

- **u32** - 32-bit unsigned integer (0 to 4,294,967,295)
- **u64** - 64-bit unsigned integer (0 to 18,446,744,073,709,551,615)
- **i128** - 128-bit signed integer (most common for financial values)

```rust
// Store and retrieve integers
let amount: i128 = 1_000_000_000; // 1 billion (common for token amounts)
let count: u32 = 42;
let large_number: u64 = 1_000_000_000_000;
```

**When to use:**
- Use `i128` for financial amounts (balances, transfers, prices)
- Use `u32` for counters, IDs, or small values
- Use `u64` for timestamps, large counters, or IDs

### Text Types

Soroban provides two text types with different characteristics:

#### Symbol
- Short, gas-efficient identifiers (≤9 characters recommended)
- Ideal for enum-like values and short labels
- Significantly cheaper than String for short text

```rust
// Token symbols, status values, action names
let token = symbol_short!("USDC");
let status = Symbol::from_str(&env, "active");
```

**When to use Symbol:**
- Token symbols (e.g., "USDC", "ETH")
- Enum-like values (e.g., "active", "paused", "pending")
- Short identifiers and keys
- Function names and action types

#### String
- Variable-length text content
- Suitable for human-readable messages
- Supports Unicode characters

```rust
// Longer text, user messages, descriptions
let message = String::from_str(&env, "Transaction completed successfully");
let description = String::from_str(&env, "Alice's savings account");
```

**When to use String:**
- User-provided descriptions
- Error messages
- Longer text content (>9 characters)
- Human-readable labels

### Binary Types

Soroban provides two binary types for different scenarios:

#### Bytes
- Variable-length binary data (0 to ~2GB)
- Ideal for arbitrary binary data of unknown size
- Supports concatenation and slicing

```rust
// Arbitrary binary data, serialized objects
let signature = Bytes::from_slice(&env, &signature_bytes);
let encoded_data = Bytes::from_slice(&env, &serialized_object);
```

**When to use Bytes:**
- Arbitrary binary data
- Serialized objects or protocols
- Data of variable length
- Signatures and cryptographic data

#### BytesN<N>
- Fixed-length binary data (compile-time size)
- More gas-efficient than Bytes for fixed-size data
- Common sizes: 32 bytes (SHA-256), 20 bytes (address hashes), 64 bytes

```rust
// Cryptographic hashes, fixed-size identifiers
let hash = BytesN::<32>::from_array(&env, &sha256_result);
let address_hash = BytesN::<20>::from_array(&env, &address_bytes);
```

**When to use BytesN:**
- Cryptographic hashes (SHA-256 = 32 bytes)
- Fixed-size identifiers
- Address hashes (20 bytes)
- Any fixed-size binary data

### Address Type

The Address type represents user accounts or contract identifiers:

```rust
// Store and verify addresses
let user = Address::generate(&env);
let contract = env.current_contract_address();

// Compare addresses
if user == contract {
    // Same address
}
```

**When to use Address:**
- User account identifiers
- Contract addresses
- Authorization and access control
- Cross-contract calls

### Collection Types

#### Vec (Vector)
- Ordered collection of values
- All elements must have the same type
- Supports dynamic growth

```rust
// Create and manipulate vectors
let mut numbers = Vec::new(&env);
numbers.push_back(1);
numbers.push_back(2);

// Access elements
let first = numbers.get(0).unwrap();
let length = numbers.len();
```

**When to use Vec:**
- Ordered lists of values
- Dynamic-size collections
- Sequences of data
- Batch operations

#### Map (Dictionary)
- Key-value associations
- Fast lookups by key
- Keys and values must have consistent types

```rust
// Create and manipulate maps
let mut settings = Map::new(&env);
settings.set(symbol_short!("theme"), 1);
settings.set(symbol_short!("language"), 2);

// Access values
let theme = settings.get(symbol_short!("theme")).unwrap();
```

**When to use Map:**
- Key-value associations
- Fast lookups
- Metadata storage
- User settings or configurations

## Type Characteristics Summary

| Type | Size | Gas Cost | Use Case |
|------|------|----------|----------|
| u32 | 4 bytes | Low | Counters, small values |
| u64 | 8 bytes | Low | Timestamps, large IDs |
| i128 | 16 bytes | Low | Financial amounts |
| Symbol | Variable | Very Low | Short identifiers (≤9 chars) |
| String | Variable | Medium | Longer text (>9 chars) |
| Bytes | Variable | Medium | Arbitrary binary data |
| BytesN<32> | 32 bytes | Low | Hashes, fixed-size data |
| Address | Fixed | Low | Account/contract IDs |
| Vec | Variable | Medium | Ordered collections |
| Map | Variable | Medium | Key-value pairs |

## Performance Considerations

### Gas Optimization Tips

1. **Prefer Symbol over String** for short text
   - Symbol: ~100 gas for short identifiers
   - String: ~1000+ gas for same content

2. **Use BytesN for fixed-size data**
   - BytesN<32>: ~500 gas
   - Bytes (32 bytes): ~1000+ gas

3. **Choose appropriate integer types**
   - u32: Cheapest for small values
   - i128: Standard for financial amounts

4. **Minimize collection operations**
   - Vec/Map lookups are O(n) for Vec, O(1) for Map
   - Use Map for frequent lookups

## Type Conversions

The example demonstrates safe type conversions:

```rust
// String to Symbol (for short strings)
let text = String::from_str(&env, "token");
let symbol = Symbol::from_str(&env, &text.to_string(&env));

// BytesN to Bytes
let hash = BytesN::<32>::from_array(&env, &hash_data);
let bytes = Bytes::from_slice(&env, hash.to_array().as_slice());

// Symbol to String
let symbol = symbol_short!("test");
let string = String::from_str(&env, &symbol.to_string(&env));
```

## Build

From repository root:

```bash
cargo build -p data-types
```

Or from this directory:

```bash
cargo build
```

## Test

```bash
cargo test -p data-types
```

Or from this directory:

```bash
cargo test
```

## Key Takeaways

1. **Choose the right type for your use case** - each type has different gas costs and characteristics
2. **Use Symbol for short identifiers** - significantly more gas-efficient than String
3. **Use BytesN for fixed-size data** - better performance than variable Bytes
4. **Use i128 for financial amounts** - standard practice in blockchain
5. **Understand collection trade-offs** - Vec for ordered data, Map for fast lookups
6. **Consider gas costs** - type choice directly impacts contract execution costs

## Related Examples

- [01-hello-world](../01-hello-world/) - Basic contract structure
- [06-soroban-types](../06-soroban-types/) - Additional type examples
- [09-primitive-types](../09-primitive-types/) - Deep dive into integer types
- [08-custom-structs](../08-custom-structs/) - Complex data structures

## References

- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)
- [Soroban Type System](https://developers.stellar.org/docs/smart-contracts/learn/storing-data)
- [Gas Optimization Guide](https://developers.stellar.org/docs/smart-contracts/learn/storing-data)
