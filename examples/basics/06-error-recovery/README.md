# Error Recovery

Comprehensive patterns for error handling and recovery in Soroban smart contracts, demonstrating try-catch patterns, fallback logic, graceful degradation, and transaction rollback.

## 📖 What You'll Learn

- Try-catch patterns with Result types
- Fallback logic for resilient operations
- Graceful degradation with partial success
- Transaction rollback and atomic operations
- Multi-layer validation strategies
- Rate limiting and error recovery

## 🔍 Contract Overview

This example demonstrates five complementary error recovery patterns:

### 1. Try-Catch Pattern

```rust
pub fn try_transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<TransferResult, Error>
```

Returns `Result` type allowing callers to handle errors explicitly. Validates inputs, checks balances, and executes transfers with proper error propagation.

### 2. Fallback Logic

```rust
pub fn transfer_with_fallback(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
    fallback_amount: i128,
) -> Result<TransferResult, Error>
```

Attempts primary operation first, automatically falls back to alternative on failure. Useful for operations where partial success is acceptable.

### 3. Graceful Degradation

```rust
pub fn batch_transfer(
    env: Env,
    from: Address,
    transfers: Vec<(Address, i128)>,
) -> Vec<Result<i128, Error>>
```

Processes multiple operations independently, returning individual results. Some operations can fail while others succeed, providing maximum resilience.

### 4. Transaction Rollback

```rust
pub fn atomic_batch_transfer(
    env: Env,
    from: Address,
    transfers: Vec<(Address, i128)>,
) -> Result<i128, Error>
```

Validates all operations before executing any. If validation fails, no state changes occur (all-or-nothing semantics).

### 5. Multi-Layer Validation

```rust
pub fn safe_transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<TransferResult, Error>
```

Implements multiple validation layers: pre-validation, rate limiting, execution with error handling, and post-execution updates.

## 💡 Key Concepts

### Error Types

Custom error enum provides clear, specific error information:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InsufficientBalance = 1,
    InvalidAmount = 2,
    Unauthorized = 3,
    TransferFailed = 4,
    ServiceUnavailable = 5,
    ValidationFailed = 6,
    RateLimitExceeded = 7,
}
```

### Result Types

Using `Result<T, Error>` enables explicit error handling:

```rust
match try_transfer(env, from, to, amount) {
    Ok(result) => // Handle success
    Err(Error::InsufficientBalance) => // Handle specific error
    Err(e) => // Handle other errors
}
```

### Atomic Operations

Two-phase commit pattern ensures atomicity:

1. **Validation Phase**: Check all preconditions
2. **Execution Phase**: Apply all changes only if validation passed

### Partial Success

Graceful degradation allows operations to partially succeed:

```rust
// Returns Vec<Result<i128, Error>>
// Each transfer result is independent
let results = batch_transfer(env, from, transfers);
```

## 🔒 Security Best Practices

1. **Always validate before state changes** — Check all preconditions before modifying storage
2. **Use Result types for recoverable errors** — Allow callers to handle errors appropriately
3. **Implement rate limiting** — Prevent abuse with time-based restrictions
4. **Validate all inputs** — Check amounts, addresses, and parameters
5. **Use atomic operations for critical paths** — Ensure consistency with all-or-nothing semantics
6. **Provide clear error messages** — Use specific error types for debugging
7. **Test error paths thoroughly** — Verify behavior under all failure conditions

## 🎯 Pattern Selection Guide

### Use Try-Catch When:
- Caller needs to handle errors
- Multiple error conditions exist
- Error recovery is caller's responsibility

### Use Fallback Logic When:
- Alternative approaches exist
- Partial success is acceptable
- User experience matters more than strict semantics

### Use Graceful Degradation When:
- Processing multiple independent items
- Some failures shouldn't block others
- Partial results are valuable

### Use Transaction Rollback When:
- All operations must succeed together
- Consistency is critical
- Partial execution would be invalid

### Use Multi-Layer Validation When:
- Security is paramount
- Multiple failure modes exist
- Rate limiting or throttling is needed

## 🧪 Testing

```bash
cargo test
```

Tests cover:

- **Try-catch patterns** — Success, insufficient balance, invalid amounts
- **Fallback logic** — Primary success, fallback activation, both fail
- **Graceful degradation** — All succeed, partial success, mixed errors
- **Transaction rollback** — Success, insufficient balance rollback, invalid amount rollback
- **Validation** — Valid transfers, invalid amounts, insufficient balance, same address
- **Safe transfers** — Success, rate limiting, cooldown periods
- **Recovery** — Default values, missing data handling

## 🚀 Building & Deployment

```bash
# Build
cargo build --target wasm32-unknown-unknown --release

# Test
cargo test

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/error_recovery.wasm \
  --source alice \
  --network testnet
```

## 📊 Error Recovery Decision Tree

```
Operation Fails
    │
    ├─ Is recovery possible?
    │   ├─ Yes → Try fallback logic
    │   └─ No → Return error to caller
    │
    ├─ Is partial success acceptable?
    │   ├─ Yes → Use graceful degradation
    │   └─ No → Use atomic operations
    │
    └─ Can operation be retried?
        ├─ Yes → Implement retry logic
        └─ No → Return error with context
```

## 🎓 Next Steps

- [Basics Index](../README.md) - Browse the full basics learning path
- [Authentication](../03-authentication/) - Combine auth with error handling
- [Events](../04-events/) - Emit events for error tracking
- [Storage Patterns](../02-storage-patterns/) - Understand storage error handling

## 📚 References

- [Soroban Error Handling](https://developers.stellar.org/docs/smart-contracts/fundamentals-and-concepts/errors)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Best Practices Guide](../../../docs/best-practices.md)

## 🔍 Real-World Use Cases

### DeFi Protocols
- Fallback to alternative liquidity pools
- Graceful degradation for batch swaps
- Atomic multi-step transactions

### Token Operations
- Try-catch for transfer validation
- Batch transfers with partial success
- Rate-limited withdrawals

### Governance Systems
- Atomic proposal execution
- Fallback voting mechanisms
- Graceful handling of invalid votes

### Escrow Services
- Multi-party validation
- Atomic fund releases
- Fallback refund mechanisms

## ⚠️ Common Pitfalls

### ❌ Don't: Ignore errors
```rust
// BAD: Silently ignoring errors
let _ = try_transfer(env, from, to, amount);
```

### ✅ Do: Handle errors explicitly
```rust
// GOOD: Explicit error handling
match try_transfer(env, from, to, amount) {
    Ok(result) => // Handle success
    Err(e) => // Handle error appropriately
}
```

### ❌ Don't: Use panics for expected errors
```rust
// BAD: Panic for recoverable errors
if balance < amount {
    panic!("Insufficient balance");
}
```

### ✅ Do: Return Result for recoverable errors
```rust
// GOOD: Return error for caller to handle
if balance < amount {
    return Err(Error::InsufficientBalance);
}
```

### ❌ Don't: Partial state changes without rollback
```rust
// BAD: State changed even if later operations fail
update_balance(from, -amount);
// If this fails, from's balance is already changed!
update_balance(to, amount);
```

### ✅ Do: Validate before any state changes
```rust
// GOOD: Validate everything first
validate_all_operations()?;
// Only then apply changes
update_balance(from, -amount);
update_balance(to, amount);
```

## 🏆 Best Practices Summary

1. Use `Result<T, Error>` for all fallible operations
2. Define specific error types for different failure modes
3. Validate inputs before state changes
4. Implement atomic operations for critical paths
5. Provide fallback mechanisms where appropriate
6. Test all error paths thoroughly
7. Document error conditions clearly
8. Use rate limiting to prevent abuse
9. Return meaningful error messages
10. Consider partial success patterns for batch operations
