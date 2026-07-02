# 02 · NFT Metadata

**Source:** [`examples/nfts/02-nft-metadata/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/02-nft-metadata)

Extends the basic NFT with a structured on-chain `NFTMetadata` struct, a `token_uri` function that falls back to `base_uri/{id}`, and a runtime toggle between on-chain and off-chain metadata modes.

## What You'll Learn

- The `token_uri` → `base_uri/{id}` fallback pattern used by most NFT collections
- When to store metadata on-chain vs off-chain
- Admin-only `set_metadata` for post-mint updates

## Metadata Modes

| Mode | Initialization | `token_uri` returns | `metadata` returns |
|------|---------------|--------------------|--------------------|
| Off-chain | `on_chain_metadata = false` | Per-token URI or `base_uri/{id}` | Error |
| On-chain | `on_chain_metadata = true` | `base_uri/{id}` | `NFTMetadata` struct |

## Quick Code

```rust
// Off-chain mode: base URI covers whole collection
client.initialize(&admin, &base_uri, &false);
client.mint(&to, &42u64, &Some(custom_uri), &None);

// On-chain mode: full struct per token
client.initialize(&admin, &base_uri, &true);
client.mint(&to, &42u64, &None, &Some(metadata));
```

## Run the Example

```bash
cd examples/nfts/02-nft-metadata
cargo test
```

## Next: [03 · NFT Metadata Standards](./03-nft-metadata-standards.md)
