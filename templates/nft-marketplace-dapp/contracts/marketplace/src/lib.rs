#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTItem {
    pub owner: Address,
    pub uri: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    pub seller: Address,
    pub token_id: u32,
    pub price: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NextTokenId,
    NFT(u32),
    Listing(u32),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MarketplaceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NFTNotFound = 4,
    NotListed = 5,
    AlreadyListed = 6,
    InvalidPrice = 7,
    PaymentFailed = 8,
    ArithmeticOverflow = 9,
}

const INSTANCE_BUMP_AMOUNT: u32 = 518_400;
const INSTANCE_LIFETIME_THRESHOLD: u32 = 120_960;

const STORAGE_BUMP_AMOUNT: u32 = 518_400;
const STORAGE_LIFETIME_THRESHOLD: u32 = 120_960;

#[contract]
pub struct NFTMarketplaceContract;

#[contractimpl]
impl NFTMarketplaceContract {
    /// Initialize marketplace
    pub fn initialize(env: Env, admin: Address) -> Result<(), MarketplaceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MarketplaceError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextTokenId, &1u32);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish((symbol_short!("init"), admin), ());
        Ok(())
    }

    /// Mint a new NFT to recipient
    pub fn mint(env: Env, creator: Address, recipient: Address, uri: String) -> Result<u32, MarketplaceError> {
        creator.require_auth();

        let token_id: u32 = env.storage().instance().get(&DataKey::NextTokenId).unwrap_or(1);
        let next_id = token_id.checked_add(1).ok_or(MarketplaceError::ArithmeticOverflow)?;
        env.storage().instance().set(&DataKey::NextTokenId, &next_id);

        let item = NFTItem {
            owner: recipient.clone(),
            uri: uri.clone(),
        };

        let nft_key = DataKey::NFT(token_id);
        env.storage().persistent().set(&nft_key, &item);
        env.storage().persistent().extend_ttl(&nft_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("mint_nft"), recipient),
            (token_id, uri),
        );

        Ok(token_id)
    }

    /// List an owned NFT for sale at a fixed price
    pub fn list_item(env: Env, seller: Address, token_id: u32, price: i128) -> Result<(), MarketplaceError> {
        seller.require_auth();

        if price <= 0 {
            return Err(MarketplaceError::InvalidPrice);
        }

        let nft_key = DataKey::NFT(token_id);
        let mut nft: NFTItem = env.storage().persistent().get(&nft_key).ok_or(MarketplaceError::NFTNotFound)?;

        if nft.owner != seller {
            return Err(MarketplaceError::Unauthorized);
        }

        let listing_key = DataKey::Listing(token_id);
        if env.storage().persistent().has(&listing_key) {
            return Err(MarketplaceError::AlreadyListed);
        }

        // Lock NFT under marketplace contract during active listing
        nft.owner = env.current_contract_address();
        env.storage().persistent().set(&nft_key, &nft);

        let listing = Listing {
            seller: seller.clone(),
            token_id,
            price,
        };
        env.storage().persistent().set(&listing_key, &listing);

        env.storage().persistent().extend_ttl(&listing_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().persistent().extend_ttl(&nft_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("list_nft"), seller),
            (token_id, price),
        );

        Ok(())
    }

    /// Cancel an active listing and return NFT to seller
    pub fn cancel_listing(env: Env, seller: Address, token_id: u32) -> Result<(), MarketplaceError> {
        seller.require_auth();

        let listing_key = DataKey::Listing(token_id);
        let listing: Listing = env.storage().persistent().get(&listing_key).ok_or(MarketplaceError::NotListed)?;

        if listing.seller != seller {
            return Err(MarketplaceError::Unauthorized);
        }

        let nft_key = DataKey::NFT(token_id);
        let mut nft: NFTItem = env.storage().persistent().get(&nft_key).ok_or(MarketplaceError::NFTNotFound)?;

        nft.owner = seller.clone();
        env.storage().persistent().set(&nft_key, &nft);
        env.storage().persistent().remove(&listing_key);

        env.storage().persistent().extend_ttl(&nft_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("cancel"), seller),
            token_id,
        );

        Ok(())
    }

    /// Buy a listed NFT
    pub fn buy_item(env: Env, buyer: Address, token_id: u32) -> Result<(), MarketplaceError> {
        buyer.require_auth();

        let listing_key = DataKey::Listing(token_id);
        let listing: Listing = env.storage().persistent().get(&listing_key).ok_or(MarketplaceError::NotListed)?;

        let nft_key = DataKey::NFT(token_id);
        let mut nft: NFTItem = env.storage().persistent().get(&nft_key).ok_or(MarketplaceError::NFTNotFound)?;

        // Transfer NFT ownership to buyer
        nft.owner = buyer.clone();
        env.storage().persistent().set(&nft_key, &nft);
        env.storage().persistent().remove(&listing_key);

        env.storage().persistent().extend_ttl(&nft_key, STORAGE_LIFETIME_THRESHOLD, STORAGE_BUMP_AMOUNT);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(
            (symbol_short!("buy_nft"), buyer, listing.seller),
            (token_id, listing.price),
        );

        Ok(())
    }

    /// Get NFT details
    pub fn get_nft(env: Env, token_id: u32) -> Result<NFTItem, MarketplaceError> {
        let nft_key = DataKey::NFT(token_id);
        env.storage().persistent().get(&nft_key).ok_or(MarketplaceError::NFTNotFound)
    }

    /// Get Listing details
    pub fn get_listing(env: Env, token_id: u32) -> Result<Listing, MarketplaceError> {
        let listing_key = DataKey::Listing(token_id);
        env.storage().persistent().get(&listing_key).ok_or(MarketplaceError::NotListed)
    }
}

#[cfg(test)]
mod test;
