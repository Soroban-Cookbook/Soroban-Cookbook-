#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

/// Custom error types for error recovery patterns
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InsufficientBalance = 1,
    InvalidAmount = 2,
    Unauthorized = 3,
    TransferFailed = 4,
    ServiceUnavailable = 5,
    ValidationFailed = 6,
    RateLimitExceeded = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferResult {
    pub success: bool,
    pub amount_transferred: i128,
    pub fallback_used: bool,
}

#[contract]
pub struct ErrorRecoveryContract;

#[contractimpl]
impl ErrorRecoveryContract {
    /// Try-catch pattern: Attempt operation and handle errors gracefully
    /// Returns Result type allowing caller to handle errors
    pub fn try_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<TransferResult, Error> {
        from.require_auth();

        // Validation
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Check balance
        let balance = Self::get_balance(env.clone(), from.clone());
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Execute transfer
        Self::set_balance(env.clone(), from.clone(), balance - amount);
        let to_balance = Self::get_balance(env.clone(), to.clone());
        Self::set_balance(env.clone(), to.clone(), to_balance + amount);

        Ok(TransferResult {
            success: true,
            amount_transferred: amount,
            fallback_used: false,
        })
    }

    /// Fallback logic: Try primary operation, fall back to alternative on failure
    pub fn transfer_with_fallback(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        fallback_amount: i128,
    ) -> Result<TransferResult, Error> {
        from.require_auth();

        // Try primary transfer
        match Self::try_transfer(env.clone(), from.clone(), to.clone(), amount) {
            Ok(result) => Ok(result),
            Err(Error::InsufficientBalance) => {
                // Fallback: try with smaller amount
                if fallback_amount > 0 && fallback_amount < amount {
                    match Self::try_transfer(env, from, to, fallback_amount) {
                        Ok(_) => Ok(TransferResult {
                            success: true,
                            amount_transferred: fallback_amount,
                            fallback_used: true,
                        }),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(Error::InsufficientBalance)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Graceful degradation: Return partial success instead of complete failure
    pub fn batch_transfer(
        env: Env,
        from: Address,
        transfers: Vec<(Address, i128)>,
    ) -> Vec<Result<i128, Error>> {
        from.require_auth();

        let mut results = Vec::new(&env);

        for i in 0..transfers.len() {
            let (to, amount) = transfers.get(i).unwrap();

            // Try each transfer independently
            let result = Self::try_transfer(env.clone(), from.clone(), to, amount);

            match result {
                Ok(_) => results.push_back(Ok(amount)),
                Err(e) => results.push_back(Err(e)),
            }
        }

        results
    }

    /// Transaction rollback: Validate all operations before executing any
    pub fn atomic_batch_transfer(
        env: Env,
        from: Address,
        transfers: Vec<(Address, i128)>,
    ) -> Result<i128, Error> {
        from.require_auth();

        // Phase 1: Validation - check all transfers are valid
        let mut total_amount: i128 = 0;

        for i in 0..transfers.len() {
            let (_, amount) = transfers.get(i).unwrap();

            if amount <= 0 {
                return Err(Error::InvalidAmount);
            }

            total_amount = total_amount
                .checked_add(amount)
                .ok_or(Error::ValidationFailed)?;
        }

        // Check total balance
        let balance = Self::get_balance(env.clone(), from.clone());
        if balance < total_amount {
            return Err(Error::InsufficientBalance);
        }

        // Phase 2: Execution - all validations passed, execute atomically
        let new_from_balance = balance - total_amount;
        Self::set_balance(env.clone(), from, new_from_balance);

        for i in 0..transfers.len() {
            let (to, amount) = transfers.get(i).unwrap();
            let to_balance = Self::get_balance(env.clone(), to.clone());
            Self::set_balance(env.clone(), to, to_balance + amount);
        }

        Ok(total_amount)
    }

    /// Validation with detailed error reporting
    pub fn validate_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<bool, Error> {
        // Check amount validity
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Check balance
        let balance = Self::get_balance(env.clone(), from.clone());
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Check addresses are different
        if from == to {
            return Err(Error::ValidationFailed);
        }

        Ok(true)
    }

    /// Safe operation with multiple validation layers
    pub fn safe_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<TransferResult, Error> {
        from.require_auth();

        // Layer 1: Pre-validation
        Self::validate_transfer(env.clone(), from.clone(), to.clone(), amount)?;

        // Layer 2: Rate limiting check (simplified)
        let last_transfer = Self::get_last_transfer_time(env.clone(), from.clone());
        let current_time = env.ledger().timestamp();

        if current_time < last_transfer + 10 {
            return Err(Error::RateLimitExceeded);
        }

        // Layer 3: Execute with error handling
        let result = Self::try_transfer(env.clone(), from.clone(), to, amount)?;

        // Layer 4: Post-execution update
        Self::set_last_transfer_time(env, from, current_time);

        Ok(result)
    }

    /// Recoverable operation: Try operation, log error, return safe default
    pub fn get_balance_or_default(env: Env, account: Address) -> i128 {
        // Try to get balance, return 0 if not found
        Self::get_balance(env, account)
    }

    // Helper functions for storage
    fn get_balance(env: Env, account: Address) -> i128 {
        env.storage().persistent().get(&account).unwrap_or(0)
    }

    fn set_balance(env: Env, account: Address, amount: i128) {
        env.storage().persistent().set(&account, &amount);
    }

    fn get_last_transfer_time(env: Env, account: Address) -> u64 {
        let key = (account, 1u32); // 1 for timestamp key
        env.storage().temporary().get(&key).unwrap_or(0)
    }

    fn set_last_transfer_time(env: Env, account: Address, time: u64) {
        let key = (account, 1u32);
        env.storage().temporary().set(&key, &time);
    }
}

mod test;
