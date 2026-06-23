# NFT Metadata Example

A Soroban NFT example that demonstrates metadata patterns for on-chain and off-chain storage.

## What this example shows

- `NFTMetadata` struct for structured token metadata
- `token_uri(token_id)` pattern support
- on-chain metadata storage option with `metadata(token_id)`
- IPFS-friendly token URI guidance and metadata management
- common NFT ownership checks and authorization patterns

## Files

- `Cargo.toml` — workspace crate setup
- `src/lib.rs` — NFT metadata contract
- `src/test.rs` — contract tests covering on-chain and off-chain metadata

## Build

```bash
cd examples/nfts/02-nft-metadata
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cd examples/nfts/02-nft-metadata
cargo test
```

## NFT Metadata Patterns

### Token URI pattern

This example uses a `base_uri` plus token-specific URI values to implement token URI patterns. For instance:

- `https://metadata.example.com/nft/123`
- `ipfs://bafy.../123`
- `onchain://<contract_id>/123`

### Off-chain metadata

For off-chain metadata, upload a standard JSON metadata file to IPFS and store an `ipfs://...` URI for each token.

### On-chain metadata option

When the contract is initialized with `on_chain_metadata = true`, each minted token can store structured metadata on-chain and still expose a token URI pattern for discoverability.

## IPFS Integration Guide

1. Create standard NFT metadata JSON:

```json
{
  "name": "Soroban NFT #1",
  "description": "Trackable on-chain metadata with IPFS-friendly URIs.",
  "image": "ipfs://bafybeih...",
  "external_url": "https://example.com/nft/1",
  "attributes": [
    { "trait_type": "Rarity", "value": "Gold" },
    { "trait_type": "Series", "value": "Genesis" }
  ]
}
```

2. Upload the JSON to IPFS and note the CID.
3. Use the contract `mint` function with a `token_uri` like `ipfs://<CID>/1`.
4. Optionally use on-chain metadata mode for structured metadata access with `metadata(token_id)`.

## API

- `initialize(admin, base_uri, on_chain_metadata)`
- `mint(to, token_id, token_uri, metadata)`
- `owner_of(token_id)`
- `token_uri(token_id)`
- `metadata(token_id)`
- `base_uri()`
- `is_on_chain_metadata()`
- `set_metadata(token_id, metadata)`
