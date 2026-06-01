#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, String, TryFromVal,
};

struct Fixture {
    env: Env,
    manager: MultiTokenBalanceManagerClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
    usdc_id: Address,
    eurc_id: Address,
    usdc: TokenClient<'static>,
    eurc: TokenClient<'static>,
}

fn metadata(env: &Env, name: &str, symbol: &str, decimals: u32) -> TokenMetadata {
    TokenMetadata {
        name: String::from_str(env, name),
        symbol: String::from_str(env, symbol),
        decimals,
        standard: MetadataStandard::Sep41,
    }
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let usdc_asset = env.register_stellar_asset_contract_v2(admin.clone());
    let eurc_asset = env.register_stellar_asset_contract_v2(admin.clone());
    let usdc_id = usdc_asset.address();
    let eurc_id = eurc_asset.address();
    let usdc = TokenClient::new(&env, &usdc_id);
    let eurc = TokenClient::new(&env, &eurc_id);
    let usdc_admin = StellarAssetClient::new(&env, &usdc_id);
    let eurc_admin = StellarAssetClient::new(&env, &eurc_id);
    usdc_admin.mint(&alice, &1_000);
    eurc_admin.mint(&alice, &500);

    let manager_id = env.register_contract(None, MultiTokenBalanceManager);
    let manager = MultiTokenBalanceManagerClient::new(&env, &manager_id);
    manager.initialize(&admin);
    manager.register_token(&admin, &usdc_id, &metadata(&env, "USD Coin", "USDC", 7));
    manager.register_token(&admin, &eurc_id, &metadata(&env, "Euro Coin", "EURC", 7));

    Fixture {
        env,
        manager,
        admin,
        alice,
        bob,
        usdc_id,
        eurc_id,
        usdc,
        eurc,
    }
}

#[test]
fn batch_balance_returns_registered_token_balances_and_metadata() {
    let f = setup();

    let balances = f.manager.batch_balance(
        &f.alice,
        &vec![&f.env, f.usdc_id.clone(), f.eurc_id.clone()],
    );

    assert_eq!(balances.len(), 2);
    assert_eq!(balances.get(0).unwrap().token, f.usdc_id);
    assert_eq!(balances.get(0).unwrap().balance, 1_000);
    assert_eq!(
        balances.get(0).unwrap().metadata.symbol,
        String::from_str(&f.env, "USDC")
    );
    assert_eq!(balances.get(1).unwrap().token, f.eurc_id);
    assert_eq!(balances.get(1).unwrap().balance, 500);
}

#[test]
fn batch_transfer_moves_multiple_registered_tokens() {
    let f = setup();
    let transfers = vec![
        &f.env,
        TransferRequest {
            token: f.usdc_id.clone(),
            to: f.bob.clone(),
            amount: 250,
        },
        TransferRequest {
            token: f.eurc_id.clone(),
            to: f.bob.clone(),
            amount: 125,
        },
    ];

    f.manager.batch_transfer(&f.alice, &transfers);

    assert_eq!(f.usdc.balance(&f.alice), 750);
    assert_eq!(f.usdc.balance(&f.bob), 250);
    assert_eq!(f.eurc.balance(&f.alice), 375);
    assert_eq!(f.eurc.balance(&f.bob), 125);
}

#[test]
fn register_token_updates_existing_metadata() {
    let f = setup();
    let updated = TokenMetadata {
        name: String::from_str(&f.env, "USD Classic Asset"),
        symbol: String::from_str(&f.env, "USDC"),
        decimals: 7,
        standard: MetadataStandard::StellarAsset,
    };

    f.manager
        .register_token(&f.admin, &f.usdc_id, &updated.clone());

    assert_eq!(f.manager.metadata(&f.usdc_id), updated);
}

#[test]
fn unregister_prevents_later_reads_and_writes() {
    let f = setup();

    f.manager.unregister_token(&f.admin, &f.eurc_id);

    assert_eq!(
        f.manager
            .try_batch_balance(&f.alice, &vec![&f.env, f.eurc_id.clone()]),
        Err(Ok(BalanceManagerError::NotRegistered))
    );
    assert_eq!(
        f.manager.try_batch_transfer(
            &f.alice,
            &vec![
                &f.env,
                TransferRequest {
                    token: f.eurc_id,
                    to: f.bob,
                    amount: 1,
                }
            ],
        ),
        Err(Ok(BalanceManagerError::NotRegistered))
    );
}

#[test]
fn rejects_empty_batches_and_invalid_amounts() {
    let f = setup();

    assert_eq!(
        f.manager.try_batch_balance(&f.alice, &Vec::new(&f.env)),
        Err(Ok(BalanceManagerError::EmptyBatch))
    );
    assert_eq!(
        f.manager.try_batch_transfer(
            &f.alice,
            &vec![
                &f.env,
                TransferRequest {
                    token: f.usdc_id,
                    to: f.bob,
                    amount: 0,
                }
            ],
        ),
        Err(Ok(BalanceManagerError::InvalidAmount))
    );
}

#[test]
fn rejects_wrong_registry_admin() {
    let f = setup();
    let not_admin = Address::generate(&f.env);

    assert_eq!(
        f.manager
            .try_register_token(&not_admin, &f.usdc_id, &metadata(&f.env, "Fake", "FAKE", 7)),
        Err(Ok(BalanceManagerError::NotAuthorized))
    );
}

#[test]
fn emits_registry_and_batch_events() {
    let f = setup();

    f.manager.batch_transfer(
        &f.alice,
        &vec![
            &f.env,
            TransferRequest {
                token: f.usdc_id,
                to: f.bob,
                amount: 10,
            },
        ],
    );

    let events = f.env.events().all();
    let (_id, topics, value) = events.get(events.len() - 1).unwrap();
    let event_name: Symbol = Symbol::try_from_val(&f.env, &topics.get(0).unwrap()).unwrap();
    let len: u32 = u32::try_from_val(&f.env, &value).unwrap();
    assert_eq!(event_name, EVENT_BATCH);
    assert_eq!(len, 1);
}
