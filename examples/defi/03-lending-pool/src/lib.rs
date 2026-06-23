#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Map};

#[derive(Clone, Copy)]
#[contracttype]
pub struct UserPosition {
    pub deposit: i128,
    pub borrow: i128,
}

#[contract]
pub struct LendingPool;

#[contractimpl]
impl LendingPool {
    pub fn initialize(env: Env, base_rate: i128, kink_rate: i128, kink_utilization: i128) {
        if env
            .storage()
            .instance()
            .has(&soroban_sdk::Symbol::new(&env, "total_deposits"))
        {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "total_deposits"), &0i128);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "total_borrows"), &0i128);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "base_rate"), &base_rate);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "kink_rate"), &kink_rate);
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "kink_utilization"),
            &kink_utilization,
        );
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "positions"),
            &Map::<Address, UserPosition>::new(&env),
        );
    }

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, UserPosition> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(UserPosition {
            deposit: 0,
            borrow: 0,
        });
        position.deposit += amount;
        positions.set(user, position);

        let mut total_deposits: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_deposits"))
            .unwrap();
        total_deposits += amount;

        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "positions"), &positions);
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "total_deposits"),
            &total_deposits,
        );
    }

    pub fn withdraw(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, UserPosition> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(UserPosition {
            deposit: 0,
            borrow: 0,
        });
        if position.deposit < amount {
            panic!("insufficient deposit");
        }
        position.deposit -= amount;
        positions.set(user, position);

        let mut total_deposits: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_deposits"))
            .unwrap();
        total_deposits -= amount;

        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "positions"), &positions);
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "total_deposits"),
            &total_deposits,
        );
    }

    pub fn borrow(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let total_deposits: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_deposits"))
            .unwrap();
        let total_borrows: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_borrows"))
            .unwrap();
        if total_borrows + amount > total_deposits {
            panic!("insufficient liquidity");
        }

        let mut positions: Map<Address, UserPosition> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(UserPosition {
            deposit: 0,
            borrow: 0,
        });
        let max_borrow = position.deposit * 80 / 100;
        if position.borrow + amount > max_borrow {
            panic!("exceeds borrow limit");
        }

        position.borrow += amount;
        positions.set(user, position);

        let mut new_total_borrows = total_borrows + amount;
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "positions"), &positions);
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "total_borrows"),
            &new_total_borrows,
        );
    }

    pub fn repay(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, UserPosition> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(UserPosition {
            deposit: 0,
            borrow: 0,
        });
        let repay_amount = if amount > position.borrow {
            position.borrow
        } else {
            amount
        };
        position.borrow -= repay_amount;
        positions.set(user, position);

        let mut total_borrows: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_borrows"))
            .unwrap();
        total_borrows -= repay_amount;

        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "positions"), &positions);
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "total_borrows"),
            &total_borrows,
        );
    }

    pub fn get_utilization(env: Env) -> i128 {
        let total_deposits: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_deposits"))
            .unwrap();
        let total_borrows: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "total_borrows"))
            .unwrap();
        if total_deposits == 0 {
            0
        } else {
            total_borrows * 100 / total_deposits
        }
    }

    pub fn get_borrow_rate(env: Env) -> i128 {
        let utilization = Self::get_utilization(env.clone());
        let base_rate: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "base_rate"))
            .unwrap();
        let kink_rate: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "kink_rate"))
            .unwrap();
        let kink_utilization: i128 = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "kink_utilization"))
            .unwrap();

        if utilization <= kink_utilization {
            base_rate + (utilization * kink_rate / kink_utilization)
        } else {
            let excess_utilization = utilization - kink_utilization;
            base_rate + kink_rate + (excess_utilization * 200 / (100 - kink_utilization))
        }
    }

    pub fn get_user_position(env: Env, user: Address) -> UserPosition {
        let positions: Map<Address, UserPosition> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "positions"))
            .unwrap();
        positions.get(user).unwrap_or(UserPosition {
            deposit: 0,
            borrow: 0,
        })
    }
}

#[cfg(test)]
mod test;
