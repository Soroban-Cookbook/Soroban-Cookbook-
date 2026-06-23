#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

const PRICE_SCALE: i128 = 1_000_000;

#[contract]
pub struct AmmPoolContract;

#[contract]
pub struct AmmOracleContract;

#[contracttype]
pub enum PoolDataKey {
    Owner,
    TokenA,
    TokenB,
    ReserveA,
    ReserveB,
    FeeBps,
}

#[contracttype]
pub enum OracleDataKey {
    Owner,
    PoolContract,
    LastTimestamp,
    PriceCumulative,
    StartTimestamp,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleUpdateEventData {
    pub timestamp: u64,
    pub price_a_in_b: i128,
    pub twap: i128,
}

const POOL_NS: Symbol = symbol_short!("amm_pool");
const ORACLE_NS: Symbol = symbol_short!("amm_oracle");
const EVENT_ORACLE_UPDATED: Symbol = symbol_short!("price_updated");

impl AmmPoolContract {
    fn require_owner(&self, env: &Env) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&PoolDataKey::Owner)
            .expect("pool not initialized");
        owner.require_auth();
    }

    fn token_a(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&PoolDataKey::TokenA)
            .expect("token A missing")
    }

    fn token_b(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&PoolDataKey::TokenB)
            .expect("token B missing")
    }

    fn reserve_a(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&PoolDataKey::ReserveA)
            .unwrap_or(0i128)
    }

    fn reserve_b(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&PoolDataKey::ReserveB)
            .unwrap_or(0i128)
    }

    fn fees(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&PoolDataKey::FeeBps)
            .unwrap_or(30i128)
    }
}

impl AmmOracleContract {
    fn require_owner(&self, env: &Env) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&OracleDataKey::Owner)
            .expect("oracle not initialized");
        owner.require_auth();
    }

    fn pool_contract(&self, env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&OracleDataKey::PoolContract)
            .expect("pool contract missing")
    }

    fn last_timestamp(&self, env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&OracleDataKey::LastTimestamp)
            .unwrap_or(0u64)
    }

    fn start_timestamp(&self, env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&OracleDataKey::StartTimestamp)
            .unwrap_or(0u64)
    }

    fn cumulative(&self, env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&OracleDataKey::PriceCumulative)
            .unwrap_or(0i128)
    }
}

#[contractimpl]
impl AmmPoolContract {
    pub fn initialize(env: Env, owner: Address, token_a: Address, token_b: Address) {
        assert!(token_a != token_b, "pool tokens must differ");
        env.storage().instance().set(&PoolDataKey::Owner, &owner);
        env.storage().instance().set(&PoolDataKey::TokenA, &token_a);
        env.storage().instance().set(&PoolDataKey::TokenB, &token_b);
        env.storage().instance().set(&PoolDataKey::ReserveA, &0i128);
        env.storage().instance().set(&PoolDataKey::ReserveB, &0i128);
        env.storage().instance().set(&PoolDataKey::FeeBps, &30i128);
    }

    pub fn deposit(env: Env, provider: Address, amount_a: i128, amount_b: i128) {
        assert!(
            amount_a > 0 && amount_b > 0,
            "deposit amounts must be positive"
        );
        let this = AmmPoolContract;
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);
        let contract_addr = env.current_contract_address();

        token::Client::new(&env, &token_a).transfer(&provider, &contract_addr, &amount_a);
        token::Client::new(&env, &token_b).transfer(&provider, &contract_addr, &amount_b);

        let reserve_a = this.reserve_a(&env) + amount_a;
        let reserve_b = this.reserve_b(&env) + amount_b;
        env.storage()
            .instance()
            .set(&PoolDataKey::ReserveA, &reserve_a);
        env.storage()
            .instance()
            .set(&PoolDataKey::ReserveB, &reserve_b);
    }

    pub fn swap(env: Env, sell_token: Address, amount_in: i128, min_amount_out: i128) -> i128 {
        assert!(amount_in > 0, "amount_in must be positive");
        let this = AmmPoolContract;
        let token_a = this.token_a(&env);
        let token_b = this.token_b(&env);
        let reserve_a = this.reserve_a(&env);
        let reserve_b = this.reserve_b(&env);
        let fee_bps = this.fees(&env);
        let contract_addr = env.current_contract_address();

        let (in_token, out_token, in_reserve, out_reserve) = if sell_token == token_a {
            (token_a, token_b, reserve_a, reserve_b)
        } else if sell_token == token_b {
            (token_b, token_a, reserve_b, reserve_a)
        } else {
            panic!("unsupported sell token");
        };

        token::Client::new(&env, &in_token).transfer(&env.invoker(), &contract_addr, &amount_in);

        let amount_after_fee = amount_in
            .checked_mul(10000 - fee_bps)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let numerator = amount_after_fee.checked_mul(out_reserve).unwrap();
        let denominator = in_reserve + amount_after_fee;
        let amount_out = numerator.checked_div(denominator).unwrap();
        assert!(amount_out >= min_amount_out, "slippage exceeded");

        token::Client::new(&env, &out_token).transfer(&contract_addr, &env.invoker(), &amount_out);

        let new_reserve_in = in_reserve + amount_in;
        let new_reserve_out = out_reserve - amount_out;
        if sell_token == token_a {
            env.storage()
                .instance()
                .set(&PoolDataKey::ReserveA, &new_reserve_in);
            env.storage()
                .instance()
                .set(&PoolDataKey::ReserveB, &new_reserve_out);
        } else {
            env.storage()
                .instance()
                .set(&PoolDataKey::ReserveB, &new_reserve_in);
            env.storage()
                .instance()
                .set(&PoolDataKey::ReserveA, &new_reserve_out);
        }
        amount_out
    }

    pub fn get_reserves(env: Env) -> (i128, i128) {
        let this = AmmPoolContract;
        (this.reserve_a(&env), this.reserve_b(&env))
    }

    pub fn current_price_a_in_b(env: Env) -> i128 {
        let this = AmmPoolContract;
        let reserve_a = this.reserve_a(&env);
        let reserve_b = this.reserve_b(&env);
        assert!(reserve_a > 0 && reserve_b > 0, "empty reserves");
        reserve_b
            .checked_mul(PRICE_SCALE)
            .unwrap()
            .checked_div(reserve_a)
            .unwrap()
    }

    pub fn current_price_b_in_a(env: Env) -> i128 {
        let this = AmmPoolContract;
        let reserve_a = this.reserve_a(&env);
        let reserve_b = this.reserve_b(&env);
        assert!(reserve_a > 0 && reserve_b > 0, "empty reserves");
        reserve_a
            .checked_mul(PRICE_SCALE)
            .unwrap()
            .checked_div(reserve_b)
            .unwrap()
    }
}

#[contractimpl]
impl AmmOracleContract {
    pub fn initialize(env: Env, owner: Address, pool_contract: Address) {
        env.storage().instance().set(&OracleDataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&OracleDataKey::PoolContract, &pool_contract);
        env.storage()
            .instance()
            .set(&OracleDataKey::LastTimestamp, &0u64);
        env.storage()
            .instance()
            .set(&OracleDataKey::PriceCumulative, &0i128);
        env.storage()
            .instance()
            .set(&OracleDataKey::StartTimestamp, &0u64);
    }

    pub fn update(env: Env) {
        let this = AmmOracleContract;
        let pool = AmmPoolContractClient::new(&env, &this.pool_contract(&env));
        let current_price = pool.current_price_a_in_b();
        let timestamp = env.ledger().timestamp();
        let last_ts = this.last_timestamp(&env);
        let start_ts = this.start_timestamp(&env);
        let cumulative = this.cumulative(&env);

        let new_cumulative = if last_ts == 0 {
            current_price.checked_mul(0i128).unwrap()
        } else {
            let elapsed = timestamp.checked_sub(last_ts).unwrap() as i128;
            cumulative
                .checked_add(current_price.checked_mul(elapsed).unwrap())
                .unwrap()
        };

        let new_start = if start_ts == 0 { timestamp } else { start_ts };

        env.storage()
            .instance()
            .set(&OracleDataKey::PriceCumulative, &new_cumulative);
        env.storage()
            .instance()
            .set(&OracleDataKey::LastTimestamp, &timestamp);
        env.storage()
            .instance()
            .set(&OracleDataKey::StartTimestamp, &new_start);

        env.events().publish(
            (ORACLE_NS, EVENT_ORACLE_UPDATED),
            OracleUpdateEventData {
                timestamp,
                price_a_in_b: current_price,
                twap: AmmOracleContract::get_twap(&env),
            },
        );
    }

    pub fn get_twap(env: &Env) -> i128 {
        let this = AmmOracleContract;
        let last_ts = this.last_timestamp(env);
        let start_ts = this.start_timestamp(env);
        let cumulative = this.cumulative(env);
        let duration = last_ts.checked_sub(start_ts).unwrap_or(1u64) as i128;
        if duration == 0 {
            return 0;
        }
        cumulative.checked_div(duration).unwrap()
    }

    pub fn current_price(env: Env) -> i128 {
        let this = AmmOracleContract;
        let pool = AmmPoolContractClient::new(&env, &this.pool_contract(&env));
        pool.current_price_a_in_b()
    }

    pub fn twap_price(env: Env) -> i128 {
        AmmOracleContract::get_twap(&env)
    }
}
