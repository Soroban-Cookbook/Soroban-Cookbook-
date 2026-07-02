#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Hierarchical Access Control Contract
//
// This contract implements a hierarchical role-based access control (RBAC)
// system with dynamic permissions and secure updates. The hierarchy is:
// - ADMIN (top-level, can manage all roles and permissions)
// - MANAGER (can manage OPERATORs and specific resources)
// - OPERATOR (can perform basic resource operations)
//
// Features:
// - Role hierarchy with implicit permissions
// - Fine-grained permission checks
// - Secure role/permissions updates
// - Event logging for all role/permission changes
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleGrantedEventData {
    pub role: Symbol,
    pub account: Address,
    pub sender: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleRevokedEventData {
    pub role: Symbol,
    pub account: Address,
    pub sender: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionGrantedEventData {
    pub permission: Symbol,
    pub role: Symbol,
    pub sender: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionRevokedEventData {
    pub permission: Symbol,
    pub role: Symbol,
    pub sender: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedCallEventData {
    pub permission: Symbol,
    pub caller: Address,
    pub timestamp: u64,
}

const CONTRACT_NS: Symbol = symbol_short!("hac");
const ACTION_ROLE_GRANT: Symbol = symbol_short!("role_grant");
const ACTION_ROLE_REVOKE: Symbol = symbol_short!("role_revoke");
const ACTION_PERM_GRANT: Symbol = symbol_short!("perm_grant");
const ACTION_PERM_REVOKE: Symbol = symbol_short!("perm_revoke");
const ACTION_CALL: Symbol = symbol_short!("call");

// ---------------------------------------------------------------------------
// Well-known roles and permissions
// ---------------------------------------------------------------------------

pub const ROLE_ADMIN: Symbol = symbol_short!("ADMIN");
pub const ROLE_MANAGER: Symbol = symbol_short!("MANAGER");
pub const ROLE_OPERATOR: Symbol = symbol_short!("OPERATOR");

pub const PERM_MANAGE_ROLES: Symbol = symbol_short!("MANAGE_ROLES");
pub const PERM_MANAGE_PERMISSIONS: Symbol = symbol_short!("MANAGE_PERMS");
pub const PERM_MANAGE_RESOURCES: Symbol = symbol_short!("MANAGE_RES");
pub const PERM_USE_RESOURCES: Symbol = symbol_short!("USE_RES");

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    RoleMembers(Symbol),
    RolePermissions(Symbol),
    Initialized,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct HierarchicalAccessControlContract;

#[contractimpl]
impl HierarchicalAccessControlContract {
    /// Initialize the contract with an initial admin.
    pub fn initialize(env: Env, initial_admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Already initialized");
        }

        // Grant ADMIN to initial admin
        let mut admin_members: Vec<Address> = Vec::new(&env);
        admin_members.push_back(initial_admin.clone());
        env.storage()
            .instance()
            .set(&DataKey::RoleMembers(ROLE_ADMIN), &admin_members);

        // Assign default permissions to roles
        let mut admin_perms: Vec<Symbol> = Vec::new(&env);
        admin_perms.push_back(PERM_MANAGE_ROLES);
        admin_perms.push_back(PERM_MANAGE_PERMISSIONS);
        admin_perms.push_back(PERM_MANAGE_RESOURCES);
        admin_perms.push_back(PERM_USE_RESOURCES);
        env.storage()
            .instance()
            .set(&DataKey::RolePermissions(ROLE_ADMIN), &admin_perms);

        let mut manager_perms: Vec<Symbol> = Vec::new(&env);
        manager_perms.push_back(PERM_MANAGE_RESOURCES);
        manager_perms.push_back(PERM_USE_RESOURCES);
        env.storage()
            .instance()
            .set(&DataKey::RolePermissions(ROLE_MANAGER), &manager_perms);

        let mut operator_perms: Vec<Symbol> = Vec::new(&env);
        operator_perms.push_back(PERM_USE_RESOURCES);
        env.storage()
            .instance()
            .set(&DataKey::RolePermissions(ROLE_OPERATOR), &operator_perms);

        // Mark as initialized
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Emit event
        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_GRANT, ROLE_ADMIN),
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

    /// Grant a role to an account. Caller must have MANAGE_ROLES permission.
    pub fn grant_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_MANAGE_ROLES);

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
            (CONTRACT_NS, ACTION_ROLE_GRANT, role.clone()),
            RoleGrantedEventData {
                role,
                account,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Revoke a role from an account. Caller must have MANAGE_ROLES permission.
    pub fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_MANAGE_ROLES);

        let key = DataKey::RoleMembers(role.clone());
        let members: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut updated: Vec<Address> = Vec::new(&env);
        for m in members.iter() {
            if m != account {
                updated.push_back(m);
            }
        }
        env.storage().instance().set(&key, &updated);

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_REVOKE, role.clone()),
            RoleRevokedEventData {
                role,
                account,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Renounce a role the caller holds.
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
            (CONTRACT_NS, ACTION_ROLE_REVOKE, role.clone()),
            RoleRevokedEventData {
                role,
                account: caller.clone(),
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Permission management
    // -----------------------------------------------------------------------

    /// Grant a permission to a role. Caller must have MANAGE_PERMISSIONS permission.
    pub fn grant_permission(env: Env, caller: Address, permission: Symbol, role: Symbol) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_MANAGE_PERMISSIONS);

        let key = DataKey::RolePermissions(role.clone());
        let mut perms: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        if !perms.contains(&permission) {
            perms.push_back(permission.clone());
            env.storage().instance().set(&key, &perms);
        }

        env.events().publish(
            (CONTRACT_NS, ACTION_PERM_GRANT, role.clone()),
            PermissionGrantedEventData {
                permission,
                role,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Revoke a permission from a role. Caller must have MANAGE_PERMISSIONS permission.
    pub fn revoke_permission(env: Env, caller: Address, permission: Symbol, role: Symbol) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_MANAGE_PERMISSIONS);

        let key = DataKey::RolePermissions(role.clone());
        let perms: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut updated: Vec<Symbol> = Vec::new(&env);
        for p in perms.iter() {
            if p != permission {
                updated.push_back(p);
            }
        }
        env.storage().instance().set(&key, &updated);

        env.events().publish(
            (CONTRACT_NS, ACTION_PERM_REVOKE, role.clone()),
            PermissionRevokedEventData {
                permission,
                role,
                sender: caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Check if an account has a specific role.
    pub fn has_role(env: Env, role: Symbol, account: Address) -> bool {
        Self::check_role(&env, &account, role)
    }

    /// Get all members of a role.
    pub fn get_role_members(env: Env, role: Symbol) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::RoleMembers(role))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Check if a role has a specific permission.
    pub fn role_has_permission(env: Env, role: Symbol, permission: Symbol) -> bool {
        let perms: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::RolePermissions(role))
            .unwrap_or_else(|| Vec::new(&env));
        perms.contains(&permission)
    }

    /// Get all permissions of a role.
    pub fn get_role_permissions(env: Env, role: Symbol) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&DataKey::RolePermissions(role))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Check if an account has a specific permission (through any of their roles).
    pub fn account_has_permission(env: Env, account: Address, permission: Symbol) -> bool {
        Self::check_account_permission(&env, &account, permission)
    }

    // -----------------------------------------------------------------------
    // Protected operations
    // -----------------------------------------------------------------------

    /// Operation requiring MANAGE_RESOURCES permission.
    pub fn manage_resource(env: Env, caller: Address, _resource_id: Symbol) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_MANAGE_RESOURCES);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, PERM_MANAGE_RESOURCES),
            ProtectedCallEventData {
                permission: PERM_MANAGE_RESOURCES,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Operation requiring USE_RESOURCES permission.
    pub fn use_resource(env: Env, caller: Address, _resource_id: Symbol) {
        caller.require_auth();
        Self::require_permission(&env, &caller, PERM_USE_RESOURCES);

        env.events().publish(
            (CONTRACT_NS, ACTION_CALL, PERM_USE_RESOURCES),
            ProtectedCallEventData {
                permission: PERM_USE_RESOURCES,
                caller,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn check_role(env: &Env, account: &Address, role: Symbol) -> bool {
        let members: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RoleMembers(role))
            .unwrap_or_else(|| Vec::new(&env));
        members.contains(account)
    }

    fn check_account_permission(env: &Env, account: &Address, permission: Symbol) -> bool {
        // Check all roles in order of hierarchy (highest first)
        let roles = [ROLE_ADMIN, ROLE_MANAGER, ROLE_OPERATOR];
        for role in roles.iter() {
            if Self::check_role(env, account, *role) {
                let perms: Vec<Symbol> = env
                    .storage()
                    .instance()
                    .get(&DataKey::RolePermissions(*role))
                    .unwrap_or_else(|| Vec::new(&env));
                if perms.contains(&permission) {
                    return true;
                }
            }
        }
        false
    }

    fn require_permission(env: &Env, account: &Address, permission: Symbol) {
        if !Self::check_account_permission(env, account, permission) {
            panic!("Caller does not have required permission");
        }
    }
}

#[cfg(test)]
mod test;
