# Basic Address Authentication

This example demonstrates how to implement basic address authentication in Soroban smart contracts using the `require_auth()` function. By the end of this example, you won't just understand *how* to authenticate callers, but you'll understand *why* it's done securely using various fundamental patterns.

## Concepts Covered

- **`require_auth()` Usage:** The core function provided by the Soroban SDK to verify that a transaction is authorized.
- **Address Parameter Patterns:** Passing explicit `Address` types to functions to specify the subject of the operation.
- **Authorization Checks:** Validating whether the authenticated user has the necessary permissions (e.g., matching a stored admin).
- **Error Handling:** Implementing custom error types for unauthorized access to provide clear failure reasons.

## Key Patterns

### 1. The Admin-Only Pattern
Restricting certain operations (like minting tokens or updating contract settings) to a specific privileged address.

```rust
pub fn admin_action(env: Env, admin: Address, value: u32) -> Result<u32, AuthError> {
    // 1. Verify that 'admin' has authorized this transaction
    admin.require_auth();
    
    // 2. Load the actual stored admin from the contract
    let stored_admin: Address = env.storage().instance()
        .get(&DataKey::Admin).ok_or(AuthError::NotAdmin)?;

    // 3. Verify that the authorized caller is the stored admin
    if admin != stored_admin {
        return Err(AuthError::NotAdmin);
    }

    Ok(value * 2)
}
```

### 2. Single-Address Auth (Transfer Pattern)
Authenticating a standard user action, such as transferring tokens. Only the owner of the tokens can authorize the debit.

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
    // Ensure only 'from' can authorize the removal of their own balance
    from.require_auth();

    // After auth, validate that the user has enough balance
    let from_balance: i128 = env.storage().persistent()
        .get(&DataKey::Balance(from.clone())).unwrap_or(0);
        
    if amount <= 0 || from_balance < amount {
        return Err(AuthError::InsufficientBalance);
    }
    
    // Proceed to deduct from 'from' and add to 'to'
    ...
}
```

### 3. Delagated Allowance Pattern
Authorizing a third party (a `spender`) to transfer assets on behalf of an owner.

```rust
pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
    // The spender authorizes the transaction
    spender.require_auth();

    // The contract queries its state to ensure the owner previously allowed this
    let allowance: i128 = env.storage().persistent()
        .get(&DataKey::Allowance(from.clone(), spender.clone())).unwrap_or(0);

    if allowance < amount {
        return Err(AuthError::Unauthorized); // Prevent unauthorized transfers
    }

    ... // process transfer
}
```

### 4. N-of-N Multi-Sig Pattern
Requiring multiple users to explicitly authorize an operation in a single call.

```rust
pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
    // The Soroban host collects all authorizations atomically
    for signer in signers.iter() {
        signer.require_auth();
    }
    value + signers.len()
}
```

## Security Best Practices

1. **Explicit Admin Checks:** Simply calling `require_auth()` guarantees the address signed the transaction, but it doesn't guarantee the address is the **admin**. Always compare against your stored admin state.
2. **Authorize Before Logic:** Place `require_auth()` calls at the very beginning of your functions to prevent unauthenticated users from consuming contract execution cycles or triggering unexpected logic.
3. **Use Custom Errors:** Use `#[contracterror]` enums (like `AuthError::NotAdmin` or `AuthError::Unauthorized`) instead of panics where feasible, so that callers get deterministic error signals.

## Running the Example

Run the comprehensive unit test suite to see these authentication patterns in action:
```bash
cd examples/basics/03-authentication
cargo test
```

## Building the Contract

Compile the contract to WebAssembly:
```bash
cargo build --target wasm32-unknown-unknown --release
```
The resulting `.wasm` file will be located at `target/wasm32-unknown-unknown/release/auth-patterns.wasm`.
