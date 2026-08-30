# Token Security Checklist

A practical pre-deployment checklist for Soroban fungible tokens that implement the [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) token interface (and common extensions such as mint, burn, clawback, and freeze). Use it during code review, testing, and audit preparation.

The examples in `examples/tokens/` demonstrate individual controls. A production token should verify every item against its own threat model.

---

## Table of Contents

1. [Authorization Checks](#1-authorization-checks)
2. [Arithmetic Safety](#2-arithmetic-safety)
3. [Supply Management](#3-supply-management)
4. [Transfer Validation](#4-transfer-validation)
5. [Testing Requirements](#5-testing-requirements)
6. [Review Record](#6-review-record)

---

## 1. Authorization Checks

Soroban does not infer authorization from the caller. Every state-changing path must call `require_auth()` on the correct `Address` **before** any storage write.

### 1.1 SEP-41 entry points

- [ ] `transfer(from, to, amount)` calls `from.require_auth()` before balances change
- [ ] `approve(from, spender, amount, expiration_ledger)` calls `from.require_auth()` before writing the allowance
- [ ] `transfer_from(spender, from, to, amount)` calls `spender.require_auth()`, not `from.require_auth()`
- [ ] `burn(from, amount)` calls `from.require_auth()` so a holder can burn only their own tokens
- [ ] `burn_from(spender, from, amount)` calls `spender.require_auth()` and consumes allowance
- [ ] Read-only queries (`balance`, `allowance`, `decimals`, `name`, `symbol`) do not require authorization

### 1.2 Admin and privileged roles

- [ ] `mint`, `set_admin`, `set_authorized` / freeze, and `clawback` verify the stored admin (or dedicated role), not merely that *some* address authorized the call
- [ ] Admin identity is read from instance storage and compared to the authenticated address
- [ ] Admin transfer is two-step (`transfer_admin` + `accept_admin`) or equivalently protected so a typo cannot permanently lock the token
- [ ] Mint, freeze, and clawback roles are separated when a single key must not control every privileged action
- [ ] `initialize` is one-shot: a second call cannot overwrite admin, decimals, or metadata

### 1.3 Allowances

- [ ] `transfer_from` and `burn_from` reject expired allowances (`expiration_ledger < current ledger` or `expiration_ledger == 0` treated as unset)
- [ ] Allowance is decremented with checked arithmetic **before** or atomically with the balance update
- [ ] Changing a non-zero allowance directly to another non-zero value is avoided or documented; prefer set-to-zero-then-set or `increase_allowance` / `decrease_allowance`
- [ ] `permit` / signed approvals (if implemented) bind `from`, `spender`, `amount`, `expiration_ledger`, and a nonce, and reject reused or expired signatures

### 1.4 Authorization testing

- [ ] Each privileged entry point fails when `env.set_auths(&[])` (or an unauthorized address) is used
- [ ] A spender cannot `transfer_from` more than the remaining unexpired allowance
- [ ] A non-admin cannot mint, freeze, clawback, or replace the admin

---

## 2. Arithmetic Safety

Token amounts on Soroban are `i128`. Negative values are representable; overflow and underflow must never wrap into a valid-looking balance.

### 2.1 Checked math

- [ ] All balance and supply updates use `checked_add` / `checked_sub` (or equivalent) and return a `#[contracterror]` on overflow or underflow
- [ ] Fee, tax, or share calculations use `checked_mul` / `checked_div` and define rounding direction (floor vs ceil) in documentation
- [ ] Division by zero is rejected before any divide
- [ ] Release profile keeps `overflow-checks = true` and `panic = "abort"`; contract code still uses checked math so failures are typed errors, not host panics

### 2.2 Decimal precision

- [ ] `decimals()` is immutable after `initialize` and matches the unit used for all amounts (smallest indivisible unit, analogous to stroops)
- [ ] Off-chain UIs convert display amounts with the on-chain decimal; the contract never accepts floating-point values
- [ ] Multi-token or wrapper contracts convert between decimal scales with checked math and documented rounding
- [ ] Fee-on-transfer or rebasing logic (if any) cannot mint or burn implicit dust that breaks `sum(balances) == total_supply`

### 2.3 Input bounds

- [ ] Amounts less than or equal to zero are rejected with a dedicated error (`InvalidAmount` or similar) before storage reads
- [ ] Operations that would push a balance or `total_supply` past `i128::MAX` fail cleanly
- [ ] Subtraction is preceded by an explicit sufficiency check (`balance >= amount`) so underflow is unreachable
- [ ] No `unwrap()`, `expect()`, or `panic!` on attacker-controlled amounts in non-test code

---

## 3. Supply Management

Unbounded or poorly authorized minting is the most common token-level failure. Treat supply as an invariant, not a convenience counter.

### 3.1 Mint and burn

- [ ] Mint is restricted to admin (or a minter role) and checks the cap **before** writing balances
- [ ] Every mint increases both the recipient balance and `total_supply` with checked arithmetic
- [ ] Every burn decreases both the holder balance and `total_supply`; burn cannot go below zero
- [ ] `burn_from` reduces supply only after a valid, unexpired allowance is consumed
- [ ] If clawback is enabled, clawed tokens are burned (or sent to a documented sink) and `total_supply` is updated consistently

### 3.2 Caps and admin mint abuse

- [ ] A maximum supply (or per-epoch mint budget) is stored at initialization and cannot be raised by a single admin without a documented governance path
- [ ] Mint amount, recipient, and new total supply are included in a `mint` event
- [ ] There is no hidden mint path (airdrop helper, “rescue”, migration hook, or constructor leftover)
- [ ] Admin cannot mint to themselves without the same cap and event rules as any other recipient
- [ ] Pause or guardian controls can halt minting without permanently bricking transfers if that is part of the threat model

### 3.3 Supply invariants

- [ ] `sum(all balances) == total_supply` holds after every mint, burn, transfer, clawback, and wrap/unwrap
- [ ] Wrapper tokens maintain a 1:1 peg: wrapped supply equals locked underlying (see `examples/tokens/06-token-wrapper`)
- [ ] Storage for zero balances is removed or left in a state that does not inflate reported supply
- [ ] Metadata (`name`, `symbol`, `decimals`) cannot be used to imply a different supply than `total_supply`

---

## 4. Transfer Validation

Transfers must move value exactly once, to a valid destination, and only when the token is allowed to move.

### 4.1 Amount and destination

- [ ] Zero-amount transfers are rejected (or are a documented no-op that emits no misleading event and does not write storage)
- [ ] Negative amounts are rejected
- [ ] Self-transfers (`from == to`) neither credit the sender twice nor decrement twice; prefer reject or a true no-op after auth and amount checks
- [ ] Destination is a valid `Address`; batch transfers reject length mismatches and any invalid recipient in the batch

### 4.2 Frozen, paused, and unauthorized accounts

- [ ] If `set_authorized` / freeze is implemented, frozen accounts cannot `transfer`, `transfer_from`, `approve`, or `burn` (admin clawback may remain available)
- [ ] Global pause blocks transfers, mints, and burns while leaving `balance` and metadata queries available
- [ ] Pause and freeze checks run before balance writes
- [ ] Unpause / unfreeze is limited to the documented role and emits an event

### 4.3 Balance and allowance integrity

- [ ] Sender balance is checked before debit; insufficient balance returns a typed error
- [ ] Debit sender, credit recipient, and (for `transfer_from`) decrement allowance in a Checks-Effects-Interactions order
- [ ] External contract calls (wrappers, hooks, callbacks) happen only after internal balances are updated
- [ ] Every successful transfer, mint, burn, approve, freeze, and admin change emits a structured event with actor, amounts, and addresses

---

## 5. Testing Requirements

Authorization and arithmetic bugs rarely show up on the happy path. Tests must cover failure cases, random inputs, and invariants.

### 5.1 Unit tests

- [ ] Happy paths: `transfer`, `approve`, `transfer_from`, `mint`, `burn` update balances and supply correctly
- [ ] Auth failures: every mutating entry point is tested with missing or wrong authorization
- [ ] Invalid amounts: zero, negative, `i128::MAX`, and `amount > balance`
- [ ] Allowance: exact spend, over-spend, expired `expiration_ledger`, and zero-then-replace updates
- [ ] Admin: unauthorized mint/freeze/clawback, one-shot `initialize`, and admin rotation
- [ ] Freeze/pause: frozen or paused accounts cannot move tokens; queries still succeed

### 5.2 Fuzz and property tests

- [ ] Random `i128` amounts (including negative and extreme values) never wrap balances
- [ ] Random sequences of `transfer` / `transfer_from` / `approve` never create tokens
- [ ] Fuzz `expiration_ledger` against current ledger sequence
- [ ] Batch transfer and permit (if present) are fuzzed for length mismatches and replayed signatures

### 5.3 Invariant tests

- [ ] **Conservation:** `total_supply` equals the sum of all tracked balances after every operation
- [ ] **Non-negative:** no balance or allowance is stored as a negative `i128`
- [ ] **Allowance:** remaining allowance is never greater than the last approved unexpired amount minus spent
- [ ] **Authorization:** state changes only occur when the required `require_auth` addresses are present
- [ ] Wrapper / multi-token contracts: peg and per-token isolation invariants hold

### 5.4 Integration and scenario tests

- [ ] Cross-contract transfers through a wrapper, vault, or allowance spender succeed and fail as specified
- [ ] Events are asserted (topics and data) so indexers can reconstruct balances
- [ ] Persistent balance entries have TTL extension behavior covered if the token is long-lived
- [ ] Review confirms `cargo test` for the token crate and the workspace security tests pass

---

## 6. Review Record

Record the contract version, network, reviewers, threat model, and unresolved risks alongside the deployment. Re-run this checklist whenever mint roles, caps, freeze/clawback, allowance logic, or decimal metadata change.

---

## Related Resources

- [Security Best Practices](./security-best-practices.md)
- [DeFi Security Checklist](./defi-security-checklist.md)
- [Testing Best Practices](./testing-best-practices.md)
- [Testing Pitfalls](./testing-pitfalls.md)
- [Common Pitfalls](./common-pitfalls.md)
- [Token examples overview](../examples/tokens.md)

## Implementation Examples

See `examples/tokens/` for implementations of these controls:

- `01-sep41-token`: SEP-41 interface (transfer, allowance, burn, metadata)
- `04-mint-burn`: admin mint with a supply cap and user burn
- `05-allowance-pattern`: `approve` / `transfer_from` with `expiration_ledger`
- `06-token-wrapper`: 1:1 wrap/unwrap peg invariant
- `10-pausable-permissions`: pause roles that can halt privileged operations
