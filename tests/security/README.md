# Soroban Token Security Cookbook & Threat Models

This directory contains the security-focused test suite for the Soroban token example implementation (`TokenWrapper`). These tests are designed to proactively detect common smart contract vulnerabilities and demonstrate safe coding patterns in the Soroban environment.

## 🔒 Threat Models & Mitigation Strategies

### 1. Reentrancy Attacks

#### The Threat
Reentrancy occurs when a contract transfers execution control to an untrusted external entity (e.g., calling a custom token or smart contract) before it updates its internal state variables. A malicious contract can exploit this intermediate, stale state by calling back into the original contract (reentering) before the initial execution completes. 

In `TokenWrapper`, the `wrap` function accepts an external `underlying` token. When wrapping, it calls the underlying token's `transfer` method to lock the collateral. If this underlying token is a custom, malicious token contract, it intercepts the `transfer` invocation and can immediately call back to `unwrap` the wrapped tokens. 
* **If State Updates are Deferred (Vulnerable)**: The `unwrap` function reads the user's stale balance (since the state updates in `wrap` have not been saved yet). It successfully unwraps the tokens, and then when the reentrant call finishes, `wrap` resumes and overwrites the balance with the newly minted amount. This allows the attacker to double-spend and mint wrapped tokens out of thin air.

#### The Mitigation: Checks-Effects-Interactions
The `TokenWrapper` example has been fortified by adhering to the **Checks-Effects-Interactions (CEI)** pattern:
1. **Checks**: Validate inputs (e.g., `require_positive(amount)`).
2. **Effects**: Perform all internal state modifications (e.g., `Balance` and `TotalSupply` storage updates) *before* interacting with external contracts.
3. **Interactions**: Perform the external underlying token transfer.

Because Soroban transactions are fully transactional and atomic, any panic during the external transfer interaction automatically reverts all state changes that occurred in the "Effects" step. By updating the state first, any reentrant call to `unwrap` or `transfer` sees the updated state, rendering reentrancy attacks completely harmless.

* **Test Case**: `test_reentrancy_prevention` deploys a custom `MaliciousToken` to attempt this exact callback exploit and asserts that the state remains secure and consistent.

---

### 2. Authorization & Access Control Bypass

#### The Threat
If privileged actions (such as `wrap`, `unwrap`, `transfer`, or initial contract configuration) can be performed without verifying that the caller actually authorized the action, malicious actors can drain users' assets, manipulate balances, or take over contract administration.

In Soroban, contracts must explicitly call `address.require_auth()` to verify that the signature or cryptographic authorization of the entity owning the address is present in the transaction.

#### The Mitigation
- **Sender/Caller Verification**: Every action that alters a user's assets (`wrap`, `unwrap`, `transfer`) invokes `user.require_auth()` or `from.require_auth()`.
- **Re-initialization Prevention**: The `initialize` function verifies whether the underlying asset is already set using `has(&DataKey::Underlying)`. If it is already initialized, it immediately aborts, preventing subsequent administrative takeovers.

* **Test Cases**: 
  - `test_unauthorized_initialize` ensures that calling `initialize` a second time is rejected.
  - `test_unauthorized_wrap` ensures that attempting to wrap tokens without the user's explicit authorization/signature fails.
  - `test_unauthorized_transfer` ensures that transferring wrapped tokens without the owner's authorization/signature fails.

---

### 3. Arithmetic Overflows & Underflows

#### The Threat
Smart contracts frequently deal with balance adjustments. If arithmetic operations fail to check for boundaries, they can wrap around:
- **Overflow**: Exceeding the maximum capacity of `i128`, causing values to wrap around to large negative numbers or panic.
- **Underflow**: Exceeding the minimum capacity (e.g., trying to subtract more than a user's balance), which could result in massive positive balances if unchecked.
- **Negative Inputs**: Passing zero or negative values for wrap or transfer operations could allow draining funds or manipulating supplies.

#### The Mitigation
- **Checked Arithmetic**: We use Rust's `.checked_add()` and similar checked operations to explicitly catch overflows. When an overflow is detected, the contract returns a clean `WrapperError::ArithmeticOverflow` instead of panicking or wrapping silently.
- **Input Validation Guardrails**: The contract strictly checks that all deposit/transfer amounts are positive (`require_positive(amount)?`), returning `WrapperError::InvalidAmount` for zero or negative values.
- **Balance Checks**: Before any subtraction (`unwrap`, `transfer`), the contract explicitly asserts that `old_balance >= amount`, returning `WrapperError::InsufficientWrappedBalance` on failure.

* **Test Cases**:
  - `test_invalid_wrap_amount` and `test_invalid_transfer_amount` check that negative and zero inputs are properly rejected.
  - `test_wrap_overflow` validates that wrapping amounts which would overflow `i128::MAX` are blocked safely.
  - `test_unwrap_insufficient_balance` and `test_transfer_insufficient_balance` confirm that spending more than the available balance is securely blocked.
