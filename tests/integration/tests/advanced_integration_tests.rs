#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Symbol, Vec,
};

#[test]
fn test_version_registry_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    let contract_addr = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    let v1 = reg.register(&contract_addr, &hash, &symbol_short!("deploy"));
    assert_eq!(v1.timestamp, 1000);
    assert_eq!(reg.get_current_version_number(), 1);

    env.ledger().with_mut(|l| l.timestamp = 2000);
    let hash2 = BytesN::from_array(&env, &[1u8; 32]);
    let v2 = reg.register(&contract_addr, &hash2, &symbol_short!("upgrade"));
    assert_eq!(v2.timestamp, 2000);
    assert_eq!(reg.get_current_version_number(), 2);

    let rollback_entry = reg.rollback();
    assert_eq!(rollback_entry.version, v2.version);
    assert_eq!(reg.get_current_version_number(), 1);

    let all = reg.get_all_versions();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_version_registry_multiple_contracts() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    let mut contracts: Vec<Address> = Vec::new(&env);
    for _ in 0..5 {
        contracts.push_back(Address::generate(&env));
    }
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    for (i, addr) in contracts.iter().enumerate() {
        let tag = Symbol::new(&env, &format!("c{}_deploy", i));
        reg.register(&addr, &hash, &tag);
    }
    assert_eq!(reg.get_current_version_number(), 5);
    assert_eq!(reg.get_all_versions().len(), 5);
}

#[test]
fn test_version_registry_contract_history_independent() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    reg.register(&c1, &hash, &symbol_short!("v1"));
    reg.register(&c2, &hash, &symbol_short!("v1"));
    reg.register(&c1, &hash, &symbol_short!("v2"));

    assert_eq!(reg.get_contract_history(&c1).len(), 2);
    assert_eq!(reg.get_contract_history(&c2).len(), 1);
}

#[test]
fn test_storage_optimization_packed_config() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let config = opt.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.fee_bps, 25);
}

#[test]
fn test_storage_optimization_withdraw_updates_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let user = Address::generate(&env);
    opt.deposit(&user, &1000);
    let remaining = opt.withdraw(&user, &300);
    assert_eq!(remaining, 700);
    assert_eq!(opt.get_balance(&user), 700);
}

#[test]
fn test_storage_optimization_batch_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let mut users: Vec<Address> = Vec::new(&env);
    for _ in 0..4 {
        users.push_back(Address::generate(&env));
    }
    let deposits = Vec::from_array(
        &env,
        [
            (users.get(0).unwrap(), 100i128),
            (users.get(1).unwrap(), 200i128),
            (users.get(2).unwrap(), 300i128),
            (users.get(3).unwrap(), 400i128),
        ],
    );

    let count = opt.batch_deposit(&deposits);
    assert_eq!(count, 4);

    let balances = opt.batch_get_balances(&users);
    assert_eq!(balances.get(0).unwrap(), 100);
    assert_eq!(balances.get(1).unwrap(), 200);
    assert_eq!(balances.get(2).unwrap(), 300);
    assert_eq!(balances.get(3).unwrap(), 400);
}

#[test]
fn test_storage_optimization_fee_update_reflected() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    opt.update_fee(&500);
    let config = opt.get_config();
    assert_eq!(config.fee_bps, 500);

    opt.update_fee(&25);
    let config2 = opt.get_config();
    assert_eq!(config2.fee_bps, 25);
}

#[test]
fn test_storage_optimization_pause_resume() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);
    assert!(!opt.get_config().paused);

    opt.set_paused(&true);
    assert!(opt.get_config().paused);

    opt.set_paused(&false);
    assert!(!opt.get_config().paused);
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_storage_optimization_deposit_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);
    opt.set_paused(&true);
    let user = Address::generate(&env);
    opt.deposit(&user, &100);
}

#[test]
fn test_cross_scenario_multi_version_storage() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let reg_admin = Address::generate(&env);
    reg.initialize(&reg_admin);

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let opt_admin = Address::generate(&env);
    opt.initialize(&opt_admin);

    let opt_addr = opt_id.clone();
    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let v1 = reg.register(&opt_addr, &hash, &symbol_short!("initial"));
    assert_eq!(v1.version, Symbol::new(&env, "v1"));

    env.ledger().with_mut(|l| l.timestamp = 2000);
    let hash2 = BytesN::from_array(&env, &[2u8; 32]);
    let v2 = reg.register(&opt_addr, &hash2, &symbol_short!("optimized"));
    assert_eq!(v2.timestamp, 2000);

    let user = Address::generate(&env);
    opt.deposit(&user, &500);
    assert_eq!(opt.get_balance(&user), 500);

    let history = reg.get_contract_history(&opt_addr);
    assert_eq!(history.len(), 2);

    let rolled = reg.rollback();
    assert_eq!(rolled.version, v2.version);
    assert_eq!(reg.get_current_version_number(), 1);
}

#[test]
fn test_version_registry_unauthorized_operations() {
    let env = Env::default();
    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    reg.initialize(&admin);
    env.set_auths(&[]);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let addr = Address::generate(&env);
    let result = reg.try_register(&addr, &hash, &symbol_short!("hack"));
    assert!(result.is_err());
}

#[test]
fn test_storage_optimization_nonce_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let user = Address::generate(&env);
    opt.deposit(&user, &100);
    let data1 = opt.get_user_data(&user);
    assert!(data1.nonce > 0);

    opt.deposit(&user, &150);
    let data2 = opt.get_user_data(&user);
    assert!(data2.nonce > data1.nonce);
}

#[test]
fn test_version_registry_multiple_rollbacks() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    let addr = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    let _v1 = reg.register(&addr, &hash, &symbol_short!("v1"));
    let _v2 = reg.register(&addr, &hash, &symbol_short!("v2"));
    let _v3 = reg.register(&addr, &hash, &symbol_short!("v3"));
    assert_eq!(reg.get_current_version_number(), 3);

    reg.rollback();
    assert_eq!(reg.get_current_version_number(), 2);
    assert_eq!(reg.get_all_versions().len(), 2);

    reg.rollback();
    assert_eq!(reg.get_current_version_number(), 1);
    assert_eq!(reg.get_all_versions().len(), 1);
}

#[test]
fn test_storage_optimization_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let user = Address::generate(&env);
    opt.deposit(&user, &100);
    let result = opt.try_withdraw(&user, &200);
    assert!(result.is_err());
}

#[test]
fn test_empty_version_registry_queries() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    assert_eq!(reg.get_current_version_number(), 0);
    assert_eq!(reg.get_all_versions().len(), 0);

    let result = reg.try_get_latest_version();
    assert!(result.is_err());
}

#[test]
fn test_version_registry_version_numbering() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let admin = Address::generate(&env);
    reg.initialize(&admin);

    let addr = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    let v1 = reg.register(&addr, &hash, &symbol_short!("a"));
    assert_eq!(v1.version, Symbol::new(&env, "v1"));

    let v2 = reg.register(&addr, &hash, &symbol_short!("b"));
    assert_eq!(v2.version, Symbol::new(&env, "v2"));

    let v3 = reg.register(&addr, &hash, &symbol_short!("c"));
    assert_eq!(v3.version, Symbol::new(&env, "v3"));

    let fetched = reg.get_version_by_number(&2);
    assert_eq!(fetched.version, Symbol::new(&env, "v2"));
}

#[test]
fn test_storage_optimization_batch_get_balances_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let opt_id = env.register_contract(None, storage_optimization::StorageOptimization);
    let opt = storage_optimization::StorageOptimizationClient::new(&env, &opt_id);
    let admin = Address::generate(&env);
    opt.initialize(&admin);

    let users = Vec::<Address>::new(&env);
    let balances = opt.batch_get_balances(&users);
    assert_eq!(balances.len(), 0);
}
