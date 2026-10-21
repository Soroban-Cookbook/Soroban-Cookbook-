# NFT Patterns on Soroban

A comprehensive reference for building Non-Fungible Token contracts on Stellar with Soroban. Covers ownership mechanics, metadata strategies, approval delegation, marketplace design, and security considerations — with annotated code drawn from the cookbook examples.

---

## Table of Contents

- [Soroban NFT vs ERC-721](#soroban-nft-vs-erc-721)
- [Core Ownership Pattern](#core-ownership-pattern)
- [Enumeration](#enumeration)
- [Approval Delegation](#approval-delegation)
- [Metadata Strategies](#metadata-strategies)
- [Marketplace Patterns](#marketplace-patterns)
- [Event Design](#event-design)
- [Storage Layout Reference](#storage-layout-reference)
- [Security Checklist](#security-checklist)
- [Examples Index](#examples-index)

---

## Soroban NFT vs ERC-721

Soroban does not have an official NFT standard equivalent to ERC-721. The cookbook examples implement a practical set of conventions that:

- Mirror ERC-721's ownership and approval API surface where sensible
- Replace Solidity-specific concepts (`msg.sender`, `_beforeTokenTransfer` hooks) with explicit Soroban equivalents
- Use Soroban's three-tier storage model rather than a flat mapping

| ERC-721 concept | Soroban equivalent |
|-----------------|-------------------|
| `msg.sender` | Explicit `from: Address` parameter + `require_auth()` |
| `ownerOf(uint256)` | `DataKey::Owner(token_id)` in `Persistent` storage |
| `_mint` internal hook | `mint(admin, to, token_id)` — admin gated |
| `transferFrom` | `transfer_from(spender, from, to, token_id)` |
| `approve` | Same name; stored in `Persistent` storage |
| `setApprovalForAll` | `set_approval_for_all(owner, operator, bool)` |
| `emit Transfer(...)` | `env.events().publish((symbol!("transfer"), ...), ...)` |
| `IERC721Metadata` | On-chain `TokenMetadata` struct or `token_uri` function |

---

## Core Ownership Pattern

Every token is identified by a `token_id` (typically `u32`) and its ownership is a single `Address` value in `Persistent` storage:

```rust
#[contracttype]
pub enum DataKey {
    Owner(u32),    // token_id → Address
    Balance(Address), // address → u32 (count of owned tokens)
    // ...
}

pub fn owner_of(env: Env, token_id: u32) -> Result<Address, NftError> {
    env.storage()
        .persistent()
        .get(&DataKey::Owner(token_id))
        .ok_or(NftError::TokenNotFound)
}
```

**Why `Persistent`?** Owner records must survive ledger entry expiry. Soroban's `Temporary` and `Instance` tiers have shorter TTLs and are unsuitable for long-lived ownership claims. See [docs/storage-types.md](./storage-types.md).

**TTL extension**: Every write to a `Persistent` owner record should extend its TTL:

```rust
env.storage().persistent().set(&DataKey::Owner(token_id), &to);
env.storage().persistent().extend_ttl(
    &DataKey::Owner(token_id),
    LEDGER_THRESHOLD,
    LEDGER_BUMP,
);
```

---

## Enumeration

NFT contracts typically expose two enumeration surfaces:

1. **Global enumeration** — iterate all tokens by sequential index
2. **Per-owner enumeration** — iterate all tokens held by a specific address

### Global Enumeration

```rust
DataKey::TotalSupply          // u32 — stored in Instance
DataKey::TokenByIndex(index)  // index → token_id in Persistent
```

On mint, append the new `token_id` at `TokenByIndex(total_supply)` and increment `TotalSupply`.

### Per-Owner Enumeration (Swap-Remove)

The per-owner list uses a **swap-remove** strategy to stay dense without gaps:

```rust
DataKey::Balance(owner)              // owner → count in Persistent
DataKey::OwnedToken(owner, index)    // (owner, index) → token_id
DataKey::OwnerTokenIndex(token_id)   // token_id → its current index in owner's list
```

**On remove** (`token B` at index 1 from list `[A, B, C]`):

1. Find `B`'s index: `OwnerTokenIndex(B) = 1`
2. Move the last token (`C` at index 2) to index 1: `OwnedToken(owner, 1) = C`
3. Update `C`'s index: `OwnerTokenIndex(C) = 1`
4. Delete the last slot: remove `OwnedToken(owner, 2)` and `OwnerTokenIndex(B)`
5. Decrement balance

This keeps every removal O(1) with no holes in the list.

---

## Approval Delegation

Soroban NFTs support two approval modes:

### Single-Token Approval

Grants a specific address the right to transfer one specific token:

```rust
pub fn approve(
    env: Env,
    owner: Address,
    approved: Address,
    token_id: u32,
) -> Result<(), NftError> {
    owner.require_auth();
    let current_owner = Self::owner_of(env.clone(), token_id)?;
    if current_owner != owner {
        return Err(NftError::NotOwner);
    }
    env.storage().persistent().set(&DataKey::Approved(token_id), &approved);
    // emit Approval event
    Ok(())
}
```

**Important**: The approval is cleared automatically on every transfer — do not carry approvals across ownership changes.

### Operator Approval

Grants a specific address the right to transfer *all* tokens owned by the granting address:

```rust
pub fn set_approval_for_all(
    env: Env,
    owner: Address,
    operator: Address,
    approved: bool,
) -> Result<(), NftError> {
    owner.require_auth();
    env.storage().persistent().set(
        &DataKey::ApproveAll(owner.clone(), operator.clone()),
        &approved,
    );
    // emit ApprovalForAll event
    Ok(())
}
```

### Approval Check Order in `transfer_from`

```rust
fn check_approved(env, spender, owner, token_id) -> Result<(), NftError> {
    if spender == owner { return Ok(()); }

    // 1. Check single-token approval
    if let Some(approved) = get_approved(env.clone(), token_id) {
        if approved == spender { return Ok(()); }
    }

    // 2. Check operator approval
    if is_approved_for_all(env, owner, spender) { return Ok(()); }

    Err(NftError::NotApproved)
}
```

### Approval Expiry (Advanced)

`03-nft-metadata-standards` adds an `expiration_ledger` to approvals. Expired approvals are treated as non-existent:

```rust
#[contracttype]
pub struct ApprovalData {
    pub approved:          Address,
    pub expiration_ledger: u32,
}

// Check
let data: ApprovalData = env.storage().persistent().get(&DataKey::Approved(token_id))?;
if env.ledger().sequence() > data.expiration_ledger {
    return Err(NftError::ApprovalExpired);
}
```

---

## Metadata Strategies

### Strategy 1: Off-Chain URI Only (Lightweight)

Store nothing on-chain beyond ownership. Return a `base_uri/{token_id}` string that points to a JSON file hosted on IPFS or a CDN.

**Best for**: Large collections where on-chain storage cost matters most.

```rust
pub fn token_uri(env: Env, token_id: u64) -> String {
    let base: String = env.storage().instance().get(&DataKey::BaseUri).unwrap();
    // returns "ipfs://Qm.../42" or "https://api.example.com/nft/42"
    format_token_uri(&env, &base, token_id)
}
```

**Trade-off**: Metadata is mutable by whoever controls the server. Use content-addressed URIs (`ipfs://`) to make metadata immutable.

### Strategy 2: Full On-Chain Metadata

Store a `TokenMetadata` struct directly in `Persistent` storage for each token. The data is immutable unless the admin explicitly updates it.

**Best for**: Small collections, dynamic NFTs, or when verifiability is paramount.

```rust
#[contracttype]
pub struct TokenMetadata {
    pub name:             String,
    pub description:      String,
    pub image:            String,           // ipfs://, https://, data:
    pub external_url:     Option<String>,
    pub animation_url:    Option<String>,
    pub background_color: Option<String>,   // 6-char hex
    pub attributes:       Vec<Attribute>,
}

env.storage().persistent().set(&DataKey::Metadata(token_id), &metadata);
```

**Trade-off**: Higher per-token storage cost; the `Persistent` entry TTL must be managed.

### Strategy 3: Hybrid (Base URI + Per-Token Override)

A base URI covers the entire collection; individual tokens can override with a specific URI:

```rust
pub fn token_uri(env: Env, token_id: u64) -> String {
    // 1. Per-token URI wins if set
    if let Some(uri) = env.storage().instance().get(&DataKey::TokenUri(token_id)) {
        return uri;
    }
    // 2. Fall back to base_uri/{token_id}
    format_token_uri(&env, &read_base_uri(&env), token_id)
}
```

### Metadata Validation

`03-nft-metadata-standards` enforces these rules at mint and update time:

| Field | Rule |
|-------|------|
| `name` | Non-empty string |
| `description` | Non-empty string |
| `image` | Non-empty; must start with `ipfs://`, `https://`, `http://`, or `data:` |
| `background_color` | If set, exactly 6 ASCII hex digits (no `#` prefix) |
| `attributes[*].trait_type` | Non-empty |
| `attributes[*].value` | Non-empty |

### URI Scheme Reference

| Scheme | Example | Notes |
|--------|---------|-------|
| `ipfs://` | `ipfs://QmHash/image.png` | Content-addressed; immutable if hash is correct |
| `https://` | `https://cdn.example.com/1.png` | Centralized; fast but mutable |
| `http://` | `http://localhost:8080/1.png` | Development only |
| `data:` | `data:image/svg+xml;base64,...` | Fully on-chain; no external dependency |

---

## Marketplace Patterns

### Fixed-Price Sale

1. Seller calls `create_fixed_price_listing(seller, items, price, royalty_recipient, royalty_bps)`.
2. The marketplace records the `Listing` and emits a `list` event.
3. A buyer calls `buy(buyer, listing_id, offer_amount)`.
4. The contract verifies `offer_amount >= price`, transfers the NFT(s), splits payment (royalty → recipient, remainder → seller), and records a `TradeRecord`.

### English Auction

1. Seller calls `create_auction_listing(seller, items, reserve_price, duration_ledgers, ...)`.
2. Bidders call `place_bid(bidder, listing_id, amount)` — each bid must exceed the current highest bid; the previous highest bidder is refunded.
3. After `end_ledger` passes, anyone calls `finalize_auction(executor, listing_id)`.
4. The contract transfers the NFT(s) to the winning bidder and distributes payment.

### Bundle Sales

A `Listing` contains `items: Vec<ListingItem>` — each item names an NFT contract and token ID. At settlement, all items are transferred atomically. If any transfer fails, the entire transaction reverts.

### Royalty Split

```
buyer_payment = total_sale_price
royalty_paid  = (total_sale_price * royalty_bps) / 10_000
seller_nets   = buyer_payment - royalty_paid

transfer royalty_paid → listing.royalty_recipient
transfer seller_nets  → listing.seller
```

Royalty basis points are capped at `10_000` (100%) at listing creation.

---

## Event Design

Follow the `(action, namespace, primary_key)` topic convention from `examples/basics/04-events`:

| Event | Topics | Data payload |
|-------|--------|-------------|
| Token minted | `("mint", "nft")` | `(to, token_id)` |
| Token transferred | `("transfer", "nft")` | `(from, to, token_id)` |
| Approval set | `("approve", "nft")` | `(owner, approved, token_id)` |
| Operator approved | `("approve_all", "nft")` | `(owner, operator, approved)` |
| Listing created | `("list", "market", listing_id)` | `(seller, price)` |
| Bid placed | `("bid", "market", listing_id)` | `(bidder, amount)` |
| Fixed sale | `("sale", "market", listing_id)` | `(buyer, seller, amount, royalty)` |
| Auction finalized | `("finalize", "market", listing_id)` | `(winner, amount)` |

Placing the listing or token ID in topic slot 2 lets indexers query "all events for token 42" or "all events for listing 7" efficiently.

---

## Storage Layout Reference

| Key | Tier | Used in | Purpose |
|-----|------|---------|---------|
| `Admin` | Instance | all | Contract administrator address |
| `Name`, `Symbol` | Instance | 01, 03 | Collection name and ticker |
| `BaseUri` | Instance | 02, 03 | URI prefix for token_uri fallback |
| `TotalSupply` | Instance | 01, 03 | Mint counter for enumeration |
| `OnChainMetadata` | Instance | 02 | Mode flag (on-chain vs off-chain) |
| `ListingCount`, `TradeCount` | Instance | 04 | Marketplace counters |
| `Owner(token_id)` | Persistent | all | Current owner of each token |
| `Balance(address)` | Persistent | 01, 03 | Per-address token count |
| `TokenByIndex(n)` | Persistent | 01 | Global enumeration index |
| `OwnedToken(owner, n)` | Persistent | 01 | Per-owner token index |
| `OwnerTokenIndex(token_id)` | Persistent | 01 | Reverse lookup for swap-remove |
| `Metadata(token_id)` | Persistent | 02, 03 | On-chain `TokenMetadata` struct |
| `Approved(token_id)` | Persistent | 01, 03 | Single-token approval |
| `ApproveAll(owner, op)` | Persistent | 01, 03 | Operator approval flag |
| `Listing(id)` | Persistent | 04 | Active marketplace listing |
| `HighestBid(id)` | Persistent | 04 | Current winning bid on auction |
| `Trade(id)` | Persistent | 04 | Immutable settlement record |

---

## Security Checklist

- [ ] **Approval cleared on transfer**: Remove `DataKey::Approved(token_id)` inside every transfer path, including `transfer_from`.
- [ ] **Owner check before approval**: `approve` must verify that the caller matches the current owner before writing the approval.
- [ ] **Token existence before every action**: Read and unwrap `Owner(token_id)` first; return `TokenNotFound` rather than panic on missing keys.
- [ ] **`require_auth()` on the acting address**: Mint uses `admin.require_auth()`, transfer uses `from.require_auth()`, bid uses `bidder.require_auth()`.
- [ ] **No gap in enumeration**: Use swap-remove for per-owner lists; never delete a middle entry without backfilling.
- [ ] **Approval expiry**: If you store approvals without TTL, they persist indefinitely. Add `expiration_ledger` or periodically require the operator to re-approve.
- [ ] **Royalty cap**: Enforce `royalty_bps <= 10_000` at listing time to prevent 100%+ royalty configurations that drain the buyer.
- [ ] **Refund before recording bid**: In auction contracts, refund the previous highest bidder *before* recording the new bid to prevent funds from being locked on failure.
- [ ] **Persistent TTL management**: Call `extend_ttl` after every write to `Persistent` entries that must not expire.

---

## Examples Index

| Example | Directory | Key concept |
|---------|-----------|------------|
| Basic NFT | [examples/nfts/01-basic-nft](../examples/nfts/01-basic-nft/) | Ownership, enumeration, approvals |
| NFT Metadata | [examples/nfts/02-nft-metadata](../examples/nfts/02-nft-metadata/) | `token_uri`, hybrid on/off-chain |
| Metadata Standards | [examples/nfts/03-nft-metadata-standards](../examples/nfts/03-nft-metadata-standards/) | JSON-schema validation, attribute system |
| NFT Marketplace | [examples/nfts/04-nft-marketplace](../examples/nfts/04-nft-marketplace/) | Listings, auctions, bundles, royalties |
