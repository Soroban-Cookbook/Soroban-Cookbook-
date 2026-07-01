extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};
use std::vec::Vec as StdVec;

// ---------------------------------------------------------------------------
// Off-chain Merkle tree builder used by tests.
//
// This mirrors exactly what a real off-chain indexer/CLI would do before
// calling `set_root`: hash all leaves, repeatedly combine sibling pairs
// (sorted, so order doesn't matter) until a single root remains, while
// carrying an unmatched trailing node up unchanged instead of duplicating
// it. Proof generation walks the same levels and records the sibling
// needed at each step (or nothing, when the node was carried).
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

/// Build every level of the tree, from leaves (level 0) to the single-node
/// root (last level).
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
                // Odd one out: carry up unchanged rather than duplicating.
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

fn leaf_from_str(env: &Env, s: &str) -> BytesN<32> {
    let data = Bytes::from_slice(env, s.as_bytes());
    env.crypto().sha256(&data).to_bytes()
}

fn build_dataset(env: &Env, n: usize) -> StdVec<BytesN<32>> {
    let mut leaves = StdVec::new();
    for i in 0..n {
        let s = std::format!("leaf-{i}");
        leaves.push(leaf_from_str(env, &s));
    }
    leaves
}

fn setup() -> (Env, Address, MerkleProofContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MerkleProofContract, ());
    let client = MerkleProofContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

// ── initialization ──────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (_env, _admin, client) = setup();
    // No root yet, so get_root_info should fail with RootNotSet.
    let result = client.try_get_root_info();
    assert_eq!(result, Err(Ok(MerkleError::RootNotSet)));
}

#[test]
fn test_initialize_twice_fails() {
    let (_env, admin, client) = setup();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(MerkleError::AlreadyInitialized)));
}

// ── set_root ─────────────────────────────────────────────────────────────────

#[test]
fn test_set_root_success_and_get_root_info() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);

    client.set_root(&admin, &root, &(leaves.len() as u32));

    let info = client.get_root_info();
    assert_eq!(info.root, root);
    assert_eq!(info.leaf_count, 4);
    assert_eq!(info.generation, 1);
}

#[test]
fn test_set_root_zero_leaf_count_fails() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);

    let result = client.try_set_root(&admin, &root, &0u32);
    assert_eq!(result, Err(Ok(MerkleError::InvalidLeafCount)));
}

#[test]
fn test_set_root_unauthorized_fails() {
    let (env, _admin, client) = setup();
    let stranger = Address::generate(&env);
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);

    let result = client.try_set_root(&stranger, &root, &(leaves.len() as u32));
    assert_eq!(result, Err(Ok(MerkleError::Unauthorized)));
}

#[test]
fn test_set_root_bumps_generation_and_resets_claims() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let proof = merkle_proof(&env, &leaves, 0);
    client.verify_and_claim(&leaves[0], &0u32, &proof);
    assert!(client.is_claimed(&0u32));

    // Publish a brand new root (new generation); old claim record must not
    // leak into the new generation's claim space.
    let leaves2 = build_dataset(&env, 5);
    let root2 = merkle_root(&env, &leaves2);
    client.set_root(&admin, &root2, &(leaves2.len() as u32));

    assert!(!client.is_claimed(&0u32));
    let info = client.get_root_info();
    assert_eq!(info.generation, 2);
}

// ── verify ───────────────────────────────────────────────────────────────────

#[test]
fn test_verify_valid_proof_returns_true() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 6);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    for (i, leaf) in leaves.iter().enumerate() {
        let proof = merkle_proof(&env, &leaves, i);
        assert!(client.verify(leaf, &(i as u32), &proof));
    }
}

#[test]
fn test_verify_single_leaf_tree() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 1);
    let root = merkle_root(&env, &leaves);
    // A single-leaf tree's root *is* the leaf hash.
    assert_eq!(root, leaves[0]);
    client.set_root(&admin, &root, &1u32);

    let proof = merkle_proof(&env, &leaves, 0);
    assert!(proof.is_empty());
    assert!(client.verify(&leaves[0], &0u32, &proof));
}

#[test]
fn test_verify_odd_leaf_count_tree() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 5);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    for (i, leaf) in leaves.iter().enumerate() {
        let proof = merkle_proof(&env, &leaves, i);
        assert!(client.verify(leaf, &(i as u32), &proof));
    }
}

#[test]
fn test_verify_tampered_leaf_returns_false() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let proof = merkle_proof(&env, &leaves, 1);
    let wrong_leaf = leaf_from_str(&env, "not-a-real-leaf");
    assert!(!client.verify(&wrong_leaf, &1u32, &proof));
}

#[test]
fn test_verify_tampered_proof_returns_false() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let mut proof = merkle_proof(&env, &leaves, 2);
    // Corrupt the first sibling hash.
    let bogus = leaf_from_str(&env, "corrupted-sibling");
    proof.set(0, bogus);
    assert!(!client.verify(&leaves[2], &2u32, &proof));
}

#[test]
fn test_verify_index_out_of_bounds_fails() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let proof = merkle_proof(&env, &leaves, 0);
    let result = client.try_verify(&leaves[0], &10u32, &proof);
    assert_eq!(result, Err(Ok(MerkleError::IndexOutOfBounds)));
}

#[test]
fn test_verify_before_root_set_fails() {
    let (env, _admin, client) = setup();
    let leaves = build_dataset(&env, 2);
    let proof = merkle_proof(&env, &leaves, 0);
    let result = client.try_verify(&leaves[0], &0u32, &proof);
    assert_eq!(result, Err(Ok(MerkleError::RootNotSet)));
}

// ── verify_and_claim ─────────────────────────────────────────────────────────

#[test]
fn test_verify_and_claim_success() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    assert!(!client.is_claimed(&2u32));
    let proof = merkle_proof(&env, &leaves, 2);
    client.verify_and_claim(&leaves[2], &2u32, &proof);
    assert!(client.is_claimed(&2u32));
}

#[test]
fn test_verify_and_claim_twice_fails() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let proof = merkle_proof(&env, &leaves, 0);
    client.verify_and_claim(&leaves[0], &0u32, &proof);

    let result = client.try_verify_and_claim(&leaves[0], &0u32, &proof);
    assert_eq!(result, Err(Ok(MerkleError::AlreadyClaimed)));
}

#[test]
fn test_verify_and_claim_invalid_proof_fails() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 4);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let bad_proof = merkle_proof(&env, &leaves, 1); // proof for the wrong leaf
    let result = client.try_verify_and_claim(&leaves[0], &0u32, &bad_proof);
    assert_eq!(result, Err(Ok(MerkleError::InvalidProof)));
    assert!(!client.is_claimed(&0u32));
}

#[test]
fn test_verify_and_claim_index_out_of_bounds_fails() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 3);
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    let proof = merkle_proof(&env, &leaves, 0);
    let result = client.try_verify_and_claim(&leaves[0], &99u32, &proof);
    assert_eq!(result, Err(Ok(MerkleError::IndexOutOfBounds)));
}

// ── hash_leaf helper ─────────────────────────────────────────────────────────

#[test]
fn test_hash_leaf_matches_offchain_sha256() {
    let env = Env::default();
    let data = Bytes::from_slice(&env, b"airdrop-recipient-1:1000");
    let contract_id = env.register(MerkleProofContract, ());
    let client = MerkleProofContractClient::new(&env, &contract_id);

    let onchain_hash = client.hash_leaf(&data);
    let expected = env.crypto().sha256(&data).to_bytes();
    assert_eq!(onchain_hash, expected);
}

// ── larger tree sanity check ────────────────────────────────────────────────

#[test]
fn test_verify_large_tree_all_leaves() {
    let (env, admin, client) = setup();
    let leaves = build_dataset(&env, 17); // forces several odd-carry levels
    let root = merkle_root(&env, &leaves);
    client.set_root(&admin, &root, &(leaves.len() as u32));

    for (i, leaf) in leaves.iter().enumerate() {
        let proof = merkle_proof(&env, &leaves, i);
        assert!(
            client.verify(leaf, &(i as u32), &proof),
            "leaf {i} failed verification"
        );
    }
}
