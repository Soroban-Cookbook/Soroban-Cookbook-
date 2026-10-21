//! # Reward Token
//!
//! Demonstrates a token contract with a multi-pool reward distribution system.
//! The admin can create reward pools, each with an independent rate. Token
//! holders accumulate proportional rewards and can claim them at any time.
//!
//! ## Storage layout
//!
//! | Key                     | Storage type | Description                         |
//! |-------------------------|--------------|-------------------------------------|
//! | `Admin`                 | instance     | Contract administrator              |
//! | `Name`                  | instance     | Token name                          |
//! | `Symbol`                | instance     | Token symbol                        |
//! | `Decimals`              | instance     | Token decimals                      |
//! | `PoolCount`             | instance     | Number of reward pools created      |
//! | `TotalSupply`           | instance     | Total token supply                  |
//! | `Pool(id)`              | instance     | `RewardPool` metadata               |
//! | `Balance(addr)`         | persistent   | Per-account token balance           |
//! | `Claimed(addr, pool)`   | persistent   | Lifetime rewards claimed by user    |
//!
//! ## Reward formula
//!
//! ```text
//! claimable = (balance * rate_per_token / 1_000_000) - already_claimed
//! ```
//!
//! `rate_per_token` is expressed in micro-units (1 = 0.000001 tokens reward
//! per token held), so a rate of `1_000_000` means 1:1 reward.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Name,
    Symbol,
    Decimals,
    PoolCount,
    TotalSupply,
    Pool(u32),
    Balance(Address),
    Claimed(Address, u32),
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A reward pool with an independent distribution rate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardPool {
    /// Unique pool identifier (auto-incremented from 0).
    pub id: u32,
    /// Reward tokens issued per token held, scaled by 1_000_000.
    /// E.g. 500_000 means 0.5 reward tokens per token held.
    pub rate_per_token: i128,
    /// Total tokens deposited into this pool as reward reserve.
    pub total_deposited: i128,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
    ArithmeticOverflow = 6,
    PoolNotFound = 7,
    NothingToClaim = 8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("rwd_tok");
const EV_MINT: Symbol = symbol_short!("mint");
const EV_BURN: Symbol = symbol_short!("burn");
const EV_XFER: Symbol = symbol_short!("transfer");
const EV_POOL: Symbol = symbol_short!("pool_new");
const EV_DEP: Symbol = symbol_short!("pool_dep");
const EV_CLAIM: Symbol = symbol_short!("claim");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct RewardToken;

#[contractimpl]
impl RewardToken {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialise the contract. Must be called exactly once.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<(), RewardError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RewardError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::PoolCount, &0u32);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Token operations
    // -----------------------------------------------------------------------

    /// Mint `amount` tokens to `to`. Admin-only.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), RewardError> {
        require_positive(amount)?;
        let admin = read_admin(&env)?;
        admin.require_auth();

        let supply = read_total_supply(&env);
        let new_supply = supply
            .checked_add(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let balance = read_balance(&env, &to);
        let new_balance = balance
            .checked_add(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);

        env.events().publish((NS, EV_MINT, to), amount);
        Ok(())
    }

    /// Burn `amount` tokens from `from`. The token holder must authorise.
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), RewardError> {
        require_positive(amount)?;
        from.require_auth();

        let balance = read_balance(&env, &from);
        if balance < amount {
            return Err(RewardError::InsufficientBalance);
        }

        let supply = read_total_supply(&env);
        let new_supply = supply
            .checked_sub(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let new_balance = balance
            .checked_sub(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_balance);

        env.events().publish((NS, EV_BURN, from), amount);
        Ok(())
    }

    /// Transfer `amount` tokens from `from` to `to`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), RewardError> {
        require_positive(amount)?;
        from.require_auth();

        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(RewardError::InsufficientBalance);
        }

        let to_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let new_from_balance = from_balance
            .checked_sub(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_from_balance);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_balance);

        env.events().publish((NS, EV_XFER, from, to), amount);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Token queries
    // -----------------------------------------------------------------------

    /// Return the token balance of `user`.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    /// Return the current total supply.
    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    /// Return the current admin address.
    pub fn admin(env: Env) -> Result<Address, RewardError> {
        read_admin(&env)
    }

    // -----------------------------------------------------------------------
    // Reward pools – admin operations
    // -----------------------------------------------------------------------

    /// Create a new reward pool with the given `rate_per_token` (scaled ×
    /// 1_000_000). Returns the new pool's id. Admin-only.
    pub fn create_pool(env: Env, rate_per_token: i128) -> Result<u32, RewardError> {
        require_positive(rate_per_token)?;
        let admin = read_admin(&env)?;
        admin.require_auth();

        let pool_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PoolCount)
            .ok_or(RewardError::NotInitialized)?;

        let pool = RewardPool {
            id: pool_count,
            rate_per_token,
            total_deposited: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pool(pool_count), &pool);
        let next = pool_count
            .checked_add(1)
            .ok_or(RewardError::ArithmeticOverflow)?;
        env.storage().instance().set(&DataKey::PoolCount, &next);

        env.events()
            .publish((NS, EV_POOL, pool_count), rate_per_token);
        Ok(pool_count)
    }

    /// Deposit `amount` tokens into a pool's reward reserve. Admin-only.
    /// Tokens are sourced from the admin's own balance.
    pub fn deposit_to_pool(env: Env, pool_id: u32, amount: i128) -> Result<(), RewardError> {
        require_positive(amount)?;
        let admin = read_admin(&env)?;
        admin.require_auth();

        let mut pool = read_pool(&env, pool_id)?;

        // Debit admin balance.
        let admin_balance = read_balance(&env, &admin);
        if admin_balance < amount {
            return Err(RewardError::InsufficientBalance);
        }
        let new_admin_balance = admin_balance
            .checked_sub(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;

        pool.total_deposited = pool
            .total_deposited
            .checked_add(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(admin.clone()), &new_admin_balance);
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);

        env.events().publish((NS, EV_DEP, pool_id), amount);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Reward pools – queries
    // -----------------------------------------------------------------------

    /// Return metadata for the given pool.
    pub fn pool_info(env: Env, pool_id: u32) -> Result<RewardPool, RewardError> {
        read_pool(&env, pool_id)
    }

    /// Return how many reward tokens `user` can currently claim from `pool_id`.
    pub fn claimable_rewards(env: Env, user: Address, pool_id: u32) -> Result<i128, RewardError> {
        let pool = read_pool(&env, pool_id)?;
        let balance = read_balance(&env, &user);
        let gross = balance
            .checked_mul(pool.rate_per_token)
            .ok_or(RewardError::ArithmeticOverflow)?
            .checked_div(1_000_000)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let claimed = read_claimed(&env, &user, pool_id);
        Ok(gross.saturating_sub(claimed).max(0))
    }

    /// Claim rewards for `user` from `pool_id`. Requires user authorisation.
    /// Returns the amount of tokens transferred to the user.
    pub fn claim_rewards(env: Env, user: Address, pool_id: u32) -> Result<i128, RewardError> {
        user.require_auth();

        let mut pool = read_pool(&env, pool_id)?;
        let balance = read_balance(&env, &user);

        let gross = balance
            .checked_mul(pool.rate_per_token)
            .ok_or(RewardError::ArithmeticOverflow)?
            .checked_div(1_000_000)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let claimed = read_claimed(&env, &user, pool_id);
        let reward = gross.saturating_sub(claimed).max(0);

        if reward == 0 {
            return Err(RewardError::NothingToClaim);
        }

        if pool.total_deposited < reward {
            return Err(RewardError::InsufficientBalance);
        }

        // Credit user, deduct from pool reserve.
        let user_balance = read_balance(&env, &user);
        let new_user_balance = user_balance
            .checked_add(reward)
            .ok_or(RewardError::ArithmeticOverflow)?;
        pool.total_deposited = pool
            .total_deposited
            .checked_sub(reward)
            .ok_or(RewardError::ArithmeticOverflow)?;

        let new_claimed = claimed
            .checked_add(reward)
            .ok_or(RewardError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), &new_user_balance);
        env.storage()
            .persistent()
            .set(&DataKey::Claimed(user.clone(), pool_id), &new_claimed);
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);

        env.events().publish((NS, EV_CLAIM, user, pool_id), reward);
        Ok(reward)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<Address, RewardError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(RewardError::NotInitialized)
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

fn read_pool(env: &Env, pool_id: u32) -> Result<RewardPool, RewardError> {
    let pool_count: u32 = env
        .storage()
        .instance()
        .get(&DataKey::PoolCount)
        .ok_or(RewardError::NotInitialized)?;
    if pool_id >= pool_count {
        return Err(RewardError::PoolNotFound);
    }
    env.storage()
        .instance()
        .get(&DataKey::Pool(pool_id))
        .ok_or(RewardError::PoolNotFound)
}

fn read_claimed(env: &Env, user: &Address, pool_id: u32) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Claimed(user.clone(), pool_id))
        .unwrap_or(0)
}

fn require_positive(amount: i128) -> Result<(), RewardError> {
    if amount <= 0 {
        Err(RewardError::InvalidAmount)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test;
