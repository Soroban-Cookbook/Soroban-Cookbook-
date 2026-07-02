# 01 · Basic NFT

**Source:** [`examples/nfts/01-basic-nft/`](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/tree/main/examples/nfts/01-basic-nft)

A complete, minimal Non-Fungible Token contract: minting, transfer, operator approval, and enumeration. The recommended starting point before adding metadata or marketplace logic.

## What You'll Learn

- Storing token ownership in `Persistent` storage keyed on `token_id`
- Global and per-owner token enumeration using the swap-remove pattern
- Single-token and operator approvals
- `transfer_from` authorization checks

## Key Concepts

| Concept | Detail |
|---------|--------|
| `Owner(token_id)` | `Persistent` storage → `Address` |
| Swap-remove | Per-owner list stays dense; removal is O(1) |
| `Approved(token_id)` | Cleared on every transfer |
| `ApproveAll(owner, op)` | Blanket operator permission |

## Quick Code

```rust
client.initialize(&admin, &String::from_str(&env, "MyNFT"), &String::from_str(&env, "NFT"));
client.mint(&admin, &alice, &42u32);
client.approve(&alice, &bob, &42u32);
client.transfer_from(&bob, &alice, &charlie, &42u32);
```

## Run the Example

```bash
cd examples/nfts/01-basic-nft
cargo test
```

## Next: [02 · NFT Metadata](./02-nft-metadata.md)
