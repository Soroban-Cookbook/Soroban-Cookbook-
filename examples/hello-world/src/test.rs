#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test_hello() {
    // 1. Initialize the environment.
    // The `Env::default()` gives us an empty execution environment for testing.
    let env = Env::default();

    // 2. Register the contract.
    // We register our `HelloContract` with the environment. This gives us an address
    // that we can use to interact with the contract.
    let contract_id = env.register_contract(None, HelloContract);

    // 3. Create a client.
    // The `HelloContractClient` is generated automatically by the `#[contractimpl]` macro.
    // It provides a convenient way to call contract functions in our tests.
    let client = HelloContractClient::new(&env, &contract_id);

    // 4. Invoke the contract function.
    // We call the `hello` function, passing a short symbol "World".
    // We don't need to pass the `env` explicitly through the client.
    let words = client.hello(&symbol_short!("World"));

    // 5. Assert the expected outcome.
    // We check if the returned vector matches our expected vector of ["Hello", "World"].
    assert_eq!(
        words,
        vec![&env, symbol_short!("Hello"), symbol_short!("World")]
    );
}
