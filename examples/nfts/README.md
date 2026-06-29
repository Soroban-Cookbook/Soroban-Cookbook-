# NFT Examples

Soroban smart contract examples for Non-Fungible Tokens — from a minimal mintable NFT through a full JSON-schema metadata standard to a marketplace with auctions, bundles, and royalties.

All examples are production-quality, fully tested, and emit structured events. They build on each other in sequence; start with `01` if you are new to NFTs on Soroban.

---

## Examples

| # | Directory | Pattern | Difficulty |
|---|-----------|---------|------------|
| 01 | [basic-nft](./01-basic-nft/) | Mintable NFT with authorization, enumeration, and approvals | Beginner |
| 02 | [nft-metadata](./02-nft-metadata/) | On-chain metadata struct, `token_uri` fallback, IPFS-friendly | Beginner |
| 03 | [nft-metadata-standards](./03-nft-metadata-standards/) | JSON-schema-compliant metadata, typed attribute system, URI validation | Intermediate |
| 04 | [nft-marketplace](./04-nft-marketplace/) | Fixed-price listings, English auctions, bundles, royalties, trade history | Advanced |

---

## Key Patterns

### Ownership and Authorization (`03-authentication` basis)

NFT ownership is stored as `Persistent` storage keyed on `token_id`. Every transfer or burn calls `require_auth()` on the current owner:

```rust
pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) -> Result<(), NftError> {
    from.require_auth();
    let owner: Address = env.storage().persistent().get(&DataKey::Owner(token_id))
        .ok_or(NftError::TokenNotFound)?;
    if owner != from {
        return Err(NftError::NotOwner);
    }
    env.storage().persistent().set(&DataKey::Owner(token_id), &to);
    // emit Transfer event
    Ok(())
}
```

### Approval Pattern

Single-token approvals (`approve`) and operator approvals (`set_approval_for_all`) mirror ERC-721's delegation model:

```rust
DataKey::Approved(token_id)            // single-token delegate — cleared on transfer
DataKey::ApproveAll(owner, operator)   // blanket operator approval
```

`transfer_from` checks both in order: single approval first, then operator approval.

### Metadata Storage Strategy

| Approach | Example | Trade-off |
|----------|---------|-----------|
| URI only (off-chain) | `02-nft-metadata` | Cheapest; metadata is mutable off-chain |
| Full on-chain struct | `03-nft-metadata-standards` | Immutable, verifiable; higher storage cost |
| Hybrid (base URI + per-token override) | `02-nft-metadata` | Flexible; base URI for bulk, override for exceptions |

### Event Topics (`04-events` basis)

All NFT events follow a `(action, "nft", token_id)` topic layout for indexer-friendly filtering:

```rust
// From 01-basic-nft
env.events().publish(
    (symbol_short!("transfer"), symbol_short!("nft")),
    (from, to, token_id),
);

// From 04-nft-marketplace
env.events().publish(
    (symbol_short!("sale"), symbol_short!("market"), listing_id),
    (buyer, seller, amount, royalty_paid),
);
```

### Storage Tiers

| Data | Tier | Why |
|------|------|-----|
| Collection name, symbol, admin, supply | `Instance` | Contract-level metadata; cheap to bundle-read |
| Owner per token, balance per address | `Persistent` | Must survive ledger expiry; TTL should be extended on every write |
| Approvals (single-token and operator) | `Persistent` | Must outlive the transaction that set them |

---

## Quick Start

```bash
# Run from the beginning — basic NFT
cd examples/nfts/01-basic-nft
cargo test

# Jump straight to the marketplace
cd examples/nfts/04-nft-marketplace
cargo test

# Build all NFT examples as WASM
cargo build -p basic-nft -p nft-metadata-v2 -p nft-metadata-standards -p nft-marketplace \
    --target wasm32-unknown-unknown --release
```

---

## Security Notes

- **Token existence checks**: Always verify a token exists (non-`None` owner) before acting on it. Return a typed error, not a panic.
- **Approval expiry**: `03-nft-metadata-standards` includes per-token approval expiry via ledger sequence. Without expiry, stale approvals can be exercised indefinitely.
- **Royalty enforcement**: Royalties are enforced by the marketplace contract only. Peer-to-peer transfers via the NFT contract bypass any royalty logic.
- **Swap-remove integrity**: The `01-basic-nft` enumeration relies on a swap-remove invariant. Never write a `remove_token_from_owner` implementation that leaves gaps in the index array.
- **Bundle atomicity**: In `04-nft-marketplace`, all NFTs in a bundle must transfer successfully. A partial transfer should revert the entire transaction.

For a full guide see [docs/nft-patterns.md](../../docs/nft-patterns.md) and [docs/security-best-practices.md](../../docs/security-best-practices.md).
