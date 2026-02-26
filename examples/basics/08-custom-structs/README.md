# Custom Struct Types in Soroban

This example demonstrates how to work with custom struct types in Soroban smart contracts, covering the `#[contracttype]` derive macro, nested structs, storage patterns, and serialization.

## Overview

Custom structs are fundamental building blocks in Soroban contracts, allowing you to organize complex data structures and create type-safe, maintainable code. This example shows comprehensive patterns for struct definition, usage, and storage.

## What's Covered

### 1. Basic Struct Definitions

#### Simple Struct with `#[contracttype]`

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

**Key Points:**
- `#[contracttype]` enables storage compatibility
- Use supported Soroban types (`Address`, `String`, `u32`, `u64`, `i128`, `bool`, `Option<T>`)
- Derive essential traits for comparison and debugging
- Avoid `u8`/`i8` types - use `u32`/`i32` instead

#### Asset Information Struct

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetInfo {
    pub contract_address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u32,  // Note: u32, not u8
    pub total_supply: Option<i128>,
    pub native: bool,
}
```

### 2. Nested Struct Examples

#### Complex User Profile

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedUserProfile {
    pub profile: UserProfile,           // Nested struct
    pub preferences: UserPreferences,    // Another nested struct
    pub statistics: UserStatistics,     // Nested with calculations
    pub security: SecuritySettings,     // Nested with vectors
}
```

#### Deep Nesting Pattern

```rust
pub struct Portfolio {
    pub holdings: Vec<AssetHolding>,           // Vector of structs
    pub metadata: PortfolioMetadata,            // Nested metadata
}

pub struct AssetHolding {
    pub asset: AssetInfo,                      // Nested asset info
    pub purchase_history: Vec<PurchaseRecord>,  // Vector of nested structs
}
```

**Benefits of Nested Structs:**
- Logical data organization
- Type safety at all levels
- Easy access patterns
- Maintainable structure

### 3. Struct Storage Patterns

#### Direct Storage

```rust
// Store single struct
env.storage().instance().set(&symbol_short!("profile"), &user_profile);

// Retrieve struct
let profile: UserProfile = env.storage()
    .instance()
    .get(&symbol_short!("profile"))
    .ok_or(ContractError::NotFound)?;
```

#### Key-Based Storage

```rust
// Store with composite key
env.storage().instance().set(
    &(symbol_short!("profile"), user_address), 
    &user_profile
);

// Retrieve with composite key
let profile: UserProfile = env.storage()
    .instance()
    .get(&(symbol_short!("profile"), user_address))
    .ok_or(ContractError::NotFound)?;
```

#### Collection Storage

```rust
// Store vector of structs
let portfolios: Vec<Portfolio> = Vec::new(&env);
env.storage().instance().set(&symbol_short!("portfolios"), &portfolios);
```

**Storage Best Practices:**
- Use short symbols (max 9 characters)
- Composite keys for user-specific data
- Consider storage limits for large structs
- Use appropriate storage patterns (instance vs temporary)

### 4. Serialization and Deserialization

#### Automatic Serialization

Soroban automatically handles serialization when storing structs:

```rust
// Store - automatically serialized
env.storage().instance().set(&key, &my_struct);

// Retrieve - automatically deserialized
let struct: MyStruct = env.storage().instance().get(&key)?;
```

#### Manual Serialization Example

```rust
pub fn serialize_struct(env: Env, profile: UserProfile) -> Result<i32, ContractError> {
    // Store and retrieve to demonstrate serialization
    let temp_key = symbol_short!("temp");
    env.storage().instance().set(&temp_key, &profile);
    
    let _retrieved: UserProfile = env.storage()
        .instance()
        .get(&temp_key)
        .ok_or(ContractError::SerializationError)?;
    
    env.storage().instance().remove(&temp_key);
    
    // Return hash/identifier
    Ok(12345)
}
```

#### Struct Validation

```rust
pub fn validate_struct(_env: Env, profile: UserProfile) -> Result<bool, ContractError> {
    // Validate field constraints
    if profile.name.len() == 0 || profile.name.len() > 100 {
        return Err(ContractError::InvalidFieldValue);
    }
    
    if profile.reputation > 1000 {
        return Err(ContractError::InvalidFieldValue);
    }
    
    Ok(true)
}
```

## Contract Functions

### User Profile Management

```rust
// Create new profile
pub fn create_user_profile(
    env: Env,
    address: Address,
    name: String,
    email: Option<String>,
) -> Result<UserProfile, ContractError>

// Get existing profile
pub fn get_user_profile(env: Env, address: Address) -> Result<UserProfile, ContractError>

// Update profile fields
pub fn update_user_profile(
    env: Env,
    address: Address,
    name: Option<String>,
    email: Option<String>,
    avatar_hash: Option<String>,
) -> Result<UserProfile, ContractError>
```

### Portfolio Management

```rust
// Create new portfolio
pub fn create_portfolio(
    env: Env,
    owner: Address,
    name: String,
    description: Option<String>,
    portfolio_type: PortfolioType,
) -> Result<Portfolio, ContractError>

// Add asset to portfolio
pub fn add_asset_to_portfolio(
    env: Env,
    owner: Address,
    portfolio_name: String,
    asset: AssetInfo,
    quantity: i128,
    price: i128,
) -> Result<(), ContractError>

// Calculate portfolio value
pub fn calculate_portfolio_value(
    env: Env,
    owner: Address,
    portfolio_name: String,
) -> Result<i128, ContractError>
```

### Extended Profile Management

```rust
// Create comprehensive user profile
pub fn create_extended_profile(
    env: Env,
    address: Address,
    name: String,
    language: String,
) -> Result<ExtendedUserProfile, ContractError>

// Get extended profile
pub fn get_extended_profile(
    env: Env,
    address: Address,
) -> Result<ExtendedUserProfile, ContractError>
```

## Usage Examples

### Basic User Profile

```rust
let user = Address::generate(&env);
let name = String::from_str(&env, "Alice");
let email = String::from_str(&env, "alice@example.com");

// Create profile
let profile = CustomStructsContract::create_user_profile(
    env.clone(),
    user.clone(),
    name.clone(),
    Some(email.clone()),
)?;

// Retrieve profile
let retrieved = CustomStructsContract::get_user_profile(env.clone(), user.clone())?;

// Update profile
let updated = CustomStructsContract::update_user_profile(
    env.clone(),
    user.clone(),
    Some(String::from_str(&env, "Alice Updated")),
    None,
    None,
)?;
```

### Portfolio Management

```rust
let owner = Address::generate(&env);
let portfolio_name = String::from_str(&env, "My Portfolio");

// Create portfolio
let portfolio = CustomStructsContract::create_portfolio(
    env.clone(),
    owner.clone(),
    portfolio_name.clone(),
    Some(String::from_str(&env, "Investment portfolio")),
    PortfolioType::Balanced,
)?;

// Add asset
let asset = AssetInfo {
    contract_address: token_address,
    symbol: String::from_str(&env, "BTC"),
    name: String::from_str(&env, "Bitcoin"),
    decimals: 8,
    total_supply: None,
    native: false,
};

CustomStructsContract::add_asset_to_portfolio(
    env.clone(),
    owner.clone(),
    portfolio_name.clone(),
    asset,
    100000000, // 1 BTC in satoshis
    50000,     // $50,000
)?;

// Calculate value
let value = CustomStructsContract::calculate_portfolio_value(
    env.clone(),
    owner.clone(),
    portfolio_name.clone(),
)?;
```

## Testing

The example includes comprehensive tests covering:

- **Struct Creation**: Basic and complex struct instantiation
- **Storage Patterns**: Direct, key-based, and collection storage
- **Nested Access**: Deep struct field access and manipulation
- **Serialization**: Storage/retrieval cycles
- **Validation**: Field validation and error handling
- **Contract Functions**: Full workflow testing

### Running Tests

```bash
cargo test
```

## Best Practices

### Struct Design

1. **Use Supported Types**: Stick to Soroban-supported types (`Address`, `String`, `u32`, `u64`, `i128`, `bool`, `Option<T>`, `Vec<T>`)
2. **Avoid `u8`/`i8`**: Use `u32`/`i32` instead for compatibility
3. **Logical Grouping**: Group related fields together
4. **Clear Naming**: Use descriptive field and struct names
5. **Documentation**: Document struct purpose and field meanings

### Storage Optimization

1. **Short Symbols**: Keep storage keys under 9 characters
2. **Composite Keys**: Use tuple keys for user-specific data
3. **Size Awareness**: Consider storage limits for large structs
4. **Efficient Access**: Design storage patterns for common access patterns

### Error Handling

1. **Custom Errors**: Use `#[contracterror]` for struct-specific errors
2. **Validation**: Validate struct fields before storage
3. **Graceful Failures**: Return meaningful error messages
4. **Consistent Patterns**: Use consistent error handling across functions

## Common Pitfalls

### Type Compatibility

❌ **Wrong:**
```rust
pub struct BadStruct {
    decimals: u8,  // u8 not supported
}
```

✅ **Correct:**
```rust
pub struct GoodStruct {
    decimals: u32,  // Use u32 instead
}
```

### Move Semantics

❌ **Wrong:**
```rust
let asset = AssetInfo { contract_address: addr, ... };
let portfolio1 = Portfolio { asset, ... };
let portfolio2 = Portfolio { asset, ... }; // Error: value moved
```

✅ **Correct:**
```rust
let asset = AssetInfo { contract_address: addr.clone(), ... };
let portfolio1 = Portfolio { asset: asset.clone(), ... };
let portfolio2 = Portfolio { asset: asset.clone(), ... };
```

### Storage Key Length

❌ **Wrong:**
```rust
env.storage().set(&symbol_short!("very_long_key"), &data); // Too long
```

✅ **Correct:**
```rust
env.storage().set(&symbol_short!("long_key"), &data); // Under 9 chars
```

## Advanced Patterns

### Struct Enums

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PortfolioType {
    Conservative = 0,
    Balanced = 1,
    Aggressive = 2,
    Custom = 3,
}
```

### Optional Fields

```rust
pub struct UserProfile {
    pub email: Option<String>,        // Optional email
    pub avatar_hash: Option<String>,  // Optional avatar
    pub verified: bool,               // Required field
}
```

### Vector Collections

```rust
pub struct Portfolio {
    pub holdings: Vec<AssetHolding>,  // Dynamic collection
    pub history: Vec<Transaction>,     // Transaction history
}
```

## Integration with Other Patterns

This example complements other Soroban patterns:

- **Authentication**: Use structs to store user permissions and roles
- **Validation**: Validate struct fields before storage
- **Error Handling**: Custom error types for struct operations
- **Events**: Emit struct data in contract events
- **Upgradability**: Design structs with version compatibility in mind

## Performance Considerations

1. **Storage Costs**: Larger structs cost more to store
2. **Access Patterns**: Design for common access patterns
3. **Serialization**: Automatic serialization is efficient but has overhead
4. **Memory Usage**: Be mindful of vector sizes in nested structs

## Security Considerations

1. **Input Validation**: Always validate struct fields
2. **Access Control**: Implement proper authorization for struct operations
3. **Data Integrity**: Ensure struct consistency across operations
4. **Privacy**: Consider sensitive data in struct fields

## Conclusion

Custom structs are powerful tools for organizing Soroban contract data. This example demonstrates comprehensive patterns for struct definition, storage, and usage. Follow the best practices and avoid common pitfalls to create maintainable, efficient, and secure smart contracts.

The key takeaways are:
- Use `#[contracttype]` for storage compatibility
- Design logical nested structures
- Implement proper validation and error handling
- Consider storage costs and access patterns
- Test thoroughly with comprehensive test suites
