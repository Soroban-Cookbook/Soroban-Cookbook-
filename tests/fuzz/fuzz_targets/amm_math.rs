#![no_main]
#![allow(deprecated)]

use constant_product_amm::{ConstantProductAmm, ConstantProductAmmClient};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::testutils::arbitrary::{arbitrary, Arbitrary};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAdminClient};
use soroban_sdk::{Address, Env};

const MAX_AMOUNT: u64 = 1_000_000;

#[derive(Arbitrary, Debug)]
struct Input {
    reserve_x: u64,
    reserve_y: u64,
    sell_amount: u64,
    min_buy_amount: u64,
    sell_x: bool,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let trader = Address::generate(&env);
    let token_x_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_y_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_x_admin = TokenAdminClient::new(&env, &token_x_id);
    let token_y_admin = TokenAdminClient::new(&env, &token_y_id);
    let token_x = TokenClient::new(&env, &token_x_id);
    let token_y = TokenClient::new(&env, &token_y_id);

    let reserve_x = (input.reserve_x % MAX_AMOUNT).max(1) as i128;
    let reserve_y = (input.reserve_y % MAX_AMOUNT).max(1) as i128;
    let sell_amount = (input.sell_amount % MAX_AMOUNT) as i128;
    let min_buy_amount = if input.min_buy_amount % 2 == 0 {
        0
    } else {
        MAX_AMOUNT as i128
    };
    token_x_admin.mint(&provider, &reserve_x);
    token_y_admin.mint(&provider, &reserve_y);
    token_x_admin.mint(&trader, &(MAX_AMOUNT as i128));
    token_y_admin.mint(&trader, &(MAX_AMOUNT as i128));

    let amm_id = env.register(ConstantProductAmm, ());
    let amm = ConstantProductAmmClient::new(&env, &amm_id);
    assert!(matches!(
        amm.try_initialize(&token_x_id, &token_y_id),
        Ok(Ok(()))
    ));
    assert!(matches!(
        amm.try_add_liquidity(&provider, &reserve_x, &reserve_y),
        Ok(Ok(_))
    ));

    let (before_x, before_y) = amm.reserves();
    let sell_token = if input.sell_x {
        token_x_id
    } else {
        token_y_id
    };
    let result = amm.try_swap(&trader, &sell_token, &sell_amount, &min_buy_amount);

    match result {
        Ok(Ok(amount_out)) => {
            let (after_x, after_y) = amm.reserves();
            if input.sell_x {
                assert_eq!(after_x, before_x + sell_amount);
                assert_eq!(after_y, before_y - amount_out);
            } else {
                assert_eq!(after_y, before_y + sell_amount);
                assert_eq!(after_x, before_x - amount_out);
            }
            assert!(amount_out > 0);
            assert!(amount_out < if input.sell_x { before_y } else { before_x });
            assert!(after_x * after_y >= before_x * before_y);
        }
        Ok(Err(_)) | Err(Ok(_)) => assert_eq!(amm.reserves(), (before_x, before_y)),
        Err(Err(error)) => panic!("swap invocation failed: {error:?}"),
    }

    let (reserve_x, reserve_y) = amm.reserves();
    assert_eq!(token_x.balance(&amm_id), reserve_x);
    assert_eq!(token_y.balance(&amm_id), reserve_y);
});
