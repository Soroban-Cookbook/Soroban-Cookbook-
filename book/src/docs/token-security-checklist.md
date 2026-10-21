# Token Security Checklist

A comprehensive security checklist for Soroban token smart contracts (including SEP-41 compliant tokens, mintable/burnable tokens, token wrappers, and multi-token balance managers).

---

## Table of Contents

1. [Authorization & Access Control](#1-authorization--access-control)
2. [Arithmetic Safety & Numerical Integrity](#2-arithmetic-safety--numerical-integrity)
3. [Supply Management & Mint/Burn Controls](#3-supply-management--mintburn-controls)
4. [Transfer & Allowance Validation](#4-transfer--allowance-validation)
5. [Storage Strategy & TTL Lifecycle](#5-storage-strategy--ttl-lifecycle)
6. [Event Emission & Auditability](#6-event-emission--auditability)
7. [Testing Requirements](#7-testing-requirements)
8. [Audit, Deployment & Verification Record](#8-audit-deployment--verification-record)

---

## 1. Authorization & Access Control

Token operations involve direct manipulation of user balances and privileged supply parameters. Authorization must be explicitly verified on all entry points.

### 1.1 Direct User Authorization
- [ ] **Transfer authentication:** `transfer(from, to, amount)` strictly requires authentication from the debit account (`from.require_auth()`).
- [ ] **Delegated transfer authentication:** `transfer_from(spender, from, to, amount)` strictly verifies `spender.require_auth()` and validates sufficient non-expired allowance granted by `from`.
- [ ] **Burn authentication:** `burn(from, amount)` strictly requires authentication from the token owner (`from.require_auth()`).
- [ ] **Delegated burn authentication:** `burn_from(spender, from, amount)` requires `spender.require_auth()` and consumes an authorized allowance.
- [ ] **Approval authentication:** `approve(from, spender, amount, expiration_ledger)` requires `from.require_auth()`.
- [ ] **Argument binding:** Use `require_auth_for_args` where granular call-parameter authorization is mandated.

### 1.2 Administrative & Privileged Roles
- [ ] **Admin role restriction:** Privileged operations (`set_admin`, `mint`, `clawback`, `pause`, `unpause`) verify caller authentication against the stored admin address.
- [ ] **Multi-sig / Governance protection:** High-impact administrative keys (e.g., minter, treasury admin) use multisignature or DAO timelock governance.
- [ ] **Role separation:** Minting, emergency pausing, and admin metadata management roles are distinct where appropriate (Principle of Least Privilege).
- [ ] **Two-step ownership transfer:** Admin updates use a two-step `propose_admin` / `accept_admin` workflow to prevent accidental transfer to unrecoverable addresses.
- [ ] **No hidden backdoors:** Ensure no hardcoded developer keys or unverified bypass paths exist in authorization logic.

---

## 2. Arithmetic Safety & Numerical Integrity

Token balances and allowances must be protected against arithmetic vulnerabilities and precision truncation.

### 2.1 Checked Operations & Safe Math
- [ ] **Overflow/Underflow prevention:** All additions and subtractions on balances and allowances use checked math (`checked_add`, `checked_sub`, `checked_mul`, `checked_div`) or panic-safe integer primitives (`i128`).
- [ ] **Positive amount enforcement:** All mutating functions (`transfer`, `transfer_from`, `mint`, `burn`, `approve`, `clawback`) enforce strictly positive amounts (`amount > 0`). Zero or negative values are explicitly rejected or handled safely.
- [ ] **Balance subtraction order:** Debits must always verify that `balance >= amount` before performing the subtraction.
- [ ] **Allowance subtraction order:** `transfer_from` and `burn_from` verify `allowance >= amount` before deducting the allowance.

### 2.2 Decimal & Precision Handling
- [ ] **Decimals consistency:** Token decimal precision (typically `7` for Stellar SEP-41 standard, or `18` for EVM compatibility) is immutably defined or strictly validated.
- [ ] **Rounding bias:** In fee calculations or reward distributions, rounding favors the protocol/pool to prevent micro-drain attacks.
- [ ] **Division by zero:** Any division operation (e.g., fee rates, exchange conversions) explicitly checks for zero divisor prior to execution.

---

## 3. Supply Management & Mint/Burn Controls

Maintaining invariant balance between total supply and circulating tokens is vital for economic stability.

### 3.1 Total Supply Invariants
- [ ] **Supply invariant:** At all times, `total_supply == sum(all_circulating_balances)`.
- [ ] **Hard cap constraints:** If a maximum supply cap is defined, `mint` operations verify `total_supply.checked_add(amount) <= max_supply`.
- [ ] **Supply accounting on mint:** Minting strictly increments both the recipient's balance and the global `total_supply` atomically.
- [ ] **Supply accounting on burn:** Burning strictly decrements both the target's balance and the global `total_supply` atomically.

### 3.2 Emission Curves & Timelocks
- [ ] **Mint rate limiting:** Large supply expansions are rate-limited or subject to governance timelocks.
- [ ] **Snapshot consistency:** For snapshot tokens, historical balances and total supply snapshots are recorded immutably and cannot be overwritten retroactively.

### 3.3 Regulated Assets & Clawback Security
- [ ] **Clawback flag verification:** Clawback capability (`clawback(admin, from, amount)`) is explicitly enabled at contract initialization and transparent to token holders.
- [ ] **Clawback authorization:** Only designated and authenticated clawback administrators can execute clawbacks.
- [ ] **Balance reduction on clawback:** Clawback burns or moves tokens with strict balance and total supply reconciliation.

---

## 4. Transfer & Allowance Validation

Transfer logic is the most frequently called code path and requires rigorous state validation.

### 4.1 Transfer Execution
- [ ] **Self-transfer safety:** Handling `from == to` does not cause double-spend, balance corruption, or arithmetic underflow.
- [ ] **Sufficient balance check:** Verifies `from_balance >= amount` and emits descriptive errors (e.g., `ContractError::InsufficientBalance`) upon failure.
- [ ] **State mutation atomicity:** Balance updates and allowance deductions are committed atomically; failed conditions abort the entire transaction.

### 4.2 Allowance & Approval Lifecycle
- [ ] **Expiration ledger validation:** `allowance.expiration_ledger >= env.ledger().sequence()` is verified on every `transfer_from` and `burn_from`. Expired allowances are treated as zero.
- [ ] **Approval overwrite safety:** `approve` properly updates or overwrites both amount and expiration ledger without leaving dangling allowances.
- [ ] **Race condition mitigation:** Documentation and helper patterns (`increase_allowance` / `decrease_allowance`) are provided to mitigate front-running on allowance changes.
- [ ] **Zero allowance reset:** Setting allowance to `0` or expired ledger properly cleans up or disables the allowance.

---

## 5. Storage Strategy & TTL Lifecycle

Soroban smart contracts manage state across Instance, Persistent, and Temporary storage tiers with active Time-To-Live (TTL) policies.

### 5.1 Storage Type Selection
- [ ] **Instance storage:** Used for global token metadata (`name`, `symbol`, `decimals`), admin address, and `total_supply`.
- [ ] **Persistent storage:** Used for user balances and allowances to ensure long-term availability.
- [ ] **Temporary storage:** Used strictly for short-lived nonces or transient multi-step signatures.

### 5.2 TTL Extension & Archival Protection
- [ ] **Instance TTL extension:** Contract extends instance TTL during state-mutating calls (`env.storage().instance().extend_ttl(...)`).
- [ ] **Balance TTL extension:** User balance entries have their TTL extended upon every transfer, mint, or balance query (`env.storage().persistent().extend_ttl(&key, ...)`).
- [ ] **Allowance TTL alignment:** Allowance expiration ledger is bounded within safe TTL extension windows.
- [ ] **Restoration handling:** Clear documentation on how archived entries can be restored via ledger footprint bump operations.

---

## 6. Event Emission & Auditability

Events enable off-chain indexers, wallets, and block explorers to track state transitions reliably.

### 6.1 Standard SEP-41 Event Topics
- [ ] **Transfer events:** Emits `transfer(from: Address, to: Address, amount: i128)` with indexed `from` and `to` topics.
- [ ] **Mint events:** Emits `mint(admin: Address, to: Address, amount: i128)`.
- [ ] **Burn events:** Emits `burn(from: Address, amount: i128)`.
- [ ] **Approve events:** Emits `approve(from: Address, spender: Address, amount: i128, expiration_ledger: u32)`.
- [ ] **Admin events:** Emits `set_admin(old_admin: Address, new_admin: Address)` on ownership handover.
- [ ] **Clawback events:** Emits `clawback(admin: Address, from: Address, amount: i128)`.

---

## 7. Testing Requirements

A production token contract must be verified with comprehensive unit, integration, and fuzz testing suites.

### 7.1 Unit Testing Coverage
- [ ] **Standard operations:** Full test coverage for `initialize`, `balance`, `allowance`, `transfer`, `transfer_from`, `approve`, `mint`, `burn`, `decimals`, `name`, `symbol`.
- [ ] **Authorization negative tests:** Tests verifying that unauthorized accounts cannot transfer, mint, burn, approve, or claim admin roles.
- [ ] **Boundary condition tests:**
  - Zero amount transfers / approvals.
  - Maximum integer (`i128::MAX`) transfers and mints.
  - Transfers exceeding current balance by 1 unit.
  - `transfer_from` exceeding authorized allowance by 1 unit.
  - Expired allowance attempts (advancing `env.ledger().set_sequence_number(...)`).
  - Self-transfers (`from == to`).

### 7.2 Invariant & Fuzz Testing
- [ ] **Supply invariant test:** Multi-step fuzzing of random sequences of `mint`, `transfer`, `transfer_from`, and `burn` asserting `total_supply == sum(balances)`.
- [ ] **Solvency preservation:** Asserting that no sequence of operations produces a negative balance.

### 7.3 Integration & Cross-Contract Testing
- [ ] **DEX / AMM integration:** Verified compatibility with liquidity pool contracts (deposits, swaps, withdrawals).
- [ ] **Lending & Vault integration:** Verified compatibility with flash loan and collateralized lending protocols.
- [ ] **TTL extension verification:** Simulating multiple ledger sequence advancements and verifying continuous entry readability.

---

## 8. Audit, Deployment & Verification Record

Before deploying to Stellar Testnet or Mainnet, record all audit parameters and sign-offs:

### 8.1 Pre-Deployment Verification
- [ ] Code passes formatting check: `cargo fmt --all -- --check`
- [ ] Code passes Clippy with zero warnings: `cargo clippy --tests --lib -- -D warnings`
- [ ] WASM size optimization verified: `cargo build --target wasm32-unknown-unknown --release` and `soroban contract optimize`
- [ ] Security checklist fully reviewed and all items marked complete.

### 8.2 Deployment Record Template
```markdown
### 📋 Token Contract Deployment Record
- **Contract Name:** [e.g., MySEP41Token]
- **Target Network:** [Testnet / Mainnet]
- **Contract ID:** `C...`
- **Admin Public Address:** `G...`
- **Initial Total Supply:** [e.g., 10,000,000.0000000]
- **Decimals:** 7
- **Soroban SDK Version:** [e.g., 22.0.0+]
- **WASM Hash:** `...`
- **Reviewed By:** [@reviewer1, @reviewer2]
- **Audit Date:** YYYY-MM-DD
- **Sign-off Status:** ✅ APPROVED FOR PRODUCTION
```

---

## Related Resources

- [SEP-41 Token Example](../examples/tokens/01-sep41-token.md)
- [SEP-41 Extensions & Allowances](../examples/tokens/02-sep41-extensions.md)
- [Pausable Token Pattern](../examples/tokens/03-optimized-operations.md)
- [DeFi Security Checklist](./defi-security-checklist.md)
- [Governance Security Checklist](./governance-security-checklist.md)
- [Security Best Practices](./security-best-practices.md)
