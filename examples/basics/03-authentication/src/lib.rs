//! # Custom Authorization Logic Patterns
//!
//! Demonstrates advanced authorization patterns for Soroban smart contracts
//! beyond basic `require_auth()`. While Soroban's built-in auth verifies that
//! a caller *is who they claim to be*, real contracts also need to verify that
//! the caller *is allowed to do what they're trying to do*.
//!
//! This contract covers three complementary patterns:
//!
//! - **Role-Based Access Control (RBAC):** Assign Admin, Moderator, or User
//!   roles and gate functions by role.
//! - **Time-Based Restrictions:** Time-locks that prevent actions before a
//!   deadline and cooldowns that throttle repeated calls.
//! - **State-Based Authorization:** A contract-wide state machine (Active,
//!   Paused, Frozen) that conditionally disables functionality.
//!
//! ## Security Design Principles
//!
//! 1. Always call `require_auth()` **first** — identity before permission.
//! 2. Perform custom permission checks **after** auth verification.
//! 3. Store authorization data in appropriate storage tiers (instance for
//!    contract-wide config, persistent for per-account data).
//! 4. Keep auth checks separate from business logic for auditability.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Roles that can be assigned to accounts. The numeric discriminants are used
/// when returning roles as `u32` to callers that cannot decode the enum.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}

/// Contract-wide operational state. Transitions are admin-only.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractState {
    Active = 0,
    Paused = 1,
    Frozen = 2,
}

/// Storage keys. Instance storage holds contract-wide config; persistent
/// storage holds per-account data that must survive across ledgers.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Role(Address),
    State,
    TimeLock,
    CooldownPeriod,
    LastAction(Address),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    // ==================== INITIALIZATION ====================

    /// Initializes the contract with the given admin address.
    ///
    /// Must be called exactly once. Panics on repeated calls to prevent
    /// admin hijacking after deployment.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().extend_ttl(100, 100);

        // Grant the Admin role to the initializing address so that
        // role-gated functions work immediately after deployment.
        env.storage()
            .persistent()
            .set(&DataKey::Role(admin.clone()), &Role::Admin);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Role(admin), 100, 100);
    }

    // ==================== ROLE-BASED ACCESS CONTROL ====================

    /// Grants a role to `account`. Only the stored admin may call this, and
    /// they must authorize the transaction.
    pub fn grant_role(env: Env, admin: Address, account: Address, role: Role) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .persistent()
            .set(&DataKey::Role(account.clone()), &role);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Role(account.clone()), 100, 100);

        env.events().publish((symbol_short!("role"),), account);
    }

    /// Revokes any role previously assigned to `account`. Admin-only.
    pub fn revoke_role(env: Env, admin: Address, account: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .persistent()
            .remove(&DataKey::Role(account.clone()));

        env.events().publish((symbol_short!("revoke"),), account);
    }

    /// Returns the role of `account` as a `u32` discriminant
    /// (0 = Admin, 1 = Moderator, 2 = User).
    ///
    /// Panics if no role has been assigned.
    pub fn get_role(env: Env, account: Address) -> u32 {
        let role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::Role(account))
            .unwrap_or_else(|| panic!("No role assigned"));
        role as u32
    }

    /// Returns `true` if `account` holds exactly the given `role`.
    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::Role(account))
            == Some(role)
    }

    // ==================== ROLE-PROTECTED ACTIONS ====================

    /// An action restricted to Admin-role callers.
    ///
    /// Demonstrates the two-step pattern: authenticate identity first, then
    /// check permission via stored role data.
    pub fn admin_action(env: Env, caller: Address, value: u64) -> u64 {
        caller.require_auth();
        Self::require_role(&env, &caller, &[Role::Admin]);

        let result = value * 2;
        env.events().publish((symbol_short!("admin"),), result);
        result
    }

    /// An action available to Admin *or* Moderator callers.
    pub fn moderator_action(env: Env, caller: Address, value: u64) -> u64 {
        caller.require_auth();
        Self::require_role(&env, &caller, &[Role::Admin, Role::Moderator]);

        let result = value + 100;
        env.events().publish((symbol_short!("mod"),), result);
        result
    }

    // ==================== TIME-BASED AUTHORIZATION ====================

    /// Sets a future timestamp before which `time_locked_action` will reject
    /// all callers. Admin-only.
    pub fn set_time_lock(env: Env, admin: Address, unlock_time: u64) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .instance()
            .set(&DataKey::TimeLock, &unlock_time);
        env.storage().instance().extend_ttl(100, 100);

        env.events()
            .publish((symbol_short!("timelock"),), unlock_time);
    }

    /// Executes only after the time-lock has expired.
    ///
    /// Compares `env.ledger().timestamp()` against the stored unlock time.
    /// Returns the current ledger timestamp on success.
    pub fn time_locked_action(env: Env, caller: Address) -> u64 {
        caller.require_auth();

        let unlock_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimeLock)
            .unwrap_or(0);

        if env.ledger().timestamp() < unlock_time {
            panic!("Action is time-locked");
        }

        env.ledger().timestamp()
    }

    /// Sets the minimum number of seconds that must elapse between successive
    /// calls to `cooldown_action` by the same caller. Admin-only.
    pub fn set_cooldown(env: Env, admin: Address, period: u64) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .instance()
            .set(&DataKey::CooldownPeriod, &period);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish((symbol_short!("cooldown"),), period);
    }

    /// Rate-limited action that enforces a per-caller cooldown period.
    ///
    /// Tracks each caller's last execution timestamp in persistent storage
    /// and rejects calls that arrive before the cooldown expires.
    /// Returns the current ledger timestamp on success.
    pub fn cooldown_action(env: Env, caller: Address) -> u64 {
        caller.require_auth();

        let period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownPeriod)
            .unwrap_or(0);

        let last_action: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LastAction(caller.clone()))
            .unwrap_or(0);

        let now = env.ledger().timestamp();
        if last_action > 0 && now < last_action + period {
            panic!("Cooldown period not elapsed");
        }

        env.storage()
            .persistent()
            .set(&DataKey::LastAction(caller.clone()), &now);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::LastAction(caller), 100, 100);

        now
    }

    // ==================== STATE-BASED AUTHORIZATION ====================

    /// Transitions the contract to a new operational state. Admin-only.
    ///
    /// Use `Paused` to temporarily halt user-facing operations or `Frozen`
    /// for a harder stop (e.g., during an incident response).
    pub fn set_state(env: Env, admin: Address, state: ContractState) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage().instance().set(&DataKey::State, &state);
        env.storage().instance().extend_ttl(100, 100);

        env.events()
            .publish((symbol_short!("state"),), state as u32);
    }

    /// Returns the current contract state as a `u32`
    /// (0 = Active, 1 = Paused, 2 = Frozen). Defaults to Active.
    pub fn get_state(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, ContractState>(&DataKey::State)
            .unwrap_or(ContractState::Active) as u32
    }

    /// An action that only executes when the contract is in the `Active`
    /// state. Returns the current ledger timestamp.
    pub fn active_only_action(env: Env, caller: Address) -> u64 {
        caller.require_auth();

        let state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active);

        if state != ContractState::Active {
            panic!("Contract is not active");
        }

        env.ledger().timestamp()
    }

    // ==================== INTERNAL HELPERS ====================

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"));
        if *caller != admin {
            panic!("Not admin");
        }
    }

    fn require_role(env: &Env, caller: &Address, allowed: &[Role]) {
        let role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::Role(caller.clone()))
            .unwrap_or_else(|| panic!("No role assigned"));
        if !allowed.contains(&role) {
            panic!("Insufficient role");
        }
    }
}

mod test;
