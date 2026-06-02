# NFT Marketplace

A Soroban marketplace example demonstrating advanced listing features for NFTs.

## Features

- Fixed-price listing support
- Auction listings with competitive bidding
- Bundle sales for multiple NFTs in a single listing
- Royalty payments on every executed sale
- Trade history recording for settled listings

## Usage

- `initialize(admin)`
- `create_fixed_price_listing(seller, items, price, royalty_recipient, royalty_bps)`
- `create_auction_listing(seller, items, reserve_price, duration, royalty_recipient, royalty_bps)`
- `place_bid(bidder, listing_id, bid_amount)`
- `buy(buyer, listing_id, offer_amount)`
- `finalize_auction(executor, listing_id)`
- `get_listing(listing_id)`
- `get_trade(trade_id)`

## Running tests

```bash
cargo test -p nft-marketplace
```
