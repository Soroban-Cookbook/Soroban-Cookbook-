# 04 · NFT Marketplace

**Source:** [`examples/nfts/04-nft-marketplace/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/04-nft-marketplace)

A full-featured marketplace: fixed-price listings, English auctions, multi-token bundle sales, royalty payments, and an immutable trade history.

## What You'll Learn

- Listing creation for both fixed-price and auction modes
- Competitive bidding with automatic refund of outbid participants
- Royalty splitting: `royalty_bps` basis points go to the creator on every sale
- Bundle atomicity: all NFTs in a bundle transfer together or not at all

## Listing Types

| Type | Created with | Settled by |
|------|-------------|-----------|
| Fixed price | `create_fixed_price_listing` | `buy` |
| Auction | `create_auction_listing` | `finalize_auction` (after `end_ledger`) |

## Quick Code

```rust
// Fixed-price: list token 42 for 1000 units, 5% royalty
let id = client.create_fixed_price_listing(
    &seller, &items, &1_000_i128, &creator, &500u32,
);

// Buy it
client.buy(&buyer, &id, &1_000_i128);
```

## Run the Example

```bash
cd examples/nfts/04-nft-marketplace
cargo test
```

## See Also

- [NFT Patterns Reference](../docs/nft-patterns.md)
- [01 · Basic NFT](./01-basic-nft.md)
