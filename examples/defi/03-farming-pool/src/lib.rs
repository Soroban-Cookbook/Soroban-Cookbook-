#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Pool(u32),
    PoolsList,
    UserInfo(u32, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolInfo {
    pub staking_token: Address,
    pub reward_token: Address,
    pub reward_rate: i128, // Rewards per ledger
    pub weight: u32,       // Relative weight of the pool
    pub last_reward_ledger: u32,
    pub acc_reward_per_share: i128, // Accumulated rewards per share, scaled
    pub total_staked: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserInfo {
    pub amount: i128,
    pub reward_debt: i128,
}

#[contract]
pub struct FarmingPoolContract;

#[contractimpl]
impl FarmingPoolContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PoolsList, &Vec::<u32>::new(&env));
    }

    /// Add a new farming pool. Only admin can call this.
    pub fn add_pool(
        env: Env,
        admin: Address,
        staking_token: Address,
        reward_token: Address,
        reward_rate: i128,
        weight: u32,
    ) -> u32 {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let mut pools: Vec<u32> = env
            .storage()
            .instance()
            .get(&DataKey::PoolsList)
            .unwrap_or_else(|| Vec::new(&env));

        let pool_id = pools.len();
        pools.push_back(pool_id);

        let pool_info = PoolInfo {
            staking_token,
            reward_token,
            reward_rate,
            weight,
            last_reward_ledger: env.ledger().sequence(),
            acc_reward_per_share: 0,
            total_staked: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pool(pool_id), &pool_info);
        env.storage().instance().set(&DataKey::PoolsList, &pools);

        pool_id
    }

    /// Remove a farming pool. Only admin can call this.
    /// Note: This simple implementation just removes it from the list.
    /// In production, you'd want to handle user funds properly.
    pub fn remove_pool(env: Env, admin: Address, pool_id: u32) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let pools: Vec<u32> = env
            .storage()
            .instance()
            .get(&DataKey::PoolsList)
            .unwrap_or_else(|| Vec::new(&env));

        let mut new_pools = Vec::new(&env);
        for p_id in pools.iter() {
            if p_id != pool_id {
                new_pools.push_back(p_id);
            }
        }

        env.storage()
            .instance()
            .set(&DataKey::PoolsList, &new_pools);
        env.storage().instance().remove(&DataKey::Pool(pool_id));
    }

    /// Adjust the reward rate for a pool. Only admin can call this.
    pub fn set_reward_rate(env: Env, admin: Address, pool_id: u32, new_rate: i128) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let mut pool = Self::get_pool(&env, pool_id);
        Self::update_pool(&env, &mut pool);
        pool.reward_rate = new_rate;
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);
    }

    /// Adjust the weight for a pool. Only admin can call this.
    pub fn set_pool_weight(env: Env, admin: Address, pool_id: u32, new_weight: u32) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let mut pool = Self::get_pool(&env, pool_id);
        Self::update_pool(&env, &mut pool);
        pool.weight = new_weight;
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);
    }

    /// Emergency withdraw tokens from the contract. Only admin can call this.
    pub fn emergency_withdraw_admin(
        env: Env,
        admin: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        // Transfer tokens out of the contract
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }

    /// Deposit staking tokens into a pool.
    pub fn deposit(env: Env, user: Address, pool_id: u32, amount: i128) {
        user.require_auth();

        let mut pool = Self::get_pool(&env, pool_id);
        let mut user_info = Self::get_user_info(&env, pool_id, &user);

        Self::update_pool(&env, &mut pool);

        if user_info.amount > 0 {
            let pending = (user_info.amount * pool.acc_reward_per_share / 1_000_000_000_000)
                - user_info.reward_debt;
            if pending > 0 {
                Self::safe_reward_transfer(&env, &pool.reward_token, &user, pending);
            }
        }

        if amount > 0 {
            let client = soroban_sdk::token::Client::new(&env, &pool.staking_token);
            client.transfer(&user, &env.current_contract_address(), &amount);
            user_info.amount += amount;
            pool.total_staked += amount;
        }

        user_info.reward_debt = user_info.amount * pool.acc_reward_per_share / 1_000_000_000_000;

        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);
        env.storage()
            .persistent()
            .set(&DataKey::UserInfo(pool_id, user.clone()), &user_info);
    }

    /// Withdraw staking tokens from a pool.
    pub fn withdraw(env: Env, user: Address, pool_id: u32, amount: i128) {
        user.require_auth();

        let mut pool = Self::get_pool(&env, pool_id);
        let mut user_info = Self::get_user_info(&env, pool_id, &user);

        if user_info.amount < amount {
            panic!("Insufficient staked amount");
        }

        Self::update_pool(&env, &mut pool);

        let pending = (user_info.amount * pool.acc_reward_per_share / 1_000_000_000_000)
            - user_info.reward_debt;
        if pending > 0 {
            Self::safe_reward_transfer(&env, &pool.reward_token, &user, pending);
        }

        if amount > 0 {
            user_info.amount -= amount;
            pool.total_staked -= amount;
            let client = soroban_sdk::token::Client::new(&env, &pool.staking_token);
            client.transfer(&env.current_contract_address(), &user, &amount);
        }

        user_info.reward_debt = user_info.amount * pool.acc_reward_per_share / 1_000_000_000_000;

        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);
        env.storage()
            .persistent()
            .set(&DataKey::UserInfo(pool_id, user.clone()), &user_info);
    }

    /// Helper to check if the caller is admin.
    fn check_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller {
            panic!("Unauthorized");
        }
    }

    /// Get pool info.
    fn get_pool(env: &Env, pool_id: u32) -> PoolInfo {
        env.storage()
            .instance()
            .get(&DataKey::Pool(pool_id))
            .expect("Pool not found")
    }

    /// Get user info.
    fn get_user_info(env: &Env, pool_id: u32, user: &Address) -> UserInfo {
        env.storage()
            .persistent()
            .get(&DataKey::UserInfo(pool_id, user.clone()))
            .unwrap_or(UserInfo {
                amount: 0,
                reward_debt: 0,
            })
    }

    /// Update pool rewards.
    fn update_pool(env: &Env, pool: &mut PoolInfo) {
        let current_ledger = env.ledger().sequence();
        if current_ledger <= pool.last_reward_ledger {
            return;
        }

        if pool.total_staked == 0 {
            pool.last_reward_ledger = current_ledger;
            return;
        }

        let ledger_count = (current_ledger - pool.last_reward_ledger) as i128;
        let reward = ledger_count * pool.reward_rate;

        // Scale by 1e12 for precision
        pool.acc_reward_per_share += reward * 1_000_000_000_000 / pool.total_staked;
        pool.last_reward_ledger = current_ledger;
    }

    /// Safely transfer rewards, ensuring the contract has enough balance.
    fn safe_reward_transfer(env: &Env, token: &Address, to: &Address, amount: i128) {
        let client = soroban_sdk::token::Client::new(env, token);
        let balance = client.balance(&env.current_contract_address());
        let transfer_amount = if amount > balance { balance } else { amount };
        if transfer_amount > 0 {
            client.transfer(&env.current_contract_address(), to, &transfer_amount);
        }
    }
}
