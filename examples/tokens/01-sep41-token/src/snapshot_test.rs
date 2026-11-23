#![cfg(test)]
#![allow(deprecated)]

use crate::{Sep41Token, Sep41TokenClient};
use soroban_sdk::{
    testutils::Address as _, xdr::ToXdr, Address, Env, IntoVal, String, Symbol, Val,
};

fn xdr_hex<T>(env: &Env, value: T) -> std::string::String
where
    T: IntoVal<Env, Val>,
{
    value
        .to_xdr(env)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn snapshot_xdr_encoded_public_return_values() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let token_id = env.register_contract(None, Sep41Token);
    let token = Sep41TokenClient::new(&env, &token_id);
    let name = String::from_str(&env, "Soroban USD");
    let symbol = Symbol::new(&env, "SUSD");
    token.initialize(&admin, &name, &symbol, &2u32, &1_000_000i128);
    token.transfer(&admin, &alice, &500_000);
    token.approve(&admin, &alice, &123_456);

    insta::assert_snapshot!("balance_xdr", xdr_hex(&env, token.balance(&alice)));
    insta::assert_snapshot!(
        "allowance_xdr",
        xdr_hex(&env, token.allowance(&admin, &alice))
    );
    insta::assert_snapshot!("name_xdr", xdr_hex(&env, token.name()));
    insta::assert_snapshot!("symbol_xdr", xdr_hex(&env, token.symbol()));
}
