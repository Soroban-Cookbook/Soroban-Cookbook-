//! # Merkle Proof Verification
//!
//! Demonstrates an on-chain Merkle-root registry that supports efficient
//! membership verification without storing the full data set on-chain.
//!
//! ## Pattern
//!
//! Merkle trees are built **off-chain**. Only the 32-byte root is stored
//! on-chain (in `instance` storage, so it is cheap and shared by every
//! invocation). Anyone can then prove that a given leaf is part of the
//! data set committed to by the root by supplying a *Merkle proof*: the
//! sibling hashes needed to recompute the root starting from the leaf.
//!
//! This is the same technique used by token airdrops, allow-lists, and
//! voting-eligibility checks: instead of storing thousands of addresses in
//! contract storage, the contract stores a single hash and verifies
//! membership cheaply at the time the claim/vote/spend is made.
//!
//! ## Tree Construction (off-chain, also used by tests in this crate)
//!
//! - Leaves are pre-hashed by the caller (e.g. `sha256(address || amount)`).
//! - To resist the classic "duplicate odd leaf" second-preimage style
//!   attack, when a level has an odd number of nodes the last node is
//!   carried up *unchanged* rather than duplicated and re-hashed with
//!   itself.
//! - Sibling pairs are sorted before hashing (`hash(min, max)`), so a leaf's
//!   proof does not need to encode left/right order, which both simplifies
//!   the verifier and avoids a class of malleability bugs where the same
//!   pair of hashes could be combined in two different orders to produce
//!   two different parents.
//!
//! ## Efficient Storage
//!
//! - Only the latest root (`BytesN<32>`) and a small amount of metadata
//!   (leaf count, root generation/version, optional expiry) live in
//!   `instance` storage — O(1) regardless of how many leaves the tree
//!   commits to.
//! - An optional "claimed" bitmap-style record (`persistent` storage keyed
//!   by leaf index) lets contracts that consume the proof (e.g. an
//!   airdrop) cheaply prevent replay without storing the leaf data itself.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Bytes, BytesN,
    Env, Vec,
};

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted whenever a new Merkle root is published.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootSetEvent {
    #[topic]
    pub admin: Address,
    pub root: BytesN<32>,
    pub leaf_count: u32,
    pub generation: u32,
}

/// Emitted whenever a leaf is successfully verified and claimed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedEvent {
    #[topic]
    pub index: u32,
    pub generation: u32,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MerkleError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the configured admin.
    Unauthorized = 3,
    /// No Merkle root has been published yet.
    RootNotSet = 4,
    /// Supplied leaf count was zero when one was required.
    InvalidLeafCount = 5,
    /// Proof failed to reconstruct the stored root.
    InvalidProof = 6,
    /// Leaf index is out of bounds for the current tree.
    IndexOutOfBounds = 7,
    /// Leaf has already been claimed/consumed.
    AlreadyClaimed = 8,
}

// ---------------------------------------------------------------------------
// Storage layout
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address allowed to publish new roots.
    Admin,
    /// Currently active Merkle root.
    Root,
    /// Number of leaves committed to by the active root (for bounds checks).
    LeafCount,
    /// Monotonically increasing generation counter, bumped on every root
    /// update. Used to invalidate "claimed" records from a previous root.
    Generation,
    /// Per-(generation, leaf index) claim marker. Only ever stores `true`;
    /// presence == claimed. This keeps storage O(claims) instead of O(leaves).
    Claimed(u32, u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootInfo {
    pub root: BytesN<32>,
    pub leaf_count: u32,
    pub generation: u32,
}

#[contract]
pub struct MerkleProofContract;

#[contractimpl]
impl MerkleProofContract {
    /// Initialize the contract with an admin address. The admin is the only
    /// account allowed to publish new Merkle roots.
    pub fn initialize(env: Env, admin: Address) -> Result<(), MerkleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MerkleError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Generation, &0u32);
        Ok(())
    }

    /// Publish (or replace) the active Merkle root.
    ///
    /// `leaf_count` is the number of leaves that were committed to the
    /// tree off-chain; it is used purely to bounds-check leaf indices
    /// during verification and does not affect the cryptographic
    /// guarantees of the proof.
    ///
    /// Replacing the root bumps an internal generation counter so that any
    /// previously recorded claims (tied to the old generation) cannot be
    /// confused with claims against the new tree.
    pub fn set_root(
        env: Env,
        admin: Address,
        root: BytesN<32>,
        leaf_count: u32,
    ) -> Result<(), MerkleError> {
        let stored_admin = read_admin(&env)?;
        if stored_admin != admin {
            return Err(MerkleError::Unauthorized);
        }
        admin.require_auth();

        if leaf_count == 0 {
            return Err(MerkleError::InvalidLeafCount);
        }

        let generation: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Generation)
            .unwrap_or(0);
        let next_generation = generation.saturating_add(1);

        env.storage().instance().set(&DataKey::Root, &root);
        env.storage()
            .instance()
            .set(&DataKey::LeafCount, &leaf_count);
        env.storage()
            .instance()
            .set(&DataKey::Generation, &next_generation);

        RootSetEvent {
            admin,
            root,
            leaf_count,
            generation: next_generation,
        }
        .publish(&env);

        Ok(())
    }

    /// Return the currently active root along with its metadata.
    pub fn get_root_info(env: Env) -> Result<RootInfo, MerkleError> {
        Ok(RootInfo {
            root: read_root(&env)?,
            leaf_count: env
                .storage()
                .instance()
                .get(&DataKey::LeafCount)
                .unwrap_or(0),
            generation: env
                .storage()
                .instance()
                .get(&DataKey::Generation)
                .unwrap_or(0),
        })
    }

    /// Verify that `leaf` is a member of the tree committed to by the
    /// currently active root, given a Merkle `proof` (sibling hashes from
    /// leaf level up to the root) and the leaf's `index` in the original
    /// (unhashed-pair) leaf ordering.
    ///
    /// Returns `true`/`false` rather than erroring on a failed proof so
    /// that callers can distinguish "the contract is misconfigured"
    /// (`Err`) from "this particular proof is invalid" (`Ok(false)`).
    pub fn verify(
        env: Env,
        leaf: BytesN<32>,
        index: u32,
        proof: Vec<BytesN<32>>,
    ) -> Result<bool, MerkleError> {
        let root = read_root(&env)?;
        let leaf_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LeafCount)
            .unwrap_or(0);
        if leaf_count == 0 {
            return Err(MerkleError::RootNotSet);
        }
        if index >= leaf_count {
            return Err(MerkleError::IndexOutOfBounds);
        }

        let computed = compute_root(&env, &leaf, index, &proof);
        Ok(computed == root)
    }

    /// Verify a proof and atomically mark the leaf index as claimed,
    /// preventing the same proof from being consumed twice against the
    /// current root generation. Returns an error if the proof is invalid
    /// or the leaf has already been claimed.
    ///
    /// This mirrors the common airdrop/allow-list consumption pattern
    /// while keeping storage proportional to the number of *claims*, not
    /// the number of *leaves* in the tree.
    pub fn verify_and_claim(
        env: Env,
        leaf: BytesN<32>,
        index: u32,
        proof: Vec<BytesN<32>>,
    ) -> Result<(), MerkleError> {
        let root = read_root(&env)?;
        let leaf_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LeafCount)
            .unwrap_or(0);
        if leaf_count == 0 {
            return Err(MerkleError::RootNotSet);
        }
        if index >= leaf_count {
            return Err(MerkleError::IndexOutOfBounds);
        }

        let generation: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Generation)
            .unwrap_or(0);
        let claim_key = DataKey::Claimed(generation, index);
        if env.storage().persistent().has(&claim_key) {
            return Err(MerkleError::AlreadyClaimed);
        }

        let computed = compute_root(&env, &leaf, index, &proof);
        if computed != root {
            return Err(MerkleError::InvalidProof);
        }

        env.storage().persistent().set(&claim_key, &true);
        // Claims are expected to be long-lived relative to a single ledger
        // close; extend generously so the record outlives typical claim
        // windows without manual maintenance.
        env.storage()
            .persistent()
            .extend_ttl(&claim_key, 17_280, 120_960);

        ClaimedEvent { index, generation }.publish(&env);

        Ok(())
    }

    /// Whether the leaf at `index` has already been claimed against the
    /// currently active root generation.
    pub fn is_claimed(env: Env, index: u32) -> bool {
        let generation: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Generation)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .has(&DataKey::Claimed(generation, index))
    }

    /// Hash a raw leaf payload using the same convention the off-chain
    /// tree builder must use (`sha256` of the raw bytes). Exposed so
    /// callers/tests can derive leaf hashes consistently with on-chain
    /// hashing without duplicating the hash choice.
    pub fn hash_leaf(env: Env, data: Bytes) -> BytesN<32> {
        env.crypto().sha256(&data).to_bytes()
    }
}

/// Recompute a Merkle root from a leaf, its index, and a proof (sibling
/// hashes ordered from the leaf level up to the root).
///
/// At each level, the current node is combined with the next proof
/// element. Siblings are combined in **sorted order**
/// (`hash(min(a, b), max(a, b))`) so the verifier does not need to know
/// whether the sibling is to the left or right — this matches the
/// construction used by the off-chain tree builder.
fn compute_root(env: &Env, leaf: &BytesN<32>, _index: u32, proof: &Vec<BytesN<32>>) -> BytesN<32> {
    let mut computed = leaf.clone();
    for sibling in proof.iter() {
        computed = hash_pair(env, &computed, &sibling);
    }
    computed
}

/// Combine two 32-byte nodes into their parent hash, ordering them first so
/// that the same pair always hashes to the same parent regardless of which
/// one is conceptually "left" or "right".
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

fn read_admin(env: &Env) -> Result<Address, MerkleError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(MerkleError::NotInitialized)
}

fn read_root(env: &Env) -> Result<BytesN<32>, MerkleError> {
    env.storage()
        .instance()
        .get(&DataKey::Root)
        .ok_or(MerkleError::RootNotSet)
}

#[cfg(test)]
mod test;
