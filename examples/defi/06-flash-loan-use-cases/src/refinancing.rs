//! # Refinancing Example
//!
//! Atomically moves a collateralised debt position from one lending pool to
//! another at a better rate using a flash loan.
//!
//! ## Flow
//! 1. Borrow the outstanding debt from the flash loan provider
//! 2. Repay the old loan, releasing the collateral
//! 3. Deposit that collateral into the new pool
//! 4. Borrow from the new pool to cover (flash loan amount + fee)
//! 5. Flash loan provider pulls back the repayment automatically

use soroban_sdk::{contract, contractclient, contractimpl, contracttype, token, Address, Env};

// ---------------------------------------------------------------------------
// External contract interfaces
// ---------------------------------------------------------------------------

/// Minimal lending pool interface used in this example.
#[contractclient(name = "LendingPoolClient")]
pub trait LendingPool {
    /// Repay `amount` of `token` and return the collateral amount released.
    fn repay_and_withdraw(env: Env, token: Address, amount: i128) -> i128;
    fn deposit_collateral(env: Env, token: Address, amount: i128);
    fn borrow(env: Env, token: Address, amount: i128);
}

/// Flash loan provider interface.
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
    OldPool,
    NewPool,
    Collateral,
    DebtAmount,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct RefinancingContract;

#[contractimpl]
impl RefinancingContract {
    pub fn init(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    /// Initiate refinancing from `old_pool` to `new_pool`.
    pub fn execute(
        env: Env,
        flash_loan: Address,
        debt_token: Address,
        collateral: Address,
        old_pool: Address,
        new_pool: Address,
        debt_amount: i128,
    ) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        env.storage().temporary().set(&DataKey::OldPool, &old_pool);
        env.storage().temporary().set(&DataKey::NewPool, &new_pool);
        env.storage()
            .temporary()
            .set(&DataKey::Collateral, &collateral);
        env.storage()
            .temporary()
            .set(&DataKey::DebtAmount, &debt_amount);

        FlashLoanClient::new(&env, &flash_loan).flash_loan(
            &env.current_contract_address(),
            &debt_token,
            &debt_amount,
        );

        env.storage().temporary().remove(&DataKey::OldPool);
        env.storage().temporary().remove(&DataKey::NewPool);
        env.storage().temporary().remove(&DataKey::Collateral);
        env.storage().temporary().remove(&DataKey::DebtAmount);
    }

    /// Flash loan callback: repay old loan → redeposit collateral → borrow
    /// enough from new pool to repay the flash loan.
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let old_pool: Address = env.storage().temporary().get(&DataKey::OldPool).unwrap();
        let new_pool: Address = env.storage().temporary().get(&DataKey::NewPool).unwrap();
        let collateral_token: Address =
            env.storage().temporary().get(&DataKey::Collateral).unwrap();

        let contract = env.current_contract_address();
        let next_seq = env.ledger().sequence() + 1;
        let debt_client = token::Client::new(&env, &token);
        let coll_client = token::Client::new(&env, &collateral_token);

        // Step 1: repay old loan, receive collateral
        let old_pool_client = LendingPoolClient::new(&env, &old_pool);
        debt_client.approve(&contract, &old_pool, &amount, &next_seq);
        let collateral_returned = old_pool_client.repay_and_withdraw(&token, &amount);

        // Step 2: deposit collateral into new pool
        let new_pool_client = LendingPoolClient::new(&env, &new_pool);
        coll_client.approve(&contract, &new_pool, &collateral_returned, &next_seq);
        new_pool_client.deposit_collateral(&collateral_token, &collateral_returned);

        // Step 3: borrow from new pool to cover flash loan repayment
        let repay = amount + fee;
        new_pool_client.borrow(&token, &repay);

        // Step 4: approve flash loan to pull repayment
        debt_client.approve(&contract, &initiator, &repay, &next_seq);
    }
}
