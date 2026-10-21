//! # Rate Limiting
//!
//! A reusable fixed-window rate limiter combining three kinds of caps:
//!
//! - **Time-based**: usage is tracked per window of `window` seconds and resets
//!   once the window elapses.
//! - **Amount-based**: the cumulative `amount` consumed within a window is capped.
//! - **Per-user**: every caller has independent usage, and an admin can grant an
//!   individual caller a limit that overrides the contract-wide default.
//!
//! The limiter stores no funds. `consume` is the guard you call at the top of a
//! rate-limited operation (withdrawal, mint, bridge transfer, faucet drip); it
//! records the usage and errors when a cap would be exceeded.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RateLimitError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// A limit field was zero or negative.
    InvalidLimit = 3,
    /// The consumed amount was zero or negative.
    InvalidAmount = 4,
    /// The caller has used up its calls for the current window.
    CallLimitExceeded = 5,
    /// The call would push the caller past its amount cap for the current window.
    AmountLimitExceeded = 6,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The caps applied to a single caller for one window.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limit {
    /// Window length in seconds. Usage resets once a window elapses.
    pub window: u64,
    /// Maximum number of `consume` calls allowed per window.
    pub max_calls: u32,
    /// Maximum cumulative amount allowed per window.
    pub max_amount: i128,
}

/// Usage accumulated by one caller inside the current window.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Usage {
    /// Ledger timestamp at which the current window opened.
    pub window_start: u64,
    /// Calls made since `window_start`.
    pub calls: u32,
    /// Amount consumed since `window_start`.
    pub amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The admin who can change limits and reset usage.
    Admin,
    /// The contract-wide default limit.
    DefaultLimit,
    /// A per-user limit that overrides the default.
    UserLimit(Address),
    /// Recorded usage for one user.
    Usage(Address),
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const CONTRACT_NS: Symbol = symbol_short!("ratelimit");
const ACTION_CONSUME: Symbol = symbol_short!("consume");
const ACTION_ADMIN: Symbol = symbol_short!("admin");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct RateLimiterContract;

#[contractimpl]
impl RateLimiterContract {
    /// Initialize with an admin and the default limit applied to every caller.
    pub fn initialize(
        env: Env,
        admin: Address,
        default_limit: Limit,
    ) -> Result<(), RateLimitError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RateLimitError::AlreadyInitialized);
        }
        validate_limit(&default_limit)?;

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::DefaultLimit, &default_limit);
        Ok(())
    }

    /// Replace the contract-wide default limit. Admin only.
    pub fn set_default_limit(env: Env, limit: Limit) -> Result<(), RateLimitError> {
        let admin = Self::require_admin(&env)?;
        validate_limit(&limit)?;

        env.storage().instance().set(&DataKey::DefaultLimit, &limit);
        env.events()
            .publish((CONTRACT_NS, ACTION_ADMIN, admin), symbol_short!("default"));
        Ok(())
    }

    /// Give `user` a limit that overrides the default. Admin only.
    pub fn set_user_limit(env: Env, user: Address, limit: Limit) -> Result<(), RateLimitError> {
        let admin = Self::require_admin(&env)?;
        validate_limit(&limit)?;

        env.storage()
            .persistent()
            .set(&DataKey::UserLimit(user.clone()), &limit);
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, user),
            symbol_short!("set_user"),
        );
        Ok(())
    }

    /// Drop `user`'s override so the default applies again. Admin only.
    pub fn clear_user_limit(env: Env, user: Address) -> Result<(), RateLimitError> {
        let admin = Self::require_admin(&env)?;

        env.storage()
            .persistent()
            .remove(&DataKey::UserLimit(user.clone()));
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, user),
            symbol_short!("clr_user"),
        );
        Ok(())
    }

    /// Consume one call and `amount` of the caller's budget for this window.
    ///
    /// Call this at the top of any operation you want rate limited. It errors
    /// without recording anything when a cap would be exceeded.
    pub fn consume(env: Env, user: Address, amount: i128) -> Result<(), RateLimitError> {
        user.require_auth();

        if amount <= 0 {
            return Err(RateLimitError::InvalidAmount);
        }

        let limit = Self::limit_of(env.clone(), user.clone())?;
        let mut usage = current_usage(&env, &user, &limit);

        if usage.calls >= limit.max_calls {
            return Err(RateLimitError::CallLimitExceeded);
        }
        let new_amount = usage
            .amount
            .checked_add(amount)
            .ok_or(RateLimitError::AmountLimitExceeded)?;
        if new_amount > limit.max_amount {
            return Err(RateLimitError::AmountLimitExceeded);
        }

        usage.calls += 1;
        usage.amount = new_amount;
        env.storage()
            .persistent()
            .set(&DataKey::Usage(user.clone()), &usage);

        env.events()
            .publish((CONTRACT_NS, ACTION_CONSUME, user), (amount, usage.calls));
        Ok(())
    }

    /// The limit that applies to `user`: their override, else the default.
    pub fn limit_of(env: Env, user: Address) -> Result<Limit, RateLimitError> {
        if let Some(limit) = env.storage().persistent().get(&DataKey::UserLimit(user)) {
            return Ok(limit);
        }
        env.storage()
            .instance()
            .get(&DataKey::DefaultLimit)
            .ok_or(RateLimitError::NotInitialized)
    }

    /// Usage for `user` in the current window, already rolled over if the
    /// previous window has elapsed.
    pub fn usage_of(env: Env, user: Address) -> Result<Usage, RateLimitError> {
        let limit = Self::limit_of(env.clone(), user.clone())?;
        Ok(current_usage(&env, &user, &limit))
    }

    /// Calls `user` may still make in the current window.
    pub fn remaining_calls(env: Env, user: Address) -> Result<u32, RateLimitError> {
        let limit = Self::limit_of(env.clone(), user.clone())?;
        let usage = current_usage(&env, &user, &limit);
        Ok(limit.max_calls.saturating_sub(usage.calls))
    }

    /// Amount `user` may still consume in the current window.
    pub fn remaining_amount(env: Env, user: Address) -> Result<i128, RateLimitError> {
        let limit = Self::limit_of(env.clone(), user.clone())?;
        let usage = current_usage(&env, &user, &limit);
        Ok(limit.max_amount.saturating_sub(usage.amount))
    }

    /// Timestamp at which `user`'s current window closes and usage resets.
    pub fn window_reset_at(env: Env, user: Address) -> Result<u64, RateLimitError> {
        let limit = Self::limit_of(env.clone(), user.clone())?;
        let usage = current_usage(&env, &user, &limit);
        Ok(usage.window_start.saturating_add(limit.window))
    }

    /// Clear `user`'s recorded usage immediately. Admin only.
    pub fn reset(env: Env, user: Address) -> Result<(), RateLimitError> {
        let admin = Self::require_admin(&env)?;

        env.storage()
            .persistent()
            .remove(&DataKey::Usage(user.clone()));
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, user),
            symbol_short!("reset"),
        );
        Ok(())
    }

    fn require_admin(env: &Env) -> Result<Address, RateLimitError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RateLimitError::NotInitialized)?;
        admin.require_auth();
        Ok(admin)
    }
}

/// Reject limits that would make the contract unusable or meaningless.
fn validate_limit(limit: &Limit) -> Result<(), RateLimitError> {
    if limit.window == 0 || limit.max_calls == 0 || limit.max_amount <= 0 {
        return Err(RateLimitError::InvalidLimit);
    }
    Ok(())
}

/// Load `user`'s usage, starting a fresh window when the stored one has elapsed.
///
/// Windows are anchored to the first consume rather than to fixed wall-clock
/// boundaries, so a caller cannot double their budget by straddling a boundary.
fn current_usage(env: &Env, user: &Address, limit: &Limit) -> Usage {
    let now = env.ledger().timestamp();
    let stored: Option<Usage> = env
        .storage()
        .persistent()
        .get(&DataKey::Usage(user.clone()));

    match stored {
        Some(usage) if now < usage.window_start.saturating_add(limit.window) => usage,
        _ => Usage {
            window_start: now,
            calls: 0,
            amount: 0,
        },
    }
}

#[cfg(test)]
mod test;
