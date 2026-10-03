#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};
use std::vec::Vec as StdVec;

// ---------------------------------------------------------------------------
// Off-chain Merkle tree builder used by tests.
// Mirrors the canonical sorting and tree building matching 21-merkle-proofs.
// ---------------------------------------------------------------------------

fn hash_pair(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
    let mut buf = Bytes::new(env);
    if a <= b {
        buf.append(&Bytes::from(a.clone()));
        buf.append(&Bytes::from(b.clone()));
    } else {
        buf.append(&Bytes::from(b.clone()));
        buf.append(&Bytes::from(a.clone()));
    }
    env.crypto().sha256(&buf).to_bytes()
}

fn build_levels(env: &Env, leaves: &StdVec<BytesN<32>>) -> StdVec<StdVec<BytesN<32>>> {
    assert!(!leaves.is_empty(), "tree must have at least one leaf");
    let mut levels: StdVec<StdVec<BytesN<32>>> = StdVec::new();
    levels.push(leaves.clone());

    while levels.last().unwrap().len() > 1 {
        let current = levels.last().unwrap();
        let mut next: StdVec<BytesN<32>> = StdVec::new();
        let mut i = 0usize;
        while i < current.len() {
            if i + 1 < current.len() {
                next.push(hash_pair(env, &current[i], &current[i + 1]));
            } else {
                next.push(current[i].clone());
            }
            i += 2;
        }
        levels.push(next);
    }
    levels
}

fn merkle_root(env: &Env, leaves: &StdVec<BytesN<32>>) -> BytesN<32> {
    let levels = build_levels(env, leaves);
    levels.last().unwrap()[0].clone()
}

fn merkle_proof(env: &Env, leaves: &StdVec<BytesN<32>>, mut index: usize) -> Vec<BytesN<32>> {
    let levels = build_levels(env, leaves);
    let mut proof = Vec::new(env);
    for level in levels.iter().take(levels.len() - 1) {
        let sibling_index = index ^ 1;
        if sibling_index < level.len() {
            proof.push_back(level[sibling_index].clone());
        }
        index /= 2;
    }
    proof
}

struct TestFixture {
    env: Env,
    admin: Address,
    token_address: Address,
    token_admin: token::StellarAssetClient<'static>,
    token_client: token::Client<'static>,
    contract_id: Address,
    client: MerkleAirdropContractClient<'static>,
}

fn setup() -> TestFixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin_addr = Address::generate(&env);

    let token_address = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    let contract_id = env.register(MerkleAirdropContract, ());
    let client = MerkleAirdropContractClient::new(&env, &contract_id);

    TestFixture {
        env,
        admin,
        token_address,
        token_admin,
        token_client,
        contract_id,
        client,
    }
}

#[test]
fn test_initialize_and_getters() {
    let f = setup();
    let root = BytesN::from_array(&f.env, &[0u8; 32]);

    f.client.initialize(&f.admin, &f.token_address, &root);

    assert_eq!(f.client.get_admin(), f.admin);
    assert_eq!(f.client.get_token(), f.token_address);
    assert_eq!(f.client.get_root(), root);

    // Verify double initialization fails
    let result = f.client.try_initialize(&f.admin, &f.token_address, &root);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_uninitialized_calls_fail() {
    let env = Env::default();
    let contract_id = env.register(MerkleAirdropContract, ());
    let client = MerkleAirdropContractClient::new(&env, &contract_id);

    let claimer = Address::generate(&env);
    let proof = Vec::new(&env);

    assert_eq!(client.try_get_admin(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_get_token(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_get_root(), Err(Ok(Error::NotInitialized)));
    assert_eq!(
        client.try_claim(&claimer, &100i128, &proof),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn test_successful_claim() {
    let f = setup();

    // Define leaves data: (Address, Amount)
    let claimers = [
        (Address::generate(&f.env), 100i128),
        (Address::generate(&f.env), 250i128),
        (Address::generate(&f.env), 500i128),
        (Address::generate(&f.env), 1000i128),
    ];

    // Compute leaves hashes using contract's own hash_leaf helper
    let mut leaves = StdVec::new();
    for (claimer, amount) in claimers.iter() {
        leaves.push(f.client.hash_leaf(claimer, amount));
    }

    // Build tree root and find proofs
    let root = merkle_root(&f.env, &leaves);
    let proof_0 = merkle_proof(&f.env, &leaves, 0);
    let proof_1 = merkle_proof(&f.env, &leaves, 1);

    // Initialize contract with the tree root
    f.client.initialize(&f.admin, &f.token_address, &root);

    // Fund the contract with enough tokens to distribute
    let total_airdrop_pool = 1850i128; // 100 + 250 + 500 + 1000
    f.token_admin.mint(&f.contract_id, &total_airdrop_pool);

    assert_eq!(f.token_client.balance(&f.contract_id), total_airdrop_pool);

    // Claimer 0 executes the claim
    let (claimer_0, amount_0) = &claimers[0];
    assert!(!f.client.is_claimed(claimer_0));

    f.client.claim(claimer_0, amount_0, &proof_0);

    // Verify claim details
    assert!(f.client.is_claimed(claimer_0));
    assert_eq!(f.token_client.balance(claimer_0), *amount_0);
    assert_eq!(
        f.token_client.balance(&f.contract_id),
        total_airdrop_pool - *amount_0
    );

    // Claimer 1 executes the claim
    let (claimer_1, amount_1) = &claimers[1];
    assert!(!f.client.is_claimed(claimer_1));

    f.client.claim(claimer_1, amount_1, &proof_1);

    // Verify claim details for claimer 1
    assert!(f.client.is_claimed(claimer_1));
    assert_eq!(f.token_client.balance(claimer_1), *amount_1);
}

#[test]
fn test_double_claim_fails() {
    let f = setup();

    let claimers = [
        (Address::generate(&f.env), 100i128),
        (Address::generate(&f.env), 200i128),
    ];

    let mut leaves = StdVec::new();
    for (claimer, amount) in claimers.iter() {
        leaves.push(f.client.hash_leaf(claimer, amount));
    }

    let root = merkle_root(&f.env, &leaves);
    let proof_0 = merkle_proof(&f.env, &leaves, 0);

    f.client.initialize(&f.admin, &f.token_address, &root);
    f.token_admin.mint(&f.contract_id, &300i128);

    let (claimer_0, amount_0) = &claimers[0];

    // First claim succeeds
    f.client.claim(claimer_0, amount_0, &proof_0);
    assert!(f.client.is_claimed(claimer_0));

    // Second claim fails
    let result = f.client.try_claim(claimer_0, amount_0, &proof_0);
    assert_eq!(result, Err(Ok(Error::AlreadyClaimed)));
}

#[test]
fn test_invalid_proof_fails() {
    let f = setup();

    let claimers = [
        (Address::generate(&f.env), 100i128),
        (Address::generate(&f.env), 200i128),
    ];

    let mut leaves = StdVec::new();
    for (claimer, amount) in claimers.iter() {
        leaves.push(f.client.hash_leaf(claimer, amount));
    }

    let root = merkle_root(&f.env, &leaves);
    let proof_0 = merkle_proof(&f.env, &leaves, 0);

    f.client.initialize(&f.admin, &f.token_address, &root);
    f.token_admin.mint(&f.contract_id, &300i128);

    let (claimer_1, amount_1) = &claimers[1];

    // Using proof of claimer 0 for claimer 1 should fail
    let result = f.client.try_claim(claimer_1, amount_1, &proof_0);
    assert_eq!(result, Err(Ok(Error::InvalidProof)));
    assert!(!f.client.is_claimed(claimer_1));
}

#[test]
fn test_tampered_amount_fails() {
    let f = setup();

    let claimers = [
        (Address::generate(&f.env), 100i128),
        (Address::generate(&f.env), 200i128),
    ];

    let mut leaves = StdVec::new();
    for (claimer, amount) in claimers.iter() {
        leaves.push(f.client.hash_leaf(claimer, amount));
    }

    let root = merkle_root(&f.env, &leaves);
    let proof_0 = merkle_proof(&f.env, &leaves, 0);

    f.client.initialize(&f.admin, &f.token_address, &root);
    f.token_admin.mint(&f.contract_id, &300i128);

    let (claimer_0, _amount_0) = &claimers[0];

    // Tampering with the claim amount should invalidate the proof verification
    let result = f.client.try_claim(claimer_0, &150i128, &proof_0);
    assert_eq!(result, Err(Ok(Error::InvalidProof)));
    assert!(!f.client.is_claimed(claimer_0));
}

#[test]
fn test_invalid_amount_fails() {
    let f = setup();
    let root = BytesN::from_array(&f.env, &[0u8; 32]);
    f.client.initialize(&f.admin, &f.token_address, &root);

    let claimer = Address::generate(&f.env);
    let proof = Vec::new(&f.env);

    let result = f.client.try_claim(&claimer, &0i128, &proof);
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));

    let result_negative = f.client.try_claim(&claimer, &-100i128, &proof);
    assert_eq!(result_negative, Err(Ok(Error::InvalidAmount)));
}
