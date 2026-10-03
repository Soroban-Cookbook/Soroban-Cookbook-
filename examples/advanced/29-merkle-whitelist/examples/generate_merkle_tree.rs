#![allow(deprecated)]
//! Off-chain Merkle tree generation tool
//!
//! This example demonstrates how to build a Merkle tree off-chain and generate
//! proofs for whitelist verification on-chain.
//!
//! Usage:
//!   cargo run --example generate_merkle_tree

use soroban_sdk::{Bytes, BytesN, Env, Address, Vec};

/// Example: Build a Merkle tree for a whitelist and generate proofs
fn main() {
    println!("=== Merkle Whitelist Tree Generator ===\n");

    let env = Env::default();

    // Step 1: Define whitelist entries
    println!("Step 1: Defining whitelist entries...");
    
    let addresses = vec![
        "GABC...", // Replace with actual addresses
        "GDEF...",
        "GHIJ...",
        "GKLM...",
    ];
    
    println!("  - {} addresses in whitelist", addresses.len());

    // Step 2: Compute leaf hashes
    println!("\nStep 2: Computing leaf hashes...");
    
    let mut leaves = Vec::new(&env);
    
    for (idx, addr_str) in addresses.iter().enumerate() {
        // In production, parse actual addresses
        // For demo, we'll create mock data
        let metadata = format!("member_{}", idx);
        let nonce = 0u64;
        
        // Compute leaf hash: sha256(address || nonce || metadata)
        let mut buf = Bytes::new(&env);
        buf.append(&Bytes::from_slice(&env, addr_str.as_bytes()));
        
        for byte in nonce.to_be_bytes().iter() {
            buf.push_back(*byte);
        }
        
        buf.append(&Bytes::from_slice(&env, metadata.as_bytes()));
        
        let leaf_hash = env.crypto().sha256(&buf).to_bytes();
        leaves.push_back(leaf_hash.clone());
        
        println!("  - Address {}: {}", idx, hex::encode(leaf_hash.to_array()));
    }

    // Step 3: Build Merkle tree
    println!("\nStep 3: Building Merkle tree...");
    
    let root = build_merkle_root(&env, &leaves);
    println!("  - Merkle Root: {}", hex::encode(root.to_array()));

    // Step 4: Generate proofs for each address
    println!("\nStep 4: Generating proofs...");
    
    for idx in 0..leaves.len() {
        let proof = generate_proof(&env, &leaves, idx);
        println!("\n  Address {} proof (depth {}):", idx, proof.len());
        for (i, sibling) in proof.iter().enumerate() {
            println!("    [{}] {}", i, hex::encode(sibling.to_array()));
        }
    }

    // Step 5: Verification example
    println!("\n\nStep 5: Verifying proof for address 0...");
    let leaf_0 = leaves.get(0).unwrap();
    let proof_0 = generate_proof(&env, &leaves, 0);
    let computed_root = verify_proof(&env, &leaf_0, &proof_0);
    
    println!("  - Original root: {}", hex::encode(root.to_array()));
    println!("  - Computed root: {}", hex::encode(computed_root.to_array()));
    println!("  - Valid: {}", root == computed_root);

    // Step 6: Export for on-chain deployment
    println!("\n\nStep 6: Deployment data");
    println!("  - Initialize contract with root: 0x{}", hex::encode(root.to_array()));
    println!("  - Store proofs off-chain (users will submit when claiming)");
    println!("  - Update root via governance when whitelist changes");
}

/// Build Merkle root from leaves
fn build_merkle_root(env: &Env, leaves: &Vec<BytesN<32>>) -> BytesN<32> {
    if leaves.is_empty() {
        return BytesN::from_array(env, &[0u8; 32]);
    }
    
    if leaves.len() == 1 {
        return leaves.get(0).unwrap();
    }

    let mut current_level = leaves.clone();

    while current_level.len() > 1 {
        let mut next_level = Vec::new(env);
        let mut i = 0;
        
        while i < current_level.len() {
            let left = current_level.get(i).unwrap();
            let right = if i + 1 < current_level.len() {
                current_level.get(i + 1).unwrap()
            } else {
                left.clone() // Duplicate if odd number
            };
            
            next_level.push_back(hash_pair(env, &left, &right));
            i += 2;
        }
        
        current_level = next_level;
    }

    current_level.get(0).unwrap()
}

/// Generate Merkle proof for a leaf at given index
fn generate_proof(env: &Env, leaves: &Vec<BytesN<32>>, leaf_index: u32) -> Vec<BytesN<32>> {
    let mut proof = Vec::new(env);

    if leaves.is_empty() || leaf_index >= leaves.len() {
        return proof;
    }

    let mut current_level = leaves.clone();
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
            next_level.push_back(hash_pair(env, &left, &right));
            i += 2;
        }

        current_level = next_level;
        index /= 2;
    }

    proof
}

/// Verify a Merkle proof
fn verify_proof(env: &Env, leaf: &BytesN<32>, proof: &Vec<BytesN<32>>) -> BytesN<32> {
    let mut computed = leaf.clone();
    
    for sibling in proof.iter() {
        computed = hash_pair(env, &computed, &sibling);
    }
    
    computed
}

/// Hash two nodes (sorted order)
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

// Mock hex encoding for demo
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}
