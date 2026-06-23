//! # Arbitrage Example
//!
//! Exploits a price difference between two AMM pools using a flash loan.
//!
//! ## Flow
//! 1. Borrow token A from flash loan provider
//! 2. Swap A → B on pool 1 at a favorable rate
//! 3. Swap B → A on pool 2 at a better rate
//! 4. Return borrowed A + fee; keep the profit

use soroban_sdk::{contract, contractclient, contractimpl, contracttype, token, Address, Env};

// ---------------------------------------------------------------------------
// External contract interfaces
// ---------------------------------------------------------------------------

/// Simplified AMM interface.
#[contractclient(name = "AMMClient")]
pub trait AMM {
    fn swap(env: Env, from_token: Address, to_token: Address, amount: i128, min_out: i128) -> i128;
}

/// Flash loan provider interface (matches 05-flash-loans).
#[contractclient(name = "FlashLoanClient")]
pub trait FlashLoan {
    fn flash_loan(env: Env, receiver: Address, token: Address, amount: i128);
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,
    Pool1,
    Pool2,
    TokenB,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ArbitrageContract;

#[contractimpl]
impl ArbitrageContract {
    pub fn init(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    /// Initiate the arbitrage.  Stores pool addresses in temporary storage
    /// so the callback can read them within the same transaction.
    pub fn execute(
        env: Env,
        flash_loan: Address,
        token_a: Address,
        token_b: Address,
        pool1: Address,
        pool2: Address,
        amount: i128,
    ) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        env.storage().temporary().set(&DataKey::Pool1, &pool1);
        env.storage().temporary().set(&DataKey::Pool2, &pool2);
        env.storage().temporary().set(&DataKey::TokenB, &token_b);

        FlashLoanClient::new(&env, &flash_loan).flash_loan(
            &env.current_contract_address(),
            &token_a,
            &amount,
        );

        env.storage().temporary().remove(&DataKey::Pool1);
        env.storage().temporary().remove(&DataKey::Pool2);
        env.storage().temporary().remove(&DataKey::TokenB);
    }

    /// Flash loan callback: perform the two-leg swap and approve repayment.
    ///
    /// Security note: in production you would validate `initiator` against a
    /// stored trusted provider address before proceeding.
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let pool1: Address = env.storage().temporary().get(&DataKey::Pool1).unwrap();
        let pool2: Address = env.storage().temporary().get(&DataKey::Pool2).unwrap();
        let token_b: Address = env.storage().temporary().get(&DataKey::TokenB).unwrap();

        let contract = env.current_contract_address();
        let next_seq = env.ledger().sequence() + 1;

        let token_a_client = token::Client::new(&env, &token);
        let token_b_client = token::Client::new(&env, &token_b);

        // Leg 1: swap A → B on pool1
        token_a_client.approve(&contract, &pool1, &amount, &next_seq);
        let received_b = AMMClient::new(&env, &pool1).swap(&token, &token_b, &amount, &1);

        // Leg 2: swap B → A on pool2
        token_b_client.approve(&contract, &pool2, &received_b, &next_seq);
        AMMClient::new(&env, &pool2).swap(&token_b, &token, &received_b, &(amount + fee));

        // Approve exact repayment to flash loan provider
        token_a_client.approve(&contract, &initiator, &(amount + fee), &next_seq);

        // Remaining profit stays in this contract for the owner to withdraw
    }

    /// Withdraw accumulated profit.
    pub fn withdraw(env: Env, token: Address, to: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&env.current_contract_address());
        if balance > 0 {
            token_client.transfer(&env.current_contract_address(), &to, &balance);
        }
    }
}
