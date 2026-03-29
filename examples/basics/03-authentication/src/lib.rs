//! # Authentication Patterns Contract
//!
//! Demonstrates core address-authentication patterns using Soroban's
//! `require_auth()` function, along with custom authorization logic including
//! role-based access control, time-based restrictions, and state-based gating.
//!
//! ## What `require_auth()` does
//!
//! - Verifies that the given `Address` has signed / authorized this invocation.
//! - Works for user accounts (ed25519 keypairs) and contract addresses alike.
//! - Protects against replays -- the host records the nonce automatically.
//! - Is essential for any state-mutating operation in multi-user contracts.
//!
//! ## Custom Authorization Patterns
//!
//! Beyond basic authentication, this contract demonstrates:
//! - **Role-Based Access Control (RBAC)**: Admin, Moderator, and User roles
//! - **Time-Based Restrictions**: Time-locks and cooldown periods
//! - **State-Based Authorization**: Contract state gating (Active/Paused/Frozen)

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol,
    Vec,
};

// ---------------------------------------------------------------------------
// Role definitions
// ---------------------------------------------------------------------------

/// Role hierarchy for access control.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Admin = 0,
    Moderator = 1,
    User = 2,
}

// ---------------------------------------------------------------------------
// Contract state
// ---------------------------------------------------------------------------

/// Global contract state for state-based authorization.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractState {
    Active = 0,
    Paused = 1,
    Frozen = 2,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// Keys used in contract storage.
///
/// * `Admin`              -- the privileged admin address (instance storage).
/// * `Balance(Address)`   -- per-account token balance (persistent storage).
/// * `Allowance(from, spender)` -- spend allowance (persistent storage).
/// * `UserRole(Address)`  -- role assigned to an address (persistent storage).
/// * `TimeLock`           -- global unlock timestamp (instance storage).
/// * `CooldownPeriod`     -- cooldown duration in seconds (instance storage).
/// * `LastAction(Address)` -- last action timestamp per address (persistent storage).
/// * `State`              -- current contract state (instance storage).
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
    UserRole(Address),
    TimeLock,
    CooldownPeriod,
    LastAction(Address),
    State,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    /// The caller does not have the required permission.
    Unauthorized = 1,
    /// The operation requires an admin that has not been set, or the provided
    /// address does not match the stored admin.
    NotAdmin = 2,
    /// `initialize()` has already been called.
    AlreadyInitialized = 3,
    /// The sender does not have enough balance to complete the transfer.
    InsufficientBalance = 4,
    /// The action is time-locked until a future timestamp.
    TimeLocked = 5,
    /// The cooldown period has not elapsed since the last action.
    CooldownActive = 6,
    /// The contract is not in the required state for this operation.
    InvalidState = 7,
    /// The caller does not have the required role.
    InsufficientRole = 8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Payload for an admin-action event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActionEventData {
    /// Identifier of the specific action performed.
    pub action: Symbol,
    /// Timestamp when the action was executed.
    pub timestamp: u64,
}

/// Payload for an audit-trail event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTrailEventData {
    /// Free-form description or reference tag.
    pub details: Symbol,
    /// Ledger timestamp at emission time.
    pub timestamp: u64,
}

/// Namespace symbol used as the first topic of every event this contract emits.
const CONTRACT_NS: Symbol = symbol_short!("auth");
/// Naming convention: `snake_case` action names in topic[1].
const ACTION_ADMIN: Symbol = symbol_short!("admin");
const ACTION_AUDIT: Symbol = symbol_short!("audit");

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
    /// Must be called exactly once. Subsequent calls return
    /// `AlreadyInitialized` so the admin cannot be hijacked after deployment.
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Audit trail for contract initialization
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, admin),
            AuditTrailEventData {
                details: symbol_short!("init"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns the current admin address, if set.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // ==================== ADMIN-ONLY PATTERNS ====================

    /// Demonstrates an admin-only gate.
    ///
    /// Pattern:
    /// 1. `require_auth` on the caller.
    /// 2. Load the stored admin and compare -- prevents anyone from passing a
    ///    random `Address` that they happen to control.
    pub fn admin_action(env: Env, admin: Address, value: u32) -> Result<u32, AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;

        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }

        // Log admin action
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin),
            AdminActionEventData {
                action: symbol_short!("action"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(value * 2)
    }

    /// Admin-only balance setter (e.g. for minting in tests or bootstrapping).
    pub fn set_balance(
        env: Env,
        admin: Address,
        user: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;

        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }

        let old_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(user.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), &amount);

        // Audit trail for balance change
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, admin, user),
            (old_balance, amount),
        );

        Ok(())
    }

    // ==================== SINGLE-ADDRESS AUTH PATTERN ====================

    /// Transfer tokens from `from` to `to`.
    ///
    /// Security:
    /// - `from.require_auth()` ensures only the owner can debit their account.
    /// - The balance check prevents the sender from going negative.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if amount <= 0 || from_balance < amount {
            return Err(AuthError::InsufficientBalance);
        }

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));

        Ok(())
    }

    // ==================== ALLOWANCE PATTERN ====================

    /// Approve `spender` to transfer up to `amount` on behalf of `from`.
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        from.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(from, spender), &amount);
        Ok(())
    }

    /// Transfer `amount` from `from` to `to` using the `spender` allowance.
    ///
    /// Security:
    /// - `spender.require_auth()` -- the spender must authorize the spend.
    /// - Allowance is checked BEFORE modifying balances.
    /// - `from_balance` is checked so the sender cannot go negative.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), AuthError> {
        spender.require_auth();

        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);

        if allowance < amount {
            return Err(AuthError::Unauthorized);
        }

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);

        if from_balance < amount {
            return Err(AuthError::InsufficientBalance);
        }

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(from, spender), &(allowance - amount));

        Ok(())
    }

    // ==================== QUERY ====================

    /// Returns the balance for `user` (0 if never set).
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }

    // ==================== MULTI-SIG PATTERN ====================

    /// Demonstrates N-of-N multi-sig: every signer in the list must authorize.
    ///
    /// The Soroban host collects authorizations before invoking the contract, so
    /// order does not matter. This function simply iterates the list calling
    /// `require_auth()` on each -- the host verifies all of them atomically.
    pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
        for signer in signers.iter() {
            signer.require_auth();
        }
        value + signers.len()
    }

    // ==================== SECURE OPERATION ====================

    /// Demonstrates authenticated operation with typed error return.
    ///
    /// Pattern: auth first, then validate, then execute.
    pub fn secure_operation(
        env: Env,
        user: Address,
        operation: Symbol,
    ) -> Result<Vec<Symbol>, AuthError> {
        user.require_auth();

        if operation == symbol_short!("invalid") {
            return Err(AuthError::Unauthorized);
        }

        Ok(vec![&env, symbol_short!("success"), operation])
    }

    // ==================== EVENTS WITH AUTH ====================

    /// Emit a named event after verifying the caller.
    pub fn emit_event(env: Env, user: Address, message: Symbol) {
        user.require_auth();
        env.events()
            .publish((symbol_short!("event"), user), message);
    }

    // ==================== ROLE-BASED ACCESS CONTROL ====================

    /// Grant a role to an address (admin-only).
    pub fn grant_role(
        env: Env,
        admin: Address,
        account: Address,
        role: Role,
    ) -> Result<(), AuthError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let old_role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(account.clone()))
            .unwrap_or(Role::User);

        env.storage()
            .persistent()
            .set(&DataKey::UserRole(account.clone()), &role);

        // Emit audit event with before/after state
        env.events()
            .publish((CONTRACT_NS, ACTION_AUDIT, account), (old_role, role));

        Ok(())
    }

    /// Revoke a role from an address (admin-only).
    pub fn revoke_role(env: Env, admin: Address, account: Address) -> Result<(), AuthError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let old_role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(account.clone()))
            .unwrap_or(Role::User);

        env.storage()
            .persistent()
            .remove(&DataKey::UserRole(account.clone()));

        // Emit audit event
        env.events()
            .publish((CONTRACT_NS, ACTION_AUDIT, account), (old_role, Role::User));

        Ok(())
    }

    /// Get the role of an address (returns User if not set).
    pub fn get_role(env: Env, account: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::UserRole(account))
            .unwrap_or(Role::User) as u32
    }

    /// Check if an address has a specific role.
    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        let user_role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(account))
            .unwrap_or(Role::User);
        user_role as u32 <= role as u32
    }

    /// Admin-only action demonstrating role-based access control.
    pub fn admin_role_action(env: Env, caller: Address, value: u64) -> Result<u64, AuthError> {
        caller.require_auth();
        Self::require_role(&env, &caller, &[Role::Admin])?;

        let result = value * 2;

        // Log admin role action
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, caller),
            AdminActionEventData {
                action: symbol_short!("role_act"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(result)
    }

    /// Moderator action (accessible by Admin and Moderator).
    pub fn moderator_action(env: Env, caller: Address, value: u64) -> Result<u64, AuthError> {
        caller.require_auth();
        Self::require_role(&env, &caller, &[Role::Admin, Role::Moderator])?;

        let result = value + 10;

        // Log moderator action (audit trail)
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, caller),
            AuditTrailEventData {
                details: symbol_short!("mod_act"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(result)
    }

    // ==================== TIME-BASED RESTRICTIONS ====================

    /// Set a global time-lock (admin-only).
    pub fn set_time_lock(env: Env, admin: Address, unlock_time: u64) -> Result<(), AuthError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let old_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimeLock)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::TimeLock, &unlock_time);

        // Audit trail for timelock configuration
        env.events()
            .publish((CONTRACT_NS, ACTION_AUDIT, admin), (old_time, unlock_time));

        Ok(())
    }

    /// Action that is blocked until the time-lock expires.
    pub fn time_locked_action(env: Env, caller: Address) -> Result<u64, AuthError> {
        caller.require_auth();

        let unlock_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimeLock)
            .unwrap_or(0);

        let current_time = env.ledger().timestamp();
        if current_time < unlock_time {
            return Err(AuthError::TimeLocked);
        }

        Ok(current_time)
    }

    /// Set the cooldown period (admin-only).
    pub fn set_cooldown(env: Env, admin: Address, period: u64) -> Result<(), AuthError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let old_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownPeriod)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::CooldownPeriod, &period);

        // Audit trail for cooldown configuration
        env.events()
            .publish((CONTRACT_NS, ACTION_AUDIT, admin), (old_period, period));

        Ok(())
    }

    /// Action with per-address cooldown enforcement.
    pub fn cooldown_action(env: Env, caller: Address) -> Result<u64, AuthError> {
        caller.require_auth();

        let cooldown_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownPeriod)
            .unwrap_or(0);

        let last_action: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LastAction(caller.clone()))
            .unwrap_or(0);

        let current_time = env.ledger().timestamp();

        if last_action > 0 && current_time < last_action + cooldown_period {
            return Err(AuthError::CooldownActive);
        }

        env.storage()
            .persistent()
            .set(&DataKey::LastAction(caller), &current_time);

        Ok(current_time)
    }

    // ==================== STATE-BASED AUTHORIZATION ====================

    /// Set the contract state (admin-only).
    pub fn set_state(env: Env, admin: Address, state: ContractState) -> Result<(), AuthError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let old_state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active);

        env.storage().instance().set(&DataKey::State, &state);

        // Audit trail for state change
        env.events()
            .publish((CONTRACT_NS, ACTION_AUDIT, admin), (old_state, state));

        Ok(())
    }

    /// Get the current contract state.
    pub fn get_state(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active) as u32
    }

    /// Action that only works when the contract is Active.
    pub fn active_only_action(env: Env, caller: Address) -> Result<u64, AuthError> {
        caller.require_auth();

        let state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active);

        if state != ContractState::Active {
            return Err(AuthError::InvalidState);
        }

        Ok(env.ledger().timestamp())
    }

    // ==================== HELPER METHODS ====================

    /// Verify that the caller is the admin.
    fn require_admin(env: &Env, caller: &Address) -> Result<(), AuthError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;

        if caller != &admin {
            return Err(AuthError::NotAdmin);
        }

        Ok(())
    }

    /// Verify that the caller has one of the required roles.
    fn require_role(env: &Env, caller: &Address, allowed_roles: &[Role]) -> Result<(), AuthError> {
        let user_role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(caller.clone()))
            .unwrap_or(Role::User);

        for role in allowed_roles {
            if user_role as u32 <= *role as u32 {
                return Ok(());
            }
        }

        Err(AuthError::InsufficientRole)
    }
}

#[cfg(test)]
mod test;
