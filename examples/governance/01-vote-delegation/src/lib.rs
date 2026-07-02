//! # Vote Delegation Contract
//!
//! This contract demonstrates a voting delegation mechanism for governance.
//! Users can delegate their voting power to another address. Delegation can
//! be chained (e.g., A delegates to B, B delegates to C), up to a maximum
//! configured depth to prevent excessive gas/recursion limits.
//!
//! The contract includes:
//! - Base balance settings by an administrator.
//! - Dynamic voting power calculation traversing delegations.
//! - Safety checks: loop/cycle detection, self-delegation prevention, and depth bounds.
//! - Clean event emissions matching the Cookbook events standard.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

/// Maximum delegation depth allowed to prevent gas exhaustion.
const MAX_DEPTH: u32 = 5;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DelegationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 3,
    SelfDelegation = 4,
    DelegationCycle = 5,
    MaxDelegationDepthExceeded = 6,
    NegativeAmount = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Delegation(Address),
    Delegators(Address),
}

#[contract]
pub struct VoteDelegationContract;

#[contractimpl]
impl VoteDelegationContract {
    /// Initialize the contract and set the administrator address.
    pub fn init(env: Env, admin: Address) -> Result<(), DelegationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(DelegationError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().extend_ttl(10000, 10000);
        Ok(())
    }

    /// Set the base voting weight/balance for a specific user.
    /// Can only be called by the admin.
    pub fn set_balance(
        env: Env,
        admin: Address,
        user: Address,
        amount: i128,
    ) -> Result<(), DelegationError> {
        // Authenticate administrator
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DelegationError::NotInitialized)?;

        if admin != stored_admin {
            return Err(DelegationError::NotAdmin);
        }

        if amount < 0 {
            return Err(DelegationError::NegativeAmount);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), &amount);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(user.clone()), 5000, 10000);

        // Emit set_balance event
        env.events()
            .publish((symbol_short!("set_bal"), user.clone()), amount);

        Ok(())
    }

    /// Delegate the caller's voting power to another address.
    pub fn delegate(env: Env, from: Address, to: Address) -> Result<(), DelegationError> {
        // Authenticate the delegating address
        from.require_auth();

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(DelegationError::NotInitialized);
        }

        if from == to {
            return Err(DelegationError::SelfDelegation);
        }

        // Cycle check: Walk from 'to' to see if it ever leads back to 'from'
        let mut current = to.clone();
        let mut forward_depth = 1;
        loop {
            if current == from {
                return Err(DelegationError::DelegationCycle);
            }

            // Check if current has delegated to someone else
            if let Some(next) = env
                .storage()
                .persistent()
                .get::<_, Address>(&DataKey::Delegation(current.clone()))
            {
                current = next;
                forward_depth += 1;
            } else {
                break;
            }
        }

        // Calculate max back depth of 'from''s delegators tree
        let back_depth = Self::get_max_back_depth(&env, from.clone());

        // Check if the total combined depth exceeds MAX_DEPTH
        if back_depth + forward_depth > MAX_DEPTH {
            return Err(DelegationError::MaxDelegationDepthExceeded);
        }

        // If 'from' already has a delegate, clean up their record first
        if let Some(old_to) = env
            .storage()
            .persistent()
            .get::<_, Address>(&DataKey::Delegation(from.clone()))
        {
            Self::remove_delegator_from_list(&env, &old_to, &from);
        }

        // Set delegation record
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(from.clone()), &to);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Delegation(from.clone()), 5000, 10000);

        // Add 'from' to the delegators list of 'to'
        Self::add_delegator_to_list(&env, &to, &from);

        // Emit delegation event
        env.events().publish(
            (symbol_short!("delegate"), from, to),
            back_depth + forward_depth,
        );

        Ok(())
    }

    /// Remove the delegation for the caller, restoring their voting power.
    pub fn undelegate(env: Env, from: Address) -> Result<(), DelegationError> {
        // Authenticate caller
        from.require_auth();

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(DelegationError::NotInitialized);
        }

        let delegation_key = DataKey::Delegation(from.clone());
        if let Some(old_to) = env
            .storage()
            .persistent()
            .get::<_, Address>(&delegation_key)
        {
            // Clean up the delegators list of the old delegate
            Self::remove_delegator_from_list(&env, &old_to, &from);

            // Remove the delegation record
            env.storage().persistent().remove(&delegation_key);

            // Emit undelegate event
            env.events()
                .publish((symbol_short!("undel"), from, old_to), ());
        }

        Ok(())
    }

    /// Get the active voting power of a user.
    /// If the user has delegated their voting power, this returns 0.
    /// Otherwise, it returns their own base balance plus the sum of all delegated power.
    pub fn get_voting_power(env: Env, user: Address) -> i128 {
        if !env.storage().instance().has(&DataKey::Admin) {
            return 0;
        }

        // If the user has delegated their vote, they have 0 active voting power.
        if env
            .storage()
            .persistent()
            .has(&DataKey::Delegation(user.clone()))
        {
            return 0;
        }

        // Recursively compute power
        Self::calculate_power_rec(&env, user, 0)
    }

    /// Query the current delegate of a user.
    pub fn get_delegate(env: Env, user: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get::<_, Address>(&DataKey::Delegation(user))
    }

    /// Query the base balance of a user.
    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(user))
            .unwrap_or(0)
    }
}

// Internal Helper Functions
impl VoteDelegationContract {
    fn add_delegator_to_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = DataKey::Delegators(delegatee.clone());
        let mut delegators = env
            .storage()
            .persistent()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));

        if !delegators.contains(delegator) {
            delegators.push_back(delegator.clone());
            env.storage().persistent().set(&key, &delegators);
            env.storage().persistent().extend_ttl(&key, 5000, 10000);
        }
    }

    fn remove_delegator_from_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = DataKey::Delegators(delegatee.clone());
        if let Some(mut delegators) = env.storage().persistent().get::<_, Vec<Address>>(&key) {
            if let Some(index) = delegators.first_index_of(delegator) {
                delegators.remove(index);
                if delegators.is_empty() {
                    env.storage().persistent().remove(&key);
                } else {
                    env.storage().persistent().set(&key, &delegators);
                    env.storage().persistent().extend_ttl(&key, 5000, 10000);
                }
            }
        }
    }

    fn calculate_power_rec(env: &Env, current: Address, depth: u32) -> i128 {
        if depth > MAX_DEPTH {
            return 0;
        }

        let base_balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(current.clone()))
            .unwrap_or(0);

        let mut total_power = base_balance;

        let key = DataKey::Delegators(current);
        if let Some(delegators) = env.storage().persistent().get::<_, Vec<Address>>(&key) {
            for delegator in delegators.iter() {
                total_power += Self::calculate_power_rec(env, delegator, depth + 1);
            }
        }

        total_power
    }

    fn get_max_back_depth(env: &Env, current: Address) -> u32 {
        let mut max_depth = 0;
        let key = DataKey::Delegators(current);
        if let Some(delegators) = env.storage().persistent().get::<_, Vec<Address>>(&key) {
            for delegator in delegators.iter() {
                let d = Self::get_max_back_depth(env, delegator) + 1;
                if d > max_depth {
                    max_depth = d;
                }
            }
        }
        max_depth
    }
}

#[cfg(test)]
mod test;
