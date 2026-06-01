# Error Handling

Master Soroban error patterns: custom errors, Result<T,E>, panic! best practices.

## 📊 Error Types Comparison

| Pattern | Use When | Pros | Cons |
|---------|----------|------|------|
| `Result<T, CustomError>` | Expected failures, user input | Type-safe, frontend codes, cheap | Verbose |
| `panic!()` | Invariants, bugs | Simple | Wastes gas, poor UX |
| `env.require_auth()` | Auth checks | Built-in | Generic message |

## 🎯 Best Practices

1. **Custom errors ≥8 variants** w/ `#[contracterror]` + u32 codes
2. **`Result<_, E>`** for business logic, `panic!` for invariants only
3. **Event on errors** for debugging
4. **Test all paths** w/ `client.try_fn()`
5. **Frontend mappable** codes (1=InvalidInput, etc.)

## 💾 Key Code

```rust
#[contracterror]
#[repr(u32)]
pub enum Error {
    InvalidInput = 1, 
    Unauthorized = 2,
    // ...
}

pub fn transfer(env: Env, amount: u64) -> Result<(), Error> {
    if amount == 0 { return Err(Error::InvalidInput); }
    // ...
    Ok(())
}
```

## 🔬 Examples

**[Primary: 03-custom-errors](../examples/basics/03-custom-errors/)**
- 8 error types (table/mapping)
- Events on failures
- Complex multi-check ops
- `cargo test -p custom-errors`

**[Panic vs Result: 05-error-handling](../examples/basics/05-error-handling/)**
- Good/bad patterns
- `transfer()` vs `transfer_panic()`

**[Foundational: 12-error-handling](../examples/basics/12-error-handling/)**
- Clean implementation of `Result` vs `panic!`
- Explicit error codes and division logic
- Comprehensive `try_` client testing

## 🧪 Testing Errors

```rust
let err = client.try_transfer(&0);
assert!(matches!(err, Err(Error::InvalidInput)));
```

## 📚 Related
- [Validation → Prevent errors early](./validation-patterns.md)  
- [Storage → Error on bad keys](./storage-patterns.md)

