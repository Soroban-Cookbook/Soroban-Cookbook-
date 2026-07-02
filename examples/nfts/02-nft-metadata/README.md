# 02 · NFT Metadata

Extends the basic NFT pattern with a structured on-chain `NFTMetadata` struct, a `token_uri` resolution function that falls back to a `base_uri/{id}` convention, and a runtime toggle between on-chain and off-chain metadata modes.

Builds directly on the patterns from [01-basic-nft](../01-basic-nft/).

---

## What This Example Covers

| Feature | Implementation |
|---------|---------------|
| `NFTMetadata` struct | `name`, `description`, `image`, `external_url`, `attributes` stored on-chain |
| `NFTAttribute` struct | `trait_type` + `value` for typed trait data |
| On-chain / off-chain toggle | `on_chain_metadata: bool` at initialization |
| `token_uri` resolution | Per-token URI first; falls back to `base_uri/{token_id}` |
| Admin-only `set_metadata` | Updates on-chain metadata after mint |
| Typed errors | `NFTError` covering invalid state, missing metadata, wrong mode |

---

## Metadata Modes

The contract is initialized with either on-chain or off-chain metadata — you cannot change modes after deployment.

**Off-chain mode** (`on_chain_metadata = false`):

- Each token requires a `token_uri` at mint time (e.g. `ipfs://Qm.../1.json`).
- `token_uri(id)` returns the per-token URI if set, or `base_uri/id` otherwise.
- `metadata(id)` returns `Err(MetadataNotEnabled)`.

**On-chain mode** (`on_chain_metadata = true`):

- Each token requires a full `NFTMetadata` struct at mint time.
- `metadata(id)` returns the stored struct.
- No `token_uri` is stored; `token_uri(id)` still resolves via `base_uri/id`.

---

## Key Patterns from Basics

### Authentication (`03-authentication` pattern)

```rust
pub fn initialize(env: Env, admin: Address, base_uri: String, on_chain_metadata: bool)
    -> Result<(), NFTError>
{
    admin.require_auth();
    // ...
}

pub fn set_metadata(env: Env, token_id: u64, metadata: NFTMetadata) -> Result<(), NFTError> {
    let admin = read_admin(&env)?;
    admin.require_auth();
    // ...
}
```

### `token_uri` Fallback Pattern

```rust
pub fn token_uri(env: Env, token_id: u64) -> Result<String, NFTError> {
    // 1. Return per-token URI if explicitly set
    if let Some(uri) = env.storage().instance().get(&DataKey::TokenUri(token_id)) {
        return Ok(uri);
    }
    // 2. Fall back to base_uri/{token_id}
    let base_uri = read_base_uri(&env)?;
    Ok(format_token_uri(&env, &base_uri, token_id))
}
```

This is the standard IPFS baseURI pattern used by most NFT collections: set a single `base_uri` pointing to a directory and let individual token metadata files be addressable at `base_uri/{id}.json`.

### Storage Tier Selection

| Key | Tier | Rationale |
|-----|------|-----------|
| `Admin`, `BaseUri`, `OnChainMetadata` | `Instance` | Contract config; read together on most calls |
| `Owner(id)`, `TokenUri(id)`, `Metadata(id)` | `Instance` | Note: this example stores per-token data in `Instance` — `Persistent` is preferred for large collections (see `03-nft-metadata-standards`) |

---

## Data Types

```rust
#[contracttype]
pub struct NFTAttribute {
    pub trait_type: String,   // e.g. "Rarity"
    pub value:      String,   // e.g. "Legendary"
}

#[contracttype]
pub struct NFTMetadata {
    pub name:         String,
    pub description:  String,
    pub image:        String,           // ipfs://, https://, or data: URI
    pub external_url: Option<String>,
    pub attributes:   Vec<NFTAttribute>,
}
```

---

## Contract Interface

```rust
fn initialize(env, admin, base_uri: String, on_chain_metadata: bool) -> Result<(), NFTError>
fn mint(env, to, token_id: u64, token_uri: Option<String>, metadata: Option<NFTMetadata>)
    -> Result<(), NFTError>
fn owner_of(env, token_id: u64)        -> Result<Address, NFTError>
fn token_uri(env, token_id: u64)       -> Result<String, NFTError>
fn metadata(env, token_id: u64)        -> Result<Option<NFTMetadata>, NFTError>
fn set_metadata(env, token_id, meta)   -> Result<(), NFTError>
fn base_uri(env)                       -> Result<String, NFTError>
fn is_on_chain_metadata(env)           -> Result<bool, NFTError>
```

---

## How to Run

```bash
cd examples/nfts/02-nft-metadata

# Run all tests
cargo test

# Build the WASM contract
cargo build --target wasm32-unknown-unknown --release
```

---

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Contract with metadata struct, URI resolution, and on/off-chain toggle |
| `src/test.rs` | Tests for both on-chain and off-chain metadata modes |
| `Cargo.toml` | Crate metadata (`nft-metadata-v2`) and workspace dependency references |

---

## Next Steps

- **[03-nft-metadata-standards](../03-nft-metadata-standards/)** — JSON-schema-compliant metadata validation, media URI checks, attribute constraints, and approval expiry
- **[04-nft-marketplace](../04-nft-marketplace/)** — List, auction, and trade NFTs with royalty enforcement
