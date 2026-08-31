#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Namespace for state channel dispute contract events
const CONTRACT_NS: Symbol = symbol_short!("dispute");

/// Action symbols for events
const ACTION_OPEN: Symbol = symbol_short!("open");
const ACTION_CHALLENGE: Symbol = symbol_short!("challenge");
const ACTION_RESPONSE: Symbol = symbol_short!("respond");
const ACTION_FRAUD: Symbol = symbol_short!("fraud");
const ACTION_FINALIZE: Symbol = symbol_short!("finalize");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidParticipant = 3,
    InvalidBalance = 4,
    InvalidDisputeWindow = 5,
    ChannelNotOpen = 6,
    ChannelNotInDispute = 7,
    DisputeAlreadyActive = 8,
    DisputeExpired = 9,
    DisputeNotExpired = 10,
    StaleSequence = 11,
    InvalidBalanceSum = 12,
    InvalidFraudProof = 13,
    ChannelClosed = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelStatus {
    Open,
    Disputed,
    Closed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelState {
    pub participant_a: Address,
    pub participant_b: Address,
    pub balance_a: i128,
    pub balance_b: i128,
    pub total_deposit: i128,
    pub dispute_window: u64,
    pub status: ChannelStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeData {
    pub challenger: Address,
    pub proposer: Address,
    pub sequence: u64,
    pub proposed_balance_a: i128,
    pub proposed_balance_b: i128,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeEventData {
    pub sequence: u64,
    pub balance_a: i128,
    pub balance_b: i128,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Channel,
    Dispute,
}

#[contract]
pub struct StateChannelDisputesContract;

#[contractimpl]
impl StateChannelDisputesContract {
    /// Opens and initializes the payment/state channel between two participants.
    ///
    /// - `participant_a`: Address of party A.
    /// - `participant_b`: Address of party B.
    /// - `balance_a`: Initial deposit for party A (must be >= 0).
    /// - `balance_b`: Initial deposit for party B (must be >= 0).
    /// - `dispute_window`: Window duration in seconds allowed for counter-challenges/responses.
    pub fn open_channel(
        env: Env,
        participant_a: Address,
        participant_b: Address,
        balance_a: i128,
        balance_b: i128,
        dispute_window: u64,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Channel) {
            return Err(Error::AlreadyInitialized);
        }

        if participant_a == participant_b {
            return Err(Error::InvalidParticipant);
        }

        participant_a.require_auth();
        participant_b.require_auth();

        if balance_a < 0 || balance_b < 0 {
            return Err(Error::InvalidBalance);
        }

        let total_deposit = balance_a
            .checked_add(balance_b)
            .ok_or(Error::InvalidBalance)?;

        if total_deposit <= 0 {
            return Err(Error::InvalidBalance);
        }

        if dispute_window == 0 {
            return Err(Error::InvalidDisputeWindow);
        }

        let channel = ChannelState {
            participant_a: participant_a.clone(),
            participant_b: participant_b.clone(),
            balance_a,
            balance_b,
            total_deposit,
            dispute_window,
            status: ChannelStatus::Open,
        };

        env.storage().instance().set(&DataKey::Channel, &channel);

        env.events().publish(
            (CONTRACT_NS, ACTION_OPEN, participant_a),
            DisputeEventData {
                sequence: 0,
                balance_a,
                balance_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Submits a challenge (opens a dispute) with an off-chain signed state.
    ///
    /// - `challenger`: Participant initiating the dispute (must be participant A or B).
    /// - `sequence`: Off-chain state sequence number.
    /// - `proposed_balance_a`: Off-chain balance allocated to party A.
    /// - `proposed_balance_b`: Off-chain balance allocated to party B.
    pub fn open_dispute(
        env: Env,
        challenger: Address,
        sequence: u64,
        proposed_balance_a: i128,
        proposed_balance_b: i128,
    ) -> Result<(), Error> {
        challenger.require_auth();

        let mut channel: ChannelState = env
            .storage()
            .instance()
            .get(&DataKey::Channel)
            .ok_or(Error::NotInitialized)?;

        if channel.status != ChannelStatus::Open {
            return Err(Error::ChannelNotOpen);
        }

        if challenger != channel.participant_a && challenger != channel.participant_b {
            return Err(Error::InvalidParticipant);
        }

        if proposed_balance_a < 0 || proposed_balance_b < 0 {
            return Err(Error::InvalidBalance);
        }

        let proposed_total = proposed_balance_a
            .checked_add(proposed_balance_b)
            .ok_or(Error::InvalidBalance)?;

        if proposed_total != channel.total_deposit {
            return Err(Error::InvalidBalanceSum);
        }

        let now = env.ledger().timestamp();
        let deadline = now
            .checked_add(channel.dispute_window)
            .ok_or(Error::InvalidDisputeWindow)?;

        channel.status = ChannelStatus::Disputed;
        env.storage().instance().set(&DataKey::Channel, &channel);

        let dispute = DisputeData {
            challenger: challenger.clone(),
            proposer: challenger.clone(),
            sequence,
            proposed_balance_a,
            proposed_balance_b,
            deadline,
        };

        env.storage().instance().set(&DataKey::Dispute, &dispute);

        env.events().publish(
            (CONTRACT_NS, ACTION_CHALLENGE, challenger),
            DisputeEventData {
                sequence,
                balance_a: proposed_balance_a,
                balance_b: proposed_balance_b,
                timestamp: now,
            },
        );

        Ok(())
    }

    /// Responds to an active challenge by submitting a newer off-chain state (higher sequence number).
    ///
    /// - `responder`: Participant responding (must be participant A or B).
    /// - `sequence`: New off-chain sequence number (must be > current dispute sequence).
    /// - `proposed_balance_a`: Updated off-chain balance for party A.
    /// - `proposed_balance_b`: Updated off-chain balance for party B.
    pub fn respond_dispute(
        env: Env,
        responder: Address,
        sequence: u64,
        proposed_balance_a: i128,
        proposed_balance_b: i128,
    ) -> Result<(), Error> {
        responder.require_auth();

        let channel: ChannelState = env
            .storage()
            .instance()
            .get(&DataKey::Channel)
            .ok_or(Error::NotInitialized)?;

        if channel.status != ChannelStatus::Disputed {
            return Err(Error::ChannelNotInDispute);
        }

        if responder != channel.participant_a && responder != channel.participant_b {
            return Err(Error::InvalidParticipant);
        }

        let mut dispute: DisputeData = env
            .storage()
            .instance()
            .get(&DataKey::Dispute)
            .ok_or(Error::ChannelNotInDispute)?;

        let now = env.ledger().timestamp();
        if now >= dispute.deadline {
            return Err(Error::DisputeExpired);
        }

        if sequence <= dispute.sequence {
            return Err(Error::StaleSequence);
        }

        if proposed_balance_a < 0 || proposed_balance_b < 0 {
            return Err(Error::InvalidBalance);
        }

        let proposed_total = proposed_balance_a
            .checked_add(proposed_balance_b)
            .ok_or(Error::InvalidBalance)?;

        if proposed_total != channel.total_deposit {
            return Err(Error::InvalidBalanceSum);
        }

        dispute.proposer = responder.clone();
        dispute.sequence = sequence;
        dispute.proposed_balance_a = proposed_balance_a;
        dispute.proposed_balance_b = proposed_balance_b;

        env.storage().instance().set(&DataKey::Dispute, &dispute);

        env.events().publish(
            (CONTRACT_NS, ACTION_RESPONSE, responder),
            DisputeEventData {
                sequence,
                balance_a: proposed_balance_a,
                balance_b: proposed_balance_b,
                timestamp: now,
            },
        );

        Ok(())
    }

    /// Submits a fraud proof to immediately slash/penalize a fraudulent challenger.
    ///
    /// A fraud proof demonstrates an invalid state (e.g. invalid balance invariant or revoked state proof).
    /// In this educational model, if a challenger submitted a state violating channel rules (or invalid proof),
    /// the victim receives 100% of the channel total deposit.
    ///
    /// - `submitter`: Honest participant submitting the fraud proof.
    /// - `invalid_sequence`: The sequence number being proven fraudulent.
    pub fn submit_fraud_proof(
        env: Env,
        submitter: Address,
        invalid_sequence: u64,
    ) -> Result<(), Error> {
        submitter.require_auth();

        let mut channel: ChannelState = env
            .storage()
            .instance()
            .get(&DataKey::Channel)
            .ok_or(Error::NotInitialized)?;

        if channel.status != ChannelStatus::Disputed {
            return Err(Error::ChannelNotInDispute);
        }

        if submitter != channel.participant_a && submitter != channel.participant_b {
            return Err(Error::InvalidParticipant);
        }

        let dispute: DisputeData = env
            .storage()
            .instance()
            .get(&DataKey::Dispute)
            .ok_or(Error::ChannelNotInDispute)?;

        // A proposer cannot submit a fraud proof against their own proposed state
        if submitter == dispute.proposer {
            return Err(Error::InvalidFraudProof);
        }

        // Verify the fraud proof target matches the disputed sequence
        if dispute.sequence != invalid_sequence {
            return Err(Error::InvalidFraudProof);
        }

        // Fraud detected: Challenger tried to cheat with an invalid/revoked state.
        // Penalty: Entire total deposit goes to the honest submitter/victim.
        if submitter == channel.participant_a {
            channel.balance_a = channel.total_deposit;
            channel.balance_b = 0;
        } else {
            channel.balance_a = 0;
            channel.balance_b = channel.total_deposit;
        }

        channel.status = ChannelStatus::Closed;
        env.storage().instance().set(&DataKey::Channel, &channel);
        env.storage().instance().remove(&DataKey::Dispute);

        env.events().publish(
            (CONTRACT_NS, ACTION_FRAUD, submitter),
            DisputeEventData {
                sequence: invalid_sequence,
                balance_a: channel.balance_a,
                balance_b: channel.balance_b,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Finalizes the dispute after the dispute deadline has passed.
    ///
    /// Sets channel balances to the latest valid agreed state and closes the channel.
    pub fn finalize_dispute(env: Env) -> Result<(), Error> {
        let mut channel: ChannelState = env
            .storage()
            .instance()
            .get(&DataKey::Channel)
            .ok_or(Error::NotInitialized)?;

        if channel.status != ChannelStatus::Disputed {
            return Err(Error::ChannelNotInDispute);
        }

        let dispute: DisputeData = env
            .storage()
            .instance()
            .get(&DataKey::Dispute)
            .ok_or(Error::ChannelNotInDispute)?;

        let now = env.ledger().timestamp();
        if now < dispute.deadline {
            return Err(Error::DisputeNotExpired);
        }

        channel.balance_a = dispute.proposed_balance_a;
        channel.balance_b = dispute.proposed_balance_b;
        channel.status = ChannelStatus::Closed;

        env.storage().instance().set(&DataKey::Channel, &channel);
        env.storage().instance().remove(&DataKey::Dispute);

        env.events().publish(
            (CONTRACT_NS, ACTION_FINALIZE, channel.participant_a.clone()),
            DisputeEventData {
                sequence: dispute.sequence,
                balance_a: channel.balance_a,
                balance_b: channel.balance_b,
                timestamp: now,
            },
        );

        Ok(())
    }

    /// Fetches the current channel state.
    pub fn get_channel(env: Env) -> Result<ChannelState, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Channel)
            .ok_or(Error::NotInitialized)
    }

    /// Fetches the current dispute data (if active).
    pub fn get_dispute(env: Env) -> Result<DisputeData, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Dispute)
            .ok_or(Error::ChannelNotInDispute)
    }
}

#[cfg(test)]
mod test;
