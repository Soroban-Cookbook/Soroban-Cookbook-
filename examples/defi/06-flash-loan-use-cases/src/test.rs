#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{
    contract, contractclient, contractimpl, symbol_short, token, Address, Env, IntoVal,
};

// ============================================================
// Security pattern tests
// ============================================================

mod security_tests {
    use super::*;
    use crate::security::{SecureReceiverContract, SecureReceiverContractClient};
    use soroban_validation::test_events::EventList;
    use soroban_sdk::testutils::Events;

    fn setup(env: &Env) -> (Address, Address, Address, SecureReceiverContractClient) {
        let owner = Address::generate(env);
        let provider = Address::generate(env);
        let addr = env.register(SecureReceiverContract, ());
        let client = SecureReceiverContractClient::new(env, &addr);
        client.init(&owner, &provider);
        (owner, provider, addr, client)
    }

    fn make_token(env: &Env, to: &Address, amount: i128) -> (Address, token::Client<'static>) {
        let admin = Address::generate(env);
        let addr = env.register_stellar_asset_contract_v2(admin).address();
        token::StellarAssetClient::new(env, &addr).mint(to, &amount);
        (addr.clone(), token::Client::new(env, &addr))
    }

    #[test]
    fn test_callback_accepts_registered_provider() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, provider, receiver_addr, client) = setup(&env);

        let (token_addr, _) = make_token(&env, &receiver_addr, 1050);
        client.on_flash_loan(&provider, &token_addr, &1000, &50);
    }

    #[test]
    #[should_panic(expected = "unauthorized provider")]
    fn test_callback_rejects_unknown_caller() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, _provider, receiver_addr, client) = setup(&env);

        let (token_addr, _) = make_token(&env, &receiver_addr, 1050);
        let rogue = Address::generate(&env);
        client.on_flash_loan(&rogue, &token_addr, &1000, &50);
    }

    #[test]
    #[should_panic(expected = "invalid amount or fee")]
    fn test_callback_rejects_zero_amount() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, provider, receiver_addr, client) = setup(&env);

        let (token_addr, _) = make_token(&env, &receiver_addr, 0);
        client.on_flash_loan(&provider, &token_addr, &0, &0);
    }

    #[test]
    #[should_panic(expected = "insufficient repayment funds")]
    fn test_callback_rejects_when_underfunded() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, provider, receiver_addr, client) = setup(&env);

        let (token_addr, _) = make_token(&env, &receiver_addr, 100);
        client.on_flash_loan(&provider, &token_addr, &1000, &50);
    }

    #[test]
    fn test_callback_approves_exact_repayment() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, provider, receiver_addr, client) = setup(&env);

        let (token_addr, token_client) = make_token(&env, &receiver_addr, 1050);
        client.on_flash_loan(&provider, &token_addr, &1000, &50);

        assert_eq!(token_client.allowance(&receiver_addr, &provider), 1050);
    }

    #[test]
    fn test_callback_emits_audit_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, provider, receiver_addr, client) = setup(&env);

        let (token_addr, _) = make_token(&env, &receiver_addr, 1050);
        client.on_flash_loan(&provider, &token_addr, &1000, &50);

        let events = EventList::new(&env, env.events().all());
        let last = events.last().unwrap();
        assert_eq!(last.0, receiver_addr);
        assert_eq!(
            last.1,
            soroban_sdk::vec![
                &env,
                symbol_short!("fl").into_val(&env),
                symbol_short!("callback").into_val(&env)
            ]
        );
    }

    #[test]
    fn test_set_provider_by_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let (_owner, _provider, _, client) = setup(&env);
        client.set_provider(&Address::generate(&env));
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_init_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (owner, provider, _, client) = setup(&env);
        client.init(&owner, &provider);
    }
}

// ============================================================
// Arbitrage tests
//
// The ArbitrageContract callback is tested by:
// 1. Wiring up mock AMMs that perform token swaps via simple transfers
// 2. Calling on_flash_loan directly (bypassing the flash loan provider)
// 3. Asserting the allowance approved to the fake provider equals amount+fee
// ============================================================

mod arbitrage_tests {
    use super::*;
    use crate::arbitrage::{ArbitrageContract, ArbitrageContractClient, DataKey};

    /// Mock AMM: given an allowance for `from_token`, swaps it at a fixed rate.
    /// With mock_all_auths any transfer/transfer_from succeeds unconditionally.
    #[contract]
    struct MockAMM;

    #[contractimpl]
    impl MockAMM {
        pub fn init(env: Env, rate_bps: u32) {
            env.storage().instance().set(&"rate", &rate_bps);
        }

        pub fn swap(
            env: Env,
            from_token: Address,
            to_token: Address,
            amount: i128,
            _min_out: i128,
        ) -> i128 {
            let rate: u32 = env.storage().instance().get(&"rate").unwrap();
            let out = amount * rate as i128 / 10000;
            // Pull input tokens into pool (mocked / skipped for test simplicity)
            out
        }
    }


    fn make_token(env: &Env) -> (Address, token::Client, token::StellarAssetClient) {
        let admin = Address::generate(env);
        let addr = env.register_stellar_asset_contract_v2(admin).address();
        (
            addr.clone(),
            token::Client::new(env, &addr),
            token::StellarAssetClient::new(env, &addr),
        )
    }

    #[test]
    fn test_on_flash_loan_approves_exact_repayment() {
        let env = Env::default();
        env.mock_all_auths();

        let pool1 = env.register(MockAMM, ());
        let pool2 = env.register(MockAMM, ());
        MockAMMClient::new(&env, &pool1).init(&11000u32); // A->B at 1.1x
        MockAMMClient::new(&env, &pool2).init(&10800u32); // B->A at 1.08x

        let owner = Address::generate(&env);
        let arb_addr = env.register(ArbitrageContract, ());
        let arb = ArbitrageContractClient::new(&env, &arb_addr);
        arb.init(&owner);

        let (ta_addr, ta_c, ta_s) = make_token(&env);
        let (tb_addr, _tb_c, tb_s) = make_token(&env);

        let amount = 1000i128;
        let fee = 5i128;

        // Simulate flash loan: arb contract holds the borrowed amount
        ta_s.mint(&arb_addr, &amount);
        // Pools need liquidity for the swaps
        tb_s.mint(&pool1, &(amount * 11000 / 10000 + 100));
        ta_s.mint(&pool2, &(amount + 200));

        // Prime temporary storage as execute() would
        env.as_contract(&arb_addr, || {
            env.storage().temporary().set(&DataKey::Pool1, &pool1);
            env.storage().temporary().set(&DataKey::Pool2, &pool2);
            env.storage().temporary().set(&DataKey::TokenB, &tb_addr);
        });

        let flash_loan_addr = Address::generate(&env);
        arb.on_flash_loan(&flash_loan_addr, &ta_addr, &amount, &fee);

        // Arb contract must approve exactly amount+fee to flash loan provider
        assert_eq!(ta_c.allowance(&arb_addr, &flash_loan_addr), amount + fee);
    }
}

// ============================================================
// Refinancing tests
// ============================================================

mod refinancing_tests {
    use super::*;
    use crate::refinancing::{DataKey, RefinancingContract, RefinancingContractClient};

    /// Mock lending pool
    #[contract]
    struct MockPool;

    #[contractimpl]
    impl MockPool {
        pub fn init(_env: Env) {}

        pub fn repay_and_withdraw(_env: Env, _token: Address, amount: i128) -> i128 {
            // Return 1.5x the debt as collateral amount
            amount * 15000 / 10000
        }

        pub fn deposit_collateral(_env: Env, _token: Address, _amount: i128) {}

        pub fn borrow(_env: Env, _token: Address, _amount: i128) {}
    }


    fn make_token(env: &Env) -> (Address, token::Client, token::StellarAssetClient) {
        let admin = Address::generate(env);
        let addr = env.register_stellar_asset_contract_v2(admin).address();
        (
            addr.clone(),
            token::Client::new(env, &addr),
            token::StellarAssetClient::new(env, &addr),
        )
    }

    #[test]
    fn test_on_flash_loan_approves_exact_repayment() {
        let env = Env::default();
        env.mock_all_auths();

        let old_pool = env.register(MockPool, ());
        let new_pool = env.register(MockPool, ());
        MockPoolClient::new(&env, &old_pool).init();
        MockPoolClient::new(&env, &new_pool).init();

        let owner = Address::generate(&env);
        let refi_addr = env.register(RefinancingContract, ());
        let refi = RefinancingContractClient::new(&env, &refi_addr);
        refi.init(&owner);

        let (debt_addr, debt_c, debt_s) = make_token(&env);
        let (coll_addr, _, _) = make_token(&env);

        let amount = 1000i128;
        let fee = 5i128;

        // Flash loan transferred `amount` to refi contract
        debt_s.mint(&refi_addr, &amount);

        // Prime temporary storage
        env.as_contract(&refi_addr, || {
            env.storage().temporary().set(&DataKey::OldPool, &old_pool);
            env.storage().temporary().set(&DataKey::NewPool, &new_pool);
            env.storage()
                .temporary()
                .set(&DataKey::Collateral, &coll_addr);
            env.storage().temporary().set(&DataKey::DebtAmount, &amount);
        });

        let flash_loan_addr = Address::generate(&env);
        refi.on_flash_loan(&flash_loan_addr, &debt_addr, &amount, &fee);

        // Must approve exactly amount+fee to flash loan provider
        assert_eq!(debt_c.allowance(&refi_addr, &flash_loan_addr), amount + fee);
    }
}
