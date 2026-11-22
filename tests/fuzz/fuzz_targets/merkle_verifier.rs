#![no_main]
#![allow(deprecated)]

use libfuzzer_sys::fuzz_target;
use merkle_proofs::{MerkleError, MerkleProofContract, MerkleProofContractClient};
use soroban_sdk::testutils::arbitrary::{arbitrary, Arbitrary};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

#[derive(Arbitrary, Debug)]
struct Input {
    leaf: [u8; 32],
    siblings: [[u8; 32]; 4],
    proof_depth: u8,
    leaf_count: u32,
    index: u32,
    invalidate_root: bool,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);
    let contract_id = env.register(MerkleProofContract, ());
    let client = MerkleProofContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let leaf = BytesN::from_array(&env, &input.leaf);
    let proof_depth = input.proof_depth.min(4);
    let mut proof = Vec::new(&env);
    let mut root = leaf.clone();

    for sibling_bytes in input.siblings.iter().take(proof_depth as usize) {
        let sibling = BytesN::from_array(&env, sibling_bytes);
        root = hash_pair(&env, &root, &sibling);
        proof.push_back(sibling);
    }
    if input.invalidate_root {
        let mut bytes = root.to_array();
        bytes[0] ^= 1;
        root = BytesN::from_array(&env, &bytes);
    }

    let leaf_count = (input.leaf_count % 64) + 1;
    let index = if input.index % 2 == 0 {
        input.index % leaf_count
    } else {
        input.index
    };
    client.set_root(&admin, &root, &leaf_count);

    let is_in_bounds = index < leaf_count;
    match client.try_verify(&leaf, &index, &proof) {
        Ok(Ok(valid)) => {
            assert!(is_in_bounds);
            assert_eq!(valid, !input.invalidate_root);
        }
        Ok(Err(MerkleError::IndexOutOfBounds)) | Err(Ok(MerkleError::IndexOutOfBounds)) => {
            assert!(!is_in_bounds)
        }
        Ok(Err(error)) => panic!("unexpected verification error: {error:?}"),
        Err(Ok(error)) => panic!("unexpected verification error: {error:?}"),
        Err(Err(error)) => panic!("verification invocation failed: {error:?}"),
    }

    match client.try_verify_and_claim(&claimer, &leaf, &index, &proof) {
        Ok(Ok(())) => {
            assert!(is_in_bounds && !input.invalidate_root);
            assert!(client.is_claimed(&claimer, &index));
            assert_eq!(
                client.try_verify_and_claim(&claimer, &leaf, &index, &proof),
                Err(Ok(MerkleError::AlreadyClaimed))
            );
        }
        Ok(Err(MerkleError::InvalidProof)) | Err(Ok(MerkleError::InvalidProof)) => {
            assert!(is_in_bounds && input.invalidate_root);
            assert!(!client.is_claimed(&claimer, &index));
        }
        Ok(Err(MerkleError::IndexOutOfBounds)) | Err(Ok(MerkleError::IndexOutOfBounds)) => {
            assert!(!is_in_bounds);
            assert!(!client.is_claimed(&claimer, &index));
        }
        Ok(Err(error)) => panic!("unexpected claim error: {error:?}"),
        Err(Ok(error)) => panic!("unexpected claim error: {error:?}"),
        Err(Err(error)) => panic!("claim invocation failed: {error:?}"),
    }
});

fn hash_pair(env: &Env, left: &BytesN<32>, right: &BytesN<32>) -> BytesN<32> {
    let mut bytes = Bytes::new(env);
    if left <= right {
        bytes.append(&Bytes::from(left.clone()));
        bytes.append(&Bytes::from(right.clone()));
    } else {
        bytes.append(&Bytes::from(right.clone()));
        bytes.append(&Bytes::from(left.clone()));
    }
    env.crypto().sha256(&bytes).to_bytes()
}
