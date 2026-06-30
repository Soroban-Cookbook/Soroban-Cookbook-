//! # DAO with Treasury
//!
//! A full-featured Decentralised Autonomous Organisation (DAO) with an
//! integrated Treasury, implementing:
//!
//! - **Treasury management** – deposit and withdraw native token balances
//!   held by the DAO contract itself.
//! - **Proposal lifecycle** – `Submit → Vote → Execute` with ledger-based
//!   voting windows and an execution queue.
//! - **Two proposal types**
//!   - [`ProposalKind::Transfer`] – disburse assets from the treasury to a
//!     recipient.
//!   - [`ProposalKind::Upgrade`] – replace the contract WASM (governance-
//!     controlled upgrades).
//! - **Weighted / token-based voting** – each voter supplies a `weight`
//!   (representing their governance-token balance).
//! - **Auth pattern** – reuses `03-authentication`: every state-mutating
//!   function calls `address.require_auth()` before touching storage.
//! - **Event pattern** – reuses `04-events`: structured topics + typed data
//!   payloads so off-chain indexers can filter efficiently.
//!
//! ## Storage layout
//!
//! | Key                        | Tier       | Value            |
//! |----------------------------|------------|------------------|
//! | `DataKey::Admin`           | Instance   | `Address`        |
//! | `DataKey::MinQuorum`       | Instance   | `i128`           |
//! | `DataKey::VotingDuration`  | Instance   | `u32` (ledgers)  |
//! | `DataKey::ExecDuration`    | Instance   | `u32` (ledgers)  |
//! | `DataKey::ProposalCount`   | Instance   | `u32`            |
//! | `DataKey::TreasuryBalance` | Persistent | `i128`           |
//! | `DataKey::Proposal(id)`    | Persistent | `Proposal`       |
//! | `DataKey::Voted(id, addr)` | Persistent | `bool`           |
//!
//! ## Event topics convention
//!
//! ```text
//! topic[0]  – contract namespace: "dao"
//! topic[1]  – action name       : "propose" | "vote" | "execute" | …
//! topic[2]  – primary key       : proposal_id or actor address
//! topic[3]  – secondary key     : (optional)
//! data      – typed payload struct (non-indexed)
//! ```

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
};

// ---------------------------------------------------------------------------
// Proposal kind
// ---------------------------------------------------------------------------

/// Discriminant identifying the proposal action type.
///
/// We use a simple u32 enum to tag the kind; the actual parameters are
/// stored directly on the [`Proposal`] struct as `Option` fields to keep
/// everything in a single persistent storage slot and avoid the
/// `contracttype` limitations with enum struct-variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalKind {
    /// Disburse assets from the treasury to a recipient.
    Transfer = 1,
    /// Replace the contract WASM with a new hash (Soroban upgrade).
    Upgrade = 2,
}

// ---------------------------------------------------------------------------
// Proposal state
// ---------------------------------------------------------------------------

/// Life-cycle states of a proposal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalState {
    /// Voting window is open.
    Active = 1,
    /// Voting closed, quorum met, yes > no.  Ready for execution.
    Passed = 2,
    /// Voting closed; quorum NOT met, or no >= yes.
    Failed = 3,
    /// Proposal was successfully executed.
    Executed = 4,
    /// Passed but the execution window expired before execution.
    Expired = 5,
    /// Cancelled by the proposer or admin before execution.
    Cancelled = 6,
}

// ---------------------------------------------------------------------------
// Proposal struct
// ---------------------------------------------------------------------------

/// A full proposal record stored on-chain.
///
/// Transfer proposals populate `transfer_recipient` and `transfer_amount`.
/// Upgrade proposals populate `upgrade_wasm_hash`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub kind: ProposalKind,
    pub votes_yes: i128,
    pub votes_no: i128,
    pub state: ProposalState,
    /// Ledger sequence at which voting closes (inclusive).
    pub voting_end_ledger: u32,
    /// Ledger sequence at which the execution window closes (inclusive).
    pub exec_end_ledger: u32,
    // --- Transfer-proposal parameters (set to proposer / 0 for Upgrade proposals) ---
    pub transfer_recipient: Address,
    pub transfer_amount: i128,
    // --- Upgrade-proposal parameters (set to zero hash for Transfer proposals) ---
    pub upgrade_wasm_hash: BytesN<32>,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Privileged admin address (instance storage).
    Admin,
    /// Minimum total-vote weight required for a proposal to pass (instance).
    MinQuorum,
    /// Default voting duration in ledgers (instance).
    VotingDuration,
    /// Default execution window in ledgers after voting ends (instance).
    ExecDuration,
    /// Running count of proposals (instance).
    ProposalCount,
    /// Treasury balance held by the DAO (persistent).
    TreasuryBalance,
    /// Individual proposal data (persistent).
    Proposal(u32),
    /// Per-voter receipt: true if `Address` has voted on proposal `u32` (persistent).
    Voted(u32, Address),
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DaoError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ProposalNotFound = 3,
    InvalidState = 4,
    VotingEnded = 5,
    VotingNotEnded = 6,
    ExecutionExpired = 7,
    AlreadyVoted = 8,
    Unauthorized = 9,
    QuorumNotMet = 10,
    ProposalFailed = 11,
    InsufficientTreasuryBalance = 12,
    InvalidAmount = 13,
}

// ---------------------------------------------------------------------------
// Events  (04-events pattern: structured topics + typed payload)
// ---------------------------------------------------------------------------

/// Emitted once when the DAO is initialised.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DaoInitialized {
    pub admin: Address,
    pub min_quorum: i128,
    pub voting_duration: u32,
    pub exec_duration: u32,
}

/// Emitted each time a new proposal is submitted.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalSubmitted {
    pub proposal_id: u32,
    pub proposer: Address,
    pub kind: ProposalKind,
    pub voting_end_ledger: u32,
    pub exec_end_ledger: u32,
}

/// Emitted for each vote cast.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCast {
    pub proposal_id: u32,
    pub voter: Address,
    pub approve: bool,
    pub weight: i128,
}

/// Emitted when a proposal is executed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecuted {
    pub proposal_id: u32,
    pub executor: Address,
}

/// Emitted when a proposal is cancelled.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelled {
    pub proposal_id: u32,
    pub cancelled_by: Address,
}

/// Emitted on treasury deposit.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryDeposit {
    pub depositor: Address,
    pub amount: i128,
    pub new_balance: i128,
}

/// Emitted on treasury withdrawal (via Transfer proposal execution).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryWithdrawal {
    pub recipient: Address,
    pub amount: i128,
    pub new_balance: i128,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DaoContract;

#[contractimpl]
impl DaoContract {
    // =======================================================================
    // Initialisation
    // =======================================================================

    /// Initialise the DAO.  Must be called exactly once.
    ///
    /// # Auth pattern (03-authentication)
    /// `admin.require_auth()` is called first so the host verifies the admin
    /// has signed this invocation before any storage is written.
    pub fn initialize(
        env: Env,
        admin: Address,
        min_quorum: i128,
        voting_duration: u32,
        exec_duration: u32,
    ) -> Result<(), DaoError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(DaoError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::MinQuorum, &min_quorum);
        env.storage()
            .instance()
            .set(&DataKey::VotingDuration, &voting_duration);
        env.storage()
            .instance()
            .set(&DataKey::ExecDuration, &exec_duration);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance, &0i128);

        // 04-events pattern: structured payload with typed fields
        DaoInitialized {
            admin,
            min_quorum,
            voting_duration,
            exec_duration,
        }
        .publish(&env);

        Ok(())
    }

    // =======================================================================
    // Treasury management
    // =======================================================================

    /// Deposit `amount` units into the treasury.
    ///
    /// In a production contract the actual token transfer would be performed
    /// via a SEP-41 token contract call; here we track the balance internally
    /// to keep the example self-contained.
    pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), DaoError> {
        Self::assert_initialized(&env)?;
        depositor.require_auth(); // 03-authentication
        if amount <= 0 {
            return Err(DaoError::InvalidAmount);
        }

        let old: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TreasuryBalance)
            .unwrap_or(0);
        let new_balance = old + amount;
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance, &new_balance);

        TreasuryDeposit {
            depositor,
            amount,
            new_balance,
        }
        .publish(&env);

        Ok(())
    }

    /// Return the current treasury balance.
    pub fn treasury_balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TreasuryBalance)
            .unwrap_or(0)
    }

    // =======================================================================
    // Proposal lifecycle – Transfer
    // =======================================================================

    /// Submit a Transfer proposal: release `amount` from the treasury to
    /// `recipient` if it passes.
    pub fn propose_transfer(
        env: Env,
        proposer: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<u32, DaoError> {
        Self::assert_initialized(&env)?;
        proposer.require_auth(); // 03-authentication
        if amount <= 0 {
            return Err(DaoError::InvalidAmount);
        }

        let (count, voting_end_ledger, exec_end_ledger) = Self::next_proposal_slots(&env)?;

        // Sentinel zero-hash for non-upgrade proposals
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);

        let proposal = Proposal {
            id: count,
            proposer: proposer.clone(),
            kind: ProposalKind::Transfer,
            votes_yes: 0,
            votes_no: 0,
            state: ProposalState::Active,
            voting_end_ledger,
            exec_end_ledger,
            transfer_recipient: recipient.clone(),
            transfer_amount: amount,
            upgrade_wasm_hash: zero_hash,
        };

        Self::save_proposal(&env, count, &proposal);

        ProposalSubmitted {
            proposal_id: count,
            proposer,
            kind: ProposalKind::Transfer,
            voting_end_ledger,
            exec_end_ledger,
        }
        .publish(&env);

        Ok(count)
    }

    // =======================================================================
    // Proposal lifecycle – Upgrade
    // =======================================================================

    /// Submit an Upgrade proposal: replace the contract WASM with
    /// `new_wasm_hash` if it passes.
    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<u32, DaoError> {
        Self::assert_initialized(&env)?;
        proposer.require_auth(); // 03-authentication

        let (count, voting_end_ledger, exec_end_ledger) = Self::next_proposal_slots(&env)?;

        // Sentinel proposer address for non-transfer proposals
        let zero_addr = proposer.clone();

        let proposal = Proposal {
            id: count,
            proposer: proposer.clone(),
            kind: ProposalKind::Upgrade,
            votes_yes: 0,
            votes_no: 0,
            state: ProposalState::Active,
            voting_end_ledger,
            exec_end_ledger,
            transfer_recipient: zero_addr,
            transfer_amount: 0,
            upgrade_wasm_hash: new_wasm_hash.clone(),
        };

        Self::save_proposal(&env, count, &proposal);

        ProposalSubmitted {
            proposal_id: count,
            proposer,
            kind: ProposalKind::Upgrade,
            voting_end_ledger,
            exec_end_ledger,
        }
        .publish(&env);

        Ok(count)
    }

    // =======================================================================
    // Voting
    // =======================================================================

    /// Cast a vote on an active proposal.
    ///
    /// `weight` represents the voter's governance-token balance (must be > 0).
    /// Each address may vote exactly once per proposal.
    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        approve: bool,
        weight: i128,
    ) -> Result<(), DaoError> {
        Self::assert_initialized(&env)?;
        voter.require_auth(); // 03-authentication

        if weight <= 0 {
            return Err(DaoError::InvalidAmount);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)?;

        // Must be in the active state
        if proposal.state != ProposalState::Active {
            return Err(DaoError::InvalidState);
        }

        // Voting window must still be open
        if env.ledger().sequence() > proposal.voting_end_ledger {
            return Err(DaoError::VotingEnded);
        }

        // Double-vote guard
        let vote_key = DataKey::Voted(proposal_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(DaoError::AlreadyVoted);
        }

        if approve {
            proposal.votes_yes += weight;
        } else {
            proposal.votes_no += weight;
        }

        env.storage().persistent().set(&vote_key, &true);
        Self::save_proposal(&env, proposal_id, &proposal);

        VoteCast {
            proposal_id,
            voter,
            approve,
            weight,
        }
        .publish(&env);

        Ok(())
    }

    // =======================================================================
    // Execution
    // =======================================================================

    /// Execute a proposal that has passed.
    ///
    /// - For `Transfer` proposals: releases assets from the treasury.
    /// - For `Upgrade` proposals: performs a WASM upgrade of this contract.
    pub fn execute(env: Env, executor: Address, proposal_id: u32) -> Result<(), DaoError> {
        Self::assert_initialized(&env)?;
        executor.require_auth(); // 03-authentication

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)?;

        let resolved = Self::resolve_state(&env, &proposal);

        match resolved {
            ProposalState::Active => return Err(DaoError::VotingNotEnded),
            ProposalState::Failed => return Err(DaoError::ProposalFailed),
            ProposalState::Expired => return Err(DaoError::ExecutionExpired),
            ProposalState::Executed => return Err(DaoError::InvalidState),
            ProposalState::Cancelled => return Err(DaoError::InvalidState),
            ProposalState::Passed => {} // proceed
        }

        // Execute the proposal action
        match proposal.kind {
            ProposalKind::Transfer => {
                Self::execute_transfer(
                    &env,
                    proposal.transfer_recipient.clone(),
                    proposal.transfer_amount,
                )?;
            }
            ProposalKind::Upgrade => {
                env.deployer()
                    .update_current_contract_wasm(proposal.upgrade_wasm_hash.clone());
            }
        }

        proposal.state = ProposalState::Executed;
        Self::save_proposal(&env, proposal_id, &proposal);

        ProposalExecuted {
            proposal_id,
            executor,
        }
        .publish(&env);

        Ok(())
    }

    // =======================================================================
    // Cancellation
    // =======================================================================

    /// Cancel a proposal (proposer or admin only).
    pub fn cancel(env: Env, caller: Address, proposal_id: u32) -> Result<(), DaoError> {
        Self::assert_initialized(&env)?;
        caller.require_auth(); // 03-authentication

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)?;

        let resolved = Self::resolve_state(&env, &proposal);
        if matches!(
            resolved,
            ProposalState::Executed | ProposalState::Cancelled | ProposalState::Expired
        ) {
            return Err(DaoError::InvalidState);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DaoError::NotInitialized)?;

        if caller != proposal.proposer && caller != admin {
            return Err(DaoError::Unauthorized);
        }

        proposal.state = ProposalState::Cancelled;
        Self::save_proposal(&env, proposal_id, &proposal);

        ProposalCancelled {
            proposal_id,
            cancelled_by: caller,
        }
        .publish(&env);

        Ok(())
    }

    // =======================================================================
    // Queries
    // =======================================================================

    /// Fetch a proposal with the **resolved** state.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError> {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)?;
        proposal.state = Self::resolve_state(&env, &proposal);
        Ok(proposal)
    }

    /// Return the resolved state of a proposal.
    pub fn proposal_state(env: Env, proposal_id: u32) -> Result<ProposalState, DaoError> {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(DaoError::ProposalNotFound)?;
        Ok(Self::resolve_state(&env, &proposal))
    }

    /// Return the admin address.
    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Return the total number of proposals submitted so far.
    pub fn proposal_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    /// Check whether a given address has already voted on a proposal.
    pub fn has_voted(env: Env, proposal_id: u32, voter: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Voted(proposal_id, voter))
    }

    // =======================================================================
    // Private helpers
    // =======================================================================

    /// Lazily resolve the true state of a proposal relative to the current ledger.
    fn resolve_state(env: &Env, proposal: &Proposal) -> ProposalState {
        // Terminal states are never overridden
        if matches!(
            proposal.state,
            ProposalState::Executed | ProposalState::Cancelled
        ) {
            return proposal.state;
        }

        let seq = env.ledger().sequence();

        if proposal.state == ProposalState::Active {
            if seq <= proposal.voting_end_ledger {
                return ProposalState::Active;
            }

            // Voting window is closed; evaluate outcome
            let min_quorum: i128 = env
                .storage()
                .instance()
                .get(&DataKey::MinQuorum)
                .unwrap_or(0);

            let total = proposal.votes_yes + proposal.votes_no;
            if total < min_quorum || proposal.votes_yes <= proposal.votes_no {
                return ProposalState::Failed;
            }

            // Passed – check execution window
            if seq <= proposal.exec_end_ledger {
                return ProposalState::Passed;
            } else {
                return ProposalState::Expired;
            }
        }

        proposal.state
    }

    /// Execute a Transfer proposal: deduct from treasury and emit event.
    fn execute_transfer(env: &Env, recipient: Address, amount: i128) -> Result<(), DaoError> {
        if amount <= 0 {
            return Err(DaoError::InvalidAmount);
        }
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TreasuryBalance)
            .unwrap_or(0);
        if balance < amount {
            return Err(DaoError::InsufficientTreasuryBalance);
        }
        let new_balance = balance - amount;
        env.storage()
            .persistent()
            .set(&DataKey::TreasuryBalance, &new_balance);

        TreasuryWithdrawal {
            recipient,
            amount,
            new_balance,
        }
        .publish(env);

        Ok(())
    }

    /// Compute the next proposal ID and ledger slots.
    fn next_proposal_slots(env: &Env) -> Result<(u32, u32, u32), DaoError> {
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .ok_or(DaoError::NotInitialized)?;

        let voting_duration: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingDuration)
            .ok_or(DaoError::NotInitialized)?;

        let exec_duration: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ExecDuration)
            .ok_or(DaoError::NotInitialized)?;

        let voting_end_ledger = env.ledger().sequence() + voting_duration;
        let exec_end_ledger = voting_end_ledger + exec_duration;

        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(count + 1));

        Ok((count, voting_end_ledger, exec_end_ledger))
    }

    /// Persist a proposal.
    fn save_proposal(env: &Env, id: u32, proposal: &Proposal) {
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), proposal);
    }

    /// Guard: return an error if the contract has not been initialised.
    fn assert_initialized(env: &Env) -> Result<(), DaoError> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(DaoError::NotInitialized)
        }
    }
}

#[cfg(test)]
mod test;
