# Error Recovery Quick Start

## 🚀 5-Minute Guide

### When to Use Each Pattern

```rust
// 1. TRY-CATCH: When caller needs to handle errors
match contract.try_transfer(&from, &to, &amount) {
    Ok(result) => println!("Success: {:?}", result),
    Err(Error::InsufficientBalance) => println!("Not enough funds"),
    Err(e) => println!("Other error: {:?}", e),
}

// 2. FALLBACK: When alternative approach exists
let result = contract.transfer_with_fallback(
    &from, &to, 
    &1000,  // Try this first
    &500    // Fall back to this if insufficient
);

// 3. GRACEFUL DEGRADATION: When processing multiple items
let results = contract.batch_transfer(&from, &transfers);
// Some succeed, some fail - all results returned

// 4. ATOMIC: When all must succeed or none
let result = contract.atomic_batch_transfer(&from, &transfers);
// Either all succeed or nothing changes

// 5. VALIDATION: When security is critical
let result = contract.safe_transfer(&from, &to, &amount);
// Multiple validation layers + rate limiting
```

## 📋 Pattern Selection Flowchart

```
Need to process multiple operations?
├─ Yes → Can some fail while others succeed?
│   ├─ Yes → Use GRACEFUL DEGRADATION (batch_transfer)
│   └─ No → Use ATOMIC (atomic_batch_transfer)
└─ No → Is there an alternative approach?
    ├─ Yes → Use FALLBACK (transfer_with_fallback)
    └─ No → Use TRY-CATCH (try_transfer)

Need extra security? → Add VALIDATION (safe_transfer)
```

## 🔧 Common Recipes

### Recipe 1: Safe Token Transfer
```rust
pub fn safe_token_transfer(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<TransferResult, Error> {
    // Validate
    validate_transfer(env.clone(), from.clone(), to.clone(), amount)?;
    
    // Execute with error handling
    try_transfer(env, from, to, amount)
}
```

### Recipe 2: Batch with Fallback
```rust
pub fn batch_with_fallback(
    env: Env,
    from: Address,
    transfers: Vec<(Address, i128, i128)>, // (to, amount, fallback)
) -> Vec<Result<TransferResult, Error>> {
    let mut results = Vec::new(&env);
    
    for transfer in transfers.iter() {
        let (to, amount, fallback) = transfer;
        let result = transfer_with_fallback(
            env.clone(), from.clone(), to, amount, fallback
        );
        results.push_back(result);
    }
    
    results
}
```

### Recipe 3: Retry Logic
```rust
pub fn transfer_with_retry(
    env: Env,
    from: Address,
    to: Address,
    amount: i128,
    max_retries: u32,
) -> Result<TransferResult, Error> {
    let mut last_error = Error::TransferFailed;
    
    for _ in 0..max_retries {
        match try_transfer(env.clone(), from.clone(), to.clone(), amount) {
            Ok(result) => return Ok(result),
            Err(Error::RateLimitExceeded) => {
                // Wait and retry
                continue;
            }
            Err(e) => {
                last_error = e;
                break; // Don't retry on other errors
            }
        }
    }
    
    Err(last_error)
}
```

## ⚡ Quick Tips

### ✅ DO
```rust
// Return Result for recoverable errors
pub fn transfer(...) -> Result<TransferResult, Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    // ...
}

// Validate before state changes
pub fn safe_operation(...) -> Result<(), Error> {
    validate_all_inputs()?;  // Check first
    update_state();          // Then modify
    Ok(())
}

// Use specific error types
return Err(Error::InsufficientBalance);
```

### ❌ DON'T
```rust
// Don't panic for expected errors
if amount <= 0 {
    panic!("Invalid amount");  // BAD
}

// Don't ignore errors
let _ = try_transfer(...);  // BAD

// Don't modify state before validation
update_balance();           // BAD
validate_balance()?;        // Too late!
```

## 🧪 Testing Checklist

```rust
#[test]
fn test_my_function() {
    // ✅ Test success case
    assert!(result.is_ok());
    
    // ✅ Test each error case
    assert_eq!(result, Err(Error::InvalidAmount));
    
    // ✅ Verify state changes
    assert_eq!(balance_after, expected);
    
    // ✅ Test edge cases
    // - Zero amounts
    // - Negative amounts
    // - Boundary conditions
    
    // ✅ Test rollback behavior
    assert_eq!(balance, original_balance); // No change on error
}
```

## 📚 Learn More

- Full documentation: [README.md](./README.md)
- Implementation: [src/lib.rs](./src/lib.rs)
- Tests: [src/test.rs](./src/test.rs)
- Validation: [VALIDATION.md](./VALIDATION.md)

## 🎯 Next Steps

1. Read through [src/lib.rs](./src/lib.rs) to see implementations
2. Run tests: `cargo test -p error-recovery`
3. Try modifying examples to fit your use case
4. Check [README.md](./README.md) for detailed patterns
5. Review [Best Practices](../../../docs/best-practices.md)
