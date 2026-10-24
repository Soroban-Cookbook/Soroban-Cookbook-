#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env,
};

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

/// Creates the test environment, registers a native (Stellar asset) mock token,
/// and registers + initializes the marketplace.
///
/// Returns `(env, marketplace_client, token_client, marketplace_contract_id)`.
fn setup() -> (
    Env,
    NftMarketplaceContractClient<'static>,
    TokenClient<'static>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Mint a real SEP-41 token we can use to verify balance changes.
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let token_sac = StellarAssetClient::new(&env, &token_addr);
    let token = TokenClient::new(&env, &token_addr);

    // Give the token admin a large initial supply so tests can mint to actors.
    token_sac.mint(&token_admin, &1_000_000_000);

    // Register and initialise the marketplace.
    let contract_id = env.register(NftMarketplaceContract, ());
    let client = NftMarketplaceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &token_addr);

    // Distribute tokens to a few common actors via the token admin.
    // Individual tests can mint more if they need fresh actors.
    let _ = token_sac; // no longer needed after setup

    (env, client, token, contract_id)
}

/// Mint `amount` of the test token to `recipient` using the token admin held
/// by the Stellar asset client.  Because `env.mock_all_auths()` is active,
/// the admin auth is satisfied automatically.
fn mint(env: &Env, token: &TokenClient, recipient: &Address, amount: i128) {
    let token_addr = token.address.clone();
    let sac = StellarAssetClient::new(env, &token_addr);
    sac.mint(recipient, &amount);
}

// ---------------------------------------------------------------------------
// Fixed-price listing tests
// ---------------------------------------------------------------------------

#[test]
fn test_fixed_price_listing_and_buy() {
    let (env, client, token, contract_id) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty = Address::generate(&env);

    // Fund the buyer.
    mint(&env, &token, &buyer, 200);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 7u32,
    };
    let listing_id =
        client.create_fixed_price_listing(&seller, &vec![&env, item.clone()], &100, &royalty, &50);

    let buyer_before = token.balance(&buyer);
    let seller_before = token.balance(&seller);
    let royalty_before = token.balance(&royalty);

    client.buy(&buyer, &listing_id, &100);

    let listing = client.get_listing(&listing_id);
    assert!(listing.sold);
    assert_eq!(client.trade_count(), 1);

    // 0.5 % royalty (50 bps) on 100 = 0 (integer division — rounds down to 0).
    // seller should receive full 100.
    let royalty_paid = 100_i128 * 50 / 10000; // = 0
    assert_eq!(token.balance(&buyer), buyer_before - 100);
    assert_eq!(token.balance(&royalty), royalty_before + royalty_paid);
    assert_eq!(
        token.balance(&seller),
        seller_before + 100 - royalty_paid
    );
    // Contract should hold nothing after settlement.
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn test_bundle_sale_tracks_trade_history() {
    let (env, client, token, contract_id) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty = Address::generate(&env);

    mint(&env, &token, &buyer, 500);

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
    let listing_id = client.create_fixed_price_listing(&seller, &items, &250, &royalty, &100);

    let seller_before = token.balance(&seller);
    let royalty_before = token.balance(&royalty);

    client.buy(&buyer, &listing_id, &250);

    let trade = client.get_trade(&0u32);
    // 1% royalty (100 bps) on 250 = 2 (integer division)
    assert_eq!(trade.amount, 250);
    assert_eq!(trade.royalty_paid, 2);
    assert_eq!(trade.items.len(), 2);

    assert_eq!(token.balance(&royalty), royalty_before + 2);
    assert_eq!(token.balance(&seller), seller_before + 248);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn test_buy_wrong_price_fails() {
    let (env, client, token, _contract_id) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let royalty = Address::generate(&env);

    mint(&env, &token, &buyer, 500);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 5u32,
    };
    let listing_id =
        client.create_fixed_price_listing(&seller, &vec![&env, item], &100, &royalty, &0);

    let err = client.try_buy(&buyer, &listing_id, &90).unwrap_err();
    assert_eq!(err, Ok(MarketplaceError::InvalidPrice));
}

// ---------------------------------------------------------------------------
// Auction tests
// ---------------------------------------------------------------------------

#[test]
fn test_auction_listing_and_finalize() {
    let (env, client, token, contract_id) = setup();
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let royalty = Address::generate(&env);

    mint(&env, &token, &bidder, 500);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 42u32,
    };
    let listing_id = client.create_auction_listing(
        &seller,
        &vec![&env, item.clone()],
        &100,
        &1u32,
        &royalty,
        &75,
    );

    let bidder_before = token.balance(&bidder);
    client.place_bid(&bidder, &listing_id, &120);

    // Funds should now be locked in the contract.
    assert_eq!(token.balance(&bidder), bidder_before - 120);
    assert_eq!(token.balance(&contract_id), 120);

    // Advance past the auction end.
    env.ledger().with_mut(|li| li.sequence_number += 2);

    let seller_before = token.balance(&seller);
    let royalty_before = token.balance(&royalty);

    client.finalize_auction(&Address::generate(&env), &listing_id);

    let listing = client.get_listing(&listing_id);
    assert!(listing.sold);
    assert_eq!(client.trade_count(), 1);

    // 75 bps royalty on 120 = 0 (rounds down).
    let royalty_paid = 120_i128 * 75 / 10000; // = 0
    assert_eq!(token.balance(&royalty), royalty_before + royalty_paid);
    assert_eq!(token.balance(&seller), seller_before + 120 - royalty_paid);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn test_outbid_refunds_previous_bidder() {
    let (env, client, token, contract_id) = setup();
    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let royalty = Address::generate(&env);

    mint(&env, &token, &bidder1, 500);
    mint(&env, &token, &bidder2, 500);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 10u32,
    };
    let listing_id = client.create_auction_listing(
        &seller,
        &vec![&env, item],
        &100,
        &5u32,
        &royalty,
        &0,
    );

    let b1_before = token.balance(&bidder1);
    let b2_before = token.balance(&bidder2);

    // First bid: 150 tokens escrowed from bidder1.
    client.place_bid(&bidder1, &listing_id, &150);
    assert_eq!(token.balance(&bidder1), b1_before - 150);
    assert_eq!(token.balance(&contract_id), 150);

    // Second bid (outbids bidder1): bidder2 sends 200, bidder1 gets 150 back.
    client.place_bid(&bidder2, &listing_id, &200);
    assert_eq!(token.balance(&bidder1), b1_before); // fully refunded
    assert_eq!(token.balance(&bidder2), b2_before - 200);
    assert_eq!(token.balance(&contract_id), 200); // only bidder2's bid locked

    // Finalize: bidder2 wins.
    env.ledger().with_mut(|li| li.sequence_number += 6);
    let seller_before = token.balance(&seller);
    client.finalize_auction(&Address::generate(&env), &listing_id);

    assert_eq!(token.balance(&seller), seller_before + 200);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
fn test_bid_too_low_fails() {
    let (env, client, _token, _contract_id) = setup();
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 3u32,
    };
    let listing_id = client.create_auction_listing(
        &seller,
        &vec![&env, item.clone()],
        &100,
        &2u32,
        &royalty,
        &50,
    );

    let err = client.try_place_bid(&bidder, &listing_id, &90).unwrap_err();
    assert_eq!(err, Ok(MarketplaceError::BidTooLow));
}

#[test]
fn test_listing_retrieval() {
    let (env, client, _token, _contract_id) = setup();
    let seller = Address::generate(&env);
    let royalty = Address::generate(&env);

    let item = ListingItem {
        nft_contract: Address::generate(&env),
        token_id: 99u32,
    };
    let listing_id =
        client.create_fixed_price_listing(&seller, &vec![&env, item.clone()], &300, &royalty, &100);

    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.seller, seller);
    assert_eq!(listing.items.len(), 1);
    assert_eq!(listing.price, 300);
}
