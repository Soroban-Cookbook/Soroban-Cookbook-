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
#[derive(Clone)]
pub struct TransferEventData {
    pub amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct ApprovalEventData {
    pub amount: i128,
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
}

const EVENT_NAMESPACE: Symbol = symbol_short!("events");
const EVENT_TRANSFER: Symbol = symbol_short!("transfer");
const EVENT_APPROVE: Symbol = symbol_short!("approve");

#[contract]
pub struct Sep41Token;

#[contractimpl]
impl Sep41Token {
    /// Initialize the token contract once.
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

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &initial_supply);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(admin.clone()), &initial_supply);

        Ok(())
    }

    /// Transfer tokens from one account to another.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        from.require_auth();
        ensure_initialized(&env)?;
        require_positive(amount)?;

        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let to_balance = read_balance(&env, &to);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(TokenError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);

        publish_transfer(&env, from, to, amount);
        Ok(())
    }

    /// Approve a spender to transfer tokens on behalf of the owner.
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        owner.require_auth();
        ensure_initialized(&env)?;
        if amount < 0 {
            return Err(TokenError::InvalidAmount);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);

        publish_approval(&env, owner, spender, amount);

        Ok(())
    }

    /// Make a transfer using an allowance previously granted by the owner.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        owner: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        spender.require_auth();
        require_positive(amount)?;
        ensure_initialized(&env)?;

        let allowance = read_allowance(&env, &owner, &spender);
        if allowance < amount {
            return Err(TokenError::AllowanceExceeded);
        }

        let owner_balance = read_balance(&env, &owner);
        if owner_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let to_balance = read_balance(&env, &to);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(TokenError::ArithmeticOverflow)?;

        env.storage().persistent().set(
            &DataKey::Allowance(owner.clone(), spender.clone()),
            &(allowance - amount),
        );
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &(owner_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);

        publish_transfer(&env, owner, to, amount);
        Ok(())
    }

    /// Return a user's token balance.
    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    /// Return the remaining allowance for a spender.
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        read_allowance(&env, &owner, &spender)
    }

    /// Return the total token supply.
    pub fn total_supply(env: Env) -> Result<i128, TokenError> {
        ensure_initialized(&env)?;
        Ok(read_total_supply(&env))
    }

    /// Return the token name.
    pub fn name(env: Env) -> Result<String, TokenError> {
        read_name(&env)
    }

    /// Return the token symbol.
    pub fn symbol(env: Env) -> Result<Symbol, TokenError> {
        read_symbol(&env)
    }

    /// Return the token decimals.
    pub fn decimals(env: Env) -> Result<u32, TokenError> {
        read_decimals(&env)
    }

    /// Return the admin/issuer address.
    pub fn admin(env: Env) -> Result<Address, TokenError> {
        read_admin(&env)
    }

    /// Mint new tokens to an account. Only admin may mint.
    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<i128, TokenError> {
        admin.require_auth();
        if admin != read_admin(&env)? {
            return Err(TokenError::Unauthorized);
        }
        require_positive(amount)?;

        let to_balance = read_balance(&env, &to);
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(TokenError::ArithmeticOverflow)?;
        let total_supply = read_total_supply(&env);
        let new_supply = total_supply
            .checked_add(amount)
            .ok_or(TokenError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        publish_transfer(&env, env.current_contract_address(), to, amount);
        Ok(new_to_balance)
    }

    /// Burn tokens from the caller's account.
    pub fn burn(env: Env, owner: Address, amount: i128) -> Result<i128, TokenError> {
        owner.require_auth();
        require_positive(amount)?;
        ensure_initialized(&env)?;

        let owner_balance = read_balance(&env, &owner);
        if owner_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let total_supply = read_total_supply(&env);
        let new_owner_balance = owner_balance - amount;
        let new_supply = total_supply - amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &new_owner_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);

        publish_transfer(&env, owner, env.current_contract_address(), amount);
        Ok(new_owner_balance)
    }
}

fn publish_transfer(env: &Env, from: Address, to: Address, amount: i128) {
    env.events().publish(
        (EVENT_NAMESPACE, EVENT_TRANSFER, from, to),
        TransferEventData { amount },
    );
}

fn publish_approval(env: &Env, owner: Address, spender: Address, amount: i128) {
    env.events().publish(
        (EVENT_NAMESPACE, EVENT_APPROVE, owner, spender),
        ApprovalEventData { amount },
    );
}

fn require_positive(amount: i128) -> Result<(), TokenError> {
    if amount <= 0 {
        return Err(TokenError::InvalidAmount);
    }
    Ok(())
}

fn ensure_initialized(env: &Env) -> Result<(), TokenError> {
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(TokenError::NotInitialized)
    }
}

fn read_admin(env: &Env) -> Result<Address, TokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(TokenError::NotInitialized)
}

fn read_name(env: &Env) -> Result<String, TokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Name)
        .ok_or(TokenError::NotInitialized)
}

fn read_symbol(env: &Env) -> Result<Symbol, TokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Symbol)
        .ok_or(TokenError::NotInitialized)
}

fn read_decimals(env: &Env) -> Result<u32, TokenError> {
    env.storage()
        .instance()
        .get(&DataKey::Decimals)
        .ok_or(TokenError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn read_allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(0)
}
