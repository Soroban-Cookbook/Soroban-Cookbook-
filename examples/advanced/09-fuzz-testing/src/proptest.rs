#![allow(deprecated)]
//! Property tests mirroring the `advanced_claimable_balance` fuzz target.
//!
//! These run under `cargo test` without nightly/`cargo-fuzz`, so they catch
//! invariant breaks in CI. The dedicated fuzz binary explores the same
//! surface more aggressively.

#![cfg(test)]

extern crate std;

use super::*;

use ::proptest::prelude::*;
use arbitrary::Arbitrary;
use proptest_arbitrary_interop::arb;
use soroban_sdk::testutils::{arbitrary::arbitrary, Address as _, Ledger};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::token::StellarAssetClient as TokenAdminClient;
use soroban_sdk::{vec, Address, Env};

#[derive(Arbitrary, Debug, Clone)]
struct Input {
    deposit_amount: i128,
    claim_amount: i128,
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn deposit_claim_invariants(input in arb::<Input>()) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp = 12_345;
            ledger_info.sequence_number = 10;
        });
        env.cost_estimate().budget().reset_unlimited();

        let depositor = Address::generate(&env);
        let claimant = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let sac = env.register_stellar_asset_contract_v2(token_admin);
        let token_id = sac.address();
        let token_client = TokenClient::new(&env, &token_id);
        let token_admin_client = TokenAdminClient::new(&env, &token_id);

        let contract_id = env.register(ClaimableBalanceContract, ());
        let client = ClaimableBalanceContractClient::new(&env, &contract_id);

        token_admin_client.mint(&depositor, &i128::MAX);

        let _ = client.try_deposit(
            &depositor,
            &token_id,
            &input.deposit_amount,
            &vec![&env, claimant.clone()],
            &TimeBound {
                kind: TimeBoundKind::Before,
                timestamp: 123_456,
            },
        );
        assert_invariants(&env, &contract_id, &token_client, &input);

        let _ = client.try_claim(&claimant, &input.claim_amount);
        assert_invariants(&env, &contract_id, &token_client, &input);
    }
}

fn assert_invariants(env: &Env, contract_id: &Address, token_client: &TokenClient, input: &Input) {
    env.as_contract(contract_id, || {
        let storage = env.storage().persistent();
        let is_init = storage.has(&DataKey::Init);
        let claimable = storage.get::<_, ClaimableBalance>(&DataKey::Balance);
        let held = token_client.balance(contract_id);

        assert!(match (is_init, claimable.is_some()) {
            (false, false) => true,
            (false, true) => false,
            (true, true) => true,
            (true, false) => true,
        });
        assert!(held >= 0);

        if let Some(balance) = claimable {
            assert!(balance.amount > 0);
            assert!(balance.amount <= input.deposit_amount);
            assert_eq!(balance.amount, held);
            assert!(!balance.claimants.is_empty());
        }
    });
}
