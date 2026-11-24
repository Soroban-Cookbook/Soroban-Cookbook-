use sep41_token::{Sep41Token, Sep41TokenClient};
use soroban_sdk::{xdr::ToXdr, Address, Env, Symbol};

const ADMIN: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const SPENDER: &str = "GAAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQDZ7H";

fn xdr_hex<T: ToXdr>(env: &Env, value: T) -> std::string::String {
    let bytes = value.to_xdr(env);
    format!("{:02x?}", bytes.iter().collect::<std::vec::Vec<u8>>())
}

#[test]
fn snapshot_public_return_values() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::from_str(&env, ADMIN);
    let spender = Address::from_str(&env, SPENDER);
    let token_id = env.register(Sep41Token, ());
    let token = Sep41TokenClient::new(&env, &token_id);
    token.initialize(
        &admin,
        &soroban_sdk::String::from_str(&env, "Soroban USD"),
        &Symbol::new(&env, "SUSD"),
        &2u32,
        &1_000_000i128,
    );
    token.approve(&admin, &spender, &300_000);
    token.transfer(&admin, &spender, &250_000);

    insta::assert_snapshot!("balance", xdr_hex(&env, token.balance(&spender)));
    insta::assert_snapshot!(
        "allowance",
        xdr_hex(&env, token.allowance(&admin, &spender))
    );
    insta::assert_snapshot!("name", xdr_hex(&env, token.name()));
    insta::assert_snapshot!("symbol", xdr_hex(&env, token.symbol()));
}
