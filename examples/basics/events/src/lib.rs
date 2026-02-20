#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_number(env: Env, value: u32) {
        env.storage().instance().set(&symbol_short!("num"), &value);

        // Emits number update with new value
        env.events().publish((symbol_short!("number"),), value);
    }

    pub fn increment(env: Env) {
        let mut num: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("num"))
            .unwrap_or(0);

        num += 1;
        env.storage().instance().set(&symbol_short!("num"), &num);

        // Emits increment event with new value
        env.events()
            .publish((symbol_short!("number"), symbol_short!("inc")), num);
    }

    pub fn decrement(env: Env) {
        let mut num: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("num"))
            .unwrap_or(0);

        num -= 1;
        env.storage().instance().set(&symbol_short!("num"), &num);

        // Emits decrement event with new value
        env.events()
            .publish((symbol_short!("number"), symbol_short!("dec")), num);
    }

    pub fn get_number(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("num"))
            .unwrap_or(0)
    }
}

mod test;
