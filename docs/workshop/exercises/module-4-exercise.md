# Module 4 Exercise: SEP-41 Token Implementation

**Objective:** Implement a SEP-41-compliant fungible token with mint, burn, and pausable functionality.

**Duration:** 60 minutes

---

## Prerequisites

- Complete Modules 1-3
- Understand storage, auth, and error handling

---

## Reference Examples

- [`examples/tokens/01-sep41-token/`](../../../examples/tokens/01-sep41-token/) — Base SEP-41 implementation
- [`examples/tokens/03-pausable-token/`](../../../examples/tokens/03-pausable-token/) — Pausable token
- [`examples/tokens/04-mint-burn/`](../../../examples/tokens/04-mint-burn/) — Mint/burn extensions

---

## Part 1: Study the Reference (10 min)

Read the SEP-41 token implementation at [`examples/tokens/01-sep41-token/src/lib.rs`](../../../examples/tokens/01-sep41-token/src/lib.rs).

Pay attention to:
- How `DataKey` enum maps storage slots
- The `initialize()` function and its guards
- The `transfer()` function — auth, balance checks, overflow protection
- The `approve()` / `transfer_from()` pattern
- How events are published with `TransferEventData` and `ApprovalEventData`

---

## Part 2: Build a Pausable Token (35 min)

Create a new contract that extends the SEP-41 token with:

1. All standard SEP-41 functions (transfer, approve, transfer_from, balance, allowance, total_supply, name, symbol, decimals)
2. **Mint** — admin only, increases supply
3. **Burn** — token holder only, decreases supply
4. **Pause/Unpause** — admin only; while paused, transfer and transfer_from fail (but balance/metadata queries still work)

### Key Design Decisions

- **Storage:** Admin, TotalSupply, Name, Symbol, Decimals, Paused → instance storage. Balances, Allowances → persistent storage.
- **Auth:** `from.require_auth()` on transfer, `owner.require_auth()` on approve, `spender.require_auth()` on transfer_from, `admin.require_auth()` on mint, `owner.require_auth()` on burn.
- **Overflow:** Use `checked_add` / `checked_sub` on all arithmetic.
- **Events:** Publish transfer events on transfer/mint/burn, approval events on approve.

### Required Function Signatures

```rust
pub fn initialize(env, admin, name, symbol, decimals, initial_supply) -> Result<(), TokenError>
pub fn transfer(env, from, to, amount) -> Result<(), TokenError>
pub fn approve(env, owner, spender, amount) -> Result<(), TokenError>
pub fn transfer_from(env, spender, owner, to, amount) -> Result<(), TokenError>
pub fn mint(env, admin, to, amount) -> Result<i128, TokenError>
pub fn burn(env, owner, amount) -> Result<i128, TokenError>
pub fn pause(env, admin) -> Result<(), TokenError>
pub fn unpause(env, admin) -> Result<(), TokenError>
pub fn balance(env, user) -> i128
pub fn allowance(env, owner, spender) -> i128
pub fn total_supply(env) -> Result<i128, TokenError>
pub fn name(env) -> Result<String, TokenError>
pub fn symbol(env) -> Result<Symbol, TokenError>
pub fn decimals(env) -> Result<u32, TokenError>
pub fn is_paused(env) -> bool
```

### Error Enum

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    InvalidAmount = 5,
    ArithmeticOverflow = 6,
    AllowanceExceeded = 7,
    ContractPaused = 8,
}
```

---

## Part 3: Write Tests (15 min)

### Required Test Cases

```rust
#[test]
fn test_initialize_sets_metadata_and_supply() { /* ... */ }

#[test]
fn test_double_initialize_rejected() { /* ... */ }

#[test]
fn test_transfer_moves_balance() {
    // Initialize, transfer from admin to user, verify balances
}

#[test]
fn test_transfer_insufficient_balance() { /* ... */ }

#[test]
fn test_transfer_zero_amount_rejected() { /* ... */ }

#[test]
fn test_approve_and_transfer_from() {
    // Approve, then transfer_from, verify allowance deducted
}

#[test]
fn test_transfer_from_exceeds_allowance() { /* ... */ }

#[test]
fn test_mint_admin_only() {
    // Admin mints → succeeds
    // Non-admin mints → fails with Unauthorized
}

#[test]
fn test_burn_owner_only() {
    // Owner burns → succeeds, supply decreases
    // Burn more than balance → fails
}

#[test]
fn test_pause_blocks_transfer() {
    // Pause, transfer → fails with ContractPaused
    // Unpause, transfer → succeeds
}

#[test]
fn test_pause_blocks_transfer_from() { /* ... */ }

#[test]
fn test_balance_works_when_paused() { /* ... */ }

#[test]
fn test_transfer_emits_event() {
    // Transfer, verify event was emitted
}

#[test]
fn test_arithmetic_overflow_rejected() {
    // Mint near i128::MAX, then try to overflow
}
```

---

## Verification Checklist

- [ ] `initialize()` sets all metadata and initial supply
- [ ] Re-initialization is rejected
- [ ] `transfer()` moves tokens and checks balances + overflow
- [ ] `approve()` + `transfer_from()` work correctly
- [ ] `mint()` is admin-only and increases total supply
- [ ] `burn()` is owner-only and decreases total supply
- [ ] `pause()` blocks transfer and transfer_from but not balance queries
- [ ] `unpause()` restores functionality
- [ ] All arithmetic uses checked operations
- [ ] Events are emitted for transfers, approvals, mint, and burn
- [ ] All tests pass
- [ ] WASM compiles successfully

---

## Solution

The full reference solution is the combination of:

1. [`examples/tokens/01-sep41-token/src/lib.rs`](../../../examples/tokens/01-sep41-token/src/lib.rs) — base token
2. [`examples/tokens/03-pausable-token/src/lib.rs`](../../../examples/tokens/03-pausable-token/src/lib.rs) — pausable extension
3. [`examples/tokens/04-mint-burn/src/lib.rs`](../../../examples/tokens/04-mint-burn/src/lib.rs) — mint/burn

Study how these examples combine the patterns. The key additions to the base SEP-41:

```rust
// Pausable guard
fn require_not_paused(env: &Env) -> Result<(), TokenError> {
    if env.storage().instance().get(&DataKey::Paused).unwrap_or(false) {
        return Err(TokenError::ContractPaused);
    }
    Ok(())
}

// Call require_not_paused() at the top of transfer() and transfer_from()
// Do NOT call it in balance(), name(), symbol(), etc.

// Mint: check admin auth, add to balance and total_supply
// Burn: check owner auth, subtract from balance and total_supply
```

---

## Next Steps

Continue to [Module 5: Advanced Patterns & Deployment](../slides/module-5-advanced.md) to learn about proxies, oracles, and deployment.
