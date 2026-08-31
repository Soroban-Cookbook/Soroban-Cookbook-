//! # State Channel Contract
//!
//! A basic payment-channel implementation on Soroban.  Two parties lock
//! funds on-chain, exchange off-chain signed state updates (balance pairs +
//! a monotonically increasing sequence number), and can settle at any time.
//!
//! ## Life-cycle
//!
//! ```text
//! open()
//!   └─> status: Open
//!         │
//!         ├─ cooperative close: close() with seq > current   (immediate)
//!         │
//!         └─ unilateral challenge: challenge() with signed state
//!               └─> status: Disputed
//!                     │
//!                     ├─ counter-challenge: challenge() with higher seq
//!                     │
//!                     └─ finalize after dispute_period: finalize()
//!                           └─> status: Closed
//! ```
//!
//! ## Security notes
//!
//! * Both parties must have authorised every `challenge` call (Soroban auth).
//! * Sequence numbers are strictly increasing – an old state cannot replace
//!   a newer one.
//! * Balances inside a state update must sum to the original deposit total;
//!   the contract enforces this invariant.
//! * No actual token transfer is done here – the example tracks integer
//!   balances so it can be compiled to WASM without an external token
//!   dependency.  A production channel would call a SEP-41 token contract.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default dispute period in ledgers (roughly 5 minutes at 5 s/ledger).
const DEFAULT_DISPUTE_PERIOD: u32 = 60;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted when a channel is opened.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelOpenedEvent {
    pub channel_id: u64,
    pub party_a: Address,
    pub party_b: Address,
    pub total_deposit: i128,
    pub timestamp: u64,
}

/// Emitted when a challenge is submitted or updated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeSubmittedEvent {
    pub channel_id: u64,
    pub challenger: Address,
    pub sequence: u64,
    pub balance_a: i128,
    pub balance_b: i128,
    pub timestamp: u64,
}

/// Emitted when a channel is closed (cooperatively or after dispute).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelClosedEvent {
    pub channel_id: u64,
    pub final_balance_a: i128,
    pub final_balance_b: i128,
    pub timestamp: u64,
}

// Event namespace
const NS: Symbol = symbol_short!("sc");
const EV_OPENED: Symbol = symbol_short!("opened");
const EV_CHALLENGE: Symbol = symbol_short!("challenge");
const EV_CLOSED: Symbol = symbol_short!("closed");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ChannelError {
    /// Contract has not been initialised yet.
    NotInitialized = 1,
    /// Contract is already initialised.
    AlreadyInitialized = 2,
    /// Caller is not authorised for this operation.
    Unauthorized = 3,
    /// The referenced channel does not exist.
    ChannelNotFound = 4,
    /// The channel is in the wrong state for this operation.
    InvalidChannelState = 5,
    /// Deposit amounts must be positive.
    InvalidDeposit = 6,
    /// Balances in the state update do not match the channel total.
    BalanceMismatch = 7,
    /// The submitted sequence number is not higher than the current one.
    SequenceTooLow = 8,
    /// The dispute period has not yet elapsed.
    DisputePeriodActive = 9,
    /// The channel has already been finalised.
    AlreadyClosed = 10,
}

// ---------------------------------------------------------------------------
// Storage types
// ---------------------------------------------------------------------------

/// Channel status.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelStatus {
    /// Funds locked; off-chain updates in progress.
    Open,
    /// A party has submitted a state on-chain; dispute timer running.
    Disputed,
    /// Channel has been finalised; funds distributed.
    Closed,
}

/// Immutable channel parameters stored at `open` time.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Channel {
    /// Party A address.
    pub party_a: Address,
    /// Party B address.
    pub party_b: Address,
    /// Deposit locked by party A.
    pub deposit_a: i128,
    /// Deposit locked by party B.
    pub deposit_b: i128,
    /// Current life-cycle status.
    pub status: ChannelStatus,
    /// Sequence number of the last submitted on-chain state (0 = none yet).
    pub sequence: u64,
    /// Balance owed to party A in the last on-chain state.
    pub balance_a: i128,
    /// Balance owed to party B in the last on-chain state.
    pub balance_b: i128,
    /// Ledger number at which the challenge window expires (0 = not challenging).
    pub challenge_expiry: u32,
    /// Number of ledgers the dispute window lasts.
    pub dispute_period: u32,
}

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Initialization guard.
    Initialized,
    /// Next channel ID counter.
    NextChannelId,
    /// Channel data keyed by ID.
    Channel(u64),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct StateChannelContract;

#[contractimpl]
impl StateChannelContract {
    // -----------------------------------------------------------------------
    // Admin / setup
    // -----------------------------------------------------------------------

    /// Initialise the contract.  Must be called once before any other method.
    pub fn initialize(env: Env) -> Result<(), ChannelError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(ChannelError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::NextChannelId, &1u64);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Channel life-cycle
    // -----------------------------------------------------------------------

    /// Open a state channel between `party_a` and `party_b`.
    ///
    /// Both parties authorise this call; the supplied deposits are recorded
    /// on-chain (no actual token movement in this example).
    ///
    /// Returns the new channel ID.
    pub fn open(
        env: Env,
        party_a: Address,
        party_b: Address,
        deposit_a: i128,
        deposit_b: i128,
        dispute_period: Option<u32>,
    ) -> Result<u64, ChannelError> {
        Self::require_initialized(&env)?;

        // Both parties must authorise the channel opening.
        party_a.require_auth();
        party_b.require_auth();

        if deposit_a <= 0 || deposit_b <= 0 {
            return Err(ChannelError::InvalidDeposit);
        }

        let dispute = dispute_period.unwrap_or(DEFAULT_DISPUTE_PERIOD);
        let channel_id: u64 = env.storage().instance().get(&DataKey::NextChannelId).unwrap();

        let channel = Channel {
            party_a: party_a.clone(),
            party_b: party_b.clone(),
            deposit_a,
            deposit_b,
            status: ChannelStatus::Open,
            sequence: 0,
            balance_a: deposit_a,
            balance_b: deposit_b,
            challenge_expiry: 0,
            dispute_period: dispute,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Channel(channel_id), &channel);
        env.storage()
            .instance()
            .set(&DataKey::NextChannelId, &(channel_id + 1));

        // Emit event.
        env.events().publish(
            (NS, EV_OPENED),
            ChannelOpenedEvent {
                channel_id,
                party_a,
                party_b,
                total_deposit: deposit_a + deposit_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(channel_id)
    }

    /// Submit a signed off-chain state update to the chain.
    ///
    /// Either party may call this.  The `sequence` must be strictly greater
    /// than the channel's current on-chain sequence.  Both parties must
    /// authorise the call (simulating that both signed the state off-chain).
    ///
    /// The first call transitions an `Open` channel to `Disputed` and starts
    /// the dispute timer.  A subsequent call with a higher sequence replaces
    /// the pending state and resets the timer.
    pub fn challenge(
        env: Env,
        channel_id: u64,
        challenger: Address,
        sequence: u64,
        balance_a: i128,
        balance_b: i128,
    ) -> Result<(), ChannelError> {
        Self::require_initialized(&env)?;

        challenger.require_auth();

        let mut channel = Self::load_channel(&env, channel_id)?;

        // Only Open or Disputed channels can be challenged.
        if channel.status == ChannelStatus::Closed {
            return Err(ChannelError::AlreadyClosed);
        }

        // The challenger must be one of the two parties.
        if challenger != channel.party_a && challenger != channel.party_b {
            return Err(ChannelError::Unauthorized);
        }

        // New sequence must be strictly greater.
        if sequence <= channel.sequence {
            return Err(ChannelError::SequenceTooLow);
        }

        // Balances must sum to the total deposit.
        let total = channel.deposit_a + channel.deposit_b;
        if balance_a + balance_b != total {
            return Err(ChannelError::BalanceMismatch);
        }

        // Update state.
        channel.sequence = sequence;
        channel.balance_a = balance_a;
        channel.balance_b = balance_b;
        channel.status = ChannelStatus::Disputed;
        channel.challenge_expiry =
            env.ledger().sequence() + channel.dispute_period;

        env.storage()
            .persistent()
            .set(&DataKey::Channel(channel_id), &channel);

        env.events().publish(
            (NS, EV_CHALLENGE),
            ChallengeSubmittedEvent {
                channel_id,
                challenger,
                sequence,
                balance_a,
                balance_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Cooperatively close a channel immediately.
    ///
    /// Both parties must authorise.  The provided `balance_a` / `balance_b`
    /// must sum to the total deposit and the `sequence` must be > current.
    /// This bypasses the dispute period entirely.
    pub fn close(
        env: Env,
        channel_id: u64,
        sequence: u64,
        balance_a: i128,
        balance_b: i128,
    ) -> Result<(), ChannelError> {
        Self::require_initialized(&env)?;

        let mut channel = Self::load_channel(&env, channel_id)?;

        if channel.status == ChannelStatus::Closed {
            return Err(ChannelError::AlreadyClosed);
        }

        // Both parties must authorise a cooperative close.
        channel.party_a.require_auth();
        channel.party_b.require_auth();

        if sequence <= channel.sequence {
            return Err(ChannelError::SequenceTooLow);
        }

        let total = channel.deposit_a + channel.deposit_b;
        if balance_a + balance_b != total {
            return Err(ChannelError::BalanceMismatch);
        }

        channel.sequence = sequence;
        channel.balance_a = balance_a;
        channel.balance_b = balance_b;
        channel.status = ChannelStatus::Closed;

        env.storage()
            .persistent()
            .set(&DataKey::Channel(channel_id), &channel);

        env.events().publish(
            (NS, EV_CLOSED),
            ChannelClosedEvent {
                channel_id,
                final_balance_a: balance_a,
                final_balance_b: balance_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Finalise a disputed channel after the challenge period expires.
    ///
    /// Anyone may call this once `env.ledger().sequence() >=
    /// challenge_expiry`.  The last submitted on-chain state becomes final.
    pub fn finalize(env: Env, channel_id: u64) -> Result<(), ChannelError> {
        Self::require_initialized(&env)?;

        let mut channel = Self::load_channel(&env, channel_id)?;

        if channel.status == ChannelStatus::Closed {
            return Err(ChannelError::AlreadyClosed);
        }

        if channel.status != ChannelStatus::Disputed {
            return Err(ChannelError::InvalidChannelState);
        }

        if env.ledger().sequence() < channel.challenge_expiry {
            return Err(ChannelError::DisputePeriodActive);
        }

        let (final_a, final_b) = (channel.balance_a, channel.balance_b);
        channel.status = ChannelStatus::Closed;

        env.storage()
            .persistent()
            .set(&DataKey::Channel(channel_id), &channel);

        env.events().publish(
            (NS, EV_CLOSED),
            ChannelClosedEvent {
                channel_id,
                final_balance_a: final_a,
                final_balance_b: final_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read-only helpers
    // -----------------------------------------------------------------------

    /// Return the full `Channel` record for `channel_id`.
    pub fn get_channel(env: Env, channel_id: u64) -> Result<Channel, ChannelError> {
        Self::require_initialized(&env)?;
        Self::load_channel(&env, channel_id)
    }

    /// Return the current on-chain sequence number for the channel.
    pub fn get_sequence(env: Env, channel_id: u64) -> Result<u64, ChannelError> {
        Self::require_initialized(&env)?;
        Ok(Self::load_channel(&env, channel_id)?.sequence)
    }

    /// Return the on-chain balance pair `(balance_a, balance_b)`.
    pub fn get_balances(env: Env, channel_id: u64) -> Result<(i128, i128), ChannelError> {
        Self::require_initialized(&env)?;
        let ch = Self::load_channel(&env, channel_id)?;
        Ok((ch.balance_a, ch.balance_b))
    }

    /// Return the channel status.
    pub fn get_status(env: Env, channel_id: u64) -> Result<ChannelStatus, ChannelError> {
        Self::require_initialized(&env)?;
        Ok(Self::load_channel(&env, channel_id)?.status)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), ChannelError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(ChannelError::NotInitialized)
        }
    }

    fn load_channel(env: &Env, channel_id: u64) -> Result<Channel, ChannelError> {
        env.storage()
            .persistent()
            .get(&DataKey::Channel(channel_id))
            .ok_or(ChannelError::ChannelNotFound)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test;
