#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction},
    vec, Address, Env, IntoVal, Symbol,
};

#[test]
fn test_auth_success() {
    let env = Env::default();

    // 1. Register the contract and create a client
    let contract_id = env.register_contract(None, AuthContract);
    let client = AuthContractClient::new(&env, &contract_id);

    // 2. Generate a random address to act as our user
    let user = Address::generate(&env);

    // 3. Enable auth mocking so we don't need real signatures for the test
    env.mock_all_auths();

    // 4. Call the function
    client.secure_action(&user);

    // 5. Verify the authentication recorded by the host
    let auths = env.auths();
    let (authorized_address, invocation) = &auths[0];

    // Check that the correct user was the one being asked for auth
    assert_eq!(authorized_address, &user);

    // Verify the structure of the call.
    // In SDK 21.7.7, Contract auth is a tuple: (Address, Symbol, Vec<Val>)
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            contract_id.clone(),
            Symbol::new(&env, "secure_action"),
            vec![&env, user.into_val(&env)],
        ))
    );
}
