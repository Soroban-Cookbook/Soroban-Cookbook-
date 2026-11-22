#![no_main]
#![allow(deprecated)]

use libfuzzer_sys::fuzz_target;
use sep41_token::{Sep41Token, Sep41TokenClient};
use soroban_sdk::testutils::arbitrary::{arbitrary, Arbitrary};
use soroban_sdk::{symbol_short, Address, Env, String};

#[derive(Arbitrary, Debug)]
struct Input {
    initial_supply: i128,
    amount: i128,
    self_transfer: bool,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let recipient = if input.self_transfer {
        admin.clone()
    } else {
        Address::generate(&env)
    };
    let contract_id = env.register(Sep41Token, ());
    let client = Sep41TokenClient::new(&env, &contract_id);
    let supply = input.initial_supply.max(0);

    let initialized = client.try_initialize(
        &admin,
        &String::from_str(&env, "Fuzz Token"),
        &symbol_short!("FUZZ"),
        &7,
        &supply,
    );
    assert!(matches!(initialized, Ok(Ok(()))));

    let result = client.try_transfer(&admin, &recipient, &input.amount);
    let should_transfer = input.amount > 0 && input.amount <= supply && !input.self_transfer;

    match result {
        Ok(Ok(())) => {
            assert!(should_transfer);
            assert_eq!(client.balance(&admin), supply - input.amount);
            assert_eq!(client.balance(&recipient), input.amount);
        }
        Ok(Err(_)) | Err(Ok(_)) => {
            assert!(!should_transfer);
            assert_eq!(client.balance(&admin), supply);
            assert_eq!(
                client.balance(&recipient),
                if input.self_transfer { supply } else { 0 }
            );
        }
        Err(Err(error)) => panic!("transfer invocation failed: {error:?}"),
    }

    assert_eq!(client.total_supply().unwrap(), supply);
    if !input.self_transfer {
        assert_eq!(
            client.balance(&admin) + client.balance(&recipient),
            supply
        );
    }
});
