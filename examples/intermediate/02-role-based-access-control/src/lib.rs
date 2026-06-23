#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RbacError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InvalidRole = 3,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Role {
    User = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    UserRole(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleChangeEvent {
    pub operator: Address,
    pub account: Address,
    pub old_role: Role,
    pub new_role: Role,
}

const CONTRACT_NS: Symbol = symbol_short!("rbac");
const ACTION_ROLE_CHANGE: Symbol = symbol_short!("role_chg");

#[contract]
pub struct RoleBasedAccessControl;

#[contractimpl]
impl RoleBasedAccessControl {
    pub fn initialize(env: Env, owner: Address) -> Result<(), RbacError> {
        owner.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(RbacError::AlreadyInitialized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::UserRole(owner.clone()), &Role::Owner);
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, owner.clone()),
            RoleChangeEvent {
                operator: owner.clone(),
                account: owner,
                old_role: Role::User,
                new_role: Role::Owner,
            },
        );

        Ok(())
    }

    pub fn grant_role(
        env: Env,
        caller: Address,
        account: Address,
        role: Role,
    ) -> Result<(), RbacError> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_can_manage(&env, &caller, &role)?;

        let old_role = Self::get_role_internal(&env, &account);
        env.storage()
            .persistent()
            .set(&DataKey::UserRole(account.clone()), &role);

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, account.clone()),
            RoleChangeEvent {
                operator: caller,
                account,
                old_role,
                new_role: role,
            },
        );

        Ok(())
    }

    pub fn revoke_role(env: Env, caller: Address, account: Address) -> Result<(), RbacError> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let caller_role = Self::get_role_internal(&env, &caller);
        let target_role = Self::get_role_internal(&env, &account);

        if !Self::can_revoke(caller_role, target_role) {
            return Err(RbacError::Unauthorized);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::UserRole(account.clone()));

        env.events().publish(
            (CONTRACT_NS, ACTION_ROLE_CHANGE, account.clone()),
            RoleChangeEvent {
                operator: caller,
                account,
                old_role: target_role,
                new_role: Role::User,
            },
        );

        Ok(())
    }

    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        let user_role = Self::get_role_internal(&env, &account);
        user_role as u32 >= role as u32
    }

    pub fn require_role(env: Env, caller: Address, allowed: Vec<Role>) -> Result<(), RbacError> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let user_role = Self::get_role_internal(&env, &caller);
        for allowed_role in allowed.iter() {
            if user_role as u32 >= *allowed_role as u32 {
                return Ok(());
            }
        }
        Err(RbacError::Unauthorized)
    }

    pub fn admin_action(env: Env, caller: Address, value: u64) -> Result<u64, RbacError> {
        Self::require_role(
            env.clone(),
            caller.clone(),
            Vec::from_array(&env, [Role::Admin]),
        )?;
        Ok(value * 2)
    }

    fn require_initialized(env: &Env) -> Result<(), RbacError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(RbacError::Unauthorized)
        }
    }

    fn get_role_internal(env: &Env, account: &Address) -> Role {
        env.storage()
            .persistent()
            .get(&DataKey::UserRole(account.clone()))
            .unwrap_or(Role::User)
    }

    fn require_can_manage(env: &Env, caller: &Address, target: &Role) -> Result<(), RbacError> {
        let caller_role = Self::get_role_internal(env, caller);
        if Self::can_grant(caller_role, *target) {
            Ok(())
        } else {
            Err(RbacError::Unauthorized)
        }
    }

    fn can_grant(caller_role: Role, target_role: Role) -> bool {
        match caller_role {
            Role::Owner => true,
            Role::Admin => matches!(target_role, Role::User | Role::Moderator),
            _ => false,
        }
    }

    fn can_revoke(caller_role: Role, target_role: Role) -> bool {
        match caller_role {
            Role::Owner => true,
            Role::Admin => matches!(target_role, Role::User | Role::Moderator),
            _ => false,
        }
    }
}
