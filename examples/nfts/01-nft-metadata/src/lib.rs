//! # NFT Metadata Standards
//!
//! A complete NFT contract implementing metadata standards for Soroban.
//!
//! ## Metadata Schema
//!
//! Each token carries on-chain metadata that follows a JSON-compatible schema:
//!
//! ```json
//! {
//!   "name":        "My NFT #1",
//!   "description": "A unique digital collectible",
//!   "image":       "ipfs://Qm.../image.png",
//!   "external_url": "https://example.com/nft/1",
//!   "attributes": [
//!     { "trait_type": "Rarity",  "value": "Legendary" },
//!     { "trait_type": "Power",   "value": "100"       }
//!   ]
//! }
//! ```
//!
//! ## Acceptance Criteria
//!
//! | Criterion            | Implementation                                      |
//! |----------------------|-----------------------------------------------------|
//! | JSON schema compliance | `TokenMetadata` struct mirrors the JSON schema    |
//! | Attribute system     | `Vec<Attribute>` with `trait_type` + `value`        |
//! | Image / media URIs   | `image` field + `media_uri` / `animation_url`       |
//! | Metadata validation  | `validate_metadata()` checks all required fields   |
//! | Documentation        | This module doc + README.md                         |
//!
//! ## Storage Layout
//!
//! | Key                    | Storage tier | Description                    |
//! |------------------------|--------------|--------------------------------|
//! | `DataKey::Admin`       | instance     | Contract admin address         |
//! | `DataKey::Name`        | instance     | Collection name                |
//! | `DataKey::Symbol`      | instance     | Collection symbol              |
//! | `DataKey::BaseUri`     | instance     | Optional base URI prefix       |
//! | `DataKey::TotalSupply` | instance     | Running mint counter           |
//! | `DataKey::Owner(id)`   | persistent   | Owner of token `id`            |
//! | `DataKey::Balance(addr)` | persistent | Number of tokens held          |
//! | `DataKey::Metadata(id)`| persistent   | Full `TokenMetadata` for token |
//! | `DataKey::Approved(id)`| persistent   | Single-token approval          |
//! | `DataKey::ApproveAll(owner, op)` | persistent | Operator approval      |

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Metadata types
// ---------------------------------------------------------------------------

/// A single trait attribute attached to a token.
///
/// Mirrors the OpenSea / ERC-721 metadata attribute object:
/// ```json
/// { "trait_type": "Rarity", "value": "Legendary" }
/// ```
///
/// Both fields are stored as `String` so numeric values like `"100"` are
/// represented as their string form, keeping the schema uniform.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    /// Human-readable category name, e.g. `"Rarity"`, `"Power"`, `"Color"`.
    pub trait_type: String,
    /// The attribute value, e.g. `"Legendary"`, `"100"`, `"Blue"`.
    pub value: String,
}

/// Full metadata record stored on-chain for each token.
///
/// ### Required fields
/// - `name`        — display name of the token
/// - `description` — human-readable description
/// - `image`       — primary media URI (IPFS, HTTPS, data URI)
///
/// ### Optional fields
/// - `external_url`    — link to an external page for this token
/// - `animation_url`   — URI for video / audio / interactive media
/// - `background_color` — hex color string without `#`, e.g. `"FF0000"`
/// - `attributes`      — list of trait attributes (may be empty)
///
/// ### URI conventions
/// - IPFS:  `ipfs://Qm.../filename.png`
/// - HTTPS: `https://cdn.example.com/nft/1.png`
/// - Data:  `data:image/svg+xml;base64,...`
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    // ---- required ----
    /// Display name of the token, e.g. `"My NFT #1"`.
    pub name: String,
    /// Human-readable description of the token.
    pub description: String,
    /// URI pointing to the primary image / media asset.
    /// Accepted schemes: `ipfs://`, `https://`, `data:`.
    pub image: String,

    // ---- optional ----
    /// External URL for this token (may be empty string to omit).
    pub external_url: String,
    /// URI for an animated or interactive version of the asset.
    pub animation_url: String,
    /// Background color as a 6-character hex string (no `#`).
    pub background_color: String,
    /// Ordered list of trait attributes.
    pub attributes: Vec<Attribute>,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

/// All storage keys used by the NFT contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin address (instance storage).
    Admin,
    /// Collection name, e.g. `"My Collection"` (instance storage).
    Name,
    /// Collection ticker symbol, e.g. `"MNFT"` (instance storage).
    Symbol,
    /// Optional base URI prepended to token IDs when building token URIs.
    BaseUri,
    /// Running count of minted tokens / next token ID (instance storage).
    TotalSupply,
    /// Maps `token_id -> owner Address` (persistent storage).
    Owner(u32),
    /// Maps `owner Address -> token count` (persistent storage).
    Balance(Address),
    /// Full metadata for a token (persistent storage).
    Metadata(u32),
    /// Single-token approval: maps `token_id -> approved Address`.
    Approved(u32),
    /// Operator approval: maps `(owner, operator) -> bool`.
    ApproveAll(Address, Address),
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Contract error codes.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum NftError {
    /// `initialize()` has already been called.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// The caller is not the contract admin.
    NotAdmin = 3,
    /// The token ID does not exist.
    TokenNotFound = 4,
    /// The token ID has already been minted.
    TokenAlreadyExists = 5,
    /// The caller is not the token owner.
    NotOwner = 6,
    /// The caller is not approved to act on this token.
    NotApproved = 7,
    /// A required metadata field is empty.
    MetadataFieldEmpty = 8,
    /// The image URI scheme is not supported.
    InvalidImageUri = 9,
    /// The background_color field is not a valid 6-char hex string.
    InvalidBackgroundColor = 10,
    /// An attribute has an empty trait_type or value.
    InvalidAttribute = 11,
    /// Transfer to the zero address is not allowed.
    TransferToZero = 12,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct NftMetadataContract;

#[contractimpl]
impl NftMetadataContract {
    // ==================== INITIALIZATION ====================

    /// Initialize the collection.
    ///
    /// Must be called exactly once by the deployer.
    ///
    /// # Parameters
    /// - `admin`    — address that can mint tokens
    /// - `name`     — collection name stored on-chain
    /// - `symbol`   — collection ticker symbol
    /// - `base_uri` — optional URI prefix (pass empty string to omit)
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        base_uri: String,
    ) -> Result<(), NftError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(NftError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::BaseUri, &base_uri);
        env.storage().instance().set(&DataKey::TotalSupply, &0u32);

        env.events().publish(
            (symbol_short!("init"), symbol_short!("nft")),
            (name, symbol),
        );

        Ok(())
    }

    // ==================== COLLECTION METADATA ====================

    /// Returns the collection name.
    pub fn name(env: Env) -> Result<String, NftError> {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .ok_or(NftError::NotInitialized)
    }

    /// Returns the collection symbol.
    pub fn symbol(env: Env) -> Result<String, NftError> {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .ok_or(NftError::NotInitialized)
    }

    /// Returns the total number of minted tokens.
    pub fn total_supply(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    // ==================== TOKEN URI ====================

    /// Returns the token URI for `token_id`.
    ///
    /// If a `base_uri` was set during initialization the URI is constructed as
    /// `{base_uri}{token_id}`.  Otherwise the `image` field from the stored
    /// `TokenMetadata` is returned directly.
    pub fn token_uri(env: Env, token_id: u32) -> Result<String, NftError> {
        // Ensure the token exists
        if !env.storage().persistent().has(&DataKey::Owner(token_id)) {
            return Err(NftError::TokenNotFound);
        }

        let base_uri: String = env
            .storage()
            .instance()
            .get(&DataKey::BaseUri)
            .unwrap_or_else(|| String::from_str(&env, ""));

        // If a base URI is configured, concatenate it with the token ID string.
        // Soroban String does not support formatting, so we build the numeric
        // suffix manually and append it byte-by-byte.
        if base_uri.len() > 0 {
            // Convert token_id to decimal string bytes
            let id_bytes = u32_to_decimal_bytes(token_id);
            let id_len = id_bytes[8] as usize; // last byte stores length
            let base_len = base_uri.len() as usize;
            let total_len = base_len + id_len;

            // Build combined byte slice on the stack (max 256 bytes)
            let mut buf = [0u8; 256];
            base_uri.copy_into_slice(&mut buf[..base_len]);
            for i in 0..id_len {
                buf[base_len + i] = id_bytes[i];
            }

            return Ok(String::from_bytes(&env, &buf[..total_len]));
        }

        // No base URI — return the image field from stored metadata
        let meta: TokenMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::Metadata(token_id))
            .ok_or(NftError::TokenNotFound)?;

        Ok(meta.image)
    }

    // ==================== OWNERSHIP ====================

    /// Returns the owner of `token_id`.
    pub fn owner_of(env: Env, token_id: u32) -> Result<Address, NftError> {
        env.storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .ok_or(NftError::TokenNotFound)
    }

    /// Returns the number of tokens owned by `owner`.
    pub fn balance_of(env: Env, owner: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    // ==================== MINTING ====================

    /// Mint a new token with full metadata.
    ///
    /// Only the admin may call this function.  The metadata is validated before
    /// storage — see [`Self::validate_metadata`] for the rules.
    ///
    /// # Parameters
    /// - `admin`    — must match the stored admin and authorize the call
    /// - `to`       — recipient address
    /// - `token_id` — unique token identifier (caller-chosen)
    /// - `metadata` — full `TokenMetadata` record
    pub fn mint(
        env: Env,
        admin: Address,
        to: Address,
        token_id: u32,
        metadata: TokenMetadata,
    ) -> Result<(), NftError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        // Prevent double-minting
        if env.storage().persistent().has(&DataKey::Owner(token_id)) {
            return Err(NftError::TokenAlreadyExists);
        }

        // Validate metadata before writing anything
        Self::validate_metadata(&env, &metadata)?;

        // Write owner
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &to);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Owner(token_id), 17_280, 120_960);

        // Update balance
        let bal: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(bal + 1));
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(to.clone()), 17_280, 120_960);

        // Write metadata
        env.storage()
            .persistent()
            .set(&DataKey::Metadata(token_id), &metadata);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Metadata(token_id), 17_280, 120_960);

        // Increment total supply
        let supply: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + 1));

        env.events().publish(
            (symbol_short!("mint"), symbol_short!("nft")),
            (to, token_id),
        );

        Ok(())
    }

    // ==================== TRANSFER ====================

    /// Transfer `token_id` from `from` to `to`.
    ///
    /// The caller must be the token owner **or** an approved address.
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError> {
        from.require_auth();
        Self::check_approved(&env, &from, token_id)?;

        Self::do_transfer(&env, &from, &to, token_id)?;

        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("nft")),
            (from, to, token_id),
        );

        Ok(())
    }

    // ==================== APPROVALS ====================

    /// Approve `approved` to transfer `token_id` on behalf of the owner.
    ///
    /// Pass the zero-equivalent address to revoke a previous approval.
    pub fn approve(
        env: Env,
        owner: Address,
        approved: Address,
        token_id: u32,
    ) -> Result<(), NftError> {
        owner.require_auth();

        let stored_owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .ok_or(NftError::TokenNotFound)?;

        if owner != stored_owner {
            return Err(NftError::NotOwner);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Approved(token_id), &approved);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Approved(token_id), 17_280, 120_960);

        env.events().publish(
            (symbol_short!("approve"), symbol_short!("nft")),
            (owner, approved, token_id),
        );

        Ok(())
    }

    /// Grant or revoke operator approval for all tokens owned by `owner`.
    pub fn set_approval_for_all(
        env: Env,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), NftError> {
        owner.require_auth();

        env.storage().persistent().set(
            &DataKey::ApproveAll(owner.clone(), operator.clone()),
            &approved,
        );
        env.storage().persistent().extend_ttl(
            &DataKey::ApproveAll(owner.clone(), operator.clone()),
            17_280,
            120_960,
        );

        env.events().publish(
            (symbol_short!("apprall"), symbol_short!("nft")),
            (owner, operator, approved),
        );

        Ok(())
    }

    /// Returns the approved address for `token_id`, or `None`.
    pub fn get_approved(env: Env, token_id: u32) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Approved(token_id))
    }

    /// Returns `true` if `operator` is approved for all tokens of `owner`.
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::ApproveAll(owner, operator))
            .unwrap_or(false)
    }

    // ==================== METADATA QUERIES ====================

    /// Returns the full `TokenMetadata` for `token_id`.
    pub fn get_metadata(env: Env, token_id: u32) -> Result<TokenMetadata, NftError> {
        env.storage()
            .persistent()
            .get(&DataKey::Metadata(token_id))
            .ok_or(NftError::TokenNotFound)
    }

    /// Returns only the attributes list for `token_id`.
    pub fn get_attributes(env: Env, token_id: u32) -> Result<Vec<Attribute>, NftError> {
        let meta: TokenMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::Metadata(token_id))
            .ok_or(NftError::TokenNotFound)?;
        Ok(meta.attributes)
    }

    // ==================== METADATA UPDATE ====================

    /// Update the metadata for an existing token (admin-only).
    ///
    /// The new metadata is fully validated before the old record is replaced.
    pub fn update_metadata(
        env: Env,
        admin: Address,
        token_id: u32,
        metadata: TokenMetadata,
    ) -> Result<(), NftError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if !env.storage().persistent().has(&DataKey::Owner(token_id)) {
            return Err(NftError::TokenNotFound);
        }

        Self::validate_metadata(&env, &metadata)?;

        env.storage()
            .persistent()
            .set(&DataKey::Metadata(token_id), &metadata);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Metadata(token_id), 17_280, 120_960);

        env.events()
            .publish((symbol_short!("updmeta"), symbol_short!("nft")), token_id);

        Ok(())
    }

    // ==================== METADATA VALIDATION ====================

    /// Validate a `TokenMetadata` record.
    ///
    /// ### Rules
    ///
    /// **Required fields (must be non-empty):**
    /// - `name`
    /// - `description`
    /// - `image`
    ///
    /// **Image URI scheme:**
    /// The `image` field must start with one of:
    /// - `ipfs://`
    /// - `https://`
    /// - `http://`
    /// - `data:`
    ///
    /// **Background color (when non-empty):**
    /// Must be exactly 6 ASCII hex characters (`0-9`, `a-f`, `A-F`).
    ///
    /// **Attributes:**
    /// Every `Attribute` must have a non-empty `trait_type` and `value`.
    pub fn validate_metadata(env: &Env, meta: &TokenMetadata) -> Result<(), NftError> {
        // Required: name
        if meta.name.len() == 0 {
            return Err(NftError::MetadataFieldEmpty);
        }
        // Required: description
        if meta.description.len() == 0 {
            return Err(NftError::MetadataFieldEmpty);
        }
        // Required: image
        if meta.image.len() == 0 {
            return Err(NftError::MetadataFieldEmpty);
        }

        // Image URI scheme validation
        if !is_valid_uri(env, &meta.image) {
            return Err(NftError::InvalidImageUri);
        }

        // Background color: must be empty OR exactly 6 hex chars
        if meta.background_color.len() > 0 && !is_valid_hex_color(&meta.background_color) {
            return Err(NftError::InvalidBackgroundColor);
        }

        // Attributes: each must have non-empty trait_type and value
        for attr in meta.attributes.iter() {
            if attr.trait_type.len() == 0 || attr.value.len() == 0 {
                return Err(NftError::InvalidAttribute);
            }
        }

        Ok(())
    }

    // ==================== ADMIN ====================

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Result<Address, NftError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(NftError::NotInitialized)
    }

    // ==================== PRIVATE HELPERS ====================

    /// Assert that `caller` is the stored admin.
    fn require_admin(env: &Env, caller: &Address) -> Result<(), NftError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(NftError::NotInitialized)?;
        if caller != &admin {
            return Err(NftError::NotAdmin);
        }
        Ok(())
    }

    /// Assert that `caller` is the owner or an approved address for `token_id`.
    fn check_approved(env: &Env, caller: &Address, token_id: u32) -> Result<(), NftError> {
        let owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .ok_or(NftError::TokenNotFound)?;

        if caller == &owner {
            return Ok(());
        }

        // Check single-token approval
        if let Some(approved) = env
            .storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::Approved(token_id))
        {
            if caller == &approved {
                return Ok(());
            }
        }

        // Check operator approval
        if env
            .storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::ApproveAll(owner, caller.clone()))
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(NftError::NotApproved)
    }

    /// Execute the token transfer: update owner, balances, clear approval.
    fn do_transfer(env: &Env, from: &Address, to: &Address, token_id: u32) -> Result<(), NftError> {
        let owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .ok_or(NftError::TokenNotFound)?;

        if from != &owner {
            return Err(NftError::NotOwner);
        }

        // Decrement sender balance
        let from_bal: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &from_bal.saturating_sub(1));

        // Increment recipient balance
        let to_bal: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_bal + 1));
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(to.clone()), 17_280, 120_960);

        // Update owner
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), to);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Owner(token_id), 17_280, 120_960);

        // Clear single-token approval on transfer
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pure helper functions (no_std compatible)
// ---------------------------------------------------------------------------

/// Returns `true` if `uri` starts with a recognised scheme.
///
/// Accepted prefixes: `ipfs://`, `https://`, `http://`, `data:`.
fn is_valid_uri(_env: &Env, uri: &String) -> bool {
    let len = uri.len() as usize;
    if len == 0 {
        return false;
    }

    // Copy up to 10 bytes to check the scheme prefix
    let check_len = if len < 10 { len } else { 10 };
    let mut buf = [0u8; 10];
    uri.copy_into_slice(&mut buf[..check_len]);

    // ipfs://  (7 bytes)
    if check_len >= 7 && &buf[..7] == b"ipfs://" {
        return true;
    }
    // https:// (8 bytes)
    if check_len >= 8 && &buf[..8] == b"https://" {
        return true;
    }
    // http://  (7 bytes)
    if check_len >= 7 && &buf[..7] == b"http://" {
        return true;
    }
    // data:    (5 bytes)
    if check_len >= 5 && &buf[..5] == b"data:" {
        return true;
    }

    false
}

/// Returns `true` if `color` is exactly 6 ASCII hex characters.
fn is_valid_hex_color(color: &String) -> bool {
    if color.len() != 6 {
        return false;
    }
    let mut buf = [0u8; 6];
    color.copy_into_slice(&mut buf);
    for b in buf.iter() {
        if !matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
            return false;
        }
    }
    true
}

/// Convert a `u32` to its decimal ASCII bytes.
///
/// Returns a 9-byte array where bytes `[0..len]` are the ASCII digits and
/// byte `[8]` stores the digit count (1–10).
fn u32_to_decimal_bytes(mut n: u32) -> [u8; 9] {
    let mut buf = [0u8; 9];
    if n == 0 {
        buf[0] = b'0';
        buf[8] = 1;
        return buf;
    }
    let mut tmp = [0u8; 8];
    let mut len = 0usize;
    while n > 0 {
        tmp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    // Reverse digits into buf
    for i in 0..len {
        buf[i] = tmp[len - 1 - i];
    }
    buf[8] = len as u8;
    buf
}

#[cfg(test)]
mod test;
