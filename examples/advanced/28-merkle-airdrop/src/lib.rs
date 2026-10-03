#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Bytes,
    BytesN, Env, Vec,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyClaimed = 3,
    InvalidProof = 4,
    InvalidAmount = 5,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Root,
    Claimed(Address),
}

// ---------------------------------------------------------------------------
// Leaf Structure (used for deterministic canonical hashing)
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimLeaf {
    pub claimer: Address,
    pub amount: i128,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AirdropClaimedEvent {
    #[topic]
    pub claimer: Address,
    pub amount: i128,
}

// ---------------------------------------------------------------------------
// Contract Definition
// ---------------------------------------------------------------------------

#[contract]
pub struct MerkleAirdropContract;

#[contractimpl]
impl MerkleAirdropContract {
    /// Initialize the Merkle Airdrop contract.
    ///
    /// - `admin`: The administrator address allowed to update settings or manage funds.
    /// - `token`: The Stellar Asset Contract (SAC) or wrapper token address being distributed.
    /// - `root`: The 32-byte root hash of the Merkle tree representing the authorized claims.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        root: BytesN<32>,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Root, &root);

        Ok(())
    }

    /// Claim tokens from the airdrop.
    ///
    /// - `claimer`: The address claiming the tokens. Must authorize the transaction.
    /// - `amount`: The exact amount of tokens to claim, committed to in the Merkle tree leaf.
    /// - `proof`: Sibling hashes in the Merkle tree from the leaf up to the root.
    pub fn claim(
        env: Env,
        claimer: Address,
        amount: i128,
        proof: Vec<BytesN<32>>,
    ) -> Result<(), Error> {
        let _admin = Self::get_admin(env.clone())?;
        let token = Self::get_token(env.clone())?;
        let root = Self::get_root(env.clone())?;

        claimer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Double-claim prevention
        let claimed_key = DataKey::Claimed(claimer.clone());
        if env.storage().persistent().has(&claimed_key) {
            return Err(Error::AlreadyClaimed);
        }

        // Reconstruct leaf hash
        let leaf = Self::hash_leaf(env.clone(), claimer.clone(), amount);

        // Verify Merkle Proof
        let computed_root = compute_root(&env, &leaf, &proof);
        if computed_root != root {
            return Err(Error::InvalidProof);
        }

        // Mark as claimed in persistent storage
        env.storage().persistent().set(&claimed_key, &true);
        // Extend TTL to ensure persistent storage is not reclaimed prematurely
        env.storage()
            .persistent()
            .extend_ttl(&claimed_key, 17_280, 120_960);

        // Execute token transfer
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &claimer, &amount);

        // Emit claimed event using modern #[contractevent] pattern
        AirdropClaimedEvent {
            claimer: claimer.clone(),
            amount,
        }
        .publish(&env);

        Ok(())
    }

    /// Get the administrator address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Get the token address.
    pub fn get_token(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)
    }

    /// Get the current stored Merkle root hash.
    pub fn get_root(env: Env) -> Result<BytesN<32>, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Root)
            .ok_or(Error::NotInitialized)
    }

    /// Check if a claimer has already claimed.
    pub fn is_claimed(env: Env, claimer: Address) -> bool {
        env.storage().persistent().has(&DataKey::Claimed(claimer))
    }

    /// Derive the leaf hash of a claim tuple (claimer, amount).
    pub fn hash_leaf(env: Env, claimer: Address, amount: i128) -> BytesN<32> {
        let value = ClaimLeaf { claimer, amount };
        env.crypto().sha256(&value.to_xdr(&env)).to_bytes()
    }
}

// ---------------------------------------------------------------------------
// Internal Merkle Proof Helpers
// ---------------------------------------------------------------------------

fn compute_root(env: &Env, leaf: &BytesN<32>, proof: &Vec<BytesN<32>>) -> BytesN<32> {
    let mut computed = leaf.clone();
    for sibling in proof.iter() {
        computed = hash_pair(env, &computed, &sibling);
    }
    computed
}

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

#[cfg(test)]
mod test;
