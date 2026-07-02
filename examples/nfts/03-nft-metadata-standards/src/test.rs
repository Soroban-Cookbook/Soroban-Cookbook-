extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deploy and initialize the contract, returning (env, contract_id, admin).
fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(NftMetadataContract, ());
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client
        .initialize(
            &admin,
            &String::from_str(&env, "My Collection"),
            &String::from_str(&env, "MNFT"),
            &String::from_str(&env, ""),
        );

    (env, contract_id, admin)
}

/// Build a minimal valid `TokenMetadata`.
fn valid_metadata(env: &Env) -> TokenMetadata {
    TokenMetadata {
        name: String::from_str(env, "My NFT #1"),
        description: String::from_str(env, "A unique digital collectible"),
        image: String::from_str(env, "ipfs://QmHash/image.png"),
        external_url: String::from_str(env, "https://example.com/nft/1"),
        animation_url: String::from_str(env, ""),
        background_color: String::from_str(env, ""),
        attributes: Vec::new(env),
    }
}

/// Build metadata with a full attribute list.
fn metadata_with_attrs(env: &Env) -> TokenMetadata {
    let attrs = vec![
        env,
        Attribute {
            trait_type: String::from_str(env, "Rarity"),
            value: String::from_str(env, "Legendary"),
        },
        Attribute {
            trait_type: String::from_str(env, "Power"),
            value: String::from_str(env, "100"),
        },
        Attribute {
            trait_type: String::from_str(env, "Color"),
            value: String::from_str(env, "Blue"),
        },
    ];

    TokenMetadata {
        name: String::from_str(env, "Legendary Sword #42"),
        description: String::from_str(env, "A legendary sword with immense power"),
        image: String::from_str(env, "https://cdn.example.com/sword.png"),
        external_url: String::from_str(env, "https://example.com/items/42"),
        animation_url: String::from_str(env, "https://cdn.example.com/sword.mp4"),
        background_color: String::from_str(env, "1A2B3C"),
        attributes: attrs,
    }
}

// ---------------------------------------------------------------------------
// Initialization tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_success() {
    let (env, contract_id, _admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    assert_eq!(
        client.name(),
        String::from_str(&env, "My Collection")
    );
    assert_eq!(client.symbol(), String::from_str(&env, "MNFT"));
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let err = client.try_initialize(
        &admin,
        &String::from_str(&env, "Second"),
        &String::from_str(&env, "SEC"),
        &String::from_str(&env, ""),
    ).unwrap_err();
    assert_eq!(err, Ok(NftError::AlreadyInitialized));
}

// ---------------------------------------------------------------------------
// Minting tests
// ---------------------------------------------------------------------------

#[test]
fn test_mint_success() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &owner, &1u32, &meta);

    assert_eq!(client.owner_of(&1u32), owner);
    assert_eq!(client.balance_of(&owner), 1);
    assert_eq!(client.total_supply(), 1);
}

#[test]
fn test_mint_multiple_tokens() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    for id in 1u32..=5 {
        let mut meta = valid_metadata(&env);
        meta.name = String::from_str(&env, "Token");
        client.mint(&admin, &owner, &id, &meta);
    }

    assert_eq!(client.balance_of(&owner), 5);
    assert_eq!(client.total_supply(), 5);
}

#[test]
fn test_mint_double_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &owner, &1u32, &meta.clone());

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::TokenAlreadyExists));
}

#[test]
fn test_mint_non_admin_fails() {
    let (env, contract_id, _admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let attacker = Address::generate(&env);
    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);

    let err = client.try_mint(&attacker, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::NotAdmin));
}

// ---------------------------------------------------------------------------
// Metadata storage & retrieval tests
// ---------------------------------------------------------------------------

#[test]
fn test_get_metadata_roundtrip() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = metadata_with_attrs(&env);

    client.mint(&admin, &owner, &42u32, &meta.clone());

    let stored = client.get_metadata(&42u32);
    assert_eq!(stored.name, meta.name);
    assert_eq!(stored.description, meta.description);
    assert_eq!(stored.image, meta.image);
    assert_eq!(stored.external_url, meta.external_url);
    assert_eq!(stored.animation_url, meta.animation_url);
    assert_eq!(stored.background_color, meta.background_color);
    assert_eq!(stored.attributes.len(), 3);
}

#[test]
fn test_get_attributes() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = metadata_with_attrs(&env);

    client.mint(&admin, &owner, &1u32, &meta);

    let attrs = client.get_attributes(&1u32);
    assert_eq!(attrs.len(), 3);

    let first = attrs.get(0).unwrap();
    assert_eq!(first.trait_type, String::from_str(&env, "Rarity"));
    assert_eq!(first.value, String::from_str(&env, "Legendary"));
}

#[test]
fn test_get_metadata_nonexistent_fails() {
    let (env, contract_id, _admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let err = client.try_get_metadata(&999u32).unwrap_err();
    assert_eq!(err, Ok(NftError::TokenNotFound));
}

// ---------------------------------------------------------------------------
// Token URI tests
// ---------------------------------------------------------------------------

#[test]
fn test_token_uri_no_base_returns_image() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &owner, &1u32, &meta);

    let uri = client.token_uri(&1u32);
    assert_eq!(uri, String::from_str(&env, "ipfs://QmHash/image.png"));
}

#[test]
fn test_token_uri_with_base_uri() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(NftMetadataContract, ());
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client
        .initialize(
            &admin,
            &String::from_str(&env, "My Collection"),
            &String::from_str(&env, "MNFT"),
            &String::from_str(&env, "https://api.example.com/metadata/"),
        );

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);
    client.mint(&admin, &owner, &7u32, &meta);

    let uri = client.token_uri(&7u32);
    assert_eq!(
        uri,
        String::from_str(&env, "https://api.example.com/metadata/7")
    );
}

#[test]
fn test_token_uri_nonexistent_fails() {
    let (env, contract_id, _admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let err = client.try_token_uri(&999u32).unwrap_err();
    assert_eq!(err, Ok(NftError::TokenNotFound));
}

// ---------------------------------------------------------------------------
// Transfer tests
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_success() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta);
    client.transfer(&alice, &bob, &1u32);

    assert_eq!(client.owner_of(&1u32), bob);
    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 1);
}

#[test]
fn test_transfer_not_owner_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta);

    let err = client.try_transfer(&charlie, &bob, &1u32).unwrap_err();
    assert_eq!(err, Ok(NftError::NotApproved));
}

#[test]
fn test_transfer_clears_approval() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta);
    client.approve(&alice, &bob, &1u32);

    // Bob transfers to Charlie
    client.transfer(&bob, &charlie, &1u32);

    // Approval should be cleared
    assert!(client.get_approved(&1u32).is_none());
}

// ---------------------------------------------------------------------------
// Approval tests
// ---------------------------------------------------------------------------

#[test]
fn test_approve_and_transfer() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta);
    client.approve(&alice, &bob, &1u32);

    assert_eq!(client.get_approved(&1u32).unwrap(), bob);

    // Bob (approved) transfers to Charlie
    client.transfer(&bob, &charlie, &1u32);
    assert_eq!(client.owner_of(&1u32), charlie);
}

#[test]
fn test_set_approval_for_all() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let operator = Address::generate(&env);
    let bob = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta.clone());
    client.mint(&admin, &alice, &2u32, &meta);

    client
        .set_approval_for_all(&alice, &operator, &true);
    assert!(client.is_approved_for_all(&alice, &operator));

    // Operator can transfer any of Alice's tokens
    client.transfer(&operator, &bob, &1u32);
    client.transfer(&operator, &bob, &2u32);

    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 2);
}

#[test]
fn test_revoke_approval_for_all() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let operator = Address::generate(&env);
    let meta = valid_metadata(&env);

    client.mint(&admin, &alice, &1u32, &meta);
    client
        .set_approval_for_all(&alice, &operator, &true);
    client
        .set_approval_for_all(&alice, &operator, &false);

    assert!(!client.is_approved_for_all(&alice, &operator));

    let err = client.try_transfer(&operator, &alice, &1u32).unwrap_err();
    assert_eq!(err, Ok(NftError::NotApproved));
}

// ---------------------------------------------------------------------------
// Metadata validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_validate_empty_name_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.name = String::from_str(&env, "");

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::MetadataFieldEmpty));
}

#[test]
fn test_validate_empty_description_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.description = String::from_str(&env, "");

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::MetadataFieldEmpty));
}

#[test]
fn test_validate_empty_image_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.image = String::from_str(&env, "");

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::MetadataFieldEmpty));
}

#[test]
fn test_validate_invalid_image_uri_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.image = String::from_str(&env, "ftp://bad-scheme.com/img.png");

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::InvalidImageUri));
}

#[test]
fn test_validate_https_image_uri_ok() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.image = String::from_str(&env, "https://cdn.example.com/img.png");

    client.mint(&admin, &owner, &1u32, &meta);
}

#[test]
fn test_validate_data_uri_ok() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.image = String::from_str(&env, "data:image/svg+xml;base64,PHN2Zy8+");

    client.mint(&admin, &owner, &1u32, &meta);
}

#[test]
fn test_validate_invalid_background_color_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    // 7 chars — too long
    meta.background_color = String::from_str(&env, "FFAABB0");

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::InvalidBackgroundColor));
}

#[test]
fn test_validate_valid_background_color_ok() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.background_color = String::from_str(&env, "FF0000");

    client.mint(&admin, &owner, &1u32, &meta);
}

#[test]
fn test_validate_empty_attribute_trait_type_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.attributes = vec![
        &env,
        Attribute {
            trait_type: String::from_str(&env, ""),
            value: String::from_str(&env, "Legendary"),
        },
    ];

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::InvalidAttribute));
}

#[test]
fn test_validate_empty_attribute_value_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let mut meta = valid_metadata(&env);
    meta.attributes = vec![
        &env,
        Attribute {
            trait_type: String::from_str(&env, "Rarity"),
            value: String::from_str(&env, ""),
        },
    ];

    let err = client.try_mint(&admin, &owner, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::InvalidAttribute));
}

// ---------------------------------------------------------------------------
// Metadata update tests
// ---------------------------------------------------------------------------

#[test]
fn test_update_metadata_success() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);
    client.mint(&admin, &owner, &1u32, &meta);

    let updated = metadata_with_attrs(&env);
    client.update_metadata(&admin, &1u32, &updated);

    let stored = client.get_metadata(&1u32);
    assert_eq!(stored.name, String::from_str(&env, "Legendary Sword #42"));
    assert_eq!(stored.attributes.len(), 3);
}

#[test]
fn test_update_metadata_non_admin_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let meta = valid_metadata(&env);
    client.mint(&admin, &owner, &1u32, &meta.clone());

    let attacker = Address::generate(&env);
    let err = client.try_update_metadata(&attacker, &1u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::NotAdmin));
}

#[test]
fn test_update_metadata_nonexistent_token_fails() {
    let (env, contract_id, admin) = setup();
    let client = NftMetadataContractClient::new(&env, &contract_id);

    let meta = valid_metadata(&env);
    let err = client.try_update_metadata(&admin, &999u32, &meta).unwrap_err();
    assert_eq!(err, Ok(NftError::TokenNotFound));
}

// ---------------------------------------------------------------------------
// JSON schema compliance documentation test
// ---------------------------------------------------------------------------

/// This test documents the expected JSON schema for token metadata.
///
/// The `TokenMetadata` struct maps 1-to-1 to the following JSON schema:
///
/// ```json
/// {
///   "$schema": "http://json-schema.org/draft-07/schema",
///   "title": "TokenMetadata",
///   "type": "object",
///   "required": ["name", "description", "image"],
///   "properties": {
///     "name":             { "type": "string", "minLength": 1 },
///     "description":      { "type": "string", "minLength": 1 },
///     "image":            { "type": "string", "pattern": "^(ipfs://|https://|http://|data:)" },
///     "external_url":     { "type": "string" },
///     "animation_url":    { "type": "string" },
///     "background_color": { "type": "string", "pattern": "^([0-9a-fA-F]{6})?$" },
///     "attributes": {
///       "type": "array",
///       "items": {
///         "type": "object",
///         "required": ["trait_type", "value"],
///         "properties": {
///           "trait_type": { "type": "string", "minLength": 1 },
///           "value":      { "type": "string", "minLength": 1 }
///         }
///       }
///     }
///   }
/// }
/// ```
#[test]
fn test_json_schema_compliance() {
    let env = Env::default();

    // Build a fully-populated metadata record
    let meta = TokenMetadata {
        name: String::from_str(&env, "My NFT #1"),
        description: String::from_str(&env, "A unique digital collectible"),
        image: String::from_str(&env, "ipfs://QmHash/image.png"),
        external_url: String::from_str(&env, "https://example.com/nft/1"),
        animation_url: String::from_str(&env, "https://cdn.example.com/nft/1.mp4"),
        background_color: String::from_str(&env, "1A2B3C"),
        attributes: vec![
            &env,
            Attribute {
                trait_type: String::from_str(&env, "Rarity"),
                value: String::from_str(&env, "Legendary"),
            },
            Attribute {
                trait_type: String::from_str(&env, "Power"),
                value: String::from_str(&env, "100"),
            },
        ],
    };

    // All validation rules must pass
    NftMetadataContract::validate_metadata(&env, &meta).unwrap();

    // Verify field presence
    assert!(meta.name.len() > 0);
    assert!(meta.description.len() > 0);
    assert!(meta.image.len() > 0);
    assert_eq!(meta.attributes.len(), 2);

    // Verify attribute schema
    for attr in meta.attributes.iter() {
        assert!(attr.trait_type.len() > 0);
        assert!(attr.value.len() > 0);
    }
}
