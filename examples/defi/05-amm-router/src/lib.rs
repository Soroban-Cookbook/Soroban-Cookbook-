#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Vec};

#[contracttype]
pub struct Pool {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: i128,
    pub reserve_b: i128,
}

#[contract]
pub struct AMMRouter;

#[contractimpl]
impl AMMRouter {
    pub fn initialize(env: Env) {
        if env
            .storage()
            .instance()
            .has(&soroban_sdk::Symbol::new(&env, "pools"))
        {
            panic!("already initialized");
        }
        env.storage().instance().set(
            &soroban_sdk::Symbol::new(&env, "pools"),
            &Map::<(Address, Address), Pool>::new(&env),
        );
    }

    pub fn add_pool(env: Env, pool: Pool) {
        let mut pools: Map<(Address, Address), Pool> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "pools"))
            .unwrap();
        let key = if pool.token_a < pool.token_b {
            (pool.token_a.clone(), pool.token_b.clone())
        } else {
            (pool.token_b.clone(), pool.token_a.clone())
        };
        pools.set(key, pool);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "pools"), &pools);
    }

    pub fn get_pool(env: Env, token_a: Address, token_b: Address) -> Option<Pool> {
        let pools: Map<(Address, Address), Pool> = env
            .storage()
            .instance()
            .get(&soroban_sdk::Symbol::new(&env, "pools"))
            .unwrap();
        let key = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        pools.get(key)
    }

    fn calculate_swap_output(reserve_in: i128, reserve_out: i128, amount_in: i128) -> i128 {
        let amount_in_with_fee = amount_in * 997;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
        numerator / denominator
    }

    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        user: Address,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> i128 {
        user.require_auth();

        if env.ledger().timestamp() > deadline {
            panic!("deadline exceeded");
        }

        if path.len() < 2 {
            panic!("invalid path");
        }

        let mut current_amount = amount_in;
        let mut current_token = path.get(0).unwrap();

        for i in 0..path.len() - 1 {
            let token_in = path.get(i).unwrap();
            let token_out = path.get(i + 1).unwrap();
            let pool = Self::get_pool(env.clone(), token_in.clone(), token_out.clone()).unwrap();

            let (reserve_in, reserve_out) = if pool.token_a == token_in {
                (pool.reserve_a, pool.reserve_b)
            } else {
                (pool.reserve_b, pool.reserve_a)
            };

            current_amount = Self::calculate_swap_output(reserve_in, reserve_out, current_amount);
            current_token = token_out;
        }

        if current_amount < amount_out_min {
            panic!("insufficient output amount");
        }

        current_amount
    }
}

#[cfg(test)]
mod test;
