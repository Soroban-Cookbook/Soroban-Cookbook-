extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, vec, Address, Env, Vec};

fn setup() -> (Env, Address, NftMarketplaceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(NftMarketplaceContract, ());
    let client = NftMarketplaceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, contract_id, client)
}

#[test]
fn test_fixed_price_listing_and_buy() {
    let (env, _contract_id, client) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 7u32,
    };
    let listing_id = client
        .create_fixed_price_listing(&seller, &vec![&env, item.clone()], &100, &royalty, &50);

    client.buy(&buyer, &listing_id, &100);
    let listing = client.get_listing(&listing_id);

    assert!(listing.sold);
    assert_eq!(listing.price, 100);
    assert_eq!(client.trade_count(), 1);
}

#[test]
fn test_bundle_sale_tracks_trade_history() {
    let (env, _contract_id, client) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty = Address::generate(&env);

    let items = vec![
        &env,
        ListingItem {
            nft_contract: Address::generate(&env),
            token_id: 1u32,
        },
        ListingItem {
            nft_contract: Address::generate(&env),
            token_id: 2u32,
        },
    ];
    let listing_id = client
        .create_fixed_price_listing(&seller, &items, &250, &royalty, &100);

    client.buy(&buyer, &listing_id, &250);
    let trade = client.get_trade(&0u32);

    assert_eq!(trade.amount, 250);
    assert_eq!(trade.royalty_paid, 2);
    assert_eq!(trade.items.len(), 2);
}

#[test]
fn test_auction_listing_and_finalize() {
    let (env, _contract_id, client) = setup();
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 42u32,
    };
    let listing_id = client
        .create_auction_listing(&seller, &vec![&env, item.clone()], &100, &1u32, &royalty, &75);

    client.place_bid(&bidder, &listing_id, &120);
    env.ledger().with_mut(|li| li.sequence_number += 2);

    client.finalize_auction(&Address::generate(&env), &listing_id);
    let listing = client.get_listing(&listing_id);

    assert!(listing.sold);
    assert_eq!(client.trade_count(), 1);
}

#[test]
fn test_bid_too_low_fails() {
    let (env, _contract_id, client) = setup();
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 3u32,
    };
    let listing_id = client
        .create_auction_listing(&seller, &vec![&env, item.clone()], &100, &2u32, &royalty, &50);

    let err = client.try_place_bid(&bidder, &listing_id, &90).unwrap_err();
    assert_eq!(err, Ok(MarketplaceError::BidTooLow));
}

#[test]
fn test_listing_retrieval() {
    let (env, _contract_id, client) = setup();
    let seller = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 99u32,
    };
    let listing_id = client
        .create_fixed_price_listing(&seller, &vec![&env, item.clone()], &300, &royalty, &100);

    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.seller, seller);
    assert_eq!(listing.items.len(), 1);
    assert_eq!(listing.price, 300);
}
