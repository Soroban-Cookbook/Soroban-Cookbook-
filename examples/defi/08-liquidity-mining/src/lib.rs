//! # Liquidity Mining / Farming Contract
//!
//! A multi-pool liquidity mining contract where users stake LP tokens and earn
//! reward tokens over time. Supports multiple independent pools, per-pool reward
//! rate adjustment by the admin, and a standard "harvest" flow.
//!
//! ## Architecture
//!
//! ```
//! Admin
//!  ├── add_pool(pool_id, lp_token, reward_token, reward_rate)
//!  └── set_reward_rate(pool_id, new_rate)
//!
//! User
//!  ├── stake(pool_id, amount)       — deposit LP tokens
//!  ├── unstake(pool_id, amount)     — withdraw LP tokens
//!  └── harvest(pool_id)             — claim accumulated reward tokens
//! ```
//!
//! ## Reward Maths
//!
//! The contract uses the standard "reward-per-share accumulator" pattern:
//!
//! ```
//! acc_reward_per_share += elapsed_ledgers * reward_rate / total_staked
//! user_pending          = user_staked * acc_reward_per_share - user_reward_debt
//! ```
//!
//! This gives O(1) updates regardless of the number of stakers.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

// ─── Storage keys ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract administrator
    Admin,
    /// Pool configuration keyed by pool id (u32)
    Pool(u32),
    /// Per-user staking state: (pool_id, user_address)
    UserInfo(u32, Address),
}

// ─── Data structures ─────────────────────────────────────────────────────────

/// Configuration and live state for a single mining pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PoolInfo {
    /// The LP token users stake into this pool
    pub lp_token: Address,
    /// The token distributed as a reward
    pub reward_token: Address,
    /// Reward tokens emitted per ledger, scaled by PRECISION
    pub reward_rate: i128,
    /// Total LP tokens currently staked in this pool
    pub total_staked: i128,
    /// Accumulated reward per staked LP token, scaled by PRECISION
    pub acc_reward_per_share: i128,
    /// Ledger sequence number of the last pool update
    pub last_update_ledger: u32,
    /// Whether this pool is active (accepting stakes / distributing rewards)
    pub active: bool,
}

/// Per-user staking record for a specific pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UserInfo {
    /// Amount of LP tokens the user has staked
    pub staked: i128,
    /// Snapshot of acc_reward_per_share at the time of last interaction,
    /// used to compute pending rewards without iterating over all ledgers.
    pub reward_debt: i128,
    /// Rewards that have been earned but not yet harvested
    pub pending_rewards: i128,
}

// ─── Constants ───────────────────────────────────────────────────────────────

/// Fixed-point precision multiplier (1e12).
/// Keeps integer arithmetic accurate when total_staked is large.
const PRECISION: i128 = 1_000_000_000_000;

/// Maximum reward rate per ledger (prevents overflow / runaway inflation).
const MAX_REWARD_RATE: i128 = 1_000_000_000_000_000; // 1e15

/// Minimum TTL extension for persistent storage entries (in ledgers).
const TTL_MIN: u32 = 17_280; // ~1 day at 5 s/ledger
/// Maximum TTL extension for persistent storage entries (in ledgers).
const TTL_MAX: u32 = 120_960; // ~7 days

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct LiquidityMining;

#[contractimpl]
impl LiquidityMining {
    // ── Admin ────────────────────────────────────────────────────────────────

    /// Initialize the contract with an admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.events()
            .publish((Symbol::new(&env, "initialized"),), admin);
    }

    /// Create a new mining pool.
    ///
    /// - `pool_id`     — caller-chosen unique identifier (u32)
    /// - `lp_token`    — address of the LP token users will stake
    /// - `reward_token`— address of the token distributed as rewards
    /// - `reward_rate` — reward tokens per ledger (scaled by PRECISION internally)
    pub fn add_pool(
        env: Env,
        pool_id: u32,
        lp_token: Address,
        reward_token: Address,
        reward_rate: i128,
    ) {
        Self::require_admin(&env);

        if env.storage().persistent().has(&DataKey::Pool(pool_id)) {
            panic!("Pool already exists");
        }
        if reward_rate <= 0 || reward_rate > MAX_REWARD_RATE {
            panic!("Invalid reward rate");
        }

        let pool = PoolInfo {
            lp_token,
            reward_token,
            reward_rate,
            total_staked: 0,
            acc_reward_per_share: 0,
            last_update_ledger: env.ledger().sequence(),
            active: true,
        };

        Self::save_pool(&env, pool_id, &pool);

        env.events()
            .publish((Symbol::new(&env, "pool_added"),), (pool_id, reward_rate));
    }

    /// Adjust the reward rate for an existing pool.
    /// Triggers an accumulator update before changing the rate so that
    /// rewards earned at the old rate are correctly accounted for.
    pub fn set_reward_rate(env: Env, pool_id: u32, new_rate: i128) {
        Self::require_admin(&env);

        if new_rate <= 0 || new_rate > MAX_REWARD_RATE {
            panic!("Invalid reward rate");
        }

        let mut pool = Self::load_pool(&env, pool_id);
        // Settle rewards at the current rate before changing it
        Self::update_pool(&env, &mut pool);
        pool.reward_rate = new_rate;
        Self::save_pool(&env, pool_id, &pool);

        env.events()
            .publish((Symbol::new(&env, "rate_changed"),), (pool_id, new_rate));
    }

    /// Pause or resume a pool.
    /// When paused, staking is disabled and the accumulator stops advancing.
    pub fn set_pool_active(env: Env, pool_id: u32, active: bool) {
        Self::require_admin(&env);
        let mut pool = Self::load_pool(&env, pool_id);
        // Settle before toggling so no rewards are lost
        Self::update_pool(&env, &mut pool);
        pool.active = active;
        Self::save_pool(&env, pool_id, &pool);

        env.events()
            .publish((Symbol::new(&env, "pool_status"),), (pool_id, active));
    }

    // ── User actions ─────────────────────────────────────────────────────────

    /// Stake `amount` LP tokens into `pool_id`.
    ///
    /// Transfers LP tokens from the caller to this contract and updates the
    /// reward accumulator so the user starts earning from this ledger onward.
    pub fn stake(env: Env, pool_id: u32, user: Address, amount: i128) {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let mut pool = Self::load_pool(&env, pool_id);
        if !pool.active {
            panic!("Pool is not active");
        }

        // Settle global accumulator
        Self::update_pool(&env, &mut pool);

        // Settle user's pending rewards before changing their stake
        let mut info = Self::load_user(&env, pool_id, &user);
        let pending = Self::calc_pending(&pool, &info);
        info.pending_rewards = info.pending_rewards.checked_add(pending).expect("Overflow");

        // Pull LP tokens from user
        let lp_client = token::Client::new(&env, &pool.lp_token);
        lp_client.transfer(&user, env.current_contract_address(), &amount);

        // Update state
        info.staked = info.staked.checked_add(amount).expect("Overflow");
        info.reward_debt = info
            .staked
            .checked_mul(pool.acc_reward_per_share)
            .expect("Overflow")
            / PRECISION;

        pool.total_staked = pool.total_staked.checked_add(amount).expect("Overflow");

        Self::save_pool(&env, pool_id, &pool);
        Self::save_user(&env, pool_id, &user, &info);

        env.events()
            .publish((Symbol::new(&env, "staked"),), (pool_id, user, amount));
    }

    /// Withdraw `amount` LP tokens from `pool_id`.
    ///
    /// Pending rewards are accumulated but NOT automatically sent — call
    /// `harvest` to claim them.
    pub fn unstake(env: Env, pool_id: u32, user: Address, amount: i128) {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let mut pool = Self::load_pool(&env, pool_id);
        let mut info = Self::load_user(&env, pool_id, &user);

        if info.staked < amount {
            panic!("Insufficient staked balance");
        }

        // Settle global accumulator
        Self::update_pool(&env, &mut pool);

        // Accumulate pending rewards
        let pending = Self::calc_pending(&pool, &info);
        info.pending_rewards = info.pending_rewards.checked_add(pending).expect("Overflow");

        // Update stake
        info.staked = info.staked.checked_sub(amount).expect("Underflow");
        info.reward_debt = info
            .staked
            .checked_mul(pool.acc_reward_per_share)
            .expect("Overflow")
            / PRECISION;

        pool.total_staked = pool.total_staked.checked_sub(amount).expect("Underflow");

        // Return LP tokens to user
        let lp_client = token::Client::new(&env, &pool.lp_token);
        lp_client.transfer(&env.current_contract_address(), &user, &amount);

        Self::save_pool(&env, pool_id, &pool);
        Self::save_user(&env, pool_id, &user, &info);

        env.events()
            .publish((Symbol::new(&env, "unstaked"),), (pool_id, user, amount));
    }

    /// Claim all accumulated reward tokens for `user` in `pool_id`.
    ///
    /// Transfers reward tokens from this contract to the user.
    /// The contract must hold sufficient reward token balance.
    pub fn harvest(env: Env, pool_id: u32, user: Address) {
        user.require_auth();

        let mut pool = Self::load_pool(&env, pool_id);
        Self::update_pool(&env, &mut pool);

        let mut info = Self::load_user(&env, pool_id, &user);
        let pending = Self::calc_pending(&pool, &info);
        let total_claimable = info.pending_rewards.checked_add(pending).expect("Overflow");

        if total_claimable == 0 {
            panic!("Nothing to harvest");
        }

        // Reset pending state
        info.pending_rewards = 0;
        info.reward_debt = info
            .staked
            .checked_mul(pool.acc_reward_per_share)
            .expect("Overflow")
            / PRECISION;

        // Transfer reward tokens to user
        let reward_client = token::Client::new(&env, &pool.reward_token);
        reward_client.transfer(&env.current_contract_address(), &user, &total_claimable);

        Self::save_pool(&env, pool_id, &pool);
        Self::save_user(&env, pool_id, &user, &info);

        env.events().publish(
            (Symbol::new(&env, "harvested"),),
            (pool_id, user, total_claimable),
        );
    }

    // ── View functions ───────────────────────────────────────────────────────

    /// Return the PoolInfo for a given pool.
    pub fn get_pool(env: Env, pool_id: u32) -> PoolInfo {
        Self::load_pool(&env, pool_id)
    }

    /// Return the UserInfo for a given (pool, user) pair.
    pub fn get_user_info(env: Env, pool_id: u32, user: Address) -> UserInfo {
        Self::load_user(&env, pool_id, &user)
    }

    /// Compute the total pending (unclaimed) rewards for a user without
    /// mutating any state.
    pub fn pending_rewards(env: Env, pool_id: u32, user: Address) -> i128 {
        let pool = Self::load_pool(&env, pool_id);
        let info = Self::load_user(&env, pool_id, &user);

        // Simulate what update_pool would do
        let simulated_acc = if pool.total_staked > 0 && pool.active {
            let elapsed = (env.ledger().sequence() - pool.last_update_ledger) as i128;
            let reward = elapsed.checked_mul(pool.reward_rate).expect("Overflow");
            pool.acc_reward_per_share
                .checked_add(reward.checked_mul(PRECISION).expect("Overflow") / pool.total_staked)
                .expect("Overflow")
        } else {
            pool.acc_reward_per_share
        };

        let simulated_pending = info.staked.checked_mul(simulated_acc).expect("Overflow")
            / PRECISION
            - info.reward_debt;

        info.pending_rewards
            .checked_add(simulated_pending)
            .expect("Overflow")
    }

    /// Return the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized")
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Advance the pool's reward accumulator to the current ledger.
    fn update_pool(env: &Env, pool: &mut PoolInfo) {
        let current_ledger = env.ledger().sequence();
        if current_ledger <= pool.last_update_ledger {
            return;
        }
        if pool.total_staked > 0 && pool.active {
            let elapsed = (current_ledger - pool.last_update_ledger) as i128;
            let reward = elapsed.checked_mul(pool.reward_rate).expect("Overflow");
            pool.acc_reward_per_share = pool
                .acc_reward_per_share
                .checked_add(reward.checked_mul(PRECISION).expect("Overflow") / pool.total_staked)
                .expect("Overflow");
        }
        pool.last_update_ledger = current_ledger;
    }

    /// Compute rewards earned by a user since their last interaction.
    fn calc_pending(pool: &PoolInfo, info: &UserInfo) -> i128 {
        info.staked
            .checked_mul(pool.acc_reward_per_share)
            .expect("Overflow")
            / PRECISION
            - info.reward_debt
    }

    /// Load a pool, panicking if it does not exist.
    fn load_pool(env: &Env, pool_id: u32) -> PoolInfo {
        env.storage()
            .persistent()
            .get(&DataKey::Pool(pool_id))
            .expect("Pool not found")
    }

    /// Persist a pool and extend its TTL.
    fn save_pool(env: &Env, pool_id: u32, pool: &PoolInfo) {
        let key = DataKey::Pool(pool_id);
        env.storage().persistent().set(&key, pool);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_MIN, TTL_MAX);
    }

    /// Load a user's staking record, returning a zeroed default if absent.
    fn load_user(env: &Env, pool_id: u32, user: &Address) -> UserInfo {
        env.storage()
            .persistent()
            .get(&DataKey::UserInfo(pool_id, user.clone()))
            .unwrap_or(UserInfo {
                staked: 0,
                reward_debt: 0,
                pending_rewards: 0,
            })
    }

    /// Persist a user record and extend its TTL.
    fn save_user(env: &Env, pool_id: u32, user: &Address, info: &UserInfo) {
        let key = DataKey::UserInfo(pool_id, user.clone());
        env.storage().persistent().set(&key, info);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_MIN, TTL_MAX);
    }

    /// Assert the caller is the stored admin.
    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();
    }
}

#[cfg(test)]
mod test;
