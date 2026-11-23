use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, IntoVal, Symbol};

const ADMIN_KEY: Symbol = symbol_short!("admin");
const IMPLEMENTATION_KEY: Symbol = symbol_short!("impl");
const COUNTER_KEY: Symbol = symbol_short!("counter");

#[contract]
pub struct ProxyContract;

#[contractimpl]
impl ProxyContract {
    pub fn init(env: Env, admin: Address, implementation: Address) {
        if env.storage().persistent().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }
        admin.require_auth();
        env.storage().persistent().set(&ADMIN_KEY, &admin);
        env.storage()
            .persistent()
            .set(&IMPLEMENTATION_KEY, &implementation);
        env.storage().persistent().set(&COUNTER_KEY, &0i128);
    }

    pub fn upgrade(env: Env, new_implementation: Address) {
        Self::admin(&env).require_auth();
        env.storage()
            .persistent()
            .set(&IMPLEMENTATION_KEY, &new_implementation);
        env.events()
            .publish((symbol_short!("upgraded"),), new_implementation);
    }

    pub fn get_implementation(env: Env) -> Address {
        Self::implementation(&env)
    }

    pub fn add(env: Env, a: i128, b: i128) -> i128 {
        Self::invoke(
            &env,
            symbol_short!("add"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    pub fn subtract(env: Env, a: i128, b: i128) -> i128 {
        Self::invoke(
            &env,
            symbol_short!("sub"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    pub fn multiply(env: Env, a: i128, b: i128) -> i128 {
        Self::invoke(
            &env,
            symbol_short!("mul"),
            vec![&env, a.into_val(&env), b.into_val(&env)],
        )
    }

    pub fn increment(env: Env, amount: i128) -> i128 {
        let delta: i128 = Self::invoke(
            &env,
            symbol_short!("increment"),
            vec![&env, amount.into_val(&env)],
        );
        let next = Self::counter_value(&env)
            .checked_add(delta)
            .unwrap_or_else(|| panic!("Counter overflow"));
        env.storage().persistent().set(&COUNTER_KEY, &next);
        next
    }

    pub fn counter(env: Env) -> i128 {
        Self::counter_value(&env)
    }

    fn admin(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    fn implementation(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&IMPLEMENTATION_KEY)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    fn counter_value(env: &Env) -> i128 {
        env.storage().persistent().get(&COUNTER_KEY).unwrap_or(0)
    }

    fn invoke<T>(env: &Env, function: Symbol, args: soroban_sdk::Vec<soroban_sdk::Val>) -> T
    where
        T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
    {
        env.invoke_contract(&Self::implementation(env), &function, args)
    }
}
