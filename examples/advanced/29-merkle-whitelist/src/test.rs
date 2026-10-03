#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Bytes, BytesN, Env, Vec,
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(env, &token_address.address());
    (token_address.address(), token_client)
}

struct TestMerkleTree {
    leaves: Vec<BytesN<32>>,
}

impl TestMerkleTree {
    fn new(env: &Env) -> Self {
        Self { leaves: Vec::new(env) }
    }

    fn add_leaf(&mut self, leaf: BytesN<32>) {
        self.leaves.push_back(leaf);
    }

    fn build_root(&self, env: &Env) -> BytesN<32> {
        if self.leaves.is_empty() {
            return BytesN::from_array(env, &[0u8; 32]);
        }
        if self.leaves.len() == 1 {
            return self.leaves.get(0).unwrap();
        }

        let mut current_level = self.leaves.clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new(env);
            let mut i = 0;
            while i < current_level.len() {
                let left = current_level.get(i).unwrap();
                let right = if i + 1 < current_level.len() {
                    current_level.get(i + 1).unwrap()
                } else {
                    left.clone()
                };
                next_level.push_back(hash_pair_test(env, &left, &right));
                i += 2;
            }
            current_level = next_level;
        }
        current_level.get(0).unwrap()
    }

    fn generate_proof(&self, env: &Env, leaf_index: u32) -> Vec<BytesN<32>> {
        let mut proof = Vec::new(env);
        if self.leaves.is_empty() || leaf_index >= self.leaves.len() {
            return proof;
        }

        let mut current_level = self.leaves.clone();
        let mut index = leaf_index;

        while current_level.len() > 1 {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            if sibling_index < current_level.len() {
                proof.push_back(current_level.get(sibling_index).unwrap());
            } else {
                proof.push_back(current_level.get(index).unwrap());
            }

            let mut next_level = Vec::new(env);
            let mut i = 0;
            while i < current_level.len() {
                let left = current_level.get(i).unwrap();
                let right = if i + 1 < current_level.len() {
                    current_level.get(i + 1).unwrap()
                } else {
                    left.clone()
                };
                next_level.push_back(hash_pair_test(env, &left, &right));
                i += 2;
            }
            current_level = next_level;
            index /= 2;
        }
        proof
    }
}

fn hash_pair_test(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
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

fn compute_leaf_hash_test(env: &Env, address: &Address, nonce: u64, metadata: &Bytes) -> BytesN<32> {
    let mut buf = Bytes::new(env);
    buf.append(&address.to_string());
    for byte in nonce.to_be_bytes().iter() {
        buf.push_back(*byte);
    }
    buf.append(metadata);
    env.crypto().sha256(&buf).to_bytes()
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);
    let initial_root = BytesN::from_array(&env, &[1u8; 32]);

    let result = client.initialize(&admin, &token_address, &1000000, &initial_root);
    assert!(result.is_ok());
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_merkle_root(), initial_root);
    assert_eq!(client.has_role(&admin, &Role::Admin), true);
}

#[test]
fn test_verify_whitelist_valid_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let mut tree = TestMerkleTree::new(&env);
    let metadata = Bytes::from_slice(&env, b"test");
    let leaf = compute_leaf_hash_test(&env, &user, 0, &metadata);
    tree.add_leaf(leaf);

    let root = tree.build_root(&env);
    client.initialize(&admin, &token_address, &1000000, &root);
    client.grant_fee_waiver(&admin, &user);

    let proof = tree.generate_proof(&env, 0);
    let result = client.try_verify_whitelist(&user, &proof, &metadata);
    assert!(result.is_ok());
    assert_eq!(client.is_whitelisted(&user), true);
}

#[test]
fn test_verify_whitelist_invalid_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);
    client.grant_fee_waiver(&admin, &user);

    let invalid_proof = Vec::new(&env);
    let metadata = Bytes::from_slice(&env, b"test");
    let result = client.try_verify_whitelist(&user, &invalid_proof, &metadata);
    assert_eq!(result, Err(Ok(Error::InvalidProof)));
}

#[test]
fn test_blacklisted_cannot_verify() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);
    client.add_to_blacklist(&admin, &user);

    let proof = Vec::new(&env);
    let metadata = Bytes::new(&env);
    let result = client.try_verify_whitelist(&user, &proof, &metadata);
    assert_eq!(result, Err(Ok(Error::Blacklisted)));
}

#[test]
fn test_governance_proposal_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let gov1 = Address::generate(&env);
    let gov2 = Address::generate(&env);
    let gov3 = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);
    client.add_role(&admin, &gov1, &Role::Governor);
    client.add_role(&admin, &gov2, &Role::Governor);
    client.add_role(&admin, &gov3, &Role::Governor);

    let new_root = BytesN::from_array(&env, &[2u8; 32]);
    let proposal_id = client.propose_root_update(&gov1, &new_root, &Bytes::new(&env));

    client.vote_on_proposal(&gov1, &proposal_id, &true);
    client.vote_on_proposal(&gov2, &proposal_id, &true);
    client.vote_on_proposal(&gov3, &proposal_id, &true);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 86401,
        protocol_version: 20,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 6312000,
    });

    client.execute_proposal(&admin, &proposal_id);
    assert_eq!(client.get_merkle_root(), new_root);
    assert_eq!(client.get_root_version(), 2);
}

#[test]
fn test_dispute_submission_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let target = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let mut tree = TestMerkleTree::new(&env);
    let metadata = Bytes::from_slice(&env, b"test");
    let leaf = compute_leaf_hash_test(&env, &target, 0, &metadata);
    tree.add_leaf(leaf);

    let root = tree.build_root(&env);
    client.initialize(&admin, &token_address, &1000000, &root);
    client.add_role(&admin, &validator1, &Role::Validator);
    client.add_role(&admin, &validator2, &Role::Validator);
    client.grant_fee_waiver(&admin, &target);
    client.grant_fee_waiver(&admin, &validator1);

    let proof = tree.generate_proof(&env, 0);
    client.verify_whitelist(&target, &proof, &metadata);

    let evidence = Bytes::from_slice(&env, b"Evidence");
    let dispute_id = client.submit_dispute(&validator1, &target, &evidence);

    client.vote_on_dispute(&validator1, &dispute_id, &DisputeDecision::Invalid);
    client.vote_on_dispute(&validator2, &dispute_id, &DisputeDecision::Invalid);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 172801,
        protocol_version: 20,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 6312000,
    });

    client.resolve_dispute(&admin, &dispute_id);
    assert_eq!(client.is_whitelisted(&target), false);
    assert_eq!(client.is_blacklisted(&target), true);
}

#[test]
fn test_role_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);

    assert_eq!(client.has_role(&user, &Role::Governor), false);
    client.add_role(&admin, &user, &Role::Governor);
    assert_eq!(client.has_role(&user, &Role::Governor), true);
    client.remove_role(&admin, &user, &Role::Governor);
    assert_eq!(client.has_role(&user, &Role::Governor), false);
}

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);

    assert_eq!(client.is_paused(), false);
    client.pause(&admin);
    assert_eq!(client.is_paused(), true);

    let proof = Vec::new(&env);
    let result = client.try_verify_whitelist(&user, &proof, &Bytes::new(&env));
    assert_eq!(result, Err(Ok(Error::ContractPaused)));

    client.unpause(&admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
fn test_fee_collection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env, &admin);
    token_client.mint(&user, &10000000);

    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let mut tree = TestMerkleTree::new(&env);
    let metadata = Bytes::from_slice(&env, b"test");
    let leaf = compute_leaf_hash_test(&env, &user, 0, &metadata);
    tree.add_leaf(leaf);

    let root = tree.build_root(&env);
    let fee = 1000000;
    client.initialize(&admin, &token_address, &fee, &root);

    let proof = tree.generate_proof(&env, 0);
    client.verify_whitelist(&user, &proof, &metadata);

    assert_eq!(client.get_accumulated_fees(), fee);

    let recipient = Address::generate(&env);
    client.collect_fees(&admin, &recipient);
    assert_eq!(client.get_accumulated_fees(), 0);
    assert_eq!(token_client.balance(&recipient), fee);
}

#[test]
fn test_nonce_increment() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &token_address, &1000000, &root);

    assert_eq!(client.get_nonce(&user), 0);
}

#[test]
fn test_multiple_leaves_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env, &admin);
    let contract_id = env.register(MerkleWhitelistContract, ());
    let client = MerkleWhitelistContractClient::new(&env, &contract_id);

    let mut tree = TestMerkleTree::new(&env);
    let metadata = Bytes::from_slice(&env, b"member");

    let leaf1 = compute_leaf_hash_test(&env, &user1, 0, &metadata);
    let leaf2 = compute_leaf_hash_test(&env, &user2, 0, &metadata);
    let leaf3 = compute_leaf_hash_test(&env, &user3, 0, &metadata);

    tree.add_leaf(leaf1);
    tree.add_leaf(leaf2);
    tree.add_leaf(leaf3);

    let root = tree.build_root(&env);
    client.initialize(&admin, &token_address, &1000000, &root);

    client.grant_fee_waiver(&admin, &user1);
    client.grant_fee_waiver(&admin, &user2);
    client.grant_fee_waiver(&admin, &user3);

    let proof1 = tree.generate_proof(&env, 0);
    let proof2 = tree.generate_proof(&env, 1);
    let proof3 = tree.generate_proof(&env, 2);

    client.verify_whitelist(&user1, &proof1, &metadata);
    client.verify_whitelist(&user2, &proof2, &metadata);
    client.verify_whitelist(&user3, &proof3, &metadata);

    assert_eq!(client.is_whitelisted(&user1), true);
    assert_eq!(client.is_whitelisted(&user2), true);
    assert_eq!(client.is_whitelisted(&user3), true);
}
