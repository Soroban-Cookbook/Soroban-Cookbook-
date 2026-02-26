#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, Symbol, Vec,
};

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

/// Unified Storage keys. 
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Role(Address),
    State,
    TimeLock,
    CooldownPeriod,
    LastAction(Address),
    Balance(Address),
    Allowance(Address, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    Unauthorized = 1,
    NotAdmin = 2,
    AlreadyInitialized = 3,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    // ==================== INITIALIZATION & ADMIN ====================

    /// Initializes the contract with the given admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), AuthError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AuthError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        // EVENT: Admin initialization
        env.events().publish((symbol_short!("init"),), admin.clone());

        Ok(())
    }

    /// Admin-only function pattern: Set a new admin
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), AuthError> {
        if let Some(stored_admin) = env.storage().instance().get::<DataKey, Address>(&DataKey::Admin) {
            if admin != stored_admin {
                return Err(AuthError::NotAdmin);
            }
            admin.require_auth();
        } else {
            admin.require_auth();
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        // EVENT: Admin change (Audit trail)
        env.events().publish((symbol_short!("admin"), symbol_short!("set")), (admin, new_admin));

        Ok(())
    }

    /// Get the current admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Admin-only action demonstration
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

        // EVENT: Admin action (Audit trail)
        env.events().publish((symbol_short!("admin"), symbol_short!("action")), admin.clone());
        
        Ok(value * 2)
    }

    // ==================== USER OPERATIONS ====================

    /// Basic authentication check
    pub fn check_auth(_env: Env, user: Address) -> bool {
        user.require_auth();
        true
    }

    pub fn secure_action(_env: Env, user: Address) {
        user.require_auth();
    }

    pub fn update_user_data(env: Env, user: Address, data: Symbol) -> bool {
        user.require_auth();
        env.storage().persistent().set(&user, &data);

        // EVENT: User data updated
        env.events().publish((symbol_short!("user"), symbol_short!("update")), user.clone());

        true
    }

    pub fn get_user_data(env: Env, user: Address) -> Option<Symbol> {
        env.storage().persistent().get(&user)
    }

    // ==================== TRANSFERS & BALANCES ====================

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        
        // EVENT: Transfer executed
        env.events().publish((symbol_short!("transfer"), from.clone(), to.clone()), amount);

        Ok(())
    }

    pub fn set_balance(env: Env, admin: Address, user: Address, amount: i128) -> Result<(), AuthError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AuthError::NotAdmin)?;
        
        if admin != stored_admin {
            return Err(AuthError::NotAdmin);
        }
        
        env.storage().persistent().set(&DataKey::Balance(user.clone()), &amount);

        // EVENT: Balance set by admin
        env.events().publish((symbol_short!("balance"), symbol_short!("set")), (user.clone(), amount));

        Ok(())
    }

    /// Get balance
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(user)).unwrap_or(0)
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128) -> Result<(), AuthError> {
        from.require_auth();
        env.storage().persistent().set(&DataKey::Allowance(from.clone(), spender.clone()), &amount);
        
        // EVENT: Allowance approved
        env.events().publish((symbol_short!("approve"), from.clone(), spender.clone()), amount);

        Ok(())
    }

    /// Transfer from allowance
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), AuthError> {
        spender.require_auth();
        
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        
        if allowance < amount {
            return Err(AuthError::Unauthorized);
        }
        
        let from_balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let to_balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage().persistent().set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        env.storage().persistent().set(&DataKey::Allowance(from.clone(), spender.clone()), &(allowance - amount));
        
        // EVENT: Transfer executed via allowance
        env.events().publish((symbol_short!("transfer"), from.clone(), to.clone()), amount);

        Ok(())
    }

    // ==================== UTILITIES ====================

    pub fn secure_operation(env: Env, user: Address, operation: Symbol) -> Result<Vec<Symbol>, AuthError> {
        user.require_auth();
        if operation == symbol_short!("invalid") {
            return Err(AuthError::Unauthorized);
        }
        let result = vec![&env, symbol_short!("success"), operation];
        Ok(result)
    }

    pub fn self_authenticate(_env: Env, self_address: Address) -> bool {
        self_address.require_auth();
        true
    }

    pub fn multi_sig_action(_env: Env, signers: Vec<Address>, value: u32) -> u32 {
        for signer in signers.iter() {
            signer.require_auth();
        }
        value + signers.len()
    }

    /// Emit event with authentication
    pub fn emit_event(env: Env, user: Address, message: Symbol) {
        user.require_auth();
        env.events().publish((symbol_short!("event"), user), message);
    }
}

mod test;