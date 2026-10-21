#![allow(deprecated)]
//! # Snapshot Token Example
//!
//! A Soroban fungible token contract that implements balance snapshot tracking.
//! It allows querying an account's balance (and total supply) at a specific historical snapshot ID.
//! This is extremely useful for governance/voting systems to prevent flash loan attacks or
//! timing-based manipulation by pinning voting weight to a pre-announced snapshot.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, String,
    Symbol,
};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEvent {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintEvent {
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BurnEvent {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotCreatedEvent {
    pub snapshot_id: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TotalSupply,
    Name,
    Symbol,
    Decimals,
    Balance(Address),
    SnapshotCounter,
    SnapshotHistory(Address), // Vec<(u32, i128)>: stores (snapshot_id, pre_change_balance)
    SupplyHistory,            // Vec<(u32, i128)>: stores (snapshot_id, pre_change_supply)
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SnapshotTokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    InvalidAmount = 5,
    ArithmeticOverflow = 6,
    SnapshotNotFound = 7,
}

#[contract]
pub struct SnapshotToken;

#[contractimpl]
impl SnapshotToken {
    /// Initialize the snapshot token contract.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: Symbol,
        decimals: u32,
    ) -> Result<(), SnapshotTokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SnapshotTokenError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::SnapshotCounter, &0u32);

        Ok(())
    }

    /// Mint tokens to a specific account. Only the admin can call this.
    pub fn mint(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<i128, SnapshotTokenError> {
        admin.require_auth();
        let stored_admin = read_admin(&env)?;
        if admin != stored_admin {
            return Err(SnapshotTokenError::Unauthorized);
        }
        require_positive(amount)?;

        let current_snapshot = read_snapshot_counter(&env);
        let to_balance = read_balance(&env, &to);
        let total_supply = read_total_supply(&env);

        // Record pre-change states before updating balances
        record_balance_snapshot(&env, &to, current_snapshot, to_balance);
        record_supply_snapshot(&env, current_snapshot, total_supply);

        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(SnapshotTokenError::ArithmeticOverflow)?;
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(SnapshotTokenError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        MintEvent { to, amount }.publish(&env);
        Ok(new_to_balance)
    }

    /// Transfer tokens from one account to another.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), SnapshotTokenError> {
        from.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let current_snapshot = read_snapshot_counter(&env);
        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(SnapshotTokenError::InsufficientBalance);
        }

        let to_balance = read_balance(&env, &to);

        // Record pre-change states before updating balances
        record_balance_snapshot(&env, &from, current_snapshot, from_balance);
        record_balance_snapshot(&env, &to, current_snapshot, to_balance);

        let new_from_balance = from_balance - amount;
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(SnapshotTokenError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_from_balance);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);

        TransferEvent { from, to, amount }.publish(&env);
        Ok(())
    }

    /// Burn tokens from the caller's account.
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<i128, SnapshotTokenError> {
        from.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let current_snapshot = read_snapshot_counter(&env);
        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(SnapshotTokenError::InsufficientBalance);
        }

        let total_supply = read_total_supply(&env);

        // Record pre-change states before updating balances
        record_balance_snapshot(&env, &from, current_snapshot, from_balance);
        record_supply_snapshot(&env, current_snapshot, total_supply);

        let new_from_balance = from_balance - amount;
        let new_supply = total_supply - amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_from_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        BurnEvent { from, amount }.publish(&env);
        Ok(new_from_balance)
    }

    /// Create a new snapshot. Only the admin can call this.
    /// Returns the new snapshot ID.
    pub fn create_snapshot(env: Env, admin: Address) -> Result<u32, SnapshotTokenError> {
        admin.require_auth();
        let stored_admin = read_admin(&env)?;
        if admin != stored_admin {
            return Err(SnapshotTokenError::Unauthorized);
        }

        let current = read_snapshot_counter(&env);
        let next = current + 1;
        env.storage()
            .instance()
            .set(&DataKey::SnapshotCounter, &next);

        SnapshotCreatedEvent { snapshot_id: next }.publish(&env);

        Ok(next)
    }

    /// Returns the current snapshot counter ID.
    pub fn current_snapshot(env: Env) -> Result<u32, SnapshotTokenError> {
        ensure_initialized(&env)?;
        Ok(read_snapshot_counter(&env))
    }

    /// Returns the total snapshots taken.
    pub fn total_snapshots(env: Env) -> Result<u32, SnapshotTokenError> {
        ensure_initialized(&env)?;
        Ok(read_snapshot_counter(&env))
    }

    /// Returns the account's balance as of a specific snapshot ID.
    pub fn balance_at_snapshot(
        env: Env,
        account: Address,
        snapshot_id: u32,
    ) -> Result<i128, SnapshotTokenError> {
        ensure_initialized(&env)?;
        let current = read_snapshot_counter(&env);
        if snapshot_id == 0 || snapshot_id > current {
            return Err(SnapshotTokenError::SnapshotNotFound);
        }

        let key = DataKey::SnapshotHistory(account.clone());
        let history_opt: Option<soroban_sdk::Vec<(u32, i128)>> =
            env.storage().persistent().get(&key);

        if let Some(history) = history_opt {
            // Find the first entry where recorded_id >= snapshot_id
            for entry in history.iter() {
                let (id, bal) = entry;
                if id >= snapshot_id {
                    return Ok(bal);
                }
            }
        }

        // If no entry exists with ID >= snapshot_id, it means the balance has not changed
        // since snapshot_id was created. Thus, the balance is the current balance.
        Ok(read_balance(&env, &account))
    }

    /// Returns the total supply as of a specific snapshot ID.
    pub fn total_supply_at_snapshot(
        env: Env,
        snapshot_id: u32,
    ) -> Result<i128, SnapshotTokenError> {
        ensure_initialized(&env)?;
        let current = read_snapshot_counter(&env);
        if snapshot_id == 0 || snapshot_id > current {
            return Err(SnapshotTokenError::SnapshotNotFound);
        }

        let key = DataKey::SupplyHistory;
        let history_opt: Option<soroban_sdk::Vec<(u32, i128)>> =
            env.storage().persistent().get(&key);

        if let Some(history) = history_opt {
            // Find the first entry where recorded_id >= snapshot_id
            for entry in history.iter() {
                let (id, supply) = entry;
                if id >= snapshot_id {
                    return Ok(supply);
                }
            }
        }

        Ok(read_total_supply(&env))
    }

    /// Return the current balance of an account.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    /// Return the current total supply.
    pub fn total_supply(env: Env) -> Result<i128, SnapshotTokenError> {
        ensure_initialized(&env)?;
        Ok(read_total_supply(&env))
    }

    /// Return the token name.
    pub fn name(env: Env) -> Result<String, SnapshotTokenError> {
        read_name(&env)
    }

    /// Return the token symbol.
    pub fn symbol(env: Env) -> Result<Symbol, SnapshotTokenError> {
        read_symbol(&env)
    }

    /// Return the token decimals.
    pub fn decimals(env: Env) -> Result<u32, SnapshotTokenError> {
        read_decimals(&env)
    }

    /// Return the admin address.
    pub fn admin(env: Env) -> Result<Address, SnapshotTokenError> {
        read_admin(&env)
    }
}

// --- Helper Functions ---

fn require_positive(amount: i128) -> Result<(), SnapshotTokenError> {
    if amount <= 0 {
        return Err(SnapshotTokenError::InvalidAmount);
    }
    Ok(())
}

fn ensure_initialized(env: &Env) -> Result<(), SnapshotTokenError> {
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(SnapshotTokenError::NotInitialized)
    }
}

fn read_admin(env: &Env) -> Result<Address, SnapshotTokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(SnapshotTokenError::NotInitialized)
}

fn read_name(env: &Env) -> Result<String, SnapshotTokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Name)
        .ok_or(SnapshotTokenError::NotInitialized)
}

fn read_symbol(env: &Env) -> Result<Symbol, SnapshotTokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Symbol)
        .ok_or(SnapshotTokenError::NotInitialized)
}

fn read_decimals(env: &Env) -> Result<u32, SnapshotTokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Decimals)
        .ok_or(SnapshotTokenError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn read_snapshot_counter(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::SnapshotCounter)
        .unwrap_or(0)
}

fn record_balance_snapshot(
    env: &Env,
    account: &Address,
    current_snapshot: u32,
    pre_change_balance: i128,
) {
    if current_snapshot == 0 {
        return;
    }
    let key = DataKey::SnapshotHistory(account.clone());
    let mut history: soroban_sdk::Vec<(u32, i128)> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| soroban_sdk::Vec::new(env));

    let has_current = if history.is_empty() {
        false
    } else {
        let (last_id, _) = history.get(history.len() - 1).unwrap();
        last_id == current_snapshot
    };

    if !has_current {
        history.push_back((current_snapshot, pre_change_balance));
        env.storage().persistent().set(&key, &history);
    }
}

fn record_supply_snapshot(env: &Env, current_snapshot: u32, pre_change_supply: i128) {
    if current_snapshot == 0 {
        return;
    }
    let key = DataKey::SupplyHistory;
    let mut history: soroban_sdk::Vec<(u32, i128)> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| soroban_sdk::Vec::new(env));

    let has_current = if history.is_empty() {
        false
    } else {
        let (last_id, _) = history.get(history.len() - 1).unwrap();
        last_id == current_snapshot
    };

    if !has_current {
        history.push_back((current_snapshot, pre_change_supply));
        env.storage().persistent().set(&key, &history);
    }
}

#[cfg(test)]
mod test;
