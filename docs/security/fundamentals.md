# Security Fundamentals for Soroban Smart Contracts

This guide provides a practical foundation for developing secure smart contracts on the Stellar network. Developing for Soroban requires an understanding of Rust, the Soroban SDK, and the specific security model of decentralized environments.

---

## 1. Threat Model Basics

Security starts with identifying what you are trying to protect and who might want to compromise it.

### Assets at Risk
*   **Tokens (XLM, SAC, Custom)**: Liquid assets stored in contract balances or managed via allowances.
*   **Contract State**: Persistent entries, instance settings, and user-specific data (e.g., account limits, roles).
*   **User Funds**: Collateral in DeFi pools, deposits in vaults, or escrowed payments.

### Attackers
*   **Malicious Users**: Can call any public function with arbitrary input to find edge cases.
*   **Malicious Contracts**: Interacting with your contract via `invoke_contract` and potentially re-entering or failing mid-call.
*   **Validators/Network (rare)**: While the network is robust, understanding resource limits (footprint, CPU, RAM) is critical to prevent DoS.

### Attack Surfaces
*   **Public Functions**: Any function not protected by authorization or internal visibility.
*   **External Calls**: Every time your contract interacts with another, you lose control of the execution flow until the call returns.
*   **State Transitions**: Modifications to storage that do not maintain consistency (invariants).

---

## 2. High-Risk Vulnerability Classes

### 2.1 Unauthorized Access / Missing Authorization
**What it is**: Allowing a user to perform an action they should not have permission for (e.g., withdrawing funds that don't belong to them).

**How it happens**: Forgetting to call `require_auth()` or using `require_auth_for_args()` incorrectly.

**Mitigation**:
*   Always call `address.require_auth()` for any sensitive operation involving that address.
*   Validate `admin` roles stored in contract storage.

**Soroban-Specific Example**:
```rust
// BAD: Missing authorization
pub fn withdraw(e: Env, user: Address, amount: i128) {
    // Anyone can call this for any user!
    transfer_funds(&e, &user, amount);
}

// GOOD: Proper authorization
pub fn withdraw(e: Env, user: Address, amount: i128) {
    user.require_auth(); // Only the user can initiate their own withdrawal
    transfer_funds(&e, &user, amount);
}
```

### 2.2 Reentrancy / Unexpected Call Flows
**What it is**: An attacker-controlled contract calling back into your contract before the first call finishes, potentially causing multiple updates to the state.

**How it happens**: Making an external call before updating the local state (e.g., transferring tokens before decreasing the user's balance).

**Mitigation**:
*   Use the **Checks-Effects-Interactions (CEI)** pattern.
*   Perform all state updates (Effects) before making external calls (Interactions).

### 2.3 Integer Overflow / Underflow
**What it is**: Arithmetic operations exceeding the maximum or minimum value of a type (e.g., `u8` wrapping from 255 to 0).

**How it happens**: Using standard operators (`+`, `-`, `*`) on user-provided values.

**Mitigation**:
*   Always use **checked arithmetic**: `checked_add`, `checked_sub`, `checked_mul`, `checked_div`.
*   Handle the `None` case with `panic_with_error!`.

### 2.4 State Inconsistency
**What it is**: Leaving the contract in a "broken" state if an operation fails or if state variables don't match (e.g., `total_supply` not matching the sum of all balances).

**How it happens**: Updating one piece of state but failing to update another related piece.

**Mitigation**:
*   Ensure all related state variables are updated atomically within the same transaction.
*   Maintain invariants (e.g., total supply must always equal the sum of ledger entries).

### 2.5 Input Validation Issues
**What it is**: Accepting invalid or malicious parameters (e.g., negative amounts, expired timestamps).

**How it happens**: Trusting that the caller provides "reasonable" inputs.

**Mitigation**:
*   Check that amounts are strictly positive (`amount > 0`).
*   Validate addresses (e.g., ensuring they are not the contract itself if that creates a bug).
*   Check deadlines/timeouts against `env.ledger().timestamp()`.

### 2.6 Denial of Service (DoS)
**What it is**: Making the contract unusable by exhausting network resources (gas, compute, storage).

**How it happens**: Using unbounded loops (looping over every user in the system) or excessive storage writes.

**Mitigation**:
*   Avoid loops that scale with the number of users or entries.
*   Use pagination or pull-based patterns instead of push-based updates.
*   Monitor Soroban resource limits (CPU, RAM, Ledger Footprint).

### 2.7 Precision / Rounding Errors
**What it is**: Losing value due to integer division or incorrect scaling.

**How it happens**: Dividing before multiplying in a sequence of calculations.

**Mitigation**:
*   **Multiply before dividing**: `(amount * price) / precision`.
*   Use a fixed scale (e.g., 7 or 9 decimal places) consistently.

---

## 3. Mitigation Checklist

Before deploying your contract, verify the following:

- [ ] **Authorization**: All sensitive functions call `require_auth()` or `require_auth_for_args()`.
- [ ] **Arithmetic**: All math uses `checked_*` methods and handles potential overflows.
- [ ] **CEI Pattern**: State is updated *before* any external contract calls or token transfers.
- [ ] **Input Validation**: All parameters (amounts, IDs, deadlines) are validated for sanity.
- [ ] **Storage Management**: No unbounded loops or storage growth that could hit ledger limits.
- [ ] **Event Logging**: State-changing operations emit clear descriptive events.
- [ ] **Error Handling**: Custom error types are used with `panic_with_error!` for clarity.
- [ ] **Invariants**: All critical invariants (e.g., token balance sums) are maintained across calls.

---

## 4. Secure Development Workflow

Follow this process to minimize the risk of vulnerabilities:

1.  **Architecture Review (Threat Modeling)**: Identify assets and potential attack paths before writing code.
2.  **Implementation**: Use idiomatic patterns and safe wrappers (like the Soroban SDK defaults).
3.  **Self-Checklist**: Run through the **Mitigation Checklist** above on every pull request.
4.  **Unit & Edge Case Testing**: Test not only "happy paths" but also invalid inputs, large numbers, and zero values.
5.  **Integration Testing**: Test interactions between multiple contracts (e.g., swapping tokens through a pool).
6.  **Peer Review**: Have another developer review the logic, specifically looking for authorization gaps.
7.  **External Audit**: For high-value contracts, involve a third-party security firm.

---

## 5. Soroban-Specific Examples

### Safe Arithmetic Pattern
```rust
// BAD: Unsafe addition
let new_balance = old_balance + amount;

// GOOD: Checked arithmetic
let new_balance = old_balance
    .checked_add(amount)
    .unwrap_or_else(|| e.panic_with_error!(Error::Overflow));
```

### Checks-Effects-Interactions (CEI)
```rust
// BAD: Interaction before Effect
pub fn claim_rewards(e: Env, user: Address) {
    user.require_auth();
    let reward = get_reward_amount(&e, &user);
    
    // Interaction (external call)
    token::Client::new(&e, &token_id).transfer(&e.current_contract_address(), &user, &reward);
    
    // Effect (state update) AFTER interaction
    set_user_reward(&e, &user, 0); // Risky!
}

// GOOD: Effect before Interaction
pub fn claim_rewards(e: Env, user: Address) {
    user.require_auth();
    let reward = get_reward_amount(&e, &user);
    
    // Effect (state update)
    set_user_reward(&e, &user, 0);
    
    // Interaction (external call)
    token::Client::new(&e, &token_id).transfer(&e.current_contract_address(), &user, &reward);
}
```
