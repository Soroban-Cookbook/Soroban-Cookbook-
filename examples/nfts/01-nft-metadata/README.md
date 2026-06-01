# NFT Metadata Standards

A complete Soroban NFT contract that implements metadata standards with JSON schema compliance, a typed attribute system, image/media URI validation, and full metadata validation.

## Metadata Schema

Each token stores a `TokenMetadata` record on-chain that mirrors the following JSON schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema",
  "title": "TokenMetadata",
  "type": "object",
  "required": ["name", "description", "image"],
  "properties": {
    "name":             { "type": "string", "minLength": 1 },
    "description":      { "type": "string", "minLength": 1 },
    "image":            { "type": "string", "pattern": "^(ipfs://|https://|http://|data:)" },
    "external_url":     { "type": "string" },
    "animation_url":    { "type": "string" },
    "background_color": { "type": "string", "pattern": "^([0-9a-fA-F]{6})?$" },
    "attributes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["trait_type", "value"],
        "properties": {
          "trait_type": { "type": "string", "minLength": 1 },
          "value":      { "type": "string", "minLength": 1 }
        }
      }
    }
  }
}
```

### Example metadata record

```json
{
  "name": "Legendary Sword #42",
  "description": "A legendary sword with immense power",
  "image": "ipfs://QmHash/sword.png",
  "external_url": "https://example.com/items/42",
  "animation_url": "https://cdn.example.com/sword.mp4",
  "background_color": "1A2B3C",
  "attributes": [
    { "trait_type": "Rarity", "value": "Legendary" },
    { "trait_type": "Power",  "value": "100"       },
    { "trait_type": "Color",  "value": "Blue"      }
  ]
}
```

## Acceptance Criteria

| Criterion              | Status | Implementation detail                                      |
|------------------------|--------|------------------------------------------------------------|
| JSON schema compliance | ✅     | `TokenMetadata` struct mirrors the JSON schema 1-to-1      |
| Attribute system       | ✅     | `Vec<Attribute>` with `trait_type` + `value` string fields |
| Image / media URIs     | ✅     | `image`, `animation_url`; URI scheme validation enforced   |
| Metadata validation    | ✅     | `validate_metadata()` called on every mint and update      |
| Documentation          | ✅     | Module docs, README, inline comments, test doc-comments    |

## Contract API

### Initialization

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    base_uri: String,   // pass "" to use per-token image URIs
) -> Result<(), NftError>
```

### Minting

```rust
pub fn mint(
    env: Env,
    admin: Address,
    to: Address,
    token_id: u32,
    metadata: TokenMetadata,
) -> Result<(), NftError>
```

### Metadata queries

```rust
pub fn get_metadata(env: Env, token_id: u32) -> Result<TokenMetadata, NftError>
pub fn get_attributes(env: Env, token_id: u32) -> Result<Vec<Attribute>, NftError>
pub fn token_uri(env: Env, token_id: u32) -> Result<String, NftError>
```

### Metadata update (admin-only)

```rust
pub fn update_metadata(
    env: Env,
    admin: Address,
    token_id: u32,
    metadata: TokenMetadata,
) -> Result<(), NftError>
```

### Transfers & approvals

```rust
pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError>
pub fn approve(env: Env, owner: Address, approved: Address, token_id: u32) -> Result<(), NftError>
pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) -> Result<(), NftError>
pub fn get_approved(env: Env, token_id: u32) -> Option<Address>
pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool
```

## Metadata Validation Rules

`validate_metadata()` enforces the following rules before any write:

| Field              | Rule                                                        |
|--------------------|-------------------------------------------------------------|
| `name`             | Must be non-empty                                           |
| `description`      | Must be non-empty                                           |
| `image`            | Must be non-empty and start with `ipfs://`, `https://`, `http://`, or `data:` |
| `background_color` | If non-empty, must be exactly 6 ASCII hex characters        |
| `attributes[*].trait_type` | Must be non-empty                                 |
| `attributes[*].value`      | Must be non-empty                                 |

## URI Conventions

| Scheme     | Example                                      | Use case                  |
|------------|----------------------------------------------|---------------------------|
| `ipfs://`  | `ipfs://QmHash/image.png`                    | Decentralized storage     |
| `https://` | `https://cdn.example.com/nft/1.png`          | Centralized CDN           |
| `http://`  | `http://localhost:8080/nft/1.png`            | Local development         |
| `data:`    | `data:image/svg+xml;base64,PHN2Zy8+`         | Fully on-chain SVG        |

## Storage Layout

| Key                          | Tier       | Description                        |
|------------------------------|------------|------------------------------------|
| `DataKey::Admin`             | instance   | Contract admin address             |
| `DataKey::Name`              | instance   | Collection name                    |
| `DataKey::Symbol`            | instance   | Collection symbol                  |
| `DataKey::BaseUri`           | instance   | Optional base URI prefix           |
| `DataKey::TotalSupply`       | instance   | Running mint counter               |
| `DataKey::Owner(id)`         | persistent | Owner of token `id`                |
| `DataKey::Balance(addr)`     | persistent | Number of tokens held by `addr`    |
| `DataKey::Metadata(id)`      | persistent | Full `TokenMetadata` for token     |
| `DataKey::Approved(id)`      | persistent | Single-token approval address      |
| `DataKey::ApproveAll(o, op)` | persistent | Operator approval flag             |

## Running the Tests

```bash
cargo test -p nft-metadata
```

## Key Concepts

- **On-chain metadata** — all metadata is stored directly in contract storage, making it immutable and verifiable without external dependencies.
- **Attribute system** — traits are stored as a `Vec<Attribute>` allowing any number of typed key-value pairs per token.
- **URI validation** — the contract enforces that image URIs use a recognised scheme at mint time, preventing garbage data from entering storage.
- **Admin-controlled updates** — metadata can be updated post-mint by the admin, enabling dynamic NFTs while keeping a clear authority model.
- **TTL management** — all persistent entries call `extend_ttl` to prevent premature expiry on Soroban's ledger.
