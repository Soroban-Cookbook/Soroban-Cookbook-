//! # Real-World Case Studies
//!
//! Three self-contained fixes for mistakes that show up repeatedly in
//! production smart contracts. Each case study pairs a vulnerable pattern
//! (documented in the README, not compiled) with the safe pattern
//! implemented here. See the README for the full problem/solution write-up
//! and lessons learned for each case study.
//!
//! 1. Reward claiming that updates state before any external interaction,
//!    so a claim can't be replayed while it is still in flight.
//! 2. Fee calculation that uses checked arithmetic, so large amounts fail
//!    loudly instead of silently wrapping.
//! 3. Commit-reveal bidding, so a bid amount isn't public (and copyable)
//!    before the bidding window closes.

#![no_std]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

const BPS_DENOMINATOR: i128 = 10_000;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    RewardBalance(Address),
    BidCommitment(Address),
    RevealedBid(Address),
    HighestBid,
    HighestBidder,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    Overflow = 5,
    NothingToClaim = 6,
    NoCommitment = 7,
    AlreadyRevealed = 8,
    CommitmentMismatch = 9,
    AlreadyCommitted = 10,
}

const EVENT_CLAIM: Symbol = symbol_short!("claim");
const EVENT_COMMIT: Symbol = symbol_short!("commit");
const EVENT_REVEAL: Symbol = symbol_short!("reveal");

#[contract]
pub struct CaseStudies;

#[contractimpl]
impl CaseStudies {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    // -----------------------------------------------------------------
    // Case study 1: checks-effects-interactions in a reward claim.
    // See "Case Study 1" in the README for the vulnerable version and why
    // it matters.
    // -----------------------------------------------------------------

    /// Admin credits a user's claimable reward balance.
    pub fn fund_reward(env: Env, admin: Address, user: Address, amount: i128) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let key = DataKey::RewardBalance(user);
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_balance = current.checked_add(amount).ok_or(Error::Overflow)?;
        env.storage().persistent().set(&key, &new_balance);
        Ok(())
    }

    /// Claim the caller's full reward balance.
    ///
    /// State is zeroed and the event is published *before* returning the
    /// amount to the caller, so nothing about this call can be re-entered
    /// or replayed to drain the balance twice: by the time any downstream
    /// effect (e.g. a token transfer in a production version) would run,
    /// the contract's own bookkeeping already reflects the withdrawal.
    pub fn claim_reward(env: Env, user: Address) -> Result<i128, Error> {
        user.require_auth();

        let key = DataKey::RewardBalance(user.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            return Err(Error::NothingToClaim);
        }

        // Effect before interaction: zero the balance first.
        env.storage().persistent().set(&key, &0i128);
        env.events().publish((EVENT_CLAIM, user), amount);
        Ok(amount)
    }

    pub fn reward_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::RewardBalance(user))
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------
    // Case study 2: checked arithmetic in a fee calculation.
    // See "Case Study 2" in the README.
    // -----------------------------------------------------------------

    /// Compute a fee in basis points using checked arithmetic, so an
    /// amount large enough to overflow the intermediate product fails
    /// with `Error::Overflow` instead of silently wrapping to a bogus fee.
    pub fn calculate_fee(amount: i128, fee_bps: u32) -> Result<i128, Error> {
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }
        if fee_bps as i128 > BPS_DENOMINATOR {
            return Err(Error::InvalidAmount);
        }

        amount
            .checked_mul(fee_bps as i128)
            .and_then(|v| v.checked_div(BPS_DENOMINATOR))
            .ok_or(Error::Overflow)
    }

    // -----------------------------------------------------------------
    // Case study 3: commit-reveal bidding to prevent front-running.
    // See "Case Study 3" in the README.
    // -----------------------------------------------------------------

    /// Submit `sha256(amount, salt)` without revealing `amount` itself, so
    /// other bidders (and anyone watching the mempool) can't see a bid and
    /// simply outbid it by a minimal margin before it lands.
    pub fn commit_bid(env: Env, bidder: Address, commitment: BytesN<32>) -> Result<(), Error> {
        bidder.require_auth();
        let key = DataKey::BidCommitment(bidder.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyCommitted);
        }
        env.storage().persistent().set(&key, &commitment);
        env.events().publish((EVENT_COMMIT, bidder), ());
        Ok(())
    }

    /// Reveal the amount and salt behind a prior commitment. Rejects the
    /// reveal unless it hashes to the exact commitment that was submitted,
    /// and updates the highest bid if this one wins.
    pub fn reveal_bid(
        env: Env,
        bidder: Address,
        amount: i128,
        salt: BytesN<32>,
    ) -> Result<(), Error> {
        bidder.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let commitment_key = DataKey::BidCommitment(bidder.clone());
        let commitment: BytesN<32> = env
            .storage()
            .persistent()
            .get(&commitment_key)
            .ok_or(Error::NoCommitment)?;

        let revealed_key = DataKey::RevealedBid(bidder.clone());
        if env.storage().persistent().has(&revealed_key) {
            return Err(Error::AlreadyRevealed);
        }

        let expected = env.crypto().sha256(&(amount, salt).to_xdr(&env)).to_bytes();
        if expected != commitment {
            return Err(Error::CommitmentMismatch);
        }

        env.storage().persistent().set(&revealed_key, &amount);

        let highest_bid: i128 = env
            .storage()
            .instance()
            .get(&DataKey::HighestBid)
            .unwrap_or(0);
        if amount > highest_bid {
            env.storage().instance().set(&DataKey::HighestBid, &amount);
            env.storage()
                .instance()
                .set(&DataKey::HighestBidder, &bidder.clone());
        }

        env.events().publish((EVENT_REVEAL, bidder), amount);
        Ok(())
    }

    pub fn highest_bid(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::HighestBid)
            .unwrap_or(0)
    }

    pub fn highest_bidder(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::HighestBidder)
    }
}

fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)?;
    if stored_admin != *admin {
        return Err(Error::Unauthorized);
    }
    admin.require_auth();
    Ok(())
}

#[cfg(test)]
mod test;
