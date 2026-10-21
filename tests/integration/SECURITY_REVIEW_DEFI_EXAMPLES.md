# Soroban-Cookbook DeFi Examples Security Review Report

**Date:** March 2025
**Auditor:** Jules, Lead Smart Contract Security Engineer
**Status:** Completed & Remediated

---

## 1. Executive Summary

This report document contains the results of the internal security review of the Decentralized Finance (DeFi) example contracts in the `Soroban-Cookbook` repository. The goal was to audit all target DeFi contracts, verify correctness under edge cases, check for common exploits (including economic vectors, reentrancy, oracle price manipulation, and precision errors), and remediate any critical-to-high severity security issues.

All identified vulnerabilities have been successfully resolved, and comprehensive integration tests simulating attacks and exploit vectors have been implemented to prevent regressions.

---

## 2. Scope

The scope of this audit covers all DeFi example contracts inside the repository:

* **01-simple-swap** (`SimpleSwapContract`) — Static token swapper.
* **02-constant-product-amm** (`ConstantProductAmm`) — Uniswap V2-style AMM with LP tokens.
* **03-lending-pool** — Decentralized lending protocol ledger-keeper.
* **04-collateralized-lending** (`LendingContract`) — Multi-role collateralized lending.
* **05-flash-loans** (`FlashLoanContract`) — Uncollateralized single-transaction borrowing.
* **06-flash-loan-use-cases** — Practical implementations (arbitrage, refinancing, security patterns).
* **07-staking-pool** (`StakingPoolContract`) — Staking reward distribution.
* **08-liquidity-mining** (`LiquidityMining`) — Multi-pool liquidity reward pool.
* **09-vault-strategies** (`VaultContract`) — Multi-strategy yield-bearing vault.
* **10-swap-liquidity** (`SwapLiquidityContract`) — AMM liquidity manager with LP token minting.
* **11-amm-price-oracle** (`AmmPoolContract` / `AmmOracleContract`) — On-chain price oracle and TWAP calculator.
* **12-farming-pool** (`FarmingPoolContract`) — Multi-pool yield farm.
* **13-amm-router** (`AMMRouter`) — Multi-hop swap path calculator.
* **staking-pool-legacy** — Time-locked staking pool with boost incentives and early withdrawal penalties.

---

## 3. Vulnerability Evaluation Categories

We audited the DeFi examples against the following vulnerability vectors:

1. **Economic Exploits & Flash Loan Attacks:** Unbounded slippage, lack of invariant/collateral checks, pool swaps/liquidation draining.
2. **Reentrancy & State Inconsistencies:** Cross-contract reentrancy vectors, improper state ordering (violating checks-effects-interactions).
3. **Oracle Manipulation & Price Feed Security:** Spot price manipulation, stale prices, lack of validation on price sources.
4. **Precision & Rounding Vulnerabilities:** Loss of precision, integer overflow/underflow, division-by-zero panics, rounding bias favorability to attackers.
5. **Authorization & Access Control:** Missing `require_auth` validations, unprotected administrative operations, privileged role escalation.

---

## 4. Findings & Remediations

Below is the detailed list of critical and high-severity vulnerabilities found during the security review, along with the remediations applied to resolve them.

### Finding 1: Collateral Check Bypass on Withdrawal in `03-lending-pool`
* **Severity:** Critical (Economic Exploit / Fund Drain)
* **Description:** In the `withdraw` function of `LendingPool`, there was no validation verifying if the user's remaining deposit (collateral) is sufficient to back their active debt (borrow). A user could deposit 1,000 tokens, borrow 800 tokens (up to the 80% limit), and then withdraw 1,000 tokens of their collateral—leaving them with 800 tokens of debt and 0 collateral. This would result in severe pool insolvency and complete drainage of depositor funds.
* **Remediation:** Enforced a post-withdrawal collateral health check to ensure remaining collateral complies with the 80% borrow limit:
  ```rust
  let max_borrow = position.deposit * 80 / 100;
  if position.borrow > max_borrow {
      panic!("insufficient collateral remaining");
  }
  ```

### Finding 2: Unprotected Initialization in `07-staking-pool`
* **Severity:** Critical (Privilege Escalation / Admin Takeover)
* **Description:** The `initialize` function in `StakingPoolContract` did not check if the contract had already been initialized. An attacker could invoke `initialize` again, overwrite the owner, change the staking/reward tokens to malicious addresses, and update the reward rate—effectively seizing control of the pool and hijacking all staker assets.
* **Remediation:** Added an initialization check verifying if the contract is already initialized:
  ```rust
  if env.storage().instance().has(&DataKey::Owner) {
      panic!("already initialized");
  }
  ```

### Finding 3: Unprotected Initialization in `10-swap-liquidity`
* **Severity:** Critical (Privilege Escalation / Admin Takeover)
* **Description:** The `initialize` function in `SwapLiquidityContract` lacked any initialization protection. An attacker could call `initialize` repeatedly to overwrite the contract `owner`, the pool assets, and LP tokens.
* **Remediation:** Integrated initialization protection using the `Owner` key:
  ```rust
  if env.storage().instance().has(&DataKey::Owner) {
      panic!("already initialized");
  }
  ```

### Finding 4: Missing Authorization (`require_auth`) in `10-swap-liquidity`
* **Severity:** High (Unauthorized Actions / Stealing User Tokens)
* **Description:** Both the `add_liquidity` and `remove_liquidity` functions in `SwapLiquidityContract` received a `provider` address but did not call `provider.require_auth()`. Any user could call these functions on behalf of another user without their consent.
* **Remediation:** Added explicit `provider.require_auth();` checks in both `add_liquidity` and `remove_liquidity`.

### Finding 5: Unprotected Initialization in `11-amm-price-oracle` Pool and Oracle Contracts
* **Severity:** Critical (Privilege Escalation / Admin Takeover)
* **Description:** Both `AmmPoolContract::initialize` and `AmmOracleContract::initialize` did not check if they were already initialized, allowing any user to overwrite critical states (e.g. pool `Owner`, pool tokens, oracle `Owner`, and the target pool contract address).
* **Remediation:** Implemented initialization checks checking if the respective `Owner` exists in storage.

### Finding 6: Missing Authorization in `11-amm-price-oracle`'s `deposit` Function
* **Severity:** High (Unauthorized Actions / Griefing)
* **Description:** The `deposit` function of `AmmPoolContract` took a `provider` address but lacked any `provider.require_auth()` verification, allowing arbitrary users to execute deposits on behalf of other addresses.
* **Remediation:** Added explicit `provider.require_auth();` check.

### Finding 7: Unvalidated `reward_rate` causing Overflow DoS in `12-farming-pool`
* **Severity:** Medium/High (Denial of Service / Arithmetic Overflow)
* **Description:** The `add_pool` and `set_reward_rate` functions did not validate `reward_rate` inputs. An administrator could accidentally set a negative rate or an excessively high reward rate. An excessively high rate would cause multiplication overflow panics during global updates (`reward * 1_000_000_000_000`), permanently locking the pool (DoS).
* **Remediation:** Enforced validation bounds on the rate input to ensure it is positive and capped at `1_000_000_000_000_000`:
  ```rust
  if reward_rate <= 0 || reward_rate > 1_000_000_000_000_000 {
      panic!("Invalid reward rate");
  }
  ```

### Finding 8: Missing Division-by-Zero and Negative Input Checks in `13-amm-router`
* **Severity:** Medium (Arithmetic Robustness / Panics)
* **Description:** The `calculate_swap_output` function of `AMMRouter` did not validate if `reserve_in` or `reserve_out` were positive. This could lead to a division-by-zero panic or negative swap outputs.
* **Remediation:** Added checks validating positive reserves and positive `amount_in` values:
  ```rust
  if reserve_in <= 0 || reserve_out <= 0 {
      panic!("insufficient liquidity in pool");
  }
  if amount_in <= 0 {
      panic!("amount_in must be positive");
  }
  ```

### Finding 9: Potential Subtraction Underflow in `04-collateralized-lending` Liquidation
* **Severity:** High (Arithmetic Safety / Accounting Inconsistencies)
* **Description:** In `LendingContract::liquidate`, if `collateral_to_seize` exceeded the borrower's total `position.collateral`, subtracting `position.collateral -= collateral_to_seize` would cause a subtraction underflow panic (blocking liquidation) or a negative collateral record.
* **Remediation:** Capped `collateral_to_seize` at `position.collateral` and adjusted the final repaid debt accordingly to maintain exact mathematical proportions:
  ```rust
  let mut collateral_to_seize = actual_repay * (100 + liquidation_incentive) / 100;
  let mut final_repay = actual_repay;
  if collateral_to_seize > position.collateral {
      collateral_to_seize = position.collateral;
      final_repay = collateral_to_seize * 100 / (100 + liquidation_incentive);
  }
  ```

---

## 5. Security Integration Tests

To verify that all the fixed vulnerabilities are robustly handled, a dedicated security test suite has been implemented at `tests/integration/tests/defi_security_tests.rs`. This suite includes:

1. **`test_lending_pool_collateral_check_on_withdraw`** — Asserts that withdrawing collateral after borrowing below the threshold is rejected with `"insufficient collateral remaining"`.
2. **`test_prevent_reinitialization_staking_pool`** — Verifies that re-initializing the staking pool triggers `"already initialized"`.
3. **`test_prevent_reinitialization_swap_liquidity`** — Verifies that re-initializing swap liquidity triggers `"already initialized"`.
4. **`test_prevent_reinitialization_amm_pool_oracle`** — Verifies that re-initializing the AMM Pool triggers `"already initialized"`.
5. **`test_prevent_reinitialization_amm_oracle`** — Verifies that re-initializing the AMM Oracle triggers `"already initialized"`.
6. **`test_farming_pool_invalid_reward_rate_add`** — Verifies that a zero or negative reward rate on adding a farming pool triggers `"Invalid reward rate"`.
7. **`test_farming_pool_excessive_reward_rate_add`** — Verifies that an excessive reward rate triggers `"Invalid reward rate"`.
8. **`test_farming_pool_invalid_reward_rate_update`** — Verifies that updating a farming pool with a negative rate triggers `"Invalid reward rate"`.
9. **`test_amm_router_robustness_zero_reserves`** — Verifies that swapping on zero reserves triggers `"insufficient liquidity in pool"`.
10. **`test_collateralized_lending_liquidation_cap_and_adjust`** — Simulates a high-incentive liquidation and asserts that the borrower's collateral and debt are safely capped, preventing underflows.

---

## 6. Reviewer Sign-off

The security posture of the `Soroban-Cookbook` DeFi examples has been significantly enhanced through these mitigations. No critical or high-severity issues remain. All tests compile and pass flawlessly.

**Reviewer Sign-off:**
Jules, Lead Smart Contract Auditor
*Soroban-Cookbook Internal Security Review Team*
*March 2025*
