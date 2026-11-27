#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Env, IntoVal,
};

#[contract]
pub struct Dummy;

#[contractimpl]
impl Dummy {
    pub fn run(_env: Env) -> u32 {
        42
    }
}

#[contract]
pub struct DummyNinetyNine;

#[contractimpl]
impl DummyNinetyNine {
    pub fn run(_env: Env) -> u32 {
        99
    }
}

#[test]
fn test_init() {
    let env = Env::default();
    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    // Can't initialize twice
    let res = client.try_init(&admin, &100, &1_000);
    assert_eq!(res.err(), Some(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_queue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env, 1u32.into_val(&env)];

    let id = client.queue(&target, &func, &args, &150);
    assert_eq!(id, 1);

    let prop = client.get_proposal(&1);
    assert_eq!(prop.id, 1);
    assert_eq!(prop.status, ProposalStatus::Queued);
}

#[test]
fn test_queue_invalid_delay() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];

    let res = client.try_queue(&target, &func, &args, &50);
    assert_eq!(res.err(), Some(Ok(Error::InvalidDelay)));
}

#[test]
fn test_queue_delay_bounds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // Allowed delays are 100..=1_000 seconds.
    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];

    // Too short.
    let res = client.try_queue(&target, &func, &args, &99);
    assert_eq!(res.err(), Some(Ok(Error::InvalidDelay)));

    // Too long, including the "lock governance forever" case.
    let res = client.try_queue(&target, &func, &args, &1_001);
    assert_eq!(res.err(), Some(Ok(Error::InvalidDelay)));
    let res = client.try_queue(&target, &func, &args, &u64::MAX);
    assert_eq!(res.err(), Some(Ok(Error::InvalidDelay)));

    // Both bounds are inclusive, and rejected calls queued nothing.
    assert_eq!(client.queue(&target, &func, &args, &100), 1);
    assert_eq!(client.queue(&target, &func, &args, &1_000), 2);
}

#[test]
fn test_init_rejects_max_delay_below_min_delay() {
    let env = Env::default();
    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let res = client.try_init(&admin, &100, &99);
    assert_eq!(res.err(), Some(Ok(Error::InvalidDelay)));

    // min == max is a valid fixed delay, and a failed init does not stick.
    client.init(&admin, &100, &100);
}

#[test]
fn test_execute_delay_not_met() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];

    let id = client.queue(&target, &func, &args, &150);

    let res = client.try_execute(&id);
    assert_eq!(res.err(), Some(Ok(Error::DelayNotMet)));
}

#[test]
fn test_execute_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let dummy_id = env.register(Dummy, ());

    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];

    env.ledger().with_mut(|l| l.timestamp = 1000);
    let id = client.queue(&target, &func, &args, &150);

    env.ledger().with_mut(|l| l.timestamp = 1151); // fast forward

    let res: soroban_sdk::Val = client.execute(&id);
    let val: u32 = res.into_val(&env);
    assert_eq!(val, 42);

    let prop = client.get_proposal(&id);
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
fn test_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];

    let id = client.queue(&target, &func, &args, &150);
    client.cancel(&id);

    let prop = client.get_proposal(&id);
    assert_eq!(prop.status, ProposalStatus::Canceled);
}

#[test]
fn test_cancel_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let res = client.try_cancel(&1);
    assert_eq!(res.err(), Some(Ok(Error::ProposalNotFound)));
}

#[test]
fn test_execute_canceled() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];

    let id = client.queue(&target, &func, &args, &150);
    client.cancel(&id);

    env.ledger().with_mut(|l| l.timestamp = 1000);
    let res = client.try_execute(&id);
    assert_eq!(res.err(), Some(Ok(Error::ProposalNotQueued)));
}

#[test]
fn test_execute_already_executed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let dummy_id = env.register(Dummy, ());

    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];

    env.ledger().with_mut(|l| l.timestamp = 1000);
    let id = client.queue(&target, &func, &args, &150);

    env.ledger().with_mut(|l| l.timestamp = 1151); // fast forward

    client.execute(&id);
    let res = client.try_execute(&id);
    assert_eq!(res.err(), Some(Ok(Error::ProposalNotQueued)));
}

#[test]
fn test_emergency_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let dummy_id = env.register(DummyNinetyNine, ());

    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];

    let id = client.queue(&target, &func, &args, &150);

    // execute immediately using emergency, skipping delay
    let res: soroban_sdk::Val = client.emergency_execute(&id);
    let val: u32 = res.into_val(&env);
    assert_eq!(val, 99);
}

#[test]
fn test_emergency_execute_already_executed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let dummy_id = env.register(DummyNinetyNine, ());

    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];

    env.ledger().with_mut(|l| l.timestamp = 1000);
    let id = client.queue(&target, &func, &args, &150);

    env.ledger().with_mut(|l| l.timestamp = 1151);
    client.execute(&id);

    let res = client.try_emergency_execute(&id);
    assert_eq!(res.err(), Some(Ok(Error::ProposalNotQueued)));
}

#[test]
fn test_emergency_execute_canceled() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TimelockGovernance, ());
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100, &1_000);

    let target = Address::generate(&env);
    let func = Symbol::new(&env, "run");
    let args = vec![&env];

    let id = client.queue(&target, &func, &args, &150);
    client.cancel(&id);

    let res = client.try_emergency_execute(&id);
    assert_eq!(res.err(), Some(Ok(Error::ProposalNotQueued)));
}
