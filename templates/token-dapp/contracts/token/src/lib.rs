#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TotalSupply,
    Name,
    Symbol,
    Decimals,
    Balance(Address),
    Allowance(Address, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    InvalidAmount = 5,
    ArithmeticOverflow = 6,
    AllowanceExceeded = 7,
    AllowanceExpired = 8,
}

const INSTANCE_BUMP_AMOUNT: u32 = 518_400; // ~30 days in ledgers (5s ledger time)
const INSTANCE_LIFETIME_THRESHOLD: u32 = 120_960; // ~7 days

const BALANCE_BUMP_AMOUNT: u32 = 518_400;
const BALANCE_LIFETIME_THRESHOLD: u32 = 120_960;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Initialize token parameters (Admin, Name, Symbol, Decimals, Initial Supply)
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: Symbol,
        decimals: u32,
        initial_supply: i128,
    ) -> Result<(), TokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TokenError::AlreadyInitialized);
        }
        if initial_supply < 0 {
            return Err(TokenError::InvalidAmount);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::TotalSupply, &initial_supply);

        if initial_supply > 0 {
            env.storage().persistent().set(&DataKey::Balance(admin.clone()), &initial_supply);
            env.storage().persistent().extend_ttl(
                &DataKey::Balance(admin.clone()),
                BALANCE_LIFETIME_THRESHOLD,
                BALANCE_BUMP_AMOUNT,
            );
        }

        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("init"), admin),
            (name, symbol, decimals, initial_supply),
        );

        Ok(())
    }

    /// Read user balance
    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let key = DataKey::Balance(id.clone());
        let bal: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        }
        bal
    }

    /// Read spending allowance
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let key = DataKey::Allowance(from, spender);
        if let Some(allowance_val) = env.storage().persistent().get::<DataKey, AllowanceValue>(&key) {
            if allowance_val.expiration_ledger >= env.ledger().sequence() {
                return allowance_val.amount;
            }
        }
        0
    }

    /// Transfer tokens from authenticated caller
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        from.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let from_key = DataKey::Balance(from.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);

        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }

        if from != to {
            let to_key = DataKey::Balance(to.clone());
            let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

            let new_from_bal = from_bal.checked_sub(amount).ok_or(TokenError::ArithmeticOverflow)?;
            let new_to_bal = to_bal.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;

            env.storage().persistent().set(&from_key, &new_from_bal);
            env.storage().persistent().set(&to_key, &new_to_bal);

            env.storage().persistent().extend_ttl(&from_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
            env.storage().persistent().extend_ttl(&to_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        }

        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("transfer"), from, to),
            amount,
        );

        Ok(())
    }

    /// Transfer tokens on behalf of another account using an active allowance
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        spender.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let allowance_key = DataKey::Allowance(from.clone(), spender.clone());
        let allowance_val = env
            .storage()
            .persistent()
            .get::<DataKey, AllowanceValue>(&allowance_key)
            .ok_or(TokenError::AllowanceExceeded)?;

        if allowance_val.expiration_ledger < env.ledger().sequence() {
            return Err(TokenError::AllowanceExpired);
        }

        if allowance_val.amount < amount {
            return Err(TokenError::AllowanceExceeded);
        }

        let from_key = DataKey::Balance(from.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);

        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let new_allowance = allowance_val.amount.checked_sub(amount).ok_or(TokenError::ArithmeticOverflow)?;
        env.storage().persistent().set(
            &allowance_key,
            &AllowanceValue {
                amount: new_allowance,
                expiration_ledger: allowance_val.expiration_ledger,
            },
        );

        if from != to {
            let to_key = DataKey::Balance(to.clone());
            let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

            let new_from_bal = from_bal.checked_sub(amount).ok_or(TokenError::ArithmeticOverflow)?;
            let new_to_bal = to_bal.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;

            env.storage().persistent().set(&from_key, &new_from_bal);
            env.storage().persistent().set(&to_key, &new_to_bal);

            env.storage().persistent().extend_ttl(&from_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
            env.storage().persistent().extend_ttl(&to_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        }

        env.storage().persistent().extend_ttl(&allowance_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("transfer"), from, to),
            amount,
        );

        Ok(())
    }

    /// Set spender allowance with an expiration ledger
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), TokenError> {
        from.require_auth();

        if amount < 0 {
            return Err(TokenError::InvalidAmount);
        }

        let key = DataKey::Allowance(from.clone(), spender.clone());
        if amount > 0 {
            env.storage().persistent().set(
                &key,
                &AllowanceValue {
                    amount,
                    expiration_ledger,
                },
            );
            env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        } else {
            env.storage().persistent().remove(&key);
        }

        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("approve"), from, spender),
            (amount, expiration_ledger),
        );

        Ok(())
    }

    /// Mint new tokens (Admin only)
    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(TokenError::NotInitialized)?;

        if admin != stored_admin {
            return Err(TokenError::Unauthorized);
        }

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let total_supply: i128 = env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0);
        let new_total_supply = total_supply.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;

        let to_key = DataKey::Balance(to.clone());
        let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
        let new_to_bal = to_bal.checked_add(amount).ok_or(TokenError::ArithmeticOverflow)?;

        env.storage().instance().set(&DataKey::TotalSupply, &new_total_supply);
        env.storage().persistent().set(&to_key, &new_to_bal);

        env.storage().persistent().extend_ttl(&to_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("mint"), admin, to),
            amount,
        );

        Ok(())
    }

    /// Burn tokens from caller account
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), TokenError> {
        from.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let from_key = DataKey::Balance(from.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);

        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let total_supply: i128 = env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0);
        let new_total_supply = total_supply.checked_sub(amount).ok_or(TokenError::ArithmeticOverflow)?;
        let new_from_bal = from_bal.checked_sub(amount).ok_or(TokenError::ArithmeticOverflow)?;

        env.storage().instance().set(&DataKey::TotalSupply, &new_total_supply);
        env.storage().persistent().set(&from_key, &new_from_bal);

        env.storage().persistent().extend_ttl(&from_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("burn"), from),
            amount,
        );

        Ok(())
    }

    /// Read token total supply
    pub fn total_supply(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0)
    }

    /// Read token name
    pub fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap()
    }

    /// Read token symbol
    pub fn symbol(env: Env) -> Symbol {
        env.storage().instance().get(&DataKey::Symbol).unwrap()
    }

    /// Read token decimals
    pub fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Decimals).unwrap_or(7)
    }
}

#[cfg(test)]
mod test;
