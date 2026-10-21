use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct ImplementationV2;

#[contractimpl]
impl ImplementationV2 {
    pub fn add(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_add(b).unwrap_or_else(|| panic!("Overflow"))
    }
    pub fn sub(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_sub(b).unwrap_or_else(|| panic!("Underflow"))
    }
    pub fn mul(_env: Env, a: i128, b: i128) -> i128 {
        a.checked_mul(b).unwrap_or_else(|| panic!("Overflow"))
    }
    pub fn increment(_env: Env, amount: i128) -> i128 {
        amount.checked_mul(2).unwrap_or_else(|| panic!("Overflow"))
    }
}
