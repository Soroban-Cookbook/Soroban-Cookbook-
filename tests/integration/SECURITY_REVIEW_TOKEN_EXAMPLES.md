# Soroban-Cookbook Token Examples Security Review Report

**Date:** August 31, 2026
**Scope:** Issue #795 — Add Security Tests for Token Examples
**Status:** Findings remediated; regression tests added

---

## 1. Executive Summary

This report covers a security review of the token example contracts in this
repository against three vulnerability classes: reentrancy, arithmetic
issues, and authorization bypass. The review found one real, exploitable
reentrancy vulnerability in `06-token-wrapper` (Finding 1), which has been
fixed in this same change alongside the regression test that demonstrates
it. It also found a test-coverage gap — not a contract bug — in
`01-sep41-token`, where the example's own test suite blanket-mocks
authorization and so never actually exercises Soroban's auth enforcement
(Finding 2). Arithmetic handling in `01-sep41-token` was reviewed and found
to already be sound (`checked_add` on every addition, pre-validated
subtractions, `overflow-checks = true` in both the `release` and `test`
workspace profiles as defense in depth); boundary tests were added as
regression coverage rather than as a fix for a bug.

## 2. Scope

- `examples/tokens/01-sep41-token` (`Sep41Token`) — canonical SEP-41
  reference token.
- `examples/tokens/06-token-wrapper` (`TokenWrapper`) — 1:1 wrapper around
  an arbitrary underlying SEP-41/token-interface contract; the only token
  example in this repository whose state-mutating functions make
  cross-contract calls, which is what makes it the relevant reentrancy
  surface among the token examples (`01-sep41-token` itself has no external
  call sites in any state-mutating function, so classic reentrancy doesn't
  apply to it directly).

Reference patterns used: `examples/advanced/05-reentrancy-guard` (the
guard pattern applied to `06-token-wrapper` below), and
`examples/intermediate/multi-sig-patterns` /
`examples/intermediate/ajo-factory` (access-control and factory/multi-contract
test-setup conventions), per this issue's implementation hints.

## 3. Vulnerability Evaluation Categories

1. **Reentrancy:** cross-contract call sites in state-mutating functions
   that could be reentered before the caller's own state settles.
2. **Arithmetic issues:** unchecked addition/subtraction, overflow at
   `i128::MAX`, underflow at zero-balance boundaries, and consistency of
   derived totals (`total_supply`) against underlying balances.
3. **Authorization bypass:** missing or ineffective `require_auth`
   enforcement, and — a distinct issue — test suites that mock
   authorization so broadly they stop verifying it at all.

## 4. Findings & Remediations

### Finding 1: Unbounded Reentrancy in `06-token-wrapper`'s `wrap`/`unwrap`
* **Severity:** High (Fund/Accounting Integrity)
* **Description:** Neither `wrap` nor `unwrap` had a reentrancy guard.
  Both call `TokenClient::transfer` on the `underlying` address configured
  at `initialize` time; `unwrap` also calls `TokenClient::balance` on it
  before the guard existed. Although both functions already applied their
  own storage writes before making that external call (correct
  checks-effects-interactions for their *own* invocation), nothing stopped
  the external call from re-entering `wrap` a second time before the first
  call returned. Because the second, reentrant `wrap` invocation reads the
  *already-updated* balance/supply as its own starting point and then
  performs a completely independent, valid state transition on top of it,
  a single real deposit could mint wrapped shares more than once — the
  reentrant call's own `TokenClient::transfer` never needs to move real
  funds a second time for the mint to "succeed" from the wrapper's
  perspective. This requires the underlying token to be malicious or to
  carry a post-transfer hook (a real, documented category of SEP-41
  extension) — not a risk under this example's own test fixture (the SDK's
  built-in Stellar Asset Contract, which has no hooks), but a live risk
  the moment `initialize` points at any token that does.
* **Remediation:** Added a shared `DataKey::Entered` guard (mirroring
  `examples/advanced/05-reentrancy-guard`) checked and set before any
  storage mutation or external call in `wrap` and `unwrap`, and checked
  (read-only, since it makes no external call itself) at the top of
  `transfer` so a malicious underlying token can't use `transfer` as an
  alternate reentry point while `wrap`/`unwrap` is mid-flight. `unwrap`'s
  guard is set before its pre-existing `TokenClient::balance` backing
  check, not after, since that read is itself an external call and was
  otherwise an unguarded reentry point in its own right. See
  `examples/tokens/06-token-wrapper/src/lib.rs` and its README's new
  "Security" section.

### Finding 2: `01-sep41-token`'s Test Suite Never Exercises Real Authorization
* **Severity:** Medium (Test Coverage Gap, not a contract vulnerability)
* **Description:** Every fixture in `examples/tokens/01-sep41-token/src/test.rs`
  calls `env.mock_all_auths()`, which makes every `require_auth()` call in
  the contract succeed unconditionally, for any address, regardless of
  whether that address's real authorization was ever declared for the
  call. The contract's own logic is correct — `transfer`, `approve`,
  `transfer_from`, `mint`, and `burn` all call `require_auth()` on the
  right party — but the existing test suite cannot detect a regression
  that removed or weakened one of those calls, because it never runs
  without the blanket mock.
* **Remediation:** No contract change was needed (the `require_auth` calls
  are all correctly placed). Added
  `tests/integration/tests/token_security_tests.rs`, using
  `env.mock_auths(&[])` (disabling the blanket mock, the same technique
  already used in `examples/basics/03-authentication/src/test.rs` and
  `tests/integration/tests/basic_security_tests.rs`) to confirm `transfer`,
  `approve`, `transfer_from`, `mint`, and `burn` each genuinely reject a
  call lacking their required signer's real authorization, plus a test
  confirming a real (non-mocked-away) but non-admin signer is still
  correctly rejected by `mint`'s own admin-identity check.

### Arithmetic Review: `01-sep41-token` (No Fix Required)
* **Severity:** N/A — reviewed, no vulnerability found.
* **Description:** Every addition in `mint`/`transfer`/`transfer_from` uses
  `checked_add(...).ok_or(TokenError::ArithmeticOverflow)?`; every
  subtraction (`from_balance - amount`, `owner_balance - amount`,
  `total_supply - amount`) is preceded by an explicit `>=`/`<` comparison
  guaranteeing it can't underflow. The workspace's `[profile.test]` and
  `[profile.release]` both set `overflow-checks = true` as defense in
  depth even for any future unchecked arithmetic.
* **Verification:** Added boundary regression tests in
  `token_security_tests.rs`: minting to exactly `i128::MAX` total supply
  then attempting one more mint returns `TokenError::ArithmeticOverflow`
  (not a panic or silent wraparound); transferring/burning a full balance
  down to exactly zero succeeds and a further transfer/burn attempt is
  cleanly rejected rather than underflowing; `total_supply` is asserted to
  equal the sum of all balances after a mint/transfer/burn sequence
  involving values near `i128::MAX / 2`.

## 5. Security Integration Tests

Added `tests/integration/tests/token_security_tests.rs`:

1. **`transfer_without_sender_auth_is_rejected`**, **`approve_without_owner_auth_is_rejected`**,
   **`transfer_from_without_spender_auth_is_rejected`**, **`mint_without_admin_auth_is_rejected`**,
   **`burn_without_owner_auth_is_rejected`** — each disables the blanket
   auth mock and asserts the call panics with a host authorization error.
2. **`mint_rejects_a_real_but_non_admin_signer`** — a genuinely
   self-authorized non-admin caller is still rejected by the contract's own
   admin check.
3. **`mint_at_i128_max_then_overflow_is_rejected_cleanly`**,
   **`transfer_of_exact_full_balance_succeeds_at_the_boundary`**,
   **`burn_of_exact_full_balance_zeroes_out_without_underflow`**,
   **`total_supply_stays_consistent_with_balances_across_a_mint_burn_cycle`**
   — arithmetic boundary regression tests described above.
4. **`wrap_succeeds_normally_against_a_non_reentrant_token`** — baseline:
   a normal deposit against a non-attacking underlying token still works
   after the guard was added.
5. **`wrap_reentrancy_attack_is_blocked`** — a `MaliciousUnderlyingToken`
   test double (mirroring `05-reentrancy-guard`'s `MaliciousContract`
   pattern) whose `transfer` calls back into `wrap` for the same deposit;
   asserts the whole transaction panics rather than double-minting.

## 6. Verification Status

`cargo test -p integration-tests`, `cargo test -p token-wrapper`, and
`cargo build --target wasm32-unknown-unknown --release -p token-wrapper`
(the acceptance criteria's stated verification commands) could **not** be
run in the environment this change was authored in — the local disk was
at 100% capacity (4.4 MB free of 136 GB) for the duration of this work, a
pre-existing environment condition unrelated to this change. In place of
running them, every modified/added file was manually re-read for
correctness after writing, including tracing the exact reentrancy call
sequence (outer `wrap` → external `transfer` → reentrant `wrap` → guard
check reading the *same* contract's storage mid-call) against the actual
`DataKey::Entered` implementation to confirm the attack test's
`#[should_panic]` expectation is consistent with the fix, and checking the
arithmetic boundary tests' expected values by hand against
`01-sep41-token`'s exact `checked_add` call order (this caught and fixed a
real error in an earlier draft of the arithmetic tests, where the second
of two chained boundary mints was wrongly assumed to succeed). Please run
the three commands above before merging.
