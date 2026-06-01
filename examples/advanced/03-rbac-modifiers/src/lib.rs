#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ---------------------------------------------------------------------------
// Role-Based Access Control (RBAC) Modifiers
//
// This contract demonstrates composable role guards for Soroban contracts.
// Roles are stored as sets of addresses. Any function can be protected by
// calling `only_role` (single role) or `any_role` (multi-role) at the top.
//
// Roles supported out of the box:
//   ADMIN  – full control, can grant/revoke any role
//   MINTER – can call mint-style operations
//   PAUSER – can pause/unpause the contract
//
// Role lifecycle events are emitted on every grant/revoke so off-chain
// indexers can reconstruct the current role set at any point in time.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted when a role is granted to an account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleGrantedEventData {
    /// The role that was granted.
    pub role: Symbol,
    /// The account that received the role.
    pub account: Address,
    /// The admin that performed the grant.
    pub sender: Address,
    /// Ledger timestamp of the grant.
    pub timestamp: u64,
}

/// Emitted when a role is revoked from an account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleRevokedEventData {
    /// The role that was revoked.
    pub role: Symbol,
    /// The account that lost the role.
    pub account: Address,
    /// The admin that performed the revoke.
    pub sender: Address,
    /// Ledger timestamp of the revoke.
    pub timestamp: u64,
}

/// Emitted when a protected function is successfully called.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedCallEventData {
    /// The role that was checked.
    pub role: Symbol,
    /// The caller that passed the check.
    pub caller: Address,
    /// Ledger timestamp.
    pub timestamp: u64,
}

/// Namespace symbol for all events emitted by this contract.
const CONTRACT_NS: Symbol = symbol_short!("rbac");
const ACTION_GRANT: Symbol = symbol_short!("grant");
const ACTION_REVOKE: Symbol = symbol_short!("revoke");
const ACTION_CALL: Symbol = symbol_short!("call");

// ---------------------------------------------------------------------------
// Well-known role symbols
// ---------------------------------------------------------------------------

/// Full-control role. Required to grant/revoke other roles.
pub const ROLE_ADMIN: Symbol = symbol_short!("ADMIN");
/// Allowed to call mint-style operations.
pub const ROLE_MINTER: Symbol = symbol_short!("MINTER");
/// Allowed to pause/unpause the contract.
pub const ROLE_PAUSER: Symbol = symbol_short!("PAUSER");

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    /// Set of addresses that hold `role`. Value: Vec<Address>.
    RoleMembers(Symbol),
    /// Whether the contract is paused.
    Paused,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Bootstrap the contract by granting ADMIN to `initial_admin`.
    ///
    /// Can only be called once. Subsequent calls panic.
    pub fn initialize(env: Env, initial_admin: Address) {
        // Guard: only callable once.
        if env
            .storage()
            .instance()
            .has(&DataKey::RoleMembers(ROLE_ADMIN))
        {
            panic!("Already initialized");
        }

        // Grant ADMIN without requiring auth (bootstrap).
        let mut members: Vec<Address> = Vec::new(&env);
        members.push_back(initial_admin.clone());
        env.storage()
            .instance()
            .set(&DataKey::RoleMembers(ROLE_ADMIN), &members);

        env.storage().instance().set(&DataKey::Paused, &false);

        env.events().publish(
            (CONTRACT_NS, ACTION_GRANT, ROLE_ADMIN),
            RoleGrantedEventData {
                role: ROLE_ADMIN,
                account: initial_admin.clone(),
                sender: initial_admin,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Role management
    // -----------------------------------------------------------------------

    /// Grant `role` to `account`. Caller must hold ADMIN.
    pub fn grant_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_ADMIN);

        let key = DataKey::RoleMembers(role.clone());
        let mut members: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        if !members.contains(&account) {
            members.push_back(account.clone());
            env.storage().instance().set(&key, &members);
        }

        env.events().publish(
            (CONTRACT_NS, ACTION_GRANT, role.clone()),
            RoleGrantedEventData {
                role,
                account,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Revoke `role` from `account`. Caller must hold ADMIN.
    pub fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_ADMIN);

        let key = DataKey::RoleMembers(role.clone());
        let members: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        // Rebuild without the revoked account.
        let mut updated: Vec<Address> = Vec::new(&env);
        for m in members.iter() {
            if m != account {
                updated.push_back(m);
            }
        }
        env.storage().instance().set(&key, &updated);

        env.events().publish(
            (CONTRACT_NS, ACTION_REVOKE, role.clone()),
            RoleRevokedEventData {
                role,
                account,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Renounce a role the caller currently holds.
    ///
    /// Useful for self-service role removal without requiring an admin.
    pub fn renounce_role(env: Env, caller: Address, role: Symbol) {
        caller.require_auth();

        let key = DataKey::RoleMembers(role.clone());
        let members: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut updated: Vec<Address> = Vec::new(&env);
        for m in members.iter() {
            if m != caller {
                updated.push_back(m);
            }
        }
        env.storage().instance().set(&key, &updated);

        env.events().publish(
            (CONTRACT_NS, ACTION_REVOKE, role.clone()),
            RoleRevokedEventData {
                role,
                account: caller.clone(),
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Role queries
    // -----------------------------------------------------------------------

    /// Return `true` if `account` holds `role`.
    pub fn has_role(env: Env, role: Symbol, account: Address) -> bool {
        Self::check_role(&env, &account, role)
    }

    /// Return all members of `role`.
    pub fn get_role_members(env: Env, role: Symbol) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::RoleMembers(role))
            .unwrap_or_else(|| Vec::new(&env))
    }

    // -----------------------------------------------------------------------
    // Protected operations (demonstrate role guards)
    // -----------------------------------------------------------------------

    /// Mint-style operation — only MINTER role may call.
    ///
    /// In a real contract this would interact with a token contract.
    pub fn protected_mint(env: Env, caller: Address, _to: Address, _amount: i128) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_MINTER);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, ROLE_MINTER),
            ProtectedCallEventData {
                role: ROLE_MINTER,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
        // ... token mint logic here
    }

    /// Pause the contract — only PAUSER role may call.
    pub fn pause(env: Env, caller: Address) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_PAUSER);

        env.storage().instance().set(&DataKey::Paused, &true);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, ROLE_PAUSER),
            ProtectedCallEventData {
                role: ROLE_PAUSER,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Unpause the contract — only PAUSER role may call.
    pub fn unpause(env: Env, caller: Address) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_PAUSER);

        env.storage().instance().set(&DataKey::Paused, &false);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, ROLE_PAUSER),
            ProtectedCallEventData {
                role: ROLE_PAUSER,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Admin-only operation — demonstrates single-role guard.
    pub fn admin_action(env: Env, caller: Address) {
        caller.require_auth();
        Self::only_role(&env, &caller, ROLE_ADMIN);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, ROLE_ADMIN),
            ProtectedCallEventData {
                role: ROLE_ADMIN,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
        // ... admin logic here
    }

    /// Multi-role guard: caller must hold ADMIN **or** MINTER.
    ///
    /// Demonstrates `any_role` composable guard.
    pub fn admin_or_minter_action(env: Env, caller: Address) {
        caller.require_auth();
        Self::any_role(&env, &caller, &[ROLE_ADMIN, ROLE_MINTER]);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, symbol_short!("adm_mnt")),
            ProtectedCallEventData {
                role: symbol_short!("adm_mnt"),
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
        // ... logic here
    }

    /// Return whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Composable role-guard helpers (private — called internally)
    // -----------------------------------------------------------------------

    /// Panic unless `account` holds `role`.
    ///
    /// Use at the top of any protected function:
    /// ```rust
    /// Self::only_role(&env, &caller, ROLE_ADMIN);
    /// ```
    fn only_role(env: &Env, account: &Address, role: Symbol) {
        if !Self::check_role(env, account, role) {
            panic!("Caller does not have required role");
        }
    }

    /// Panic unless `account` holds **at least one** of the supplied `roles`.
    ///
    /// Use for multi-role guards:
    /// ```rust
    /// Self::any_role(&env, &caller, &[ROLE_ADMIN, ROLE_MINTER]);
    /// ```
    fn any_role(env: &Env, account: &Address, roles: &[Symbol]) {
        for role in roles.iter() {
            if Self::check_role(env, account, role.clone()) {
                return;
            }
        }
        panic!("Caller does not have any required role");
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn check_role(env: &Env, account: &Address, role: Symbol) -> bool {
        let members: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleMembers(role))
            .unwrap_or_else(|| Vec::new(env));
        members.contains(account)
    }
}

#[cfg(test)]
mod test;
