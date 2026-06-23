//! # Security Patterns for Flash Loan Receivers
//!
//! Demonstrates best practices every flash loan receiver must follow to avoid
//! losing funds or being exploited.
//!
//! ## Patterns
//! 1. **Validate caller** — only accept callbacks from the registered provider
//! 2. **Validate parameters** — amount must be positive
//! 3. **Approve only exact repayment** — never approve more than `amount + fee`
//! 4. **Short-lived approval** — expire allowance after this ledger sequence
//! 5. **Emit audit event** — log every callback for off-chain visibility
//!
//! ## Anti-patterns to avoid
//! - Never approve `i128::MAX` (unlimited spend)
//! - Never persist borrowed funds across transactions
//! - Never skip caller validation inside `on_flash_loan`

#![allow(unused_variables)]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Registered trusted flash loan provider
    Provider,
    Owner,
}

#[contract]
pub struct SecureReceiverContract;

#[contractimpl]
impl SecureReceiverContract {
    /// Register the single trusted flash loan provider.
    pub fn init(env: Env, owner: Address, provider: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Provider, &provider);
    }

    /// Update the trusted provider (admin-only).
    pub fn set_provider(env: Env, provider: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        env.storage().instance().set(&DataKey::Provider, &provider);
    }

    /// Flash loan callback with all security checks applied.
    ///
    /// # Security checks (in order)
    /// 1. `initiator` must match the registered `Provider`
    /// 2. `amount` must be positive and `fee` non-negative
    /// 3. Contract must hold ≥ `amount + fee` before approving
    /// 4. Approve **exactly** `amount + fee`, expiring after this sequence
    /// 5. Emit audit event
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        // Check 1: caller identity
        let provider: Address = env.storage().instance().get(&DataKey::Provider).unwrap();
        if initiator != provider {
            panic!("unauthorized provider");
        }

        // Check 2: parameter sanity
        if amount <= 0 || fee < 0 {
            panic!("invalid amount or fee");
        }

        // ----------------------------------------------------------------
        // Insert custom use-case logic here (arbitrage, liquidation, etc.)
        // ----------------------------------------------------------------

        let repay = amount + fee;
        let token_client = token::Client::new(&env, &token);
        let contract = env.current_contract_address();

        // Check 3: ensure repayment funds are available
        if token_client.balance(&contract) < repay {
            panic!("insufficient repayment funds");
        }

        // Check 4: approve **only** the exact repayment, expiring immediately
        token_client.approve(
            &contract,
            &initiator,
            &repay,
            &(env.ledger().sequence() + 1),
        );

        // Check 5: emit audit event
        env.events().publish(
            (symbol_short!("fl"), symbol_short!("callback")),
            (initiator, token, amount, fee),
        );
    }
}
