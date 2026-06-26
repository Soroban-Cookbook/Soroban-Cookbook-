#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, IssuerFlags},
    token::{StellarAssetClient, TokenClient},
    Address, Env, TryFromVal,
};

struct Fixture {
    env: Env,
    wrapper_id: Address,
    wrapper: TokenWrapperClient<'static>,
    underlying: TokenClient<'static>,
    underlying_admin: StellarAssetClient<'static>,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(admin.clone());
    asset.issuer().set_flag(IssuerFlags::ClawbackEnabledFlag);
    let underlying_id = asset.address();
    let underlying = TokenClient::new(&env, &underlying_id);
    let underlying_admin = StellarAssetClient::new(&env, &underlying_id);

    let wrapper_id = env.register_contract(None, TokenWrapper);
    let wrapper = TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    underlying_admin.mint(&alice, &1_000);
    underlying_admin.mint(&bob, &500);

    Fixture {
        env,
        wrapper_id,
        wrapper,
        underlying,
        underlying_admin,
        alice,
        bob,
    }
}

#[test]
fn wrap_mints_one_to_one_shares() {
    let f = setup();

    assert_eq!(f.wrapper.wrap(&f.alice, &250), 250);

    assert_eq!(f.wrapper.balance(&f.alice), 250);
    assert_eq!(f.wrapper.total_supply(), 250);
    assert_eq!(f.underlying.balance(&f.alice), 750);
    assert_eq!(f.underlying.balance(&f.wrapper_id), 250);

    let backing = f.wrapper.backing();
    assert!(backing.fully_backed);
    assert!(backing.exactly_backed);
    assert_eq!(backing.surplus, 0);
}

#[test]
fn unwrap_burns_shares_and_returns_underlying() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &300);
    assert_eq!(f.wrapper.unwrap(&f.alice, &125), 175);

    assert_eq!(f.wrapper.balance(&f.alice), 175);
    assert_eq!(f.wrapper.total_supply(), 175);
    assert_eq!(f.underlying.balance(&f.alice), 825);
    assert_eq!(f.underlying.balance(&f.wrapper_id), 175);

    let backing = f.wrapper.backing();
    assert!(backing.fully_backed);
    assert!(backing.exactly_backed);
}

#[test]
fn wrapped_transfer_preserves_backing() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &400);
    f.wrapper.transfer(&f.alice, &f.bob, &150);

    assert_eq!(f.wrapper.balance(&f.alice), 250);
    assert_eq!(f.wrapper.balance(&f.bob), 150);
    assert_eq!(f.wrapper.total_supply(), 400);
    assert_eq!(f.underlying.balance(&f.wrapper_id), 400);
    assert!(f.wrapper.backing().exactly_backed);
}

#[test]
fn rejects_zero_and_negative_amounts() {
    let f = setup();

    assert_eq!(
        f.wrapper.try_wrap(&f.alice, &0),
        Err(Ok(WrapperError::InvalidAmount))
    );
    assert_eq!(
        f.wrapper.try_wrap(&f.alice, &-1),
        Err(Ok(WrapperError::InvalidAmount))
    );
    assert_eq!(
        f.wrapper.try_unwrap(&f.alice, &0),
        Err(Ok(WrapperError::InvalidAmount))
    );
}

#[test]
fn rejects_unwrap_above_wrapped_balance() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &100);

    assert_eq!(
        f.wrapper.try_unwrap(&f.alice, &101),
        Err(Ok(WrapperError::InsufficientWrappedBalance))
    );
}

#[test]
fn rejects_double_initialization_and_uninitialized_usage() {
    let f = setup();

    assert_eq!(
        f.wrapper.try_initialize(&f.wrapper_id),
        Err(Ok(WrapperError::AlreadyInitialized))
    );

    let fresh_id = f.env.register_contract(None, TokenWrapper);
    let fresh = TokenWrapperClient::new(&f.env, &fresh_id);
    assert_eq!(
        fresh.try_wrap(&f.alice, &10),
        Err(Ok(WrapperError::NotInitialized))
    );
    assert_eq!(fresh.try_backing(), Err(Ok(WrapperError::NotInitialized)));
}

#[test]
fn backing_reports_surplus_from_direct_underlying_transfer() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &200);
    f.underlying.transfer(&f.bob, &f.wrapper_id, &50);

    let backing = f.wrapper.backing();
    assert_eq!(backing.underlying_balance, 250);
    assert_eq!(backing.wrapped_supply, 200);
    assert_eq!(backing.surplus, 50);
    assert!(backing.fully_backed);
    assert!(!backing.exactly_backed);
}

#[test]
fn unwrap_fails_if_underlying_backing_is_clawed_back() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &200);
    f.underlying_admin.clawback(&f.wrapper_id, &50);

    let backing = f.wrapper.backing();
    assert_eq!(backing.underlying_balance, 150);
    assert_eq!(backing.wrapped_supply, 200);
    assert!(!backing.fully_backed);

    assert_eq!(
        f.wrapper.try_unwrap(&f.alice, &1),
        Err(Ok(WrapperError::NotFullyBacked))
    );
}

fn get_event_topics_and_data(
    env: &Env,
    events: &soroban_sdk::testutils::ContractEvents,
    idx: usize,
) -> (soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val) {
    let event = events.events().get(idx).unwrap();
    let soroban_sdk::xdr::ContractEventBody::V0(body) = &event.body;
    let mut topics = soroban_sdk::Vec::new(env);
    for topic in body.topics.iter() {
        topics.push_back(soroban_sdk::Val::try_from_val(env, topic).unwrap());
    }
    let data = soroban_sdk::Val::try_from_val(env, &body.data).unwrap();
    (topics, data)
}

#[test]
fn emits_wrap_and_unwrap_events() {
    let f = setup();

    f.wrapper.wrap(&f.alice, &100);
    let events_wrap = f.env.events().all();
    assert_eq!(events_wrap.events().len(), 2);

    let (wrap_topics, wrap_amount) = get_event_topics_and_data(&f.env, &events_wrap, 1);
    let wrap_event: Symbol = Symbol::try_from_val(&f.env, &wrap_topics.get(0).unwrap()).unwrap();
    let wrapped: i128 = i128::try_from_val(&f.env, &wrap_amount).unwrap();
    assert_eq!(wrap_event, EVENT_WRAP);
    assert_eq!(wrapped, 100);

    f.wrapper.unwrap(&f.alice, &40);
    let events_unwrap = f.env.events().all();
    assert_eq!(events_unwrap.events().len(), 2);

    let (unwrap_topics, unwrap_amount) = get_event_topics_and_data(&f.env, &events_unwrap, 1);
    let unwrap_event: Symbol =
        Symbol::try_from_val(&f.env, &unwrap_topics.get(0).unwrap()).unwrap();
    let unwrapped: i128 = i128::try_from_val(&f.env, &unwrap_amount).unwrap();
    assert_eq!(unwrap_event, EVENT_UNWRAP);
    assert_eq!(unwrapped, 40);
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------
// Run with: cargo test -p token-wrapper -- --nocapture bench

#[cfg(test)]
mod bench {
    extern crate std;

    use super::*;
    use soroban_sdk::{
        testutils::IssuerFlags,
        token::{StellarAssetClient, TokenClient},
        Address, Env,
    };

    fn setup_bench() -> (
        Env,
        Address,
        TokenWrapperClient<'static>,
        TokenClient<'static>,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let asset = env.register_stellar_asset_contract_v2(admin.clone());
        asset.issuer().set_flag(IssuerFlags::ClawbackEnabledFlag);
        let underlying_id = asset.address();
        let underlying = TokenClient::new(&env, &underlying_id);
        let underlying_admin = StellarAssetClient::new(&env, &underlying_id);

        let wrapper_id = env.register_contract(None, TokenWrapper);
        let wrapper = TokenWrapperClient::new(&env, &wrapper_id);
        wrapper.initialize(&underlying_id);

        let alice = Address::generate(&env);
        underlying_admin.mint(&alice, &10_000);

        (env, wrapper_id, wrapper, underlying, alice)
    }

    #[test]
    fn bench_wrap() {
        let (env, _wrapper_id, wrapper, _underlying, alice) = setup_bench();
        env.budget().reset_default();
        wrapper.wrap(&alice, &1_000);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-wrapper::wrap  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_unwrap() {
        let (env, _wrapper_id, wrapper, _underlying, alice) = setup_bench();
        wrapper.wrap(&alice, &1_000);
        env.budget().reset_default();
        wrapper.unwrap(&alice, &500);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-wrapper::unwrap  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_transfer() {
        let (env, _wrapper_id, wrapper, _underlying, alice) = setup_bench();
        let bob = Address::generate(&env);
        wrapper.wrap(&alice, &1_000);
        env.budget().reset_default();
        wrapper.transfer(&alice, &bob, &400);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-wrapper::transfer  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_backing_query() {
        let (env, _wrapper_id, wrapper, _underlying, alice) = setup_bench();
        wrapper.wrap(&alice, &1_000);
        env.budget().reset_default();
        let _ = wrapper.backing();
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-wrapper::backing  cpu={cpu}  mem={mem}");
    }
}
