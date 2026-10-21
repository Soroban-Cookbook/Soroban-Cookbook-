#![no_main]
#![allow(deprecated)]

//! Fuzz encode / decode / validate for multi-party auth vectors.

use libfuzzer_sys::fuzz_target;
use multi_party_auth::{MultiPartyAuthContract, MultiPartyAuthContractClient};
use soroban_sdk::testutils::{
    arbitrary::{arbitrary, Arbitrary, SorobanArbitrary},
    Address as _,
};
use soroban_sdk::{Address, Env, IntoVal, Vec};

#[derive(Arbitrary, Debug)]
struct Input {
    count: u8,
    extra: <Vec<Address> as SorobanArbitrary>::Prototype,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let client = MultiPartyAuthContractClient::new(&env, &env.register(MultiPartyAuthContract, ()));

    let mut signers = Vec::new(&env);
    for _ in 0..(input.count as u32).min(25) {
        signers.push_back(Address::generate(&env));
    }
    // The arbitrary prototype can yield vectors holding non-`Address` values;
    // `try_get` skips those instead of panicking in the harness itself.
    let extras: Vec<Address> = input.extra.into_val(&env);
    for i in 0..extras.len().min(5) {
        if let Ok(Some(a)) = extras.try_get(i) {
            signers.push_back(a);
        }
    }

    let Ok(Ok(encoded)) = client.try_encode_auth_vec(&signers) else {
        return;
    };

    assert!(client.validate_auth_vec(&encoded));
    let decoded = client.decode_auth_vec(&encoded);
    assert_eq!(decoded.len(), client.auth_vec_len(&encoded));
    assert_eq!(encoded, client.encode_auth_vec(&decoded));

    if decoded.len() > 0 {
        assert!(client.auth_vec_contains(&encoded, &decoded.get(0).unwrap()));
    }
});
