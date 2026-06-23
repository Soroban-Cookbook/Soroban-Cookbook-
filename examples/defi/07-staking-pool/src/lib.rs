#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

const REWARD_PRECISION: i128 = 1_000_000_000_000_000_000;

#[contract]
pub struct StakingPoolContract;

#[contracttype]
pub enum DataKey {
    Owner,
    StakingToken,
    RewardToken,
    RewardRate,
    LastUpdateTime,
    RewardPerTokenStored,
    TotalSupply,
    Balance(Address),
    UserRewardPerTokenPaid(Address),
    Rewards(Address),
}

impl StakingPoolContract {
    fn require_owner(&self, env: &Env) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();
    }

    fn staking_token(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::StakingToken)
            .expect("staking token missing")
    }

    fn reward_token(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::RewardToken)
            .expect("reward token missing")
    }

    fn reward_rate(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RewardRate)
            .unwrap_or(0i128)
    }

    fn last_update_time(&self, env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastUpdateTime)
            .unwrap_or(env.ledger().timestamp())
    }

    fn reward_per_token_stored(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RewardPerTokenStored)
            .unwrap_or(0i128)
    }

    fn total_supply(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0i128)
    }

    fn balance_of(&self, env: &Env, account: &Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(account.clone()))
            .unwrap_or(0i128)
    }

    fn user_reward_per_token_paid(&self, env: &Env, account: &Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::UserRewardPerTokenPaid(account.clone()))
            .unwrap_or(0i128)
    }

    fn rewards(&self, env: &Env, account: &Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Rewards(account.clone()))
            .unwrap_or(0i128)
    }

    fn update_reward(&self, env: &Env, account: &Address) {
        let reward_per_token = self.reward_per_token(env);
        env.storage()
            .instance()
            .set(&DataKey::RewardPerTokenStored, &reward_per_token);
        env.storage()
            .instance()
            .set(&DataKey::LastUpdateTime, &env.ledger().timestamp());

        let earned = self.earned_at(env, account, reward_per_token);
        env.storage()
            .instance()
            .set(&DataKey::Rewards(account.clone()), &earned);
        env.storage().instance().set(
            &DataKey::UserRewardPerTokenPaid(account.clone()),
            &reward_per_token,
        );
    }

    fn reward_per_token(&self, env: &Env) -> i128 {
        let total_supply = self.total_supply(env);
        if total_supply == 0 {
            return self.reward_per_token_stored(env);
        }
        let last_time = self.last_update_time(env);
        let now = env.ledger().timestamp();
        let elapsed = now.checked_sub(last_time).unwrap_or(0u64) as i128;
        let accumulated = elapsed
            .checked_mul(self.reward_rate(env))
            .unwrap()
            .checked_mul(REWARD_PRECISION)
            .unwrap()
            .checked_div(total_supply)
            .unwrap();
        self.reward_per_token_stored(env)
            .checked_add(accumulated)
            .unwrap()
    }

    fn earned_at(&self, env: &Env, account: &Address, reward_per_token: i128) -> i128 {
        let balance = self.balance_of(env, account);
        let paid = self.user_reward_per_token_paid(env, account);
        let reward = balance
            .checked_mul(reward_per_token.checked_sub(paid).unwrap())
            .unwrap()
            .checked_div(REWARD_PRECISION)
            .unwrap();
        self.rewards(env, account).checked_add(reward).unwrap()
    }
}

#[contractimpl]
impl StakingPoolContract {
    pub fn initialize(
        env: Env,
        owner: Address,
        staking_token: Address,
        reward_token: Address,
        reward_rate: i128,
    ) {
        assert!(reward_rate >= 0, "reward rate must be non-negative");
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::StakingToken, &staking_token);
        env.storage()
            .instance()
            .set(&DataKey::RewardToken, &reward_token);
        env.storage()
            .instance()
            .set(&DataKey::RewardRate, &reward_rate);
        env.storage()
            .instance()
            .set(&DataKey::LastUpdateTime, &env.ledger().timestamp());
        env.storage()
            .instance()
            .set(&DataKey::RewardPerTokenStored, &0i128);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
    }

    pub fn stake(env: Env, staker: Address, amount: i128) {
        assert!(amount > 0, "stake amount must be positive");
        let this = StakingPoolContract;
        this.update_reward(&env, &staker);

        let contract = env.current_contract_address();
        token::Client::new(&env, &this.staking_token(&env)).transfer(&staker, &contract, &amount);

        let new_balance = this.balance_of(&env, &staker) + amount;
        env.storage()
            .instance()
            .set(&DataKey::Balance(staker.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(this.total_supply(&env) + amount));
    }

    pub fn unstake(env: Env, staker: Address, amount: i128) {
        assert!(amount > 0, "unstake amount must be positive");
        let this = StakingPoolContract;
        let balance = this.balance_of(&env, &staker);
        assert!(balance >= amount, "insufficient staked balance");

        this.update_reward(&env, &staker);
        let new_balance = balance - amount;
        env.storage()
            .instance()
            .set(&DataKey::Balance(staker.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(this.total_supply(&env) - amount));

        let contract = env.current_contract_address();
        token::Client::new(&env, &this.staking_token(&env)).transfer(&contract, &staker, &amount);
    }

    pub fn claim_rewards(env: Env, staker: Address) {
        let this = StakingPoolContract;
        this.update_reward(&env, &staker);
        let reward = this.rewards(&env, &staker);
        assert!(reward > 0, "no rewards to claim");

        env.storage()
            .instance()
            .set(&DataKey::Rewards(staker.clone()), &0i128);
        let contract = env.current_contract_address();
        token::Client::new(&env, &this.reward_token(&env)).transfer(&contract, &staker, &reward);
    }

    pub fn earned(env: Env, staker: Address) -> i128 {
        let this = StakingPoolContract;
        let reward_per_token = this.reward_per_token(&env);
        this.earned_at(&env, &staker, reward_per_token)
    }

    pub fn balance_of(env: Env, staker: Address) -> i128 {
        let this = StakingPoolContract;
        this.balance_of(&env, &staker)
    }

    pub fn reward_per_token(env: Env) -> i128 {
        let this = StakingPoolContract;
        this.reward_per_token(&env)
    }
}
