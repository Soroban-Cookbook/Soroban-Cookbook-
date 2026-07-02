//! # Vault Strategies
//!
//! Demonstrates a yield-bearing vault that supports multiple pluggable strategies.
//! Users deposit tokens into the vault; the vault allocates funds to an active
//! strategy that earns yield.  The admin can switch strategies at any time,
//! subject to risk-management guards.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────┐  deposit/withdraw  ┌───────────────────────────────────────┐
//! │  User    │ ─────────────────► │              VaultContract             │
//! └──────────┘                    │                                        │
//!                                 │  active_strategy ──► ConservativeStrat │
//!                                 │                  ──► BalancedStrat      │
//!                                 │                  ──► AggressiveStrat    │
//!                                 └───────────────────────────────────────┘
//! ```
//!
//! ## Strategy Interface
//!
//! Every strategy is represented by a [`StrategyType`] variant and must satisfy:
//! - `max_allocation_bps` – maximum % of vault TVL it may hold (basis points, 1 bps = 0.01 %)
//! - `expected_apy_bps`   – indicative annual yield in basis points
//! - `risk_level`         – [`RiskLevel`] enum used by the risk-management layer
//!
//! ## Risk Management
//!
//! The vault enforces three risk controls:
//! 1. **Allocation cap** – each strategy declares a `max_allocation_bps`; the vault
//!    rejects deposits that would exceed it.
//! 2. **Emergency pause** – the admin can pause the vault; all deposits are blocked
//!    while withdrawals remain open (users can always exit).
//! 3. **Strategy switch guard** – switching to a higher-risk strategy requires the
//!    vault to be below a configurable `max_tvl_for_aggressive` threshold.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Basis-point denominator (10 000 bps = 100 %).
const BPS_DENOM: i128 = 10_000;

/// Maximum TVL (in token units) allowed when switching to the Aggressive strategy.
/// Acts as a circuit-breaker: large vaults must stay in lower-risk strategies.
const MAX_TVL_FOR_AGGRESSIVE: i128 = 1_000_000;

// ─────────────────────────────────────────────────────────────────────────────
// Storage keys
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin address.
    Admin,
    /// Whether the vault is paused (bool).
    Paused,
    /// Currently active strategy.
    ActiveStrategy,
    /// Total value locked in the vault (i128 token units).
    TotalDeposits,
    /// Per-user deposit balance.
    UserBalance(Address),
}

// ─────────────────────────────────────────────────────────────────────────────
// Strategy interface types
// ─────────────────────────────────────────────────────────────────────────────

/// Risk classification used by the risk-management layer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    /// Capital-preservation focus; lowest expected yield.
    Low,
    /// Balanced risk/reward; moderate yield.
    Medium,
    /// Aggressive yield-seeking; highest risk.
    High,
}

/// The three built-in yield strategies.
///
/// In a production system each variant would correspond to an external protocol
/// (e.g. a lending pool, an AMM LP position, or a leveraged vault).  Here the
/// yield simulation is kept intentionally simple so the focus stays on the
/// strategy-switching and risk-management patterns.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrategyType {
    /// **Conservative** – money-market / stable lending.
    /// Low risk, ~3 % APY, up to 100 % allocation.
    Conservative,
    /// **Balanced** – diversified LP positions.
    /// Medium risk, ~8 % APY, up to 80 % allocation.
    Balanced,
    /// **Aggressive** – leveraged yield farming.
    /// High risk, ~20 % APY, up to 50 % allocation.
    Aggressive,
}

/// Static parameters that describe a strategy.
/// This is the "strategy interface" – every strategy must expose these fields.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StrategyParams {
    /// Human-readable name (max 9 chars for `symbol_short!` compatibility).
    pub name: Symbol,
    /// Maximum fraction of vault TVL this strategy may hold, in basis points.
    pub max_allocation_bps: i128,
    /// Indicative annual yield in basis points.
    pub expected_apy_bps: i128,
    /// Risk classification.
    pub risk_level: RiskLevel,
}

// ─────────────────────────────────────────────────────────────────────────────
// Strategy implementations
// ─────────────────────────────────────────────────────────────────────────────

/// Return the static [`StrategyParams`] for a given [`StrategyType`].
///
/// This function acts as the strategy registry / factory.  Adding a new
/// strategy means adding a new variant to [`StrategyType`] and a new arm here.
pub fn strategy_params(env: &Env, strategy: &StrategyType) -> StrategyParams {
    match strategy {
        // ── Conservative ────────────────────────────────────────────────────
        // Deposits into a stable money-market; principal is protected.
        // Suitable for risk-averse users or large vaults.
        StrategyType::Conservative => StrategyParams {
            name: symbol_short!("conserve"),
            max_allocation_bps: BPS_DENOM, // 100 % – no cap
            expected_apy_bps: 300,         // ~3 % APY
            risk_level: RiskLevel::Low,
        },

        // ── Balanced ────────────────────────────────────────────────────────
        // Splits funds across several LP pools; moderate impermanent-loss risk.
        StrategyType::Balanced => StrategyParams {
            name: symbol_short!("balanced"),
            max_allocation_bps: 8_000, // 80 % cap
            expected_apy_bps: 800,     // ~8 % APY
            risk_level: RiskLevel::Medium,
        },

        // ── Aggressive ──────────────────────────────────────────────────────
        // Leveraged yield farming; high returns but liquidation risk.
        // Blocked for vaults above MAX_TVL_FOR_AGGRESSIVE.
        StrategyType::Aggressive => StrategyParams {
            name: symbol_short!("aggressiv"),
            max_allocation_bps: 5_000, // 50 % cap
            expected_apy_bps: 2_000,   // ~20 % APY
            risk_level: RiskLevel::High,
        },
    }
}

/// Simulate the yield earned by `amount` tokens over `periods` ledger-periods
/// using the strategy's APY.
///
/// Formula: `yield = amount * apy_bps * periods / (BPS_DENOM * 365)`
///
/// This is a simplified linear approximation; a real implementation would use
/// compound interest and an on-chain oracle for the period length.
pub fn simulate_yield(amount: i128, apy_bps: i128, periods: i128) -> i128 {
    amount
        .checked_mul(apy_bps)
        .and_then(|v| v.checked_mul(periods))
        .and_then(|v| v.checked_div(BPS_DENOM * 365))
        .unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Vault contract
// ─────────────────────────────────────────────────────────────────────────────

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    // ── Initialisation ───────────────────────────────────────────────────────

    /// Initialise the vault with an admin and a starting strategy.
    ///
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address, initial_strategy: StrategyType) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ActiveStrategy, &initial_strategy);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &0i128);
        env.storage().instance().set(&DataKey::Paused, &false);

        env.events().publish(
            (symbol_short!("vault"), symbol_short!("init")),
            (admin, initial_strategy),
        );
    }

    // ── Deposits & Withdrawals ───────────────────────────────────────────────

    /// Deposit `amount` tokens into the vault on behalf of `user`.
    ///
    /// # Risk checks
    /// - Vault must not be paused.
    /// - `amount` must be positive.
    /// - The resulting allocation must not exceed the active strategy's
    ///   `max_allocation_bps` cap.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Risk check 1: vault must not be paused
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("Vault is paused");
        }

        let strategy: StrategyType = env
            .storage()
            .instance()
            .get(&DataKey::ActiveStrategy)
            .expect("Not initialized");

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0);

        // Risk check 2: allocation cap
        let params = strategy_params(&env, &strategy);
        let new_total = total.checked_add(amount).expect("Overflow");
        // max_allowed = new_total * max_allocation_bps / BPS_DENOM
        // We want: amount <= new_total * max_allocation_bps / BPS_DENOM
        // Rearranged: amount * BPS_DENOM <= new_total * max_allocation_bps
        if amount
            .checked_mul(BPS_DENOM)
            .expect("Overflow")
            > new_total
                .checked_mul(params.max_allocation_bps)
                .expect("Overflow")
        {
            panic!("Exceeds strategy allocation cap");
        }

        // Update balances
        let user_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::UserBalance(user.clone()))
            .unwrap_or(0);

        let new_user_bal = user_bal.checked_add(amount).expect("Overflow");
        env.storage()
            .persistent()
            .set(&DataKey::UserBalance(user.clone()), &new_user_bal);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::UserBalance(user.clone()), 17_280, 120_960);

        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);

        env.events().publish(
            (symbol_short!("vault"), symbol_short!("deposit")),
            (user, amount, new_total),
        );
    }

    /// Withdraw `amount` tokens from the vault.
    ///
    /// Withdrawals are always permitted, even when the vault is paused,
    /// so users can always exit.
    pub fn withdraw(env: Env, user: Address, amount: i128) {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let user_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::UserBalance(user.clone()))
            .unwrap_or(0);

        if user_bal < amount {
            panic!("Insufficient balance");
        }

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0);

        let new_user_bal = user_bal.checked_sub(amount).expect("Underflow");
        let new_total = total.checked_sub(amount).expect("Underflow");

        env.storage()
            .persistent()
            .set(&DataKey::UserBalance(user.clone()), &new_user_bal);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::UserBalance(user.clone()), 17_280, 120_960);

        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);

        env.events().publish(
            (symbol_short!("vault"), symbol_short!("withdraw")),
            (user, amount, new_total),
        );
    }

    // ── Strategy Switching ───────────────────────────────────────────────────

    /// Switch the active strategy.  Only the admin may call this.
    ///
    /// # Risk checks
    /// - Caller must be the admin.
    /// - Switching to [`StrategyType::Aggressive`] is blocked when the vault's
    ///   TVL exceeds [`MAX_TVL_FOR_AGGRESSIVE`].
    pub fn switch_strategy(env: Env, admin: Address, new_strategy: StrategyType) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if admin != stored_admin {
            panic!("Unauthorized");
        }

        // Risk check 3: TVL guard for aggressive strategy
        if new_strategy == StrategyType::Aggressive {
            let total: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TotalDeposits)
                .unwrap_or(0);
            if total > MAX_TVL_FOR_AGGRESSIVE {
                panic!("TVL too high for aggressive strategy");
            }
        }

        let old_strategy: StrategyType = env
            .storage()
            .instance()
            .get(&DataKey::ActiveStrategy)
            .expect("Not initialized");

        env.storage()
            .instance()
            .set(&DataKey::ActiveStrategy, &new_strategy);

        env.events().publish(
            (symbol_short!("vault"), symbol_short!("switch")),
            (old_strategy, new_strategy),
        );
    }

    // ── Emergency Controls ───────────────────────────────────────────────────

    /// Pause the vault (admin only).  Deposits are blocked; withdrawals remain open.
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events()
            .publish((symbol_short!("vault"), symbol_short!("pause")), ());
    }

    /// Unpause the vault (admin only).
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events()
            .publish((symbol_short!("vault"), symbol_short!("unpause")), ());
    }

    // ── View Functions ───────────────────────────────────────────────────────

    /// Return the user's current deposit balance.
    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::UserBalance(user))
            .unwrap_or(0)
    }

    /// Return the total value locked in the vault.
    pub fn total_deposits(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0)
    }

    /// Return the currently active strategy.
    pub fn active_strategy(env: Env) -> StrategyType {
        env.storage()
            .instance()
            .get(&DataKey::ActiveStrategy)
            .expect("Not initialized")
    }

    /// Return the static parameters for the currently active strategy.
    pub fn strategy_info(env: Env) -> StrategyParams {
        let strategy: StrategyType = env
            .storage()
            .instance()
            .get(&DataKey::ActiveStrategy)
            .expect("Not initialized");
        strategy_params(&env, &strategy)
    }

    /// Return whether the vault is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Estimate the yield that `amount` tokens would earn over `periods` days
    /// using the active strategy's APY.
    pub fn estimate_yield(env: Env, amount: i128, periods: i128) -> i128 {
        let strategy: StrategyType = env
            .storage()
            .instance()
            .get(&DataKey::ActiveStrategy)
            .expect("Not initialized");
        let params = strategy_params(&env, &strategy);
        simulate_yield(amount, params.expected_apy_bps, periods)
    }

    // ── Internal Helpers ─────────────────────────────────────────────────────

    fn assert_admin(env: &Env, caller: &Address) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if *caller != stored_admin {
            panic!("Unauthorized");
        }
    }
}

#[cfg(test)]
mod test;
