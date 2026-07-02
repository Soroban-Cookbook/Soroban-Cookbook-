#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, vec, Address, Env, IntoVal, Symbol,
};

#[contracttype]
pub enum MaliciousDataKey {
    MainContract,
    AttackType,
}

#[contract]
pub struct MaliciousContract;

#[contractimpl]
impl MaliciousContract {
    pub fn init(env: Env, main_contract: Address, attack_type: u32) {
        env.storage()
            .instance()
            .set(&MaliciousDataKey::MainContract, &main_contract);
        env.storage()
            .instance()
            .set(&MaliciousDataKey::AttackType, &attack_type);
    }

    pub fn receive_funds(env: Env, to: Address, amount: i128) {
        let main_contract: Address = env
            .storage()
            .instance()
            .get(&MaliciousDataKey::MainContract)
            .unwrap();
        let attack_type: u32 = env
            .storage()
            .instance()
            .get(&MaliciousDataKey::AttackType)
            .unwrap();

        if attack_type == 1 {
            // Attempt to re-enter withdraw
            let _: () = env.invoke_contract(
                &main_contract,
                &Symbol::new(&env, "withdraw"),
                vec![
                    &env,
                    to.into_val(&env),
                    amount.into_val(&env),
                    env.current_contract_address().into_val(&env),
                ],
            );
        } else if attack_type == 2 {
            // Attempt to read-only re-enter
            let _: i128 = env.invoke_contract(
                &main_contract,
                &Symbol::new(&env, "get_balance"),
                vec![&env, to.into_val(&env)],
            );
        }
    }
}

#[test]
fn test_reentrancy_guard_deposit_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReentrancyGuardContract, ());
    let client = ReentrancyGuardContractClient::new(&env, &contract_id);

    client.init();

    let user = Address::generate(&env);
    client.deposit(&user, &1000);
    assert_eq!(client.get_balance(&user), 1000);

    // Normal withdraw to a non-malicious target contract
    let safe_target = env.register(MaliciousContract, ());
    let safe_target_client = MaliciousContractClient::new(&env, &safe_target);
    safe_target_client.init(&contract_id, &0); // Attack type 0 = do nothing

    client.withdraw(&user, &500, &safe_target);
    assert_eq!(client.get_balance(&user), 500);
}

#[test]
#[should_panic]
fn test_reentrancy_attack_prevented() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReentrancyGuardContract, ());
    let client = ReentrancyGuardContractClient::new(&env, &contract_id);
    client.init();

    let user = Address::generate(&env);
    client.deposit(&user, &1000);

    // Malicious target contract that attempts to call withdraw again
    let malicious_target = env.register(MaliciousContract, ());
    let malicious_target_client = MaliciousContractClient::new(&env, &malicious_target);
    malicious_target_client.init(&contract_id, &1); // Attack type 1 = re-enter withdraw

    client.withdraw(&user, &500, &malicious_target);
}

#[test]
#[should_panic]
fn test_readonly_reentrancy_prevented() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReentrancyGuardContract, ());
    let client = ReentrancyGuardContractClient::new(&env, &contract_id);
    client.init();

    let user = Address::generate(&env);
    client.deposit(&user, &1000);

    // Malicious target contract that attempts to read balance during withdraw
    let malicious_target = env.register(MaliciousContract, ());
    let malicious_target_client = MaliciousContractClient::new(&env, &malicious_target);
    malicious_target_client.init(&contract_id, &2); // Attack type 2 = re-enter get_balance

    client.withdraw(&user, &500, &malicious_target);
}
