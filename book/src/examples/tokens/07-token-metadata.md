# 07 · Token Metadata

**Source:** [`examples/tokens/07-token-metadata/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/tokens/07-token-metadata)

On-chain token metadata management: mutable fields (`name`, `symbol`, `icon_uri`) that the admin can update post-deploy, and immutable fields (`decimals`) that are fixed at initialization.

## What You'll Learn

- Separating mutable and immutable token metadata
- Admin-gated metadata updates with `require_auth()`
- Emitting `metadata_updated` events for off-chain caches
- Querying all metadata fields in a single call

## Mutable vs Immutable Fields

| Field | Mutable | Reason |
|-------|---------|--------|
| `name` | Yes | Rebranding support |
| `symbol` | Yes | Ticker changes |
| `icon_uri` | Yes | CDN migration |
| `decimals` | No | Fixed at deploy; changing would break all balance displays |

## Run the Example

```bash
cd examples/tokens/07-token-metadata
cargo test
```

## Next: [08 · Multi-Token Balance Manager](./08-multi-token-balance-manager.md)
