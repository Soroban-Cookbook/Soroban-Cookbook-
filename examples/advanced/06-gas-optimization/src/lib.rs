#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

//! # Gas Optimization Patterns for Soroban
//!
//! This contract demonstrates 12 gas optimization techniques:
//! 1. Storage tier selection (Instance vs Persistent vs Temporary)
//! 2. Caching frequently accessed values
//! 3. Batch operations vs individual operations
//! 4. Symbol interning and short symbols
//! 5. Using enums instead of strings for state
//! 6. Minimizing storage reads per operation
//! 7. Lazy initialization
//! 8. Checked arithmetic vs unchecked
//! 9. Short-circuit evaluation
//! 10. Efficient error handling with typed errors
//! 11. Bitflags for boolean state packing
//! 12. Struct packing and layout optimization

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

/// DataKey enum for typed, efficient storage access.
///
/// Optimization 1: no explicit discriminants on tuple variants —
/// `#[contracttype]` does not support mixing explicit integer discriminants
/// with tuple variants.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Instance storage: contract-wide config
    Config,
    /// Persistent storage: per-user balance (survives upgrades)
    Balance(Address),
    /// Temporary storage: session cache keyed by session id
    SessionCache(u64),
    /// Benchmark key for storage performance tests
    Benchmark(u64),
}

/// Optimization 11: bitflags pack multiple booleans into a single `u32`,
/// which is one Soroban-compatible storage word.
#[contracttype]
#[derive(Clone)]
pub struct Config {
    /// Packed flags: bit 0 = paused, bit 1 = emergency_mode, bits 2-31 reserved
    pub flags: u32,
    /// Fee rate in basis points.  Uses `u32` because Soroban does not provide
    /// `TryFromVal` for `u16`; range enforcement happens in application logic.
    pub fee_bps: u32,
    /// Administrator address
    pub admin: Address,
}

impl Config {
    pub fn is_paused(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            self.flags |= 0x01;
        } else {
            self.flags &= !0x01;
        }
    }

    pub fn is_emergency(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn set_emergency(&mut self, emergency: bool) {
        if emergency {
            self.flags |= 0x02;
        } else {
            self.flags &= !0x02;
        }
    }
}

/// Optimization 10: typed errors via `#[contracterror]` are more efficient
/// than string panics and let callers pattern-match on specific failure modes.
///
/// Note: the type cannot be named `Error` because that conflicts with a
/// reserved name inside the Soroban SDK macros; use `ContractError` instead.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Paused = 1,
    EmergencyMode = 2,
    InsufficientBalance = 3,
    InvalidAmount = 4,
    Unauthorized = 5,
}

#[contract]
pub struct GasOptimizationContract;

/// Optimization 4: `symbol_short!` creates a compile-time `Symbol` constant
/// that is cheaper to compare and store than a heap-allocated string.
const CONFIG_KEY: Symbol = symbol_short!("cfg");

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Load config from instance storage, falling back to a zeroed default keyed
/// on the current contract address.  Centralising the load avoids repeated
/// multi-line `get(&CONFIG_KEY).unwrap_or(Config { … })` expressions.
fn load_config(env: &Env) -> Config {
    env.storage().instance().get(&CONFIG_KEY).unwrap_or(Config {
        flags: 0,
        fee_bps: 0,
        admin: env.current_contract_address(),
    })
}

fn save_config(env: &Env, config: &Config) {
    env.storage().instance().set(&CONFIG_KEY, config);
}

fn read_balance(env: &Env, account: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(account.clone()))
        .unwrap_or(0)
}

fn write_balance(env: &Env, account: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(account.clone()), &amount);
}

// ---------------------------------------------------------------------------
// Contract implementation
// ---------------------------------------------------------------------------

#[contractimpl]
impl GasOptimizationContract {
    /// Initialize contract config once.
    ///
    /// Optimization 7: lazy initialization — config is written exactly once;
    /// subsequent calls are rejected so callers only pay the write cost once.
    pub fn initialize(env: Env, admin: Address, fee_bps: u32) -> Result<(), ContractError> {
        if env.storage().instance().has(&CONFIG_KEY) {
            return Err(ContractError::Unauthorized);
        }
        save_config(
            &env,
            &Config {
                flags: 0,
                fee_bps,
                admin,
            },
        );
        Ok(())
    }

    /// Transfer tokens from `from` to `to`.
    ///
    /// Optimization 2 & 6: config is read once and cached in a local variable
    /// rather than issuing multiple individual storage reads throughout the
    /// function body.
    ///
    /// Optimization 9: short-circuit on `paused` / `emergency` before
    /// touching balance storage — zero balance-read gas on blocked calls.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let config = load_config(&env);

        if config.is_paused() {
            return Err(ContractError::Paused);
        }

        // Optimization 5: typed enum state — block transfers during emergency.
        if config.is_emergency() {
            return Err(ContractError::EmergencyMode);
        }

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Optimization 6: single read per account.
        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        // Optimization 8: checked arithmetic to catch overflow without panic.
        let new_from = from_balance
            .checked_sub(amount)
            .ok_or(ContractError::InvalidAmount)?;

        // Fee via integer arithmetic — no floating point.
        let fee = (amount * config.fee_bps as i128) / 10_000;
        let to_amount = amount
            .checked_sub(fee)
            .ok_or(ContractError::InvalidAmount)?;

        // Optimization 3 & 6: batch both balance writes together.
        write_balance(&env, &from, new_from);

        let new_to = read_balance(&env, &to)
            .checked_add(to_amount)
            .ok_or(ContractError::InvalidAmount)?;
        write_balance(&env, &to, new_to);

        Ok(())
    }

    /// Return the balance of `account`.
    ///
    /// Optimization 1: balances live in persistent storage so they survive
    /// contract upgrades without a migration step.
    pub fn get_balance(env: Env, account: Address) -> i128 {
        read_balance(&env, &account)
    }

    /// Return balances for multiple accounts in a single call.
    ///
    /// Optimization 6: batching queries reduces per-call overhead versus N
    /// individual cross-contract `get_balance` calls.
    pub fn get_balances(env: Env, accounts: Vec<Address>) -> Vec<i128> {
        let mut balances = Vec::new(&env);
        for account in accounts.iter() {
            balances.push_back(read_balance(&env, &account));
        }
        balances
    }

    /// Pause the contract (admin only).
    pub fn pause(env: Env) -> Result<(), ContractError> {
        let mut config = load_config(&env);
        config.admin.require_auth();
        config.set_paused(true);
        save_config(&env, &config);
        Ok(())
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let mut config = load_config(&env);
        config.admin.require_auth();
        config.set_paused(false);
        save_config(&env, &config);
        Ok(())
    }

    /// Enable or disable emergency mode (admin only).
    ///
    /// Optimization 5: state is stored as a bitflag rather than a string,
    /// making reads and comparisons significantly cheaper.
    pub fn set_emergency(env: Env, emergency: bool) -> Result<(), ContractError> {
        let mut config = load_config(&env);
        config.admin.require_auth();
        config.set_emergency(emergency);
        save_config(&env, &config);
        Ok(())
    }

    /// Mint tokens to multiple recipients in a single call.
    ///
    /// Optimization 3: batching writes reduces per-call invocation overhead
    /// compared with N individual mint calls.
    pub fn batch_mint(env: Env, recipients: Vec<(Address, i128)>) -> Result<(), ContractError> {
        let config = load_config(&env);
        config.admin.require_auth();

        for (recipient, amount) in recipients.iter() {
            if amount > 0 {
                let current = read_balance(&env, &recipient);
                let new_bal = current
                    .checked_add(amount)
                    .ok_or(ContractError::InvalidAmount)?;
                write_balance(&env, &recipient, new_bal);
            }
        }
        Ok(())
    }

    /// Burn tokens from multiple accounts in a single call.
    ///
    /// Optimization 3: same batching benefit as `batch_mint`.
    pub fn batch_burn(env: Env, accounts: Vec<(Address, i128)>) -> Result<(), ContractError> {
        let config = load_config(&env);
        config.admin.require_auth();

        for (account, amount) in accounts.iter() {
            let current = read_balance(&env, &account);
            if current < amount {
                return Err(ContractError::InsufficientBalance);
            }
            write_balance(&env, &account, current - amount);
        }
        Ok(())
    }

    /// Benchmark write operations across storage tiers.
    ///
    /// `storage` selects the tier:
    /// 0 = instance, 1 = persistent, 2 = temporary.
    /// Returns the number of entries written.
    pub fn benchmark_write(env: Env, storage: u32, count: u64) -> u64 {
        for i in 0..count {
            let key = DataKey::Benchmark(i);
            let value = i as i128;
            match storage {
                0 => env.storage().instance().set(&key, &value),
                1 => env.storage().persistent().set(&key, &value),
                _ => env.storage().temporary().set(&key, &value),
            }
        }
        count
    }

    /// Benchmark read operations across storage tiers.
    ///
    /// Reads `count` entries and returns a checksum so the compiler cannot
    /// elide the reads.
    pub fn benchmark_read(env: Env, storage: u32, count: u64) -> u64 {
        let mut checksum: u64 = 0;
        for i in 0..count {
            let key = DataKey::Benchmark(i);
            let value: i128 = match storage {
                0 => env.storage().instance().get(&key).unwrap_or(0),
                1 => env.storage().persistent().get(&key).unwrap_or(0),
                _ => env.storage().temporary().get(&key).unwrap_or(0),
            };
            checksum = checksum.wrapping_add(value as u64);
        }
        checksum
    }

    /// Benchmark iteration over a stored vector.
    ///
    /// Writes `count` i128 values into a single temporary-storage vector,
    /// reads it back, and returns the sum of all values.
    pub fn benchmark_iteration(env: Env, count: u64) -> u64 {
        let mut vec = Vec::new(&env);
        for i in 0..count {
            vec.push_back(i as i128);
        }
        let key = DataKey::SessionCache(u64::MAX);
        env.storage().temporary().set(&key, &vec);
        let loaded: Vec<i128> = env.storage().temporary().get(&key).unwrap();
        let mut sum: u64 = 0;
        for item in loaded.iter() {
            sum = sum.wrapping_add(item as u64);
        }
        sum
    }
}
