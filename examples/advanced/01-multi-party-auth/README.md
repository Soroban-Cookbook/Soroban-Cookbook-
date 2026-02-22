# Multi-Party Authorization Patterns

This example demonstrates advanced authorization patterns in Soroban that require multiple parties to authorize an action.

## Patterns Demonstrated

### 1. Multi-Signature Transfer (AND Logic)
The `multi_sig_transfer` function requires authorization from **all** addresses provided in a `Vec<Address>`. This is useful for joint custody or mandatory multi-approval actions.

```rust
pub fn multi_sig_transfer(
    env: Env,
    signers: Vec<Address>,
    to: Address,
    amount: i128
) {
    for signer in signers.iter() {
        signer.require_auth();
    }
    // Action only proceeds if all signers authorized
}
```

### 2. Threshold Authorization (M-of-N)
The `proposal_approval` function demonstrates how to implement threshold logic where at least `M` out of `N` authorized signers must approve an action.

- Useful for DAOs, governance, and shared wallets.
- Shows how to use storage to track valid signers and thresholds.
- Demonstrates preventing duplicate approvals.

### 3. Sequential Authorization (Workflow)
The `sequential_auth_escrow` function shows a multi-step workflow.
- **Step 1:** Buyer authorizes funding.
- **Step 2:** Both Buyer and Seller must authorize release (2-of-2).
- Demonstrates how state can track the progress of multi-party actions across multiple transactions.

## Security Considerations

1. **Atomic Authorization:** `require_auth()` ensures that the specified address has signed the transaction for the current contract call.
2. **Order Independence:** The order in which `require_auth()` is called for different addresses does not matter; the host environment collects and verifies all required authorizations.
3. **Dynamic Lists:** When using `Vec<Address>` for signers, ensure the list is bounded to prevent excessive gas costs or potential "unbounded loop" attacks.
4. **Threshold Integrity:** Always verify that approvers are part of the recognized "valid signers" set to prevent unauthorized parties from contributing to a threshold.

## Gas Considerations
- Gas costs scale linearly with the number of authorizations.
- Each `require_auth()` call involves cryptographic verification of a signature (or contract authorization), which is a relatively expensive operation.

## How to run tests

```bash
cargo test -p multi-party-auth
```
