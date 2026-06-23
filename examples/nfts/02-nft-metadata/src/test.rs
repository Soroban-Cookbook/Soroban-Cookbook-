#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

struct Fixture {
    env: Env,
    contract: NFTMetadataContractClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn test_metadata(env: &Env, name: &str, description: &str, image: &str) -> NFTMetadata {
    let mut attributes = Vec::new(env);
    attributes.push_back(NFTAttribute {
        trait_type: String::from_str(env, "Rarity"),
        value: String::from_str(env, "Legendary"),
    });
    NFTMetadata {
        name: String::from_str(env, name),
        description: String::from_str(env, description),
        image: String::from_str(env, image),
        external_url: Some(String::from_str(env, "https://example.com/nft/1")),
        attributes,
    }
}

fn setup(on_chain_metadata: bool) -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, NFTMetadataContract);
    let contract = NFTMetadataContractClient::new(&env, &token_id);
    let base_uri = String::from_str(&env, "https://metadata.example.com/nft");
    contract
        .initialize(&admin, &base_uri, &on_chain_metadata)
        .unwrap();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    Fixture {
        env,
        contract,
        admin,
        alice,
        bob,
    }
}

#[test]
fn initialize_sets_contract_options() {
    let f = setup(false);

    assert_eq!(f.contract.base_uri().unwrap(), String::from_str(&f.env, "https://metadata.example.com/nft"));
    assert_eq!(f.contract.is_on_chain_metadata().unwrap(), false);
}

#[test]
fn mint_with_off_chain_uri_stores_token_uri() {
    let f = setup(false);

    let token_uri = String::from_str(&f.env, "ipfs://bafybeihmytoken/1");
    f.contract
        .mint(&f.admin, &f.alice, &1u64, &Some(token_uri.clone()), &None)
        .unwrap();

    assert_eq!(f.contract.owner_of(&1u64).unwrap(), f.alice);
    assert_eq!(f.contract.token_uri(&1u64).unwrap(), token_uri);
}

#[test]
fn mint_with_on_chain_metadata_stores_structured_metadata() {
    let f = setup(true);
    let metadata = test_metadata(&f.env, "Soroban NFT #1", "On-chain metadata example", "ipfs://bafybeihimage/1");

    f.contract
        .mint(&f.admin, &f.alice, &2u64, &None, &Some(metadata.clone()))
        .unwrap();

    assert_eq!(f.contract.owner_of(&2u64).unwrap(), f.alice);
    assert_eq!(f.contract.metadata(&2u64).unwrap().unwrap(), metadata);
    assert_eq!(f.contract.token_uri(&2u64).unwrap(), String::from_str(&f.env, "https://metadata.example.com/nft/2"));
}

#[test]
fn token_uri_pattern_uses_base_uri_when_uri_is_missing() {
    let f = setup(true);
    let metadata = test_metadata(&f.env, "Soroban NFT #2", "Pattern metadata", "ipfs://bafybeihimage/2");

    f.contract
        .mint(&f.admin, &f.bob, &3u64, &None, &Some(metadata))
        .unwrap();

    assert_eq!(f.contract.token_uri(&3u64).unwrap(), String::from_str(&f.env, "https://metadata.example.com/nft/3"));
}

#[test]
fn setting_metadata_requires_admin_on_on_chain_mode() {
    let f = setup(true);
    let metadata = test_metadata(&f.env, "Soroban NFT #3", "Metadata update", "ipfs://bafybeihimage/3");

    f.contract
        .mint(&f.admin, &f.alice, &4u64, &None, &Some(test_metadata(&f.env, "Soroban NFT #3", "Original metadata", "ipfs://bafybeihimage/3")))
        .unwrap();

    f.contract
        .set_metadata(&f.admin, &4u64, &metadata.clone())
        .unwrap();

    assert_eq!(f.contract.metadata(&4u64).unwrap().unwrap(), metadata);
}

#[test]
fn non_admin_cannot_mint_or_set_metadata() {
    let f = setup(true);
    let metadata = test_metadata(&f.env, "Soroban NFT #4", "Unauthorized test", "ipfs://bafybeihimage/4");

    assert_eq!(
        f.contract.try_mint(&f.bob, &f.alice, &5u64, &None, &Some(metadata.clone())),
        Err(Ok(NFTError::Unauthorized))
    );

    f.contract
        .mint(&f.admin, &f.alice, &5u64, &None, &Some(metadata.clone()))
        .unwrap();

    assert_eq!(
        f.contract.try_set_metadata(&f.bob, &5u64, &metadata),
        Err(Ok(NFTError::Unauthorized))
    );
}

#[test]
fn mint_requires_uri_for_off_chain_mode() {
    let f = setup(false);
    assert_eq!(
        f.contract.try_mint(&f.admin, &f.alice, &6u64, &None, &None),
        Err(Ok(NFTError::TokenUriRequired))
    );
}

#[test]
fn cannot_mint_existing_token_id() {
    let f = setup(false);
    let token_uri = String::from_str(&f.env, "ipfs://bafybeihmytoken/7");

    f.contract
        .mint(&f.admin, &f.alice, &7u64, &Some(token_uri.clone()), &None)
        .unwrap();

    assert_eq!(
        f.contract.try_mint(&f.admin, &f.bob, &7u64, &Some(token_uri), &None),
        Err(Ok(NFTError::TokenAlreadyMinted))
    );
}
