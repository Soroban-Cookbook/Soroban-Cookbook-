#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingItem {
    pub nft_contract: Address,
    pub token_id: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    pub seller: Address,
    pub items: Vec<ListingItem>,
    pub is_auction: bool,
    pub price: i128,
    pub reserve_price: i128,
    pub end_ledger: u32,
    pub royalty_recipient: Address,
    pub royalty_bps: u32,
    pub sold: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeRecord {
    pub buyer: Address,
    pub seller: Address,
    pub items: Vec<ListingItem>,
    pub amount: i128,
    pub royalty_paid: i128,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PaymentToken,
    ListingCount,
    Listing(u32),
    HighestBid(u32),
    TradeCount,
    Trade(u32),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MarketplaceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidPrice = 4,
    ListingNotFound = 5,
    TradeNotFound = 6,
    IncorrectListingType = 7,
    AlreadySold = 8,
    AuctionNotActive = 9,
    BidTooLow = 10,
    NoBids = 11,
    ListingExpired = 12,
    InvalidRoyalty = 13,
}

#[contract]
pub struct NftMarketplaceContract;

// ---------------------------------------------------------------------------
// Internal helper — retrieves a token Client for the configured payment token.
// ---------------------------------------------------------------------------
fn payment_token_client(env: &Env) -> token::Client {
    let token_addr: Address = env
        .storage()
        .instance()
        .get(&DataKey::PaymentToken)
        .expect("marketplace not initialized");
    token::Client::new(env, &token_addr)
}

#[contractimpl]
impl NftMarketplaceContract {
    /// One-time initializer.  `payment_token` is the SEP-41 token address used
    /// for all bids and purchases (e.g. a stablecoin or wrapped-XLM contract).
    pub fn initialize(
        env: Env,
        admin: Address,
        payment_token: Address,
    ) -> Result<(), MarketplaceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MarketplaceError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PaymentToken, &payment_token);
        env.storage().instance().set(&DataKey::ListingCount, &0u32);
        env.storage().instance().set(&DataKey::TradeCount, &0u32);
        env.events().publish(
            (symbol_short!("init"), symbol_short!("mkt")),
            (admin, payment_token),
        );
        Ok(())
    }

    pub fn create_fixed_price_listing(
        env: Env,
        seller: Address,
        items: Vec<ListingItem>,
        price: i128,
        royalty_recipient: Address,
        royalty_bps: u32,
    ) -> Result<u32, MarketplaceError> {
        seller.require_auth();
        if items.is_empty() || price <= 0 {
            return Err(MarketplaceError::InvalidPrice);
        }
        if royalty_bps > 1000 {
            return Err(MarketplaceError::InvalidRoyalty);
        }

        let listing_id = env
            .storage()
            .instance()
            .get(&DataKey::ListingCount)
            .unwrap_or(0);
        let listing = Listing {
            seller: seller.clone(),
            items: items.clone(),
            is_auction: false,
            price,
            reserve_price: price,
            end_ledger: 0,
            royalty_recipient: royalty_recipient.clone(),
            royalty_bps,
            sold: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);
        env.storage()
            .instance()
            .set(&DataKey::ListingCount, &(listing_id + 1));
        env.events().publish(
            (symbol_short!("listing"), symbol_short!("fixed")),
            (seller, listing_id, price, royalty_recipient, royalty_bps),
        );
        Ok(listing_id)
    }

    pub fn create_auction_listing(
        env: Env,
        seller: Address,
        items: Vec<ListingItem>,
        reserve_price: i128,
        duration: u32,
        royalty_recipient: Address,
        royalty_bps: u32,
    ) -> Result<u32, MarketplaceError> {
        seller.require_auth();
        if items.is_empty() || reserve_price <= 0 || duration == 0 {
            return Err(MarketplaceError::InvalidPrice);
        }
        if royalty_bps > 1000 {
            return Err(MarketplaceError::InvalidRoyalty);
        }

        let start_ledger = env.ledger().sequence();
        let end_ledger = start_ledger + duration;
        let listing_id = env
            .storage()
            .instance()
            .get(&DataKey::ListingCount)
            .unwrap_or(0);

        let listing = Listing {
            seller: seller.clone(),
            items: items.clone(),
            is_auction: true,
            price: reserve_price,
            reserve_price,
            end_ledger,
            royalty_recipient: royalty_recipient.clone(),
            royalty_bps,
            sold: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);
        env.storage()
            .instance()
            .set(&DataKey::ListingCount, &(listing_id + 1));
        env.events().publish(
            (symbol_short!("listing"), symbol_short!("auction")),
            (
                seller,
                listing_id,
                reserve_price,
                end_ledger,
                royalty_recipient,
                royalty_bps,
            ),
        );
        Ok(listing_id)
    }

    /// Place a bid on an active auction.
    ///
    /// The bid amount is transferred from `bidder` into this contract for
    /// escrow.  If a previous highest bid exists it is refunded to that bidder
    /// before the new bid is recorded, so funds are never double-locked.
    pub fn place_bid(
        env: Env,
        bidder: Address,
        listing_id: u32,
        bid_amount: i128,
    ) -> Result<(), MarketplaceError> {
        bidder.require_auth();

        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(MarketplaceError::ListingNotFound)?;
        if !listing.is_auction {
            return Err(MarketplaceError::IncorrectListingType);
        }
        if listing.sold {
            return Err(MarketplaceError::AlreadySold);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger > listing.end_ledger {
            return Err(MarketplaceError::ListingExpired);
        }

        let current_bid: Option<Bid> = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBid(listing_id));
        let minimum = match current_bid.clone() {
            Some(ref bid) => bid.amount + 1,
            None => listing.reserve_price,
        };

        if bid_amount < minimum {
            return Err(MarketplaceError::BidTooLow);
        }

        let token = payment_token_client(&env);
        let contract_addr = env.current_contract_address();

        // 1. Refund the outbid participant before accepting the new bid.
        //    This ordering ensures we never leave the previous bidder's funds
        //    locked if the refund transfer were to fail for any reason.
        if let Some(prev) = current_bid {
            token.transfer(&contract_addr, &prev.bidder, &prev.amount);
        }

        // 2. Escrow the new bid inside this contract.
        token.transfer_from(
            &contract_addr, // spender (uses the bidder's approval)
            &bidder,
            &contract_addr,
            &bid_amount,
        );

        let bid = Bid {
            bidder: bidder.clone(),
            amount: bid_amount,
        };
        env.storage()
            .persistent()
            .set(&DataKey::HighestBid(listing_id), &bid);
        env.events().publish(
            (symbol_short!("bid"), symbol_short!("auction")),
            (bidder, listing_id, bid_amount),
        );
        Ok(())
    }

    /// Buy a fixed-price listing.
    ///
    /// Transfers `offer_amount` from `buyer` into this contract and immediately
    /// disburses proceeds to the seller and royalty recipient via `settle_sale`.
    pub fn buy(
        env: Env,
        buyer: Address,
        listing_id: u32,
        offer_amount: i128,
    ) -> Result<(), MarketplaceError> {
        buyer.require_auth();

        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(MarketplaceError::ListingNotFound)?;
        if listing.is_auction {
            return Err(MarketplaceError::IncorrectListingType);
        }
        if listing.sold {
            return Err(MarketplaceError::AlreadySold);
        }
        if offer_amount != listing.price {
            return Err(MarketplaceError::InvalidPrice);
        }

        // Pull payment from buyer into escrow, then settle immediately.
        let token = payment_token_client(&env);
        let contract_addr = env.current_contract_address();
        token.transfer_from(&contract_addr, &buyer, &contract_addr, &offer_amount);

        Self::settle_sale(&env, &listing, buyer, offer_amount, listing_id)
    }

    /// Finalize an auction after its `end_ledger` has passed.
    ///
    /// The winning bid is already held in escrow; this function disburses it
    /// to the seller and royalty recipient.  Anyone may call this entry-point.
    pub fn finalize_auction(
        env: Env,
        _executor: Address,
        listing_id: u32,
    ) -> Result<(), MarketplaceError> {
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(MarketplaceError::ListingNotFound)?;
        if !listing.is_auction {
            return Err(MarketplaceError::IncorrectListingType);
        }
        if listing.sold {
            return Err(MarketplaceError::AlreadySold);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger <= listing.end_ledger {
            return Err(MarketplaceError::AuctionNotActive);
        }

        let highest_bid: Bid = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBid(listing_id))
            .ok_or(MarketplaceError::NoBids)?;

        Self::settle_sale(
            &env,
            &listing,
            highest_bid.bidder,
            highest_bid.amount,
            listing_id,
        )
    }

    pub fn get_listing(env: Env, listing_id: u32) -> Result<Listing, MarketplaceError> {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(MarketplaceError::ListingNotFound)
    }

    pub fn get_trade(env: Env, trade_id: u32) -> Result<TradeRecord, MarketplaceError> {
        env.storage()
            .persistent()
            .get(&DataKey::Trade(trade_id))
            .ok_or(MarketplaceError::TradeNotFound)
    }

    pub fn trade_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TradeCount)
            .unwrap_or(0)
    }

    /// Internal settlement: disburse escrowed funds and record the trade.
    ///
    /// At this point the `amount` in payment tokens already sits in this
    /// contract (transferred by `buy` or escrowed by `place_bid`).
    fn settle_sale(
        env: &Env,
        listing: &Listing,
        buyer: Address,
        amount: i128,
        listing_id: u32,
    ) -> Result<(), MarketplaceError> {
        let royalty_paid = amount * (listing.royalty_bps as i128) / 10000;
        let seller_proceeds = amount - royalty_paid;

        let token = payment_token_client(env);
        let contract_addr = env.current_contract_address();

        // Royalty first, then seller proceeds.
        if royalty_paid > 0 {
            token.transfer(&contract_addr, &listing.royalty_recipient, &royalty_paid);
        }
        token.transfer(&contract_addr, &listing.seller, &seller_proceeds);

        let trade = TradeRecord {
            buyer: buyer.clone(),
            seller: listing.seller.clone(),
            items: listing.items.clone(),
            amount,
            royalty_paid,
            ledger: env.ledger().sequence(),
        };

        let trade_id = env
            .storage()
            .instance()
            .get(&DataKey::TradeCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Trade(trade_id), &trade);
        env.storage()
            .instance()
            .set(&DataKey::TradeCount, &(trade_id + 1));

        let mut completed_listing = listing.clone();
        completed_listing.sold = true;
        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &completed_listing);

        env.events().publish(
            (symbol_short!("trade"), symbol_short!("mkt")),
            (
                buyer,
                listing.seller.clone(),
                amount,
                royalty_paid,
                listing_id,
            ),
        );

        Ok(())
    }
}

#[cfg(test)]
mod test;
