# 04 · NFT Marketplace

A full-featured NFT marketplace contract for Soroban supporting fixed-price listings, English auctions, multi-token bundle sales, royalty payments, and a persistent trade history.

Builds on the authorization and event patterns from the basics and the ownership model established in [01-basic-nft](../01-basic-nft/).

---

## What This Example Covers

| Feature | Implementation |
|---------|---------------|
| Fixed-price listings | `create_fixed_price_listing` — instant buy at a set price |
| English auctions | `create_auction_listing` — time-bounded, highest-bid-wins |
| Bundle sales | `items: Vec<ListingItem>` — multiple NFTs from any contract in one listing |
| Royalty payments | `royalty_bps` — basis-point royalty split on every settlement |
| Competitive bidding | `place_bid` — new bid must exceed the current highest bid |
| Auction finalization | `finalize_auction` — callable by anyone after `end_ledger` |
| Trade history | `TradeRecord` — immutable log of every settled sale |
| Typed errors | `MarketplaceError` enum covering all invalid-state cases |
| Structured events | `list`, `bid`, `sale`, `finalize` emit `(action, "market")` topics |

---

## Core Data Types

```rust
/// One item in a listing — identifies an NFT by its contract and token ID.
#[contracttype]
pub struct ListingItem {
    pub nft_contract: Address,
    pub token_id:     u32,
}

/// A listing in the marketplace (fixed-price or auction).
#[contracttype]
pub struct Listing {
    pub seller:            Address,
    pub items:             Vec<ListingItem>,
    pub is_auction:        bool,
    pub price:             i128,       // fixed price (auction: 0)
    pub reserve_price:     i128,       // auction minimum (fixed: 0)
    pub end_ledger:        u32,        // 0 means no expiry for fixed-price
    pub royalty_recipient: Address,
    pub royalty_bps:       u32,        // basis points, max 10_000
    pub sold:              bool,
}

/// The winning bid on an active auction.
#[contracttype]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
}

/// Immutable record written after every completed sale.
#[contracttype]
pub struct TradeRecord {
    pub buyer:        Address,
    pub seller:       Address,
    pub items:        Vec<ListingItem>,
    pub amount:       i128,
    pub royalty_paid: i128,
    pub ledger:       u32,
}
```

---

## Key Patterns from Basics

### Authentication (`03-authentication` pattern)

Every action that transfers value or changes listing state requires authentication from the acting address:

```rust
pub fn create_fixed_price_listing(
    env: Env,
    seller: Address,
    items: Vec<ListingItem>,
    price: i128,
    royalty_recipient: Address,
    royalty_bps: u32,
) -> Result<u32, MarketplaceError> {
    seller.require_auth();
    // ...
}

pub fn place_bid(env: Env, bidder: Address, listing_id: u32, amount: i128)
    -> Result<(), MarketplaceError>
{
    bidder.require_auth();
    // ...
}
```

The admin is only needed for `initialize`; all subsequent marketplace actions are permissionless (any address can list, bid, or buy).

### Event Emission (`04-events` pattern)

Every state transition emits a structured event. Topics follow `(action, "market", listing_id)` so indexers can reconstruct the full lifecycle of any listing:

```rust
// New listing created
env.events().publish(
    (symbol_short!("list"), symbol_short!("market"), listing_id),
    (seller, price),
);

// Bid placed on auction
env.events().publish(
    (symbol_short!("bid"), symbol_short!("market"), listing_id),
    (bidder, amount),
);

// Fixed-price sale completed
env.events().publish(
    (symbol_short!("sale"), symbol_short!("market"), listing_id),
    (buyer, seller, amount, royalty_paid),
);

// Auction finalized
env.events().publish(
    (symbol_short!("finalize"), symbol_short!("market"), listing_id),
    (winner, amount),
);
```

### Royalty Calculation

Royalties are calculated from the sale price and split before paying the seller:

```rust
let royalty_paid = (amount * royalty_bps as i128) / 10_000;
let seller_proceeds = amount - royalty_paid;
// transfer royalty_paid → royalty_recipient
// transfer seller_proceeds → seller
```

Royalties are bounded to `royalty_bps <= 10_000` (100%) at listing creation.

### Storage Layout

| Key | Tier | Rationale |
|-----|------|-----------|
| `Admin`, `ListingCount`, `TradeCount` | `Instance` | Global counters; read on every operation |
| `Listing(id)`, `HighestBid(id)` | `Persistent` | Active listings and bids must survive ledger gaps |
| `Trade(id)` | `Persistent` | Trade history is permanent; should not expire |

---

## Contract Interface

```rust
fn initialize(env, admin: Address) -> Result<(), MarketplaceError>

fn create_fixed_price_listing(
    env, seller, items, price, royalty_recipient, royalty_bps
) -> Result<u32, MarketplaceError>

fn create_auction_listing(
    env, seller, items, reserve_price, duration_ledgers, royalty_recipient, royalty_bps
) -> Result<u32, MarketplaceError>

fn place_bid(env, bidder, listing_id, amount) -> Result<(), MarketplaceError>

fn buy(env, buyer, listing_id, offer_amount)  -> Result<(), MarketplaceError>

fn finalize_auction(env, executor, listing_id) -> Result<(), MarketplaceError>

fn get_listing(env, listing_id)               -> Result<Listing, MarketplaceError>
fn get_trade(env, trade_id)                   -> Result<TradeRecord, MarketplaceError>
```

---

## How to Run

```bash
cd examples/nfts/04-nft-marketplace

# Run all tests
cargo test

# Build the WASM contract
cargo build --target wasm32-unknown-unknown --release
```

---

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Marketplace contract — listing creation, bidding, settlement, royalties |
| `src/test.rs` | Tests for fixed-price buys, auction flows, royalty splits, error cases |
| `Cargo.toml` | Crate metadata (`nft-marketplace`) and workspace dependency references |

---

## Security Notes

- **Royalty enforcement**: Royalties are only enforced by this marketplace contract. Peer-to-peer transfers using the NFT contract directly bypass the royalty split. This is a known limitation of contract-level royalty systems.
- **Auction sniping**: The `end_ledger` is fixed at listing creation. Consider adding a buffer (e.g. extend `end_ledger` if a bid arrives in the last N ledgers) to prevent last-second snipe exploits.
- **Bundle atomicity**: All items in a bundle listing are transferred together during settlement. A failure to transfer any single item reverts the entire transaction.
- **Bid refunds**: Outbid participants must be refunded before recording a new highest bid. Always handle refunds before accepting the new bid to avoid locking funds.
- **Price validation**: `price` and `reserve_price` must be `> 0` to prevent zero-cost acquisition.

---

## Previous Examples

- **[01-basic-nft](../01-basic-nft/)** — Ownership, approval, and transfer patterns
- **[02-nft-metadata](../02-nft-metadata/)** — token_uri and on-chain metadata
- **[03-nft-metadata-standards](../03-nft-metadata-standards/)** — JSON-schema metadata validation
