#![no_main]
#![allow(deprecated)]

//! Fuzz the advanced timelock queue / execute / cancel surface.

use libfuzzer_sys::fuzz_target;
use soroban_sdk::testutils::{
    arbitrary::{arbitrary, Arbitrary},
    Address as _, Ledger,
};
use soroban_sdk::{Address, Bytes, Env};
use timelock::{OperationState, TimelockContract, TimelockContractClient};

#[derive(Arbitrary, Debug)]
struct Input {
    delay: u64,
    advance: u64,
    cancel_first: bool,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let client = TimelockContractClient::new(&env, &env.register(TimelockContract, ()));
    client.initialize(&admin);

    let (min_delay, max_delay) = client.get_delay_bounds();
    let op = Bytes::from_slice(&env, b"fuzz-op");

    let _ = client.try_queue(&op, &input.delay);

    if input.cancel_first && client.get_state(&op) == OperationState::Pending {
        let _ = client.try_cancel(&op);
        return;
    }

    let advance = input.advance.min(max_delay.saturating_mul(2));
    env.ledger().with_mut(|l| {
        l.timestamp = l.timestamp.saturating_add(advance);
    });

    let _ = client.try_execute(&op);

    let state = client.get_state(&op);
    assert!(matches!(
        state,
        OperationState::Unknown
            | OperationState::Pending
            | OperationState::Ready
            | OperationState::Done
    ));

    if matches!(state, OperationState::Pending | OperationState::Ready) {
        assert!((min_delay..=max_delay).contains(&input.delay));
    }
});
