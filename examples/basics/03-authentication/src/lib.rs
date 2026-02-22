#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, IntoVal};

#[contract]
pub struct AuthContract;

#[contractimpl]
impl AuthContract {
    /// Demonstrates basic address-based authentication.
    /// Only the 'user' can successfully call this function.
    pub fn secure_action(env: Env, user: Address) {
        // 1. The magic line: checks signature and protects against replays.
        user.require_auth();

        // If code execution reaches here, 'user' is authenticated.
        // We might log an event or update state safely.
        env.events().publish((symbol_short!("auth"),), user);
    }

    /// Demonstrates authentication with specific arguments.
    /// Ensures the user specifically authorized THIS amount.
    pub fn secure_transfer(env: Env, user: Address, amount: u64) {
        // Checks that the user authorized the call with these exact arguments.
        user.require_auth_for_args((amount,).into_val(&env));

        // Logic for transfer...
    }
}

mod test;
