# Authentication Patterns

This example demonstrates basic address authentication patterns using Soroban's `require_auth()` function, showing how to verify caller identity in smart contracts.

## Concepts Covered

- **`require_auth()`**: Core function for verifying transaction authorization
- **Address Verification**: Confirming the caller's identity
- **Authorization Patterns**: Different ways to implement auth checks
- **Error Handling**: Proper responses to auth failures

## Key Functions

### 1. Basic Authentication Pattern
```rust
pub fn basic_auth(env: Env, user: Address) -> bool {
    user.require_auth();  // Verify the user authorized this transaction
    true
}
```

### 2. Transfer Pattern
```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> bool {
    from.require_auth();  // Only 'from' address can initiate transfer
    // Transfer logic...
    true
}
```

### 3. Admin-Only Pattern
```rust
pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), AuthError> {
    // Verify current admin status
    if admin != stored_admin {
        return Err(AuthError::AdminOnly);
    }
    admin.require_auth();  // Admin must authorize the change
    env.storage().instance().set(&ADMIN_KEY, &new_admin);
    Ok(())
}
```

## Security Considerations

### ✅ Best Practices
- **Always call `require_auth()` before state changes**
- **Place auth checks early in function**
- **Validate inputs after authentication**
- **Use custom error types for different failure scenarios**

### ❌ Common Mistakes to Avoid
- **Forgetting to call `require_auth()`**
- **Calling it after state changes**
- **Not handling auth failures properly**
- **Confusing authorization with authentication**

## How Authentication Works

The `require_auth()` function:

1. **Verifies Transaction Signatures**: Ensures the address has signed the current transaction
2. **Prevents Unauthorized Access**: Stops malicious actors from calling functions on behalf of others
3. **Enables Secure Operations**: Allows only authorized parties to perform sensitive actions
4. **Works with Both Accounts and Contracts**: Can authenticate both user accounts and smart contracts

## When to Use `require_auth()`

Use `require_auth()` whenever:
- Transferring assets or value
- Modifying user-specific data
- Changing contract configuration
- Performing privileged operations
- Accessing sensitive information

## Error Handling

The example demonstrates proper error handling with custom error types:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized = 1,
    AdminOnly = 2,
    InvalidAddress = 3,
}
```

## Running Tests

To run the tests for this example:

```bash
cd examples/basics/03-authentication
cargo test
```

## Deployment

To build for deployment:

```bash
cd examples/basics/03-authentication
cargo build --target wasm32-unknown-unknown --release
```

The resulting WASM file will be in `target/wasm32-unknown-unknown/release/auth-patterns.wasm`.

## Additional Resources

- [Soroban Authentication Guide](https://developers.stellar.org/docs/glossary/authentication)
- [Authorization Best Practices](https://developers.stellar.org/docs/guides/security-best-practices)
- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)