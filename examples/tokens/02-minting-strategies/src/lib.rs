//! Minting strategies contract demonstrating fixed-cap, unlimited, and scheduled issuance.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TotalSupply,
    SupplyCap,
    ScheduleStart,
    ScheduleInterval,
    ScheduleRate,
    ScheduledMinted,
    Balance(Address),
}

#[contracttype]
pub struct MintEventData {
    pub strategy: Symbol,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MintingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    SupplyCapExceeded = 5,
    InvalidCap = 6,
    InvalidSchedule = 7,
    ScheduleNotStarted = 8,
    ScheduleUnavailable = 9,
    Overflow = 10,
}

const EVENT_NAMESPACE: Symbol = symbol_short!("mint");
const STRATEGY_FIXED: Symbol = symbol_short!("fixed");
const STRATEGY_UNLIMITED: Symbol = symbol_short!("unlimited");
const STRATEGY_SCHEDULED: Symbol = symbol_short!("scheduled");

#[contract]
pub struct MintingStrategiesToken;

#[contractimpl]
impl MintingStrategiesToken {
    pub fn initialize(
        env: Env,
        admin: Address,
        supply_cap: i128,
        schedule_start: u64,
        schedule_interval: u64,
        schedule_rate: i128,
    ) -> Result<(), MintingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MintingError::AlreadyInitialized);
        }
        if supply_cap < 0 || schedule_interval == 0 || schedule_rate < 0 {
            return Err(MintingError::InvalidSchedule);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage().instance().set(&DataKey::SupplyCap, &supply_cap);
        env.storage().instance().set(&DataKey::ScheduleStart, &schedule_start);
        env.storage().instance().set(&DataKey::ScheduleInterval, &schedule_interval);
        env.storage().instance().set(&DataKey::ScheduleRate, &schedule_rate);
        env.storage()
            .instance()
            .set(&DataKey::ScheduledMinted, &0i128);

        Ok(())
    }

    pub fn mint_with_cap(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<i128, MintingError> {
        admin.require_auth();
        let stored_admin = read_admin(&env)?;
        if admin != stored_admin {
            return Err(MintingError::Unauthorized);
        }
        require_positive(amount)?;

        let cap = read_supply_cap(&env)?;
        if cap == 0 {
            return Err(MintingError::InvalidCap);
        }

        let total_supply = read_total_supply(&env);
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;
        if new_supply > cap {
            return Err(MintingError::SupplyCapExceeded);
        }

        let new_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        publish_mint(&env, STRATEGY_FIXED, to, amount);
        Ok(new_balance)
    }

    pub fn mint_unlimited(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<i128, MintingError> {
        admin.require_auth();
        let stored_admin = read_admin(&env)?;
        if admin != stored_admin {
            return Err(MintingError::Unauthorized);
        }
        require_positive(amount)?;

        let new_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;
        let total_supply = read_total_supply(&env);
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        publish_mint(&env, STRATEGY_UNLIMITED, to, amount);
        Ok(new_balance)
    }

    pub fn mint_scheduled(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<i128, MintingError> {
        admin.require_auth();
        let stored_admin = read_admin(&env)?;
        if admin != stored_admin {
            return Err(MintingError::Unauthorized);
        }
        require_positive(amount)?;

        let available = read_scheduled_available(&env)?;
        if available < amount {
            return Err(MintingError::ScheduleUnavailable);
        }

        let total_supply = read_total_supply(&env);
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;
        let cap = read_supply_cap(&env)?;
        if cap > 0 && new_supply > cap {
            return Err(MintingError::SupplyCapExceeded);
        }

        let new_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;
        let new_scheduled = read_scheduled_minted(&env)
            .checked_add(amount)
            .ok_or(MintingError::Overflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.storage()
            .instance()
            .set(&DataKey::ScheduledMinted, &new_scheduled);
        publish_mint(&env, STRATEGY_SCHEDULED, to, amount);
        Ok(new_balance)
    }

    pub fn scheduled_available(env: Env) -> Result<i128, MintingError> {
        read_scheduled_available(&env)
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn supply_cap(env: Env) -> Option<i128> {
        let cap = env
            .storage()
            .instance()
            .get(&DataKey::SupplyCap)
            .unwrap_or(0);
        if cap == 0 {
            None
        } else {
            Some(cap)
        }
    }

    pub fn admin(env: Env) -> Result<Address, MintingError> {
        read_admin(&env)
    }
}

fn require_positive(amount: i128) -> Result<(), MintingError> {
    if amount <= 0 {
        Err(MintingError::InvalidAmount)
    } else {
        Ok(())
    }
}

fn read_admin(env: &Env) -> Result<Address, MintingError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(MintingError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_supply_cap(env: &Env) -> Result<i128, MintingError> {
    Ok(env
        .storage()
        .instance()
        .get(&DataKey::SupplyCap)
        .unwrap_or(0))
}

fn read_schedule_start(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ScheduleStart)
        .unwrap_or(0)
}

fn read_schedule_interval(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ScheduleInterval)
        .unwrap_or(0)
}

fn read_schedule_rate(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::ScheduleRate)
        .unwrap_or(0)
}

fn read_scheduled_minted(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::ScheduledMinted)
        .unwrap_or(0)
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn read_scheduled_available(env: &Env) -> Result<i128, MintingError> {
    let start = read_schedule_start(env);
    let interval = read_schedule_interval(env);
    let rate = read_schedule_rate(env);

    if interval == 0 {
        return Err(MintingError::InvalidSchedule);
    }
    if rate <= 0 {
        return Err(MintingError::InvalidSchedule);
    }

    let now = env.ledger().timestamp();
    if now < start {
        return Err(MintingError::ScheduleNotStarted);
    }

    let elapsed = now - start;
    let periods = elapsed / interval;
    let total_available = (periods as i128)
        .checked_mul(rate)
        .ok_or(MintingError::Overflow)?;
    let minted = read_scheduled_minted(env);

    if minted >= total_available {
        Ok(0)
    } else {
        Ok(total_available - minted)
    }
}

fn publish_mint(env: &Env, strategy: Symbol, to: Address, amount: i128) {
    env.events().publish(
        (EVENT_NAMESPACE, strategy, to.clone()),
        MintEventData {
            strategy,
            amount,
            timestamp: env.ledger().timestamp(),
        },
    );
}

#[cfg(test)]
mod test;
