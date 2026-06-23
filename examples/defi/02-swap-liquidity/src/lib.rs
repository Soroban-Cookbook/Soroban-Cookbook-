#![no_std]

use core::cmp;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

#[contract]
pub struct SwapLiquidityContract;

#[contracttype]
pub enum DataKey {
    Owner,
    TokenA,
    TokenB,
    LPToken,
    ReserveA,
    ReserveB,
    TotalShares,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityEventData {
    pub provider: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub shares: i128,
}

const EVENT_NS: Symbol = symbol_short!("swap_liq");
const EVENT_ADD: Symbol = symbol_short!("liquidity_added");
const EVENT_REMOVE: Symbol = symbol_short!("liquidity_removed");

impl SwapLiquidityContract {
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
            .expect("token A not configured")
    }

    fn token_b(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenB)
            .expect("token B not configured")
    }

    fn lp_token(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::LPToken)
            .expect("lp token not configured")
    }

    fn reserve_a(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::ReserveA)
            .unwrap_or(0i128)
    }

    fn reserve_b(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::ReserveB)
            .unwrap_or(0i128)
    }

    fn total_shares(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0i128)
    }

    fn contract_address(&self, env: &Env) -> Address {
        env.current_contract_address()
    }
}

#[contractimpl]
impl SwapLiquidityContract {
    pub fn initialize(
        env: Env,
        owner: Address,
        token_a: Address,
        token_b: Address,
        lp_token: Address,
    ) {
        assert!(token_a != token_b, "tokens must differ");
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::TokenA, &token_a);
        env.storage().instance().set(&DataKey::TokenB, &token_b);
        env.storage().instance().set(&DataKey::LPToken, &lp_token);
        env.storage().instance().set(&DataKey::ReserveA, &0i128);
        env.storage().instance().set(&DataKey::ReserveB, &0i128);
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
    }

    pub fn add_liquidity(env: Env, provider: Address, amount_a: i128, amount_b: i128) {
        assert!(
            amount_a > 0 && amount_b > 0,
            "liquidity amounts must be positive"
        );
        let this = SwapLiquidityContract;
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);
        let lp_token = this.lp_token(&env);
        let reserve_a = this.reserve_a(&env);
        let reserve_b = this.reserve_b(&env);
        let total_shares = this.total_shares(&env);

        let contract_addr = this.contract_address(&env);
        token::Client::new(&env, &token_a).transfer(&provider, &contract_addr, &amount_a);
        token::Client::new(&env, &token_b).transfer(&provider, &contract_addr, &amount_b);

        let minted_shares = if total_shares == 0 {
            amount_a
                .checked_mul(amount_b)
                .expect("overflow in initial shares")
        } else {
            let share_a = amount_a
                .checked_mul(total_shares)
                .unwrap()
                .checked_div(reserve_a)
                .unwrap();
            let share_b = amount_b
                .checked_mul(total_shares)
                .unwrap()
                .checked_div(reserve_b)
                .unwrap();
            cmp::min(share_a, share_b)
        };

        assert!(minted_shares > 0, "insufficient liquidity added");
        token::Client::new(&env, &lp_token).mint(&provider, &minted_shares);

        env.storage()
            .instance()
            .set(&DataKey::ReserveA, &(reserve_a + amount_a));
        env.storage()
            .instance()
            .set(&DataKey::ReserveB, &(reserve_b + amount_b));
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares + minted_shares));

        env.events().publish(
            (EVENT_NS, EVENT_ADD),
            LiquidityEventData {
                provider,
                amount_a,
                amount_b,
                shares: minted_shares,
            },
        );
    }

    pub fn remove_liquidity(env: Env, provider: Address, shares: i128) {
        assert!(shares > 0, "shares must be positive");
        let this = SwapLiquidityContract;
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);
        let lp_token = this.lp_token(&env);
        let reserve_a = this.reserve_a(&env);
        let reserve_b = this.reserve_b(&env);
        let total_shares = this.total_shares(&env);
        assert!(shares <= total_shares, "not enough pool shares");

        let amount_a = reserve_a
            .checked_mul(shares)
            .unwrap()
            .checked_div(total_shares)
            .unwrap();
        let amount_b = reserve_b
            .checked_mul(shares)
            .unwrap()
            .checked_div(total_shares)
            .unwrap();
        let contract_addr = this.contract_address(&env);

        token::Client::new(&env, &lp_token).burn(&provider, &shares);
        token::Client::new(&env, &token_a).transfer(&contract_addr, &provider, &amount_a);
        token::Client::new(&env, &token_b).transfer(&contract_addr, &provider, &amount_b);

        env.storage()
            .instance()
            .set(&DataKey::ReserveA, &(reserve_a - amount_a));
        env.storage()
            .instance()
            .set(&DataKey::ReserveB, &(reserve_b - amount_b));
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares - shares));

        env.events().publish(
            (EVENT_NS, EVENT_REMOVE),
            LiquidityEventData {
                provider,
                amount_a,
                amount_b,
                shares,
            },
        );
    }

    pub fn pool_share(env: Env, provider: Address) -> i128 {
        let this = SwapLiquidityContract;
        let total_shares = this.total_shares(&env);
        if total_shares == 0 {
            return 0;
        }
        let provider_shares = token::Client::new(&env, &this.lp_token(&env)).balance(&provider);
        provider_shares
            .checked_mul(10000)
            .unwrap()
            .checked_div(total_shares)
            .unwrap()
    }

    pub fn get_reserves(env: Env) -> (i128, i128) {
        let this = SwapLiquidityContract;
        (this.reserve_a(&env), this.reserve_b(&env))
    }

    pub fn get_total_shares(env: Env) -> i128 {
        let this = SwapLiquidityContract;
        this.total_shares(&env)
    }
}
