#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env,
};

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    Address,
    ClaimableBalanceContractClient<'static>,
    TokenClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| {
        l.timestamp = 1_000;
    });

    let depositor = Address::generate(&env);
    let claimant = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(token_admin);
    let token_id = sac.address();
    let token = TokenClient::new(&env, &token_id);
    let token_admin_client = StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(ClaimableBalanceContract, ());
    let client = ClaimableBalanceContractClient::new(&env, &contract_id);

    token_admin_client.mint(&depositor, &10_000);

    (
        env,
        depositor,
        claimant,
        token_id,
        contract_id,
        client,
        token,
    )
}

#[test]
fn deposit_and_claim_full() {
    let (env, depositor, claimant, token_id, contract_id, client, token) = setup();

    client.deposit(
        &depositor,
        &token_id,
        &500,
        &vec![&env, claimant.clone()],
        &TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 500,
        },
    );

    assert_eq!(token.balance(&contract_id), 500);
    client.claim(&claimant, &500);
    assert_eq!(token.balance(&claimant), 500);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn partial_claim_leaves_remainder() {
    let (env, depositor, claimant, token_id, contract_id, client, token) = setup();

    client.deposit(
        &depositor,
        &token_id,
        &1_000,
        &vec![&env, claimant.clone()],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 9_999,
        },
    );

    client.claim(&claimant, &400);
    assert_eq!(token.balance(&claimant), 400);
    assert_eq!(token.balance(&contract_id), 600);
}

#[test]
#[should_panic(expected = "time predicate is not fulfilled")]
fn claim_before_time_bound_panics() {
    let (env, depositor, claimant, token_id, _contract_id, client, _) = setup();

    client.deposit(
        &depositor,
        &token_id,
        &100,
        &vec![&env, claimant.clone()],
        &TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 50_000,
        },
    );

    client.claim(&claimant, &50);
}

#[test]
#[should_panic(expected = "deposit must be positive")]
fn zero_deposit_rejected() {
    let (env, depositor, claimant, token_id, _contract_id, client, _) = setup();
    client.deposit(
        &depositor,
        &token_id,
        &0,
        &vec![&env, claimant],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 9_999,
        },
    );
}
