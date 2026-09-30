#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    vec, Env, IntoVal,
};

// `#[contractimpl]` cannot resolve a contract declared inside a function body,
// so the dummy targets invoked by the execution tests live at module scope.
#[contract]
pub struct Dummy42;

#[contractimpl]
impl Dummy42 {
    pub fn run(_env: Env) -> u32 {
        42
    }
}

#[contract]
pub struct Dummy99;

#[contractimpl]
impl Dummy99 {
    pub fn run(_env: Env) -> u32 {
        99
    }
}

#[test]
fn test_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.init(&admin, &100);

    // Can't initialize twice
    let res = client.try_init(&admin, &100);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_queue() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
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
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];
    
    let res = client.try_queue(&target, &func, &args, &50);
    assert_eq!(res, Err(Ok(Error::InvalidDelay)));
}

#[test]
fn test_execute_delay_not_met() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];
    
    let id = client.queue(&target, &func, &args, &150);
    
// `execute` returns `Val`, which has no `PartialEq`, so match the error instead
    // of comparing the whole `Result`.
    assert!(matches!(
        client.try_execute(&id),
        Err(Ok(Error::DelayNotMet))
    ));
}

#[test]
fn test_execute_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    // We need a dummy contract to invoke
    let dummy_id = env.register_contract(None, Dummy42);
    
    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];
    
    env.ledger().set_timestamp(1000);
    let id = client.queue(&target, &func, &args, &150);
    
    env.ledger().set_timestamp(1151); // fast forward
    
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
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
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
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
assert!(matches!(
        client.try_cancel(&1),
        Err(Ok(Error::ProposalNotFound))
    ));
}

#[test]
fn test_execute_canceled() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let target = Address::generate(&env);
    let func = Symbol::new(&env, "some_func");
    let args = vec![&env];
    
    let id = client.queue(&target, &func, &args, &150);
    client.cancel(&id);
    
    env.ledger().set_timestamp(1000);
    assert!(matches!(
        client.try_execute(&id),
        Err(Ok(Error::ProposalNotQueued))
    ));
}

#[test]
fn test_execute_already_executed() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let dummy_id = env.register_contract(None, Dummy42);
    
    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];
    
    env.ledger().set_timestamp(1000);
    let id = client.queue(&target, &func, &args, &150);
    
    env.ledger().set_timestamp(1151); // fast forward
    
    client.execute(&id);
    assert!(matches!(
        client.try_execute(&id),
        Err(Ok(Error::ProposalNotQueued))
    ));
}

#[test]
fn test_emergency_execute() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let dummy_id = env.register_contract(None, Dummy99);
    
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
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let dummy_id = env.register_contract(None, Dummy99);
    
    let target = dummy_id;
    let func = Symbol::new(&env, "run");
    let args = vec![&env];
    
    env.ledger().set_timestamp(1000);
    let id = client.queue(&target, &func, &args, &150);
    
    env.ledger().set_timestamp(1151);
    client.execute(&id);
    
    assert!(matches!(
        client.try_emergency_execute(&id),
        Err(Ok(Error::ProposalNotQueued))
    ));
}

#[test]
fn test_emergency_execute_canceled() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, TimelockGovernance);
    let client = TimelockGovernanceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.init(&admin, &100);
    
    let target = Address::generate(&env);
    let func = Symbol::new(&env, "run");
    let args = vec![&env];
    
    let id = client.queue(&target, &func, &args, &150);
    client.cancel(&id);
    
    assert!(matches!(
        client.try_emergency_execute(&id),
        Err(Ok(Error::ProposalNotQueued))
    ));
}
