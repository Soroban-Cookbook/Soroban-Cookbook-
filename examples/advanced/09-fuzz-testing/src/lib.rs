#![allow(deprecated)]
//! # Fuzz Testing for Advanced Contracts
//!
//! Demonstrates how to structure a Soroban contract so it can be fuzzed with
//! `cargo-fuzz` and property-tested with `proptest`. Builds on the timelock
//! / claimable-balance pattern from the advanced examples.
//!
//! ## What it shows
//!
//! - Depositing tokens into a claimable balance with a time bound
//! - Claiming (partially or fully) after the time predicate holds
//! - Invariant checks that remain true for *any* fuzzed deposit/claim amounts
//!
//! ## How to run
//!
//! Property tests (no nightly required):
//! ```bash
//! cargo test -p fuzz-testing
//! ```
//!
//! Continuous fuzzing (requires nightly + `cargo-fuzz`):
//! ```bash
//! cargo +nightly fuzz run --fuzz-dir tests/fuzz advanced_claimable_balance
//! ```
//!
//! See also: `guides/fuzz-testing.md`.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Init,
    Balance,
}

#[derive(Clone, Debug)]
#[contracttype]
pub enum TimeBoundKind {
    Before,
    After,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct TimeBound {
    pub kind: TimeBoundKind,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ClaimableBalance {
    pub token: Address,
    pub amount: i128,
    pub claimants: Vec<Address>,
    pub time_bound: TimeBound,
}

#[contract]
pub struct ClaimableBalanceContract;

fn check_time_bound(env: &Env, time_bound: &TimeBound) -> bool {
    let ledger_timestamp = env.ledger().timestamp();
    match time_bound.kind {
        TimeBoundKind::Before => ledger_timestamp <= time_bound.timestamp,
        TimeBoundKind::After => ledger_timestamp >= time_bound.timestamp,
    }
}

fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Init)
}

#[contractimpl]
impl ClaimableBalanceContract {
    /// Deposit `amount` of `token` and allow any of `claimants` to claim it
    /// when `time_bound` is satisfied.
    pub fn deposit(
        env: Env,
        from: Address,
        token: Address,
        amount: i128,
        claimants: Vec<Address>,
        time_bound: TimeBound,
    ) {
        if amount <= 0 {
            panic!("deposit must be positive");
        }
        if claimants.is_empty() {
            panic!("need more than 0 claimants");
        }
        if claimants.len() > 10 {
            panic!("too many claimants");
        }
        if is_initialized(&env) {
            panic!("contract has been already initialized");
        }

        from.require_auth();

        token::Client::new(&env, &token).transfer(&from, env.current_contract_address(), &amount);

        env.storage().persistent().set(
            &DataKey::Balance,
            &ClaimableBalance {
                token,
                amount,
                time_bound,
                claimants,
            },
        );
        env.storage().persistent().set(&DataKey::Init, &());
    }

    /// Claim up to `amount` tokens. Fails if the time bound is not met, the
    /// caller is not a claimant, or `amount` exceeds the remaining balance.
    pub fn claim(env: Env, claimant: Address, amount: i128) {
        claimant.require_auth();

        let mut claimable_balance: ClaimableBalance = env
            .storage()
            .persistent()
            .get(&DataKey::Balance)
            .expect("no claimable balance");

        if !check_time_bound(&env, &claimable_balance.time_bound) {
            panic!("time predicate is not fulfilled");
        }
        if !claimable_balance.claimants.contains(&claimant) {
            panic!("claimant is not allowed to claim this balance");
        }
        if amount <= 0 {
            panic!("claim must be positive");
        }
        if amount > claimable_balance.amount {
            panic!("claimed amount greater than balance");
        }

        token::Client::new(&env, &claimable_balance.token).transfer(
            &env.current_contract_address(),
            &claimant,
            &amount,
        );

        let new_balance = claimable_balance.amount - amount;
        if new_balance > 0 {
            claimable_balance.amount = new_balance;
            env.storage()
                .persistent()
                .set(&DataKey::Balance, &claimable_balance);
        } else {
            env.storage().persistent().remove(&DataKey::Balance);
        }
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod proptest;
