# Soroban Security Cookbook & Threat Models

This directory contains the security-focused test suite for the Soroban token example implementation (`TokenWrapper`) and for the cookbook's access-control patterns (RBAC, multisig, timelock). These tests are designed to proactively detect common smart contract vulnerabilities and demonstrate safe coding patterns in the Soroban environment.

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

---

### 4. Access Control: RBAC, Multisig & Timelock

Tests live in [`tests/access_control.rs`](tests/access_control.rs) and target:

| Pattern  | Contract                                                                                          |
| -------- | ------------------------------------------------------------------------------------------------- |
| RBAC     | [`examples/intermediate/02-role-based-access-control`](../../examples/intermediate/02-role-based-access-control) |
| Multisig | [`examples/intermediate/multi-sig-patterns`](../../examples/intermediate/multi-sig-patterns)      |
| Timelock | [`examples/advanced/02-timelock`](../../examples/advanced/02-timelock)                            |

#### Signature spoofing methodology
`mock_all_auths()` approves every `require_auth()` call, so it cannot show that a
spoofed signature is rejected. The spoofing tests use `env.mock_auths(&[..])`
instead. That supplies only the attacker's signature, bound to the exact contract,
function and arguments. The host must then fail the call with
`Error(Auth, InvalidAction)`. The same approach shows that a real signature is
**bound to its arguments**: it cannot be replayed for a different proposal id or a
higher role.

#### RBAC: privilege escalation
| Threat | Test |
| ------ | ---- |
| Outsider grants itself any role | `outsider_cannot_grant_any_role` |
| Moderator promotes itself or others | `moderator_cannot_escalate_self_or_others` |
| Admin creates Admins/Owners | `admin_cannot_mint_admins_or_owners` |
| Admin demotes the Owner or a peer Admin by *granting* a lower role | `admin_cannot_demote_owner_via_grant`, `admin_cannot_downgrade_peer_admin_via_grant` |
| Admin/Moderator revokes a higher or equal role | `admin_cannot_revoke_owner_or_peer_admin`, `moderator_cannot_revoke_anyone` |
| Revoked admin keeps its privileges | `revoked_admin_loses_all_privileges` |
| Re-initialization hijacks ownership | `reinitialize_cannot_hijack_ownership` |
| Role management before init | `uninitialized_contract_rejects_role_management` |
| Attacker passes Owner as `caller` | `spoofed_owner_caller_rejected` |
| Owner's signature reused for a higher role | `owner_signature_bound_to_granted_role` |

> **Fixed vulnerability:** `grant_role` used to check only the *new* role, so an
> Admin could call `grant_role(admin, owner, User)` and strip ownership. The
> contract now also requires the caller to outrank the account's **current**
> role (`require_can_manage`).

#### Multisig: invalid signer sets, missing approvals, spoofing
| Threat | Test |
| ------ | ---- |
| Zero, unreachable or empty threshold/signer set | `invalid_signer_sets_rejected` |
| Signer set replaced via re-init | `reinitialize_cannot_replace_signer_set` |
| Non-signer creates, approves or cancels | `outsider_with_valid_signature_is_not_a_signer` |
| Execution below threshold | `missing_approvals_block_execution`, `outsider_executor_cannot_bypass_threshold` |
| One signer approves twice / is listed twice | `repeated_approval_cannot_fake_quorum`, `duplicate_signer_entries_do_not_inflate_voting_power` |
| Replay of an executed proposal | `executed_proposal_cannot_be_replayed_or_reapproved` |
| Executing a cancelled proposal | `cancelled_proposal_cannot_execute_even_with_quorum` |
| Acting on non-existent proposals | `unknown_proposal_rejected` |
| Approval submitted on behalf of a signer | `spoofed_signer_approval_rejected` |
| Approval replayed onto another proposal | `approval_signature_bound_to_proposal_id` |
| Partial signer set on joint-auth actions | `multi_auth_action_rejects_partial_signatures`, `require_all_signers_rejects_missing_signer` |

> `execute` is intentionally permissionless once quorum is reached. The
> approval threshold is the security gate, not the executor's identity.

#### Timelock: bypass attempts
| Threat | Test |
| ------ | ---- |
| Executing before `execute_at` (including 1 s early) | `execute_before_delay_rejected_at_every_boundary` |
| Queuing with zero, too short or too long delay | `delay_outside_bounds_rejected` |
| Re-queuing to reset the schedule | `requeue_cannot_reset_schedule` |
| Lowering bounds to accelerate queued ops | `lowering_delay_bounds_does_not_accelerate_queued_ops` |
| Configuring the delay below the absolute floor | `delay_bounds_cannot_disable_timelock` |
| Replay after execution | `executed_operation_cannot_be_replayed` |
| Executing cancelled/unknown ops | `cancelled_or_unknown_operation_cannot_execute` |
| Executing while paused | `pause_blocks_ready_operations_and_new_queues` |
| Admin takeover via re-init or after handover | `reinitialize_cannot_replace_admin`, `previous_admin_loses_rights_after_handover` |
| Attacker-signed `queue`/`execute`/`cancel`/`set_admin`/`update_delay_bounds`/`set_pause` | `spoofed_*_rejected` |

## Running

```bash
cargo test -p security-tests                        # full suite
cargo test -p security-tests --test access_control  # access control only
```

## Status

- [x] Issue #799: Security tests for access control (RBAC, multisig, timelock)
