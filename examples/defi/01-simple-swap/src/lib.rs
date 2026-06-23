#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

#[contract]
pub struct SimpleSwapContract;

#[contracttype]
pub enum DataKey {
    Owner,
    TokenA,
    TokenB,
    RateNum,
    RateDen,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapEventData {
    pub trader: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: i128,
    pub buy_amount: i128,
    pub min_buy_amount: i128,
}

const EVENT_NS: Symbol = symbol_short!("swap");
const EVENT_SWAP: Symbol = symbol_short!("swap_executed");
const EVENT_PAIR: Symbol = symbol_short!("pair_updated");

impl SimpleSwapContract {
    fn require_owner(&self, env: &Env) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("contract not initialized");
        owner.require_auth();
    }

    fn token_a(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenA)
            .expect("token pair not configured")
    }

    fn token_b(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenB)
            .expect("token pair not configured")
    }

    fn rate(&self, env: &Env) -> (i128, i128) {
        let num: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RateNum)
            .expect("rate not configured");
        let den: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RateDen)
            .expect("rate not configured");
        (num, den)
    }

    fn contract_address(&self, env: &Env) -> Address {
        env.current_contract_address()
    }
}

#[contractimpl]
impl SimpleSwapContract {
    pub fn initialize(
        env: Env,
        owner: Address,
        token_a: Address,
        token_b: Address,
        rate_num: i128,
        rate_den: i128,
    ) {
        assert!(rate_num > 0 && rate_den > 0, "rate must be positive");
        assert!(token_a != token_b, "tokens must differ");

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::TokenA, &token_a);
        env.storage().instance().set(&DataKey::TokenB, &token_b);
        env.storage().instance().set(&DataKey::RateNum, &rate_num);
        env.storage().instance().set(&DataKey::RateDen, &rate_den);

        env.events().publish(
            (EVENT_NS, EVENT_PAIR),
            (token_a, token_b, rate_num, rate_den),
        );
    }

    pub fn update_pair(
        env: Env,
        token_a: Address,
        token_b: Address,
        rate_num: i128,
        rate_den: i128,
    ) {
        let this = SimpleSwapContract;
        this.require_owner(&env);
        assert!(rate_num > 0 && rate_den > 0, "rate must be positive");
        assert!(token_a != token_b, "tokens must differ");

        env.storage().instance().set(&DataKey::TokenA, &token_a);
        env.storage().instance().set(&DataKey::TokenB, &token_b);
        env.storage().instance().set(&DataKey::RateNum, &rate_num);
        env.storage().instance().set(&DataKey::RateDen, &rate_den);

        env.events().publish(
            (EVENT_NS, EVENT_PAIR),
            (token_a, token_b, rate_num, rate_den),
        );
    }

    pub fn quote(env: Env, sell_token: Address, sell_amount: i128) -> i128 {
        assert!(sell_amount > 0, "sell amount must be positive");
        let this = SimpleSwapContract;
        let (num, den) = this.rate(&env);
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);

        if sell_token == token_a {
            sell_amount
                .checked_mul(num)
                .unwrap()
                .checked_div(den)
                .unwrap()
        } else if sell_token == token_b {
            sell_amount
                .checked_mul(den)
                .unwrap()
                .checked_div(num)
                .unwrap()
        } else {
            panic!("unsupported sell token");
        }
    }

    pub fn swap(
        env: Env,
        sell_token: Address,
        sell_amount: i128,
        min_buy_amount: i128,
        recipient: Address,
    ) {
        assert!(sell_amount > 0, "sell amount must be positive");
        assert!(min_buy_amount > 0, "min buy amount must be positive");

        let this = SimpleSwapContract;
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);
        let (num, den) = this.rate(&env);
        let buy_token = if sell_token == token_a {
            token_b.clone()
        } else if sell_token == token_b {
            token_a.clone()
        } else {
            panic!("unsupported sell token");
        };

        let buy_amount = if sell_token == token_a {
            sell_amount
                .checked_mul(num)
                .unwrap()
                .checked_div(den)
                .unwrap()
        } else {
            sell_amount
                .checked_mul(den)
                .unwrap()
                .checked_div(num)
                .unwrap()
        };

        assert!(buy_amount >= min_buy_amount, "slippage exceeded");

        let contract_addr = this.contract_address(&env);

        token::Client::new(&env, &sell_token).transfer(
            &env.invoker(),
            &contract_addr,
            &sell_amount,
        );
        token::Client::new(&env, &buy_token).transfer(&contract_addr, &recipient, &buy_amount);

        env.events().publish(
            (EVENT_NS, EVENT_SWAP),
            SwapEventData {
                trader: env.invoker(),
                sell_token,
                buy_token,
                sell_amount,
                buy_amount,
                min_buy_amount,
            },
        );
    }

    pub fn get_pair(env: Env) -> (Address, Address) {
        let this = SimpleSwapContract;
        (this.token_a(&env), this.token_b(&env))
    }

    pub fn get_rate(env: Env) -> (i128, i128) {
        let this = SimpleSwapContract;
        this.rate(&env)
    }
}
