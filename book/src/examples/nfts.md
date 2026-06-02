# NFT Examples

Non-fungible tokens and marketplaces on Soroban.

## Examples

### 01 — Basic NFT

**Path:** `examples/nfts/01-basic-nft`

A mintable NFT contract with ownership queries, transfer mechanics, enumerability, and both single-token and operator approvals.

**Key concepts:**
- Mint function guarded by an admin role
- `owner_of` + `balance_of` queries
- Global and owner-specific token enumeration
- `approve`, `set_approval_for_all`, and `transfer_from` patterns

---

### 02 — NFT Metadata Standards

**Path:** `examples/nfts/01-nft-metadata`

A complete NFT contract implementing metadata standards with JSON schema compliance, a typed attribute system, image/media URI validation, and full metadata validation.

**Key concepts:**
- `TokenMetadata` struct that mirrors the JSON metadata schema
- `Vec<Attribute>` attribute system with `trait_type` + `value` fields
- Image and media URI validation (`ipfs://`, `https://`, `http://`, `data:`)
- `validate_metadata()` called on every mint and update
- Admin-controlled minting and metadata updates
- Single-token and operator approvals

---

### 03 — NFT Marketplace

**Path:** `examples/nfts/02-nft-marketplace`

An advanced marketplace contract demonstrating fixed-price listings, auction bidding, bundle sales, automatic royalty accounting, and trading history.

**Key concepts:**
- Fixed-price and auction listing flows
- Bundle sales for multiple NFTs in one order
- Royalty calculation and payout accounting
- Trade history storage for settlement auditing

---

## Metadata Schema

Each token carries a `TokenMetadata` record stored on-chain. The struct maps 1-to-1 to this JSON schema:

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

---

## Rust Types

### `Attribute`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub trait_type: String,   // e.g. "Rarity"
    pub value: String,        // e.g. "Legendary"
}
```

### `TokenMetadata`

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    // Required
    pub name: String,
    pub description: String,
    pub image: String,          // must start with ipfs://, https://, http://, or data:

    // Optional (pass empty string to omit)
    pub external_url: String,
    pub animation_url: String,
    pub background_color: String,  // 6-char hex, e.g. "FF0000"

    // Attributes
    pub attributes: Vec<Attribute>,
}
```

---

## Core API

### Initialize the collection

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    base_uri: String,   // pass "" to use per-token image URIs
) -> Result<(), NftError>
```

### Mint a token

```rust
pub fn mint(
    env: Env,
    admin: Address,
    to: Address,
    token_id: u32,
    metadata: TokenMetadata,
) -> Result<(), NftError>
```

### Query metadata

```rust
pub fn get_metadata(env: Env, token_id: u32) -> Result<TokenMetadata, NftError>
pub fn get_attributes(env: Env, token_id: u32) -> Result<Vec<Attribute>, NftError>
pub fn token_uri(env: Env, token_id: u32) -> Result<String, NftError>
```

### Transfer and approvals

```rust
pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError>
pub fn approve(env: Env, owner: Address, approved: Address, token_id: u32) -> Result<(), NftError>
pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) -> Result<(), NftError>
```

---

## Metadata Validation

`validate_metadata()` is called on every `mint` and `update_metadata` call:

| Field                      | Rule                                                                 |
|----------------------------|----------------------------------------------------------------------|
| `name`                     | Must be non-empty                                                    |
| `description`              | Must be non-empty                                                    |
| `image`                    | Must be non-empty and start with `ipfs://`, `https://`, `http://`, or `data:` |
| `background_color`         | If non-empty, must be exactly 6 ASCII hex characters                 |
| `attributes[*].trait_type` | Must be non-empty                                                    |
| `attributes[*].value`      | Must be non-empty                                                    |

---

## URI Conventions

| Scheme     | Example                               | Use case              |
|------------|---------------------------------------|-----------------------|
| `ipfs://`  | `ipfs://QmHash/image.png`             | Decentralized storage |
| `https://` | `https://cdn.example.com/nft/1.png`   | Centralized CDN       |
| `http://`  | `http://localhost:8080/nft/1.png`     | Local development     |
| `data:`    | `data:image/svg+xml;base64,PHN2Zy8+`  | Fully on-chain SVG    |

---

## Token URI Resolution

When a `base_uri` is set during initialization, `token_uri(id)` returns `{base_uri}{id}`:

```
base_uri = "https://api.example.com/metadata/"
token_id = 7
→ token_uri = "https://api.example.com/metadata/7"
```

When no `base_uri` is set, `token_uri(id)` returns the `image` field from the stored metadata directly.

---

## Testing

```bash
cargo test -p nft-metadata
```

The test suite covers:

- Collection initialization and double-init guard
- Minting with valid and invalid metadata
- All metadata validation error cases
- Token URI resolution (with and without base URI)
- Transfers and ownership updates
- Single-token and operator approvals
- Metadata update (admin-only)
- JSON schema compliance documentation test

---

## Coming Soon

### NFT Marketplace
Fixed-price and auction listings with escrow patterns, bid management, and royalty splits.

### Enumerable NFT
NFT with iteration capabilities — list all tokens owned by an address.

### Dynamic NFTs
NFTs whose metadata changes over time based on on-chain state.

### Soulbound Tokens
Non-transferable NFTs for credentials and achievements.

---

## Prerequisites

- [Basics](../examples/basics.md)
- [Storage Patterns](../examples/storage-patterns.md)

## Next

- [Governance](../examples/governance.md)
- [Tokens](../examples/tokens.md)
