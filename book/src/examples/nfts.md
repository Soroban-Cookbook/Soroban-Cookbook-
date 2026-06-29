# NFT Examples

Non-Fungible Token contracts on Soroban — from a minimal mintable NFT through a full JSON-schema metadata standard to a marketplace with auctions, bundles, and royalties.

## Examples

| # | Example | Pattern | Difficulty |
|---|---------|---------|------------|
| 01 | [Basic NFT](./nfts/01-basic-nft.md) | Mintable NFT with authorization, enumeration, and approvals | Beginner |
| 02 | [NFT Metadata](./nfts/02-nft-metadata.md) | On-chain metadata struct, `token_uri` fallback, IPFS-friendly | Beginner |
| 03 | [NFT Metadata Standards](./nfts/03-nft-metadata-standards.md) | JSON-schema-compliant metadata, attribute system, URI validation | Intermediate |
| 04 | [NFT Marketplace](./nfts/04-nft-marketplace.md) | Fixed-price listings, auctions, bundles, royalties, trade history | Advanced |

## Key Concepts

- **Ownership in `Persistent` storage** — `DataKey::Owner(token_id)` → `Address`
- **Swap-remove enumeration** — per-owner token lists stay dense with O(1) removal
- **Single-token and operator approvals** — cleared on transfer
- **Metadata strategies** — off-chain URI, on-chain struct, or hybrid
- **Marketplace royalties** — `royalty_bps` basis points split on every sale

## See Also

- [NFT Patterns Reference](./docs/nft-patterns.md) — deep-dive guide
- [Security Best Practices](./docs/security-best-practices.md)

## Prerequisites

- [Basics](./basics.md)

## Next: [Governance](./governance.md)
