#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{vec, Address, Env, IntoVal};
use soroban_validation::test_events::EventList;

// Mock Receiver Contract
#[contract]
pub struct Receiver;

#[contractimpl]
impl Receiver {
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let token_client = token::Client::new(&env, &token);
        // Approve the flash loan contract to pull the funds back (amount + fee)
        token_client.approve(
            &env.current_contract_address(),
            &initiator,
            &(amount + fee),
            &(env.ledger().sequence() + 1),
        );
    }

    pub fn try_reenter(env: Env, flash_loan_address: Address, token: Address) {
        let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_address);
        flash_loan_client.flash_loan(&env.current_contract_address(), &token, &100);
    }
}

// Reentrant Receiver Contract
#[contract]
pub struct ReentrantReceiver;

#[contractimpl]
impl ReentrantReceiver {
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, _fee: i128) {
        let flash_loan_client = FlashLoanContractClient::new(&env, &initiator);
        // Try to re-enter the flash loan function
        flash_loan_client.flash_loan(&env.current_contract_address(), &token, &amount);
    }
}

fn setup_test(
    env: &Env,
) -> (
    Address,
    Address,
    Address,
    FlashLoanContractClient,
    token::Client,
    token::StellarAssetClient,
) {
    let admin = Address::generate(env);
    let flash_loan_address = env.register_contract(None, FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(env, &flash_loan_address);
    flash_loan_client.init(&admin, &50); // 0.5% fee

    let token_admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::Client::new(env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(env, &token_address);

    (
        admin,
        token_admin,
        flash_loan_address,
        flash_loan_client,
        token_client,
        token_admin_client,
    )
}

#[test]
fn test_successful_flash_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);

    let receiver_address = env.register_contract(None, Receiver);

    // Fund the flash loan contract
    token_admin_client.mint(&flash_loan_address, &10000);

    // Fund the receiver for the fee (0.5% of 1000 = 5)
    token_admin_client.mint(&receiver_address, &5);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);

    assert_eq!(token_client.balance(&flash_loan_address), 10005);
    assert_eq!(token_client.balance(&receiver_address), 0);
}

#[test]
fn test_successful_flash_loan_zero_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);

    flash_loan_client.set_fee(&0);

    let receiver_address = env.register_contract(None, Receiver);
    token_admin_client.mint(&flash_loan_address, &10000);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);

    assert_eq!(token_client.balance(&flash_loan_address), 10000);
}

#[test]
#[should_panic(expected = "insufficient liquidity")]
fn test_fail_insufficient_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, _, flash_loan_client, token_client, _) = setup_test(&env);
    let receiver_address = env.register_contract(None, Receiver);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_fail_zero_amount() {
    let env = Env::default();
    let (_, _, _, flash_loan_client, token_client, _) = setup_test(&env);
    let receiver_address = env.register_contract(None, Receiver);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &0);
}

#[test]
#[should_panic(expected = "reentrancy detected")]
fn test_fail_reentrancy() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);

    let reentrant_receiver = env.register_contract(None, ReentrantReceiver);
    token_admin_client.mint(&flash_loan_address, &10000);

    flash_loan_client.flash_loan(&reentrant_receiver, &token_client.address, &1000);
}

#[test]
#[should_panic] // Should fail because Receiver didn't approve enough
fn test_fail_no_repayment() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);

    // Use a receiver that doesn't repay/approve
    #[contract]
    pub struct BadReceiver;
    #[contractimpl]
    impl BadReceiver {
        pub fn on_flash_loan(
            _env: Env,
            _initiator: Address,
            _token: Address,
            _amount: i128,
            _fee: i128,
        ) {
        }
    }

    let bad_receiver = env.register_contract(None, BadReceiver);
    token_admin_client.mint(&flash_loan_address, &10000);

    flash_loan_client.flash_loan(&bad_receiver, &token_client.address, &1000);
}

#[test]
fn test_admin_set_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, _, flash_loan_client, _, _) = setup_test(&env);

    flash_loan_client.set_fee(&100);
    assert_eq!(flash_loan_client.get_fee(), 100);
}

#[test]
#[should_panic]
fn test_fail_non_admin_set_fee() {
    let env = Env::default();
    // No mock_all_auths, or we can manually check auth
    let (_, _, _, flash_loan_client, _, _) = setup_test(&env);

    // This will fail because it's not the admin address calling it
    flash_loan_client.set_fee(&100);
}

#[test]
fn test_sequential_loans_work() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);
    let receiver_address = env.register_contract(None, Receiver);

    token_admin_client.mint(&flash_loan_address, &10000);
    token_admin_client.mint(&receiver_address, &100);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);
    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);

    assert_eq!(token_client.balance(&flash_loan_address), 10010);
}

#[test]
fn test_multiple_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client1, token_admin_client1) =
        setup_test(&env);

    let token_admin2 = Address::generate(&env);
    let token_address2 = env.register_stellar_asset_contract(token_admin2);
    let token_client2 = token::Client::new(&env, &token_address2);
    let token_admin_client2 = token::StellarAssetClient::new(&env, &token_address2);

    let receiver_address = env.register_contract(None, Receiver);

    token_admin_client1.mint(&flash_loan_address, &10000);
    token_admin_client2.mint(&flash_loan_address, &20000);
    token_admin_client1.mint(&receiver_address, &5);
    token_admin_client2.mint(&receiver_address, &10);

    flash_loan_client.flash_loan(&receiver_address, &token_client1.address, &1000);
    flash_loan_client.flash_loan(&receiver_address, &token_client2.address, &2000);

    assert_eq!(token_client1.balance(&flash_loan_address), 10005);
    assert_eq!(token_client2.balance(&flash_loan_address), 20010);
}

#[test]
fn test_events_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, flash_loan_address, flash_loan_client, token_client, token_admin_client) =
        setup_test(&env);
    let receiver_address = env.register_contract(None, Receiver);

    token_admin_client.mint(&flash_loan_address, &10000);
    token_admin_client.mint(&receiver_address, &5);

    flash_loan_client.flash_loan(&receiver_address, &token_client.address, &1000);

    let events = EventList::new(&env, env.events().all());
    let last_event = events.last().unwrap();

    assert_eq!(last_event.contract_id, flash_loan_address);
    assert_eq!(
        last_event.topics,
        vec![
            &env,
            (symbol_short!("flash"), symbol_short!("loan")).into_val(&env)
        ]
    );
    // The data contains (receiver, token, amount, fee)
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_fail_double_init() {
    let env = Env::default();
    let (admin, _, _, flash_loan_client, _, _) = setup_test(&env);
    flash_loan_client.init(&admin, &100);
}
