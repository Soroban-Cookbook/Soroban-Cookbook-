//! # Optimized Token Operations
//!
//! This module demonstrates best practices for optimizing token transfers and storage patterns
//! on the Soroban network. It includes two implementations:
//! - `standard`: Basic token wrapper (for comparison)
//! - `optimized`: Enhanced version with batched operations and efficient storage
//!
//! Key optimizations:
//! 1. **Batched Transfers**: Process multiple transfers in a single contract call
//! 2. **Efficient Storage Structures**: Use Maps and Vecs instead of individual storage keys
//! 3. **Reduced Storage Operations**: Minimize reads/writes through better data organization
//! 4. **Early Validation**: Check constraints before expensive operations

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token::TokenClient, vec, Address, Env,
    Map, Vec,
};

// ============================================================================
// Standard Implementation (Baseline for benchmarking)
// ============================================================================

#[cfg(any(not(target_arch = "wasm32"), test, feature = "testutils"))]
#[contracttype]
#[derive(Clone)]
pub enum StandardDataKey {
    Underlying,
    TotalSupply,
    Balance(Address),
}

#[cfg(any(not(target_arch = "wasm32"), test, feature = "testutils"))]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StandardError {
    NotInitialized = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    ArithmeticOverflow = 4,
}

#[cfg(any(not(target_arch = "wasm32"), test, feature = "testutils"))]
#[contract]
pub struct StandardTokenOps;

#[cfg(any(not(target_arch = "wasm32"), test, feature = "testutils"))]
#[contractimpl]
impl StandardTokenOps {
    pub fn initialize(env: Env, underlying: Address) -> Result<(), StandardError> {
        env.storage()
            .instance()
            .set(&StandardDataKey::Underlying, &underlying);
        env.storage()
            .instance()
            .set(&StandardDataKey::TotalSupply, &0i128);
        Ok(())
    }

    pub fn wrap(env: Env, user: Address, amount: i128) -> Result<i128, StandardError> {
        if amount <= 0 {
            return Err(StandardError::InvalidAmount);
        }

        let underlying_addr: Address = env
            .storage()
            .instance()
            .get(&StandardDataKey::Underlying)
            .ok_or(StandardError::NotInitialized)?;

        let old_balance: i128 = env
            .storage()
            .persistent()
            .get(&StandardDataKey::Balance(user.clone()))
            .unwrap_or(0);

        let new_balance = old_balance
            .checked_add(amount)
            .ok_or(StandardError::ArithmeticOverflow)?;

        let old_supply: i128 = env
            .storage()
            .instance()
            .get(&StandardDataKey::TotalSupply)
            .unwrap_or(0);

        let new_supply = old_supply
            .checked_add(amount)
            .ok_or(StandardError::ArithmeticOverflow)?;

        user.require_auth();

        let wrapper = env.current_contract_address();
        TokenClient::new(&env, &underlying_addr).transfer(&user, &wrapper, &amount);

        env.storage()
            .persistent()
            .set(&StandardDataKey::Balance(user), &new_balance);
        env.storage()
            .instance()
            .set(&StandardDataKey::TotalSupply, &new_supply);

        Ok(new_balance)
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&StandardDataKey::Balance(user))
            .unwrap_or(0)
    }
}

// ============================================================================
// Optimized Implementation
// ============================================================================

#[contracttype]
#[derive(Clone)]
pub enum OptimizedDataKey {
    Underlying,
    Balances,
}

#[contracttype]
#[derive(Clone)]
pub struct BatchTransfer {
    pub recipient: Address,
    pub amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OptimizedError {
    NotInitialized = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    ArithmeticOverflow = 4,
    InvalidBatch = 5,
}

#[contract]
pub struct OptimizedTokenOps;

#[contractimpl]
impl OptimizedTokenOps {
    /// Initialize once with the underlying token address.
    pub fn initialize(env: Env, underlying: Address) -> Result<(), OptimizedError> {
        if env.storage().instance().has(&OptimizedDataKey::Underlying) {
            return Err(OptimizedError::InvalidAmount);
        }

        env.storage()
            .instance()
            .set(&OptimizedDataKey::Underlying, &underlying);

        // Initialize balances map
        let empty_map: Map<Address, i128> = Map::new(&env);
        env.storage()
            .persistent()
            .set(&OptimizedDataKey::Balances, &empty_map);

        Ok(())
    }

    /// **Optimization 1**: Batch multiple transfers in a single call.
    /// This reduces the number of contract invocations and storage updates.
    ///
    /// # Arguments
    /// * `user` - The sender authorizing the transfers
    /// * `recipients` - Vector of BatchTransfer containing recipient address and amount
    ///
    /// # Example
    /// Transfer 100 tokens each to alice and bob:
    /// ```ignore
    /// batch_transfer(sender, vec![
    ///     BatchTransfer { recipient: alice, amount: 100 },
    ///     BatchTransfer { recipient: bob, amount: 100 },
    /// ])
    /// ```
    pub fn batch_transfer(
        env: Env,
        user: Address,
        recipients: Vec<BatchTransfer>,
    ) -> Result<(), OptimizedError> {
        if recipients.is_empty() {
            return Err(OptimizedError::InvalidBatch);
        }

        user.require_auth();

        // **Optimization 2**: Validate ALL inputs before making changes.
        // This prevents partial failures and reduces unnecessary storage ops.
        let mut total_amount: i128 = 0;
        for recipient_data in recipients.iter() {
            if recipient_data.amount <= 0 {
                return Err(OptimizedError::InvalidAmount);
            }
            total_amount = total_amount
                .checked_add(recipient_data.amount)
                .ok_or(OptimizedError::ArithmeticOverflow)?;
        }

        // Load balances map once (optimization 3: single storage read)
        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&OptimizedDataKey::Balances)
            .ok_or(OptimizedError::NotInitialized)?;

        let user_balance = balances.get(user.clone()).unwrap_or(0);
        if user_balance < total_amount {
            return Err(OptimizedError::InsufficientBalance);
        }

        // Update sender balance
        balances.set(user.clone(), user_balance - total_amount);

        // Update all recipient balances (optimization 4: batch updates)
        for recipient_data in recipients.iter() {
            let current_balance = balances.get(recipient_data.recipient.clone()).unwrap_or(0);
            let new_balance = current_balance
                .checked_add(recipient_data.amount)
                .ok_or(OptimizedError::ArithmeticOverflow)?;
            balances.set(recipient_data.recipient.clone(), new_balance);
        }

        // Write back the updated balances map once (optimization 5: single storage write)
        env.storage()
            .persistent()
            .set(&OptimizedDataKey::Balances, &balances);

        Ok(())
    }

    /// Single transfer operation optimized for gas efficiency.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), OptimizedError> {
        if amount <= 0 {
            return Err(OptimizedError::InvalidAmount);
        }

        from.require_auth();

        let mut balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&OptimizedDataKey::Balances)
            .ok_or(OptimizedError::NotInitialized)?;

        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            return Err(OptimizedError::InsufficientBalance);
        }

        let to_balance = balances.get(to.clone()).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(OptimizedError::ArithmeticOverflow)?;

        balances.set(from, from_balance - amount);
        balances.set(to, new_to_balance);

        env.storage()
            .persistent()
            .set(&OptimizedDataKey::Balances, &balances);

        Ok(())
    }

    /// Get balance for a user with efficient map lookup.
    pub fn balance(env: Env, user: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&OptimizedDataKey::Balances)
            .unwrap_or_else(|| Map::new(&env));

        balances.get(user).unwrap_or(0)
    }

    /// Get total supply by summing all balances (could be optimized with separate tracking).
    pub fn total_supply(env: Env) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&OptimizedDataKey::Balances)
            .unwrap_or_else(|| Map::new(&env));

        let mut total: i128 = 0;
        for entry in balances.iter() {
            total = total.checked_add(entry.1).unwrap_or(i128::MAX);
        }
        total
    }
}

#[cfg(test)]
mod tests;
