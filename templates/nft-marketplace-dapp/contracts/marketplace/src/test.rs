#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, String};

#[test]
fn test_mint_list_and_buy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(NFTMarketplaceContract, ());
    let client = NFTMarketplaceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    client.initialize(&admin);

    let uri = String::from_str(&env, "ipfs://bafybeicookbooknft123");
    let token_id = client.mint(&creator, &creator, &uri);
    assert_eq!(token_id, 1);

    let nft = client.get_nft(&1);
    assert_eq!(nft.owner, creator);
    assert_eq!(nft.uri, uri);

    // Creator lists NFT for 500 XLM
    client.list_item(&creator, &1, &500);
    let listing = client.get_listing(&1);
    assert_eq!(listing.seller, creator);
    assert_eq!(listing.price, 500);

    // Buyer purchases NFT
    client.buy_item(&buyer, &1);
    let updated_nft = client.get_nft(&1);
    assert_eq!(updated_nft.owner, buyer);

    // Listing should now be removed
    assert!(client.try_get_listing(&1).is_err());
}

#[test]
fn test_cancel_listing() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(NFTMarketplaceContract, ());
    let client = NFTMarketplaceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    client.initialize(&admin);

    let uri = String::from_str(&env, "ipfs://nft-cancel-test");
    let token_id = client.mint(&creator, &creator, &uri);

    client.list_item(&creator, &token_id, &1000);
    assert_eq!(client.get_listing(&token_id).price, 1000);

    client.cancel_listing(&creator, &token_id);
    let nft = client.get_nft(&token_id);
    assert_eq!(nft.owner, creator);
    assert!(client.try_get_listing(&token_id).is_err());
}
