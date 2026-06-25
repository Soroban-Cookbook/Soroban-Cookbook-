#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, vec, Address, Env, IntoVal, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Entered,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    ReentrancyError = 1,
}

#[contract]
pub struct ReentrancyGuardContract;

#[contractimpl]
impl ReentrancyGuardContract {
    pub fn init(env: Env) {
        env.storage().instance().set(&DataKey::Entered, &false);
    }

    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), ContractError> {
        from.require_auth();
        
        if Self::is_entered(&env) {
            return Err(ContractError::ReentrancyError);
        }
        env.storage().instance().set(&DataKey::Entered, &true);

        let mut balance: i128 = env.storage().persistent().get(&DataKey::Balance(from.clone())).unwrap_or(0);
        balance += amount;
        env.storage().persistent().set(&DataKey::Balance(from), &balance);
        
        env.storage().instance().set(&DataKey::Entered, &false);
        Ok(())
    }

    pub fn withdraw(env: Env, to: Address, amount: i128, target_contract: Address) -> Result<(), ContractError> {
        to.require_auth();

        if Self::is_entered(&env) {
            return Err(ContractError::ReentrancyError);
        }
        env.storage().instance().set(&DataKey::Entered, &true);

        let mut balance: i128 = env.storage().persistent().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        if balance >= amount {
            balance -= amount;
            env.storage().persistent().set(&DataKey::Balance(to.clone()), &balance);

            // Cross-contract call which opens up a reentrancy vector
            env.invoke_contract::<()>(
                &target_contract,
                &Symbol::new(&env, "receive_funds"),
                vec![&env, to.into_val(&env), amount.into_val(&env)],
            );
        }

        env.storage().instance().set(&DataKey::Entered, &false);
        Ok(())
    }

    pub fn get_balance(env: Env, user: Address) -> Result<i128, ContractError> {
        // Read-only reentrancy guard
        if Self::is_entered(&env) {
            return Err(ContractError::ReentrancyError);
        }
        
        Ok(env.storage().persistent().get(&DataKey::Balance(user)).unwrap_or(0))
    }

    fn is_entered(env: &Env) -> bool {
        env.storage().instance().get(&DataKey::Entered).unwrap_or(false)
    }
}

#[cfg(test)]
mod test;
