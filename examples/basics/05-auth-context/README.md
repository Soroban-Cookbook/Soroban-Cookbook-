# Auth Context Example

This example demonstrates how to retrieve and use authorization context in Soroban smart contracts, specifically focusing on the difference between the `invoker` and the `current_contract_address`.

## Overview

Authorization context provides important information about:

- **Invoker**: Who invoked the current function (can be a user's address or another contract's address).
- **Current Contract**: The address of the contract currently being executed.

Understanding the authorization context is crucial for building inter-contract calls, implementing proxy patterns, and handling security-sensitive operations.

## Key Concepts

### Invoker vs Current Contract Address

- **Invoker Address parameters**: Rather than a global `env.invoker()` (which was removed in newer SDKs), contracts receive the invoker's `Address` as a function argument and verify it using `address.require_auth()`.
- `env.current_contract_address()`: Returns the address of the contract being executed.

### Inspecting Authorization Context (`env.auths()`)

- `env.auths()`: Returns the authorizations that have been approved or provided in the current invocation.
- In tests, this allows developers to inspect the call stack of authorizations to verify that a specific call chain was properly authenticated.
- **Note on Mocking:** When using `env.mock_all_auths()` in tests, authorizations are mocked at the host level. The contract's local `env.auths()` array behaves differently than it would with a real transaction payload, but correctly populated test payloads will demonstrate the expected context.

### When does Context Matter?

The distinction is critical when dealing with nested calls. If User A calls Contract Proxy, and Contract Proxy calls Contract Target, inside Contract Target:

- The **User A** address is passed through and authenticated via `user_address.require_auth()`.
- `env.current_contract_address()` = **Contract Target**'s address.

Because the immediate caller is Contract Proxy, Contract Target must ensure that `require_auth` is used so that the authentication checks the entire call stack back to the original User A.

### Security Considerations

1. **Don't confuse the immediate caller with the authorized admin**: For robust authentication where you need to guarantee a specific user authorized the action (even through call chains), always use `address.require_auth()`.
2. **Proxies act on behalf of users**: When your contract acts as a proxy, it must require the user's auth _before_ making the cross-contract call.
3. **Use `require_auth` for explicit checks**:
   - Use `address.require_auth()` when you want to ensure `address` approved the operation somewhere in the call stack.
   - You can also compare an authenticated address with an expected admin address (`if invoker == expected_admin`) to enforce access control boundaries.

## Use Cases

1. **Proxy Contracts and Factory Patterns**: A proxy or factory needs to track caller context across boundaries.
2. **Access Control Systems**: Certain functions (like contract upgrades or configuration changes) should only be callable directly by a specific admin address, enforcing strict `invoker` boundary checks.
3. **Inter-contract Communication**: Trusting specific sender contracts to update shared state.

## Run the Tests

```bash
cargo test
```

## Build the Contract

```bash
cargo build --target wasm32-unknown-unknown --release
```
