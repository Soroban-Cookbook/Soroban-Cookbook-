//! # Flash Loan Contract
//!
//! A simple flash loan implementation for Soroban.
//!
//! Features:
//! - Flash loan function with callback mechanism
//! - Fee collection (configurable basis points)
//! - Reentrancy protection using transaction-scoped flags
//! - Event emission for loan tracking

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Locked,
    FeeBps,
    Admin,
}

#[contract]
pub struct FlashLoanContract;

#[contractimpl]
impl FlashLoanContract {
    /// Initialize the contract with an admin and fee in basis points (1/10000).
    pub fn init(env: Env, admin: Address, fee_bps: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
    }

    /// Execute a flash loan.
    ///
    /// # Arguments
    /// * `receiver` - The address of the contract that will receive the funds and implement the callback
    /// * `token` - The address of the token to be borrowed
    /// * `amount` - The amount of tokens to borrow
    pub fn flash_loan(env: Env, receiver: Address, token: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        // 1. Reentrancy protection
        if env.storage().temporary().has(&DataKey::Locked) {
            panic!("reentrancy detected");
        }
        env.storage().temporary().set(&DataKey::Locked, &true);

        // 2. Calculate fee
        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0);
        let fee = (amount * fee_bps as i128) / 10000;

        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        // 3. Verify contract has enough liquidity
        let balance = token_client.balance(&contract_address);
        if balance < amount {
            panic!("insufficient liquidity");
        }

        // 4. Transfer tokens to receiver
        token_client.transfer(&contract_address, &receiver, &amount);

        // 5. Call receiver's callback
        let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);
        receiver_client.on_flash_loan(&contract_address, &token, &amount, &fee);

        // 6. Verify repayment
        // We use transfer_from to pull the funds (amount + fee) back from the receiver.
        // The receiver MUST have approved the flash loan contract to spend this amount.
        token_client.transfer_from(&receiver, &contract_address, &(amount + fee));

        // 7. Release lock
        env.storage().temporary().remove(&DataKey::Locked);

        // 8. Emit event
        env.events().publish(
            (symbol_short!("flash"), symbol_short!("loan")),
            (receiver, token, amount, fee),
        );
    }

    /// Update the fee basis points. Only admin can call this.
    pub fn set_fee(env: Env, new_fee_bps: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::FeeBps, &new_fee_bps);
    }

    /// Get the current fee in basis points.
    pub fn get_fee(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0)
    }
}

/// Interface for the flash loan receiver contract.
#[soroban_sdk::contractclient(name = "FlashLoanReceiverClient")]
pub trait FlashLoanReceiver {
    /// Callback executed during a flash loan.
    ///
    /// # Arguments
    /// * `initiator` - The address of the flash loan contract
    /// * `token` - The address of the borrowed token
    /// * `amount` - The borrowed amount
    /// * `fee` - The fee to be paid back
    fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128);
}

#[cfg(test)]
mod test;
