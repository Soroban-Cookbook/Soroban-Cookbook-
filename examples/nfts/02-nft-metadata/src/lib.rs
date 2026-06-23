//! NFT with metadata storage and optional on-chain metadata.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    BaseUri,
    OnChainMetadata,
    Owner(u64),
    TokenUri(u64),
    Metadata(u64),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NFTError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    TokenAlreadyMinted = 3,
    TokenNotFound = 4,
    MetadataNotEnabled = 5,
    MetadataNotFound = 6,
    TokenUriRequired = 7,
    InvalidTokenId = 8,
    NotInitialized = 9,
}

#[contract]
pub struct NFTMetadataContract;

#[contractimpl]
impl NFTMetadataContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        base_uri: String,
        on_chain_metadata: bool,
    ) -> Result<(), NFTError> {
        let storage = env.storage();
        if storage.instance().has(&DataKey::Admin) {
            return Err(NFTError::AlreadyInitialized);
        }

        admin.require_auth();
        storage.instance().set(&DataKey::Admin, &admin);
        storage.instance().set(&DataKey::BaseUri, &base_uri);
        storage
            .instance()
            .set(&DataKey::OnChainMetadata, &on_chain_metadata);
        Ok(())
    }

    pub fn mint(
        env: Env,
        to: Address,
        token_id: u64,
        token_uri: Option<String>,
        metadata: Option<NFTMetadata>,
    ) -> Result<(), NFTError> {
        if token_id == 0 {
            return Err(NFTError::InvalidTokenId);
        }

        let admin = read_admin(&env)?;
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Owner(token_id)) {
            return Err(NFTError::TokenAlreadyMinted);
        }

        let on_chain_metadata = read_on_chain_metadata(&env)?;
        if on_chain_metadata {
            if metadata.is_none() {
                return Err(NFTError::MetadataNotEnabled);
            }
        } else {
            if token_uri.is_none() {
                return Err(NFTError::TokenUriRequired);
            }
            if metadata.is_some() {
                return Err(NFTError::MetadataNotEnabled);
            }
        }

        env.storage().instance().set(&DataKey::Owner(token_id), &to);
        if let Some(uri) = token_uri {
            env.storage()
                .instance()
                .set(&DataKey::TokenUri(token_id), &uri);
        }

        if let Some(metadata) = metadata {
            env.storage()
                .instance()
                .set(&DataKey::Metadata(token_id), &metadata);
        }

        Ok(())
    }

    pub fn owner_of(env: Env, token_id: u64) -> Result<Address, NFTError> {
        read_owner(&env, token_id)
    }

    pub fn token_uri(env: Env, token_id: u64) -> Result<String, NFTError> {
        let _ = read_owner(&env, token_id)?;
        let uri = env.storage().instance().get(&DataKey::TokenUri(token_id));
        if let Some(token_uri) = uri {
            return Ok(token_uri);
        }
        let base_uri = read_base_uri(&env)?;
        Ok(format_token_uri(&env, &base_uri, token_id))
    }

    pub fn metadata(env: Env, token_id: u64) -> Result<Option<NFTMetadata>, NFTError> {
        let _ = read_owner(&env, token_id)?;
        if !read_on_chain_metadata(&env)? {
            return Err(NFTError::MetadataNotEnabled);
        }
        let metadata = env.storage().instance().get(&DataKey::Metadata(token_id));
        Ok(metadata)
    }

    pub fn set_metadata(env: Env, token_id: u64, metadata: NFTMetadata) -> Result<(), NFTError> {
        let admin = read_admin(&env)?;
        admin.require_auth();
        let _ = read_owner(&env, token_id)?;
        if !read_on_chain_metadata(&env)? {
            return Err(NFTError::MetadataNotEnabled);
        }
        env.storage()
            .instance()
            .set(&DataKey::Metadata(token_id), &metadata);
        Ok(())
    }

    pub fn base_uri(env: Env) -> Result<String, NFTError> {
        read_base_uri(&env)
    }

    pub fn is_on_chain_metadata(env: Env) -> Result<bool, NFTError> {
        read_on_chain_metadata(&env)
    }
}

fn read_admin(env: &Env) -> Result<Address, NFTError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(NFTError::NotInitialized)
}

fn read_base_uri(env: &Env) -> Result<String, NFTError> {
    env.storage()
        .instance()
        .get(&DataKey::BaseUri)
        .ok_or(NFTError::NotInitialized)
}

fn read_on_chain_metadata(env: &Env) -> Result<bool, NFTError> {
    env.storage()
        .instance()
        .get(&DataKey::OnChainMetadata)
        .ok_or(NFTError::NotInitialized)
}

fn read_owner(env: &Env, token_id: u64) -> Result<Address, NFTError> {
    env.storage()
        .instance()
        .get(&DataKey::Owner(token_id))
        .ok_or(NFTError::TokenNotFound)
}

fn format_token_uri(env: &Env, base_uri: &String, token_id: u64) -> String {
    let base_len = base_uri.len() as usize;
    let mut buf = [0u8; 1024];
    if base_len + 1 >= buf.len() {
        panic!("base URI too long");
    }
    base_uri.copy_into_slice(&mut buf[..base_len]);
    buf[base_len] = b'/';
    let token_len = write_u64_decimal(token_id, &mut buf[base_len + 1..]);
    let total = base_len + 1 + token_len;
    String::from_bytes(env, &buf[..total])
}

fn write_u64_decimal(value: u64, buf: &mut [u8]) -> usize {
    let mut n = value;
    if n == 0 {
        if buf.is_empty() {
            panic!("buffer too short for token id");
        }
        buf[0] = b'0';
        return 1;
    }
    let mut i = 0;
    let mut reversed = [0u8; 20];
    while n > 0 {
        reversed[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    if i > buf.len() {
        panic!("buffer too short for token id");
    }
    for j in 0..i {
        buf[j] = reversed[i - 1 - j];
    }
    i
}
