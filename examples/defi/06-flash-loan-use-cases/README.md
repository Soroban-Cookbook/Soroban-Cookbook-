# Flash Loan Use Cases

Practical flash loan receiver contracts built on top of the core flash loan
provider in [`05-flash-loans`](../05-flash-loans/).

---

> ⚠️ **WARNING — READ BEFORE USING IN PRODUCTION**
>
> Flash loans are powerful but dangerous. Errors in receiver logic can drain
> every token the contract holds in a single transaction. All examples here are
> for **educational purposes only**. Before deploying any receiver contract:
>
> 1. Get an independent security audit.
> 2. Test against a forked mainnet environment, not just unit tests.
> 3. Implement rate-limiting / circuit-breakers for high-value pools.
> 4. Never approve more than `amount + fee` — ever.

---

## Overview

| Contract | File | Description |
|---|---|---|
| `ArbitrageContract` | `src/arbitrage.rs` | Two-leg AMM arbitrage |
| `RefinancingContract` | `src/refinancing.rs` | Atomic debt refinancing |
| `SecureReceiverContract` | `src/security.rs` | Security pattern reference |

---

## Core Concept

A flash loan lets you borrow any amount with **no collateral**, provided the
loan (plus fee) is repaid **within the same transaction**. If repayment fails,
the entire transaction reverts as if nothing happened.

```
borrower ──flash_loan()──► provider
provider ──transfer()────► receiver
provider ──on_flash_loan()► receiver   ← your logic runs here
provider ◄──transfer_from()────────── receiver  (repayment)
```

The receiver contract must:
1. Implement `on_flash_loan(initiator, token, amount, fee)`.
2. Perform its logic and end with enough tokens to repay.
3. Call `token.approve(self, initiator, amount + fee, ...)` before returning.

---

## Use Case 1: Arbitrage (`arbitrage.rs`)

Exploits a price discrepancy between two AMM pools atomically:

```
borrow A  →  swap A→B on pool1 (better rate)
           →  swap B→A on pool2 (worse rate)
           →  repay A + fee
           →  keep profit
```

Key points:
- Profit is only possible when the price difference exceeds the flash loan fee.
- Both swaps and the repayment happen inside one Soroban transaction; if any
  step fails the entire transaction reverts, preventing partial losses.
- Store intermediate parameters (pool addresses, token) in **temporary storage**
  so they are available to the callback and cleaned up automatically.

---

## Use Case 2: Refinancing (`refinancing.rs`)

Moves a collateralised debt position from one lending pool to another with a
better interest rate, without needing the full debt amount up-front:

```
borrow debt_token  →  repay old_pool  →  receive collateral
                   →  deposit collateral in new_pool
                   →  borrow from new_pool to cover repayment
                   →  repay flash loan + fee
```

Key points:
- The borrower never needs to hold liquid funds equal to their debt; the flash
  loan bridges the gap.
- If the new pool's terms are worse than expected the transaction reverts,
  keeping the original position intact.

---

## Use Case 3: Security Patterns (`security.rs`)

Demonstrates the mandatory safety checks every receiver must apply:

| Check | Why |
|---|---|
| Validate `initiator == registered_provider` | Prevents anyone from triggering your callback with crafted parameters |
| `amount > 0 && fee >= 0` | Guards against zero-value or negative-fee edge cases |
| `balance >= amount + fee` before approving | Ensures the contract can actually repay before granting the allowance |
| Approve **exactly** `amount + fee` | An unlimited approval (`i128::MAX`) lets the provider drain your contract |
| Allowance expires at `ledger().sequence() + 1` | Leftover allowances are a persistent attack surface |
| Emit audit event | Enables off-chain monitoring and forensics |

---

## Security Checklist

Before deploying any flash loan receiver:

- [ ] `on_flash_loan` validates `initiator` against a stored, immutable provider address
- [ ] Allowance is set to **exactly** `amount + fee`, not more
- [ ] Allowance expires at `sequence + 1` (single-transaction scope)
- [ ] Contract does not persist borrowed funds in persistent/instance storage
- [ ] No re-entrant call paths exist within the callback
- [ ] Slippage / min-out parameters are set for every swap
- [ ] Profit maths accounts for all fees (swap fees, flash loan fee)
- [ ] Contract has been audited before mainnet deployment

---

## Running the Tests

```bash
cd examples/defi/06-flash-loan-use-cases
cargo test
```

Tests cover:
- Security: valid callback, rogue caller, zero amount, underfunded receiver,
  exact allowance, audit event, admin controls, duplicate init.
- Arbitrage: callback approves the correct repayment after two mock swaps.
- Refinancing: callback approves the correct repayment after mock pool ops.

---

## Related Examples

- [`05-flash-loans`](../05-flash-loans/) — the flash loan provider (reentrancy
  guard, fee collection, callback dispatch).
- [`04-collateralized-lending`](../04-collateralized-lending/) — lending pool
  patterns used in the refinancing example.
- [`02-constant-product-amm`](../02-constant-product-amm/) — AMM swap
  mechanics used in the arbitrage example.
