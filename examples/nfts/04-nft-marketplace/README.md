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
| Competitive bidding | `place_bid` — new bid must exceed the current highest bid; previous bid is refunded |
| Bid escrow | `place_bid` transfers the bid amount into the marketplace contract immediately |
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

### Payment Token (escrow model)

The marketplace uses a single SEP-41 token for all payments, configured at `initialize`:

```rust
fn initialize(env: Env, admin: Address, payment_token: Address) -> Result<(), MarketplaceError>
```

Before calling `buy` or `place_bid`, the buyer/bidder must call `approve` on the payment token contract to allow the marketplace to pull funds:

```
token_contract.approve(&bidder, &marketplace_contract, &bid_amount, &expiry_ledger);
marketplace_contract.place_bid(&bidder, &listing_id, &bid_amount);
```

### Bid Escrow Flow

`place_bid` enforces binding bids by locking the token amount immediately:

1. Validate the bid meets the reserve/current-minimum.
2. **Refund** the previous highest bidder (if any) back from contract escrow.
3. **Transfer** the new bid amount from the bidder into the contract via `transfer_from`.
4. Record the new highest bid in persistent storage.

```
bidder ──transfer_from──► marketplace_contract  (new bid locked)
marketplace_contract ──transfer──► prev_bidder  (previous bid returned)
```

### Settlement Flow

When a sale completes (either via `buy` or `finalize_auction`), `settle_sale` disburses
the escrowed funds from the contract to the recipients:

```
marketplace_contract ──transfer──► royalty_recipient  (royalty_paid)
marketplace_contract ──transfer──► seller             (amount − royalty_paid)
```

Fixed-price buys pull from buyer → contract → seller in the same transaction:

```
buyer ──transfer_from──► marketplace_contract ──transfer──► seller/royalty
```

### Royalty Calculation

Royalties are calculated from the sale price and split before paying the seller:

```rust
let royalty_paid = (amount * royalty_bps as i128) / 10_000;
let seller_proceeds = amount - royalty_paid;
// transfer royalty_paid → royalty_recipient
// transfer seller_proceeds → seller
```

Royalties are bounded to `royalty_bps <= 1_000` (10%) at listing creation.

### Event Emission (`04-events` pattern)

Every state transition emits a structured event. Topics follow `(action, "market")` so indexers can reconstruct the full lifecycle of any listing:

```rust
// New listing created
env.events().publish(
    (symbol_short!("listing"), symbol_short!("fixed"), listing_id),
    (seller, price),
);

// Bid placed on auction
env.events().publish(
    (symbol_short!("bid"), symbol_short!("auction"), listing_id),
    (bidder, amount),
);

// Sale completed (fixed-price or auction)
env.events().publish(
    (symbol_short!("trade"), symbol_short!("mkt"), listing_id),
    (buyer, seller, amount, royalty_paid),
);
```

### Storage Layout

| Key | Tier | Rationale |
|-----|------|-----------|
| `Admin`, `PaymentToken`, `ListingCount`, `TradeCount` | `Instance` | Global state; read on every operation |
| `Listing(id)`, `HighestBid(id)` | `Persistent` | Active listings and bids must survive ledger gaps |
| `Trade(id)` | `Persistent` | Trade history is permanent; should not expire |

---

## Contract Interface

```rust
fn initialize(env, admin: Address, payment_token: Address) -> Result<(), MarketplaceError>

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
| `src/test.rs` | Tests for fixed-price buys, auction flows, royalty splits, bid escrow, refunds |
| `Cargo.toml` | Crate metadata (`nft-marketplace`) and workspace dependency references |

---

## Security Notes

- **Payment token approval required**: Buyers and bidders must call `approve` on the payment token contract before interacting with the marketplace. The marketplace uses `transfer_from` to pull funds, which requires a prior allowance.
- **Royalty enforcement**: Royalties are only enforced by this marketplace contract. Peer-to-peer transfers using the NFT contract directly bypass the royalty split. This is a known limitation of contract-level royalty systems.
- **Auction sniping**: The `end_ledger` is fixed at listing creation. Consider adding a buffer (e.g. extend `end_ledger` if a bid arrives in the last N ledgers) to prevent last-second snipe exploits.
- **Bundle atomicity**: All items in a bundle listing are transferred together during settlement. A failure to transfer any single item reverts the entire transaction.
- **Price validation**: `price` and `reserve_price` must be `> 0` to prevent zero-cost acquisition.
- **Single payment token**: The marketplace is initialized with one payment token and cannot be changed. Deploy separate instances for different currencies.

---

## Previous Examples

- **[01-basic-nft](../01-basic-nft/)** — Ownership, approval, and transfer patterns
- **[02-nft-metadata](../02-nft-metadata/)** — token_uri and on-chain metadata
- **[03-nft-metadata-standards](../03-nft-metadata-standards/)** — JSON-schema metadata validation
