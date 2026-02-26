# Soroban Types Demonstration

This example demonstrates the unique types available in Soroban smart contracts and their proper usage patterns.

## Overview

Soroban provides several specialized types optimized for blockchain use cases:

- **Address** - Account and contract identifiers
- **Bytes** - Variable-length binary data
- **BytesN** - Fixed-length binary data (ideal for hashes)
- **Symbol** - Short, efficient string-like identifiers
- **String** - Standard string handling for longer text

## Type Characteristics

### Address
- Represents user accounts or contract identifiers
- Essential for authentication and access control
- Supports equality comparison
- Can be generated randomly for testing

### Bytes
- Variable-length byte arrays (0 to ~2GB)
- Ideal for arbitrary binary data
- Supports concatenation and slicing
- Perfect for serialized objects or protocols

### BytesN<N>
- Fixed-length byte arrays (compile-time size)
- Optimized for constant-size data like cryptographic hashes
- More gas-efficient than Bytes for fixed-size data
- Common sizes: 32 bytes (SHA-256), 20 bytes (address hashes), 64 bytes

### Symbol
- Short strings (≤9 characters recommended)
- Highly gas-efficient for identifiers
- Perfect for keys, enum values, and short labels
- Use instead of String for short text

### String
- Variable-length text content
- Suitable for human-readable messages
- Supports Unicode characters
- Use for longer text content

## Usage Patterns

### When to Use Each Type

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
```

### Performance Considerations

- **Symbol vs String**: Use Symbol for text ≤9 characters - significantly more gas-efficient
- **BytesN vs Bytes**: Use BytesN when size is known at compile time - better performance
- **Address**: Always use Address type for account identifiers - built-in validation

## Examples

### Address Operations
```rust
// Store and retrieve addresses
client.store_address(&user);
let stored_user = client.get_address();

// Compare addresses
let is_same = client.verify_address(&addr1, &addr2);

// Generate new address
let new_user = client.generate_address();
```

### Binary Data Handling
```rust
// Variable-length data
let data = Bytes::from_slice(&env, b"arbitrary data");
client.store_bytes(&data);

// Fixed-size hash
let hash = BytesN::<32>::from_array(&env, &hash_result);
client.store_fixed_bytes(&hash);

// Convert between types
let variable = client.fixed_to_variable_bytes(hash);
```

### Efficient Identifiers
```rust
// Short identifiers (use Symbol)
let token = symbol_short!("ETH");
let status = Symbol::from_str(&env, "active");

// Longer text (use String)
let description = String::from_str(&env, "Ethereum cryptocurrency");
let full_name = String::from_str(&env, "Alice Johnson");
```

## Cross-Type Operations

The example demonstrates type interoperability:

```rust
// String to Symbol conversion (for short strings)
let short_text = String::from_str(&env, "token");
let symbol = Symbol::from_str(&env, &short_text.to_string(&env));

// BytesN to Bytes conversion
let hash = BytesN::<32>::from_array(&env, &hash_data);
let bytes = Bytes::from_slice(&env, hash.to_array().as_slice());
```

## Best Practices

1. **Use Symbol for identifiers**: More gas-efficient than String for short text
2. **Prefer BytesN for fixed-size data**: Better performance than variable Bytes
3. **Validate input types**: Ensure data meets expected constraints
4. **Choose appropriate types**: Consider gas costs and use case requirements
5. **Handle conversions carefully**: Some type conversions may have limitations

## Testing

The example includes comprehensive tests covering:
- Basic type operations
- Storage and retrieval
- Type conversions
- Edge cases and performance
- Cross-type integrations

Run tests with:
```bash
cargo test -p soroban_types_example
```

## Build

```bash
# Build the contract
cargo build -p soroban_types_example

# Build for deployment
cargo build -p soroban_types_example --target wasm32-unknown-unknown --release
```
