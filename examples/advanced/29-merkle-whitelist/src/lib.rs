#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Bytes, BytesN, Env,
    Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidProof = 4,
    AlreadyRegistered = 5,
    NotWhitelisted = 6,
    InvalidFee = 7,
    ProposalNotFound = 8,
    ProposalNotPassed = 9,
    TimelockNotExpired = 10,
    AlreadyVoted = 11,
    DisputeNotFound = 12,
    DisputePeriodActive = 13,
    RateLimitExceeded = 14,
    Blacklisted = 15,
    ContractPaused = 16,
    InvalidRole = 17,
    InvalidProposal = 18,
    DisputeAlreadyResolved = 19,
    InsufficientStake = 20,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    MerkleRoot,
    RootVersion,
    FeeConfig,
    GovernanceConfig,
    Paused,
    Role(Address, Role),
    WhitelistEntry(Address),
    Proposal(u64),
    ProposalCount,
    Vote(u64, Address),
    Dispute(u64),
    DisputeCount,
    DisputeVote(u64, Address),
    RateLimit(Address),
    Blacklist(Address),
    Nonce(Address),
    AccumulatedFees,
    FeeWaiver(Address),
}

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Governor,
    Validator,
}

// ---------------------------------------------------------------------------
// Whitelist Entry
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhitelistEntry {
    pub verified: bool,
    pub registered_at: u64,
    pub metadata: Bytes,
    pub root_version: u64,
    pub dispute_count: u32,
}

// ---------------------------------------------------------------------------
// Fee Configuration
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub token: Address,
    pub registration_fee: i128,
    pub dispute_fee: i128,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Governance Configuration
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceConfig {
    pub quorum: u32,
    pub timelock_duration: u64,
    pub proposal_duration: u64,
    pub dispute_period: u64,
    pub validator_stake_required: i128,
}

// ---------------------------------------------------------------------------
// Proposal
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub new_root: BytesN<32>,
    pub metadata: Bytes,
    pub created_at: u64,
    pub votes_for: u32,
    pub votes_against: u32,
    pub executed: bool,
    pub timelock_ends: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

// ---------------------------------------------------------------------------
// Dispute
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub id: u64,
    pub target: Address,
    pub submitter: Address,
    pub evidence: Bytes,
    pub created_at: u64,
    pub votes_invalid: u32,
    pub votes_valid: u32,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeDecision {
    Invalid,
    Valid,
}

// ---------------------------------------------------------------------------
// Rate Limit
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitData {
    pub count: u32,
    pub window_start: u64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct MerkleWhitelistContract;

#[contractimpl]
impl MerkleWhitelistContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the contract with admin, fee token, and initial Merkle root.
    ///
    /// # Arguments
    /// * `admin` - Contract administrator with full control
    /// * `fee_token` - Token address for fee payments
    /// * `registration_fee` - Fee required to register an entry
    /// * `initial_root` - Initial Merkle tree root hash
    pub fn initialize(
        env: Env,
        admin: Address,
        fee_token: Address,
        registration_fee: i128,
        initial_root: BytesN<32>,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        // Store admin
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Store initial Merkle root
        env.storage().instance().set(&DataKey::MerkleRoot, &initial_root);
        env.storage().instance().set(&DataKey::RootVersion, &1u64);

        // Store fee configuration
        let fee_config = FeeConfig {
            token: fee_token,
            registration_fee,
            dispute_fee: registration_fee / 10, // 10% of registration fee
            enabled: true,
        };
        env.storage().instance().set(&DataKey::FeeConfig, &fee_config);

        // Store governance configuration with sensible defaults
        let gov_config = GovernanceConfig {
            quorum: 3,                         // Require 3 votes minimum
            timelock_duration: 86400,          // 1 day timelock
            proposal_duration: 259200,         // 3 days voting period
            dispute_period: 172800,            // 2 days challenge period
            validator_stake_required: 1000000, // Stake required for validators
        };
        env.storage()
            .instance()
            .set(&DataKey::GovernanceConfig, &gov_config);

        // Initialize counters
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);
        env.storage().instance().set(&DataKey::DisputeCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::AccumulatedFees, &0i128);
        env.storage().instance().set(&DataKey::Paused, &false);

        // Grant admin role to initializer
        env.storage()
            .instance()
            .set(&DataKey::Role(admin.clone(), Role::Admin), &true);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Core Whitelist Functions
    // -----------------------------------------------------------------------

    /// Verify and register a whitelisted address using a Merkle proof.
    ///
    /// # Arguments
    /// * `address` - Address to verify and register
    /// * `proof` - Merkle proof (sibling hashes from leaf to root)
    /// * `metadata` - Additional metadata about the entry
    ///
    /// # Fees
    /// Requires payment of registration fee (unless waived)
    pub fn verify_whitelist(
        env: Env,
        address: Address,
        proof: Vec<BytesN<32>>,
        metadata: Bytes,
    ) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        address.require_auth();

        // Check blacklist
        if Self::is_blacklisted(&env, &address) {
            return Err(Error::Blacklisted);
        }

        // Check rate limit
        Self::check_rate_limit(&env, &address)?;

        // Check if already registered
        let entry_key = DataKey::WhitelistEntry(address.clone());
        if env.storage().persistent().has(&entry_key) {
            return Err(Error::AlreadyRegistered);
        }

        // Get current root and version
        let root = Self::get_merkle_root(&env)?;
        let root_version: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RootVersion)
            .unwrap_or(1);

        // Get nonce for replay protection
        let nonce = Self::get_nonce(&env, &address);

        // Compute leaf hash: hash(address || nonce || metadata)
        let leaf = Self::compute_leaf_hash(&env, &address, nonce, &metadata);

        // Verify Merkle proof
        let computed_root = Self::compute_root_from_proof(&env, &leaf, &proof);
        if computed_root != root {
            return Err(Error::InvalidProof);
        }

        // Collect registration fee
        Self::collect_registration_fee(&env, &address)?;

        // Increment nonce
        env.storage()
            .instance()
            .set(&DataKey::Nonce(address.clone()), &(nonce + 1));

        // Store whitelist entry
        let entry = WhitelistEntry {
            verified: true,
            registered_at: env.ledger().timestamp(),
            metadata: metadata.clone(),
            root_version,
            dispute_count: 0,
        };
        env.storage().persistent().set(&entry_key, &entry);
        env.storage().persistent().extend_ttl(&entry_key, 17280, 120960);

        // Update rate limit
        Self::update_rate_limit(&env, &address);

        Ok(())
    }

    /// Check if an address is whitelisted (has verified entry).
    pub fn is_whitelisted(env: Env, address: Address) -> bool {
        let entry_key = DataKey::WhitelistEntry(address);
        if let Some(entry) = env.storage().persistent().get::<_, WhitelistEntry>(&entry_key) {
            entry.verified
        } else {
            false
        }
    }

    /// Get whitelist entry status and metadata.
    pub fn get_entry(env: Env, address: Address) -> Result<WhitelistEntry, Error> {
        let entry_key = DataKey::WhitelistEntry(address);
        env.storage()
            .persistent()
            .get(&entry_key)
            .ok_or(Error::NotWhitelisted)
    }

    /// Revoke a whitelist entry (admin only).
    pub fn revoke_entry(env: Env, caller: Address, target: Address) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        let entry_key = DataKey::WhitelistEntry(target);
        if !env.storage().persistent().has(&entry_key) {
            return Err(Error::NotWhitelisted);
        }

        env.storage().persistent().remove(&entry_key);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Governance Functions
    // -----------------------------------------------------------------------

    /// Propose a new Merkle root update.
    ///
    /// # Arguments
    /// * `proposer` - Address creating the proposal (must be governor)
    /// * `new_root` - New Merkle root to be adopted
    /// * `metadata` - Description or justification for the update
    pub fn propose_root_update(
        env: Env,
        proposer: Address,
        new_root: BytesN<32>,
        metadata: Bytes,
    ) -> Result<u64, Error> {
        Self::ensure_not_paused(&env)?;
        proposer.require_auth();
        Self::ensure_role(&env, &proposer, Role::Governor)?;

        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);
        let new_id = proposal_id + 1;

        let gov_config = Self::get_governance_config(&env)?;
        let now = env.ledger().timestamp();

        let proposal = Proposal {
            id: new_id,
            proposer: proposer.clone(),
            new_root,
            metadata,
            created_at: now,
            votes_for: 0,
            votes_against: 0,
            executed: false,
            timelock_ends: now + gov_config.timelock_duration,
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(new_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &new_id);

        Ok(new_id)
    }

    /// Vote on an active proposal.
    ///
    /// # Arguments
    /// * `voter` - Address casting the vote (must be governor)
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `support` - true = vote for, false = vote against
    pub fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: bool,
    ) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        voter.require_auth();
        Self::ensure_role(&env, &voter, Role::Governor)?;

        let proposal_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&proposal_key)
            .ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::InvalidProposal);
        }

        let gov_config = Self::get_governance_config(&env)?;
        let now = env.ledger().timestamp();

        // Check if voting period has expired
        if now > proposal.created_at + gov_config.proposal_duration {
            return Err(Error::InvalidProposal);
        }

        // Check if already voted
        let vote_key = DataKey::Vote(proposal_id, voter.clone());
        if env.storage().temporary().has(&vote_key) {
            return Err(Error::AlreadyVoted);
        }

        // Record vote
        env.storage().temporary().set(&vote_key, &support);

        // Update vote counts
        if support {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        env.storage().instance().set(&proposal_key, &proposal);

        Ok(())
    }

    /// Execute a passed proposal after timelock expires.
    ///
    /// # Arguments
    /// * `executor` - Address executing the proposal
    /// * `proposal_id` - ID of the proposal to execute
    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u64,
    ) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        executor.require_auth();

        let proposal_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&proposal_key)
            .ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::InvalidProposal);
        }

        let gov_config = Self::get_governance_config(&env)?;
        let now = env.ledger().timestamp();

        // Check if proposal passed
        if proposal.votes_for < gov_config.quorum
            || proposal.votes_for <= proposal.votes_against
        {
            return Err(Error::ProposalNotPassed);
        }

        // Check if timelock expired
        if now < proposal.timelock_ends {
            return Err(Error::TimelockNotExpired);
        }

        // Execute: Update Merkle root
        env.storage()
            .instance()
            .set(&DataKey::MerkleRoot, &proposal.new_root);

        // Increment root version
        let current_version: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RootVersion)
            .unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::RootVersion, &(current_version + 1));

        // Mark as executed
        proposal.executed = true;
        env.storage().instance().set(&proposal_key, &proposal);

        Ok(())
    }

    /// Get proposal details.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)
    }

    /// Get proposal status.
    pub fn get_proposal_status(env: Env, proposal_id: u64) -> Result<ProposalStatus, Error> {
        let proposal: Proposal = Self::get_proposal(env.clone(), proposal_id)?;
        let gov_config = Self::get_governance_config(&env)?;
        let now = env.ledger().timestamp();

        if proposal.executed {
            return Ok(ProposalStatus::Executed);
        }

        // Check if voting period expired
        if now > proposal.created_at + gov_config.proposal_duration {
            if proposal.votes_for >= gov_config.quorum
                && proposal.votes_for > proposal.votes_against
            {
                return Ok(ProposalStatus::Passed);
            } else {
                return Ok(ProposalStatus::Rejected);
            }
        }

        Ok(ProposalStatus::Active)
    }

    // -----------------------------------------------------------------------
    // Dispute Functions
    // -----------------------------------------------------------------------

    /// Submit a dispute against a whitelisted entry.
    ///
    /// # Arguments
    /// * `submitter` - Address submitting the dispute (must be validator)
    /// * `target` - Address being disputed
    /// * `evidence` - Evidence supporting the dispute (hash or description)
    pub fn submit_dispute(
        env: Env,
        submitter: Address,
        target: Address,
        evidence: Bytes,
    ) -> Result<u64, Error> {
        Self::ensure_not_paused(&env)?;
        submitter.require_auth();
        Self::ensure_role(&env, &submitter, Role::Validator)?;

        // Check if target is whitelisted
        let entry_key = DataKey::WhitelistEntry(target.clone());
        if !env.storage().persistent().has(&entry_key) {
            return Err(Error::NotWhitelisted);
        }

        // Collect dispute fee
        Self::collect_dispute_fee(&env, &submitter)?;

        let dispute_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0);
        let new_id = dispute_id + 1;

        let dispute = Dispute {
            id: new_id,
            target: target.clone(),
            submitter,
            evidence,
            created_at: env.ledger().timestamp(),
            votes_invalid: 0,
            votes_valid: 0,
            resolved: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Dispute(new_id), &dispute);
        env.storage()
            .instance()
            .set(&DataKey::DisputeCount, &new_id);

        Ok(new_id)
    }

    /// Vote on a dispute.
    ///
    /// # Arguments
    /// * `voter` - Address casting vote (must be validator)
    /// * `dispute_id` - ID of the dispute
    /// * `decision` - Invalid or Valid
    pub fn vote_on_dispute(
        env: Env,
        voter: Address,
        dispute_id: u64,
        decision: DisputeDecision,
    ) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        voter.require_auth();
        Self::ensure_role(&env, &voter, Role::Validator)?;

        let dispute_key = DataKey::Dispute(dispute_id);
        let mut dispute: Dispute = env
            .storage()
            .instance()
            .get(&dispute_key)
            .ok_or(Error::DisputeNotFound)?;

        if dispute.resolved {
            return Err(Error::DisputeAlreadyResolved);
        }

        // Check if already voted
        let vote_key = DataKey::DisputeVote(dispute_id, voter.clone());
        if env.storage().temporary().has(&vote_key) {
            return Err(Error::AlreadyVoted);
        }

        // Record vote
        env.storage().temporary().set(&vote_key, &decision);

        // Update vote counts
        match decision {
            DisputeDecision::Invalid => dispute.votes_invalid += 1,
            DisputeDecision::Valid => dispute.votes_valid += 1,
        }

        env.storage().instance().set(&dispute_key, &dispute);

        Ok(())
    }

    /// Resolve a dispute after voting period.
    ///
    /// # Arguments
    /// * `resolver` - Address resolving the dispute
    /// * `dispute_id` - ID of the dispute to resolve
    pub fn resolve_dispute(
        env: Env,
        resolver: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        Self::ensure_not_paused(&env)?;
        resolver.require_auth();

        let dispute_key = DataKey::Dispute(dispute_id);
        let mut dispute: Dispute = env
            .storage()
            .instance()
            .get(&dispute_key)
            .ok_or(Error::DisputeNotFound)?;

        if dispute.resolved {
            return Err(Error::DisputeAlreadyResolved);
        }

        let gov_config = Self::get_governance_config(&env)?;
        let now = env.ledger().timestamp();

        // Check if dispute period has passed
        if now < dispute.created_at + gov_config.dispute_period {
            return Err(Error::DisputePeriodActive);
        }

        // Resolve based on votes
        if dispute.votes_invalid > dispute.votes_valid {
            // Dispute upheld - revoke whitelist entry
            let entry_key = DataKey::WhitelistEntry(dispute.target.clone());
            env.storage().persistent().remove(&entry_key);

            // Add to blacklist
            env.storage()
                .instance()
                .set(&DataKey::Blacklist(dispute.target.clone()), &true);
        }
        // If votes_valid >= votes_invalid, entry remains whitelisted

        dispute.resolved = true;
        env.storage().instance().set(&dispute_key, &dispute);

        Ok(())
    }

    /// Get dispute details.
    pub fn get_dispute(env: Env, dispute_id: u64) -> Result<Dispute, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Dispute(dispute_id))
            .ok_or(Error::DisputeNotFound)
    }

    // -----------------------------------------------------------------------
    // Role Management
    // -----------------------------------------------------------------------

    /// Grant a role to an address (admin only).
    pub fn add_role(env: Env, caller: Address, target: Address, role: Role) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage()
            .instance()
            .set(&DataKey::Role(target, role), &true);

        Ok(())
    }

    /// Revoke a role from an address (admin only).
    pub fn remove_role(
        env: Env,
        caller: Address,
        target: Address,
        role: Role,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage().instance().remove(&DataKey::Role(target, role));

        Ok(())
    }

    /// Check if an address has a specific role.
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Role(address, role))
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Fee Management
    // -----------------------------------------------------------------------

    /// Update registration fee (admin only).
    pub fn set_registration_fee(env: Env, caller: Address, new_fee: i128) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        let mut fee_config = Self::get_fee_config(&env)?;
        fee_config.registration_fee = new_fee;
        env.storage()
            .instance()
            .set(&DataKey::FeeConfig, &fee_config);

        Ok(())
    }

    /// Grant fee waiver to an address (admin only).
    pub fn grant_fee_waiver(env: Env, caller: Address, target: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage()
            .instance()
            .set(&DataKey::FeeWaiver(target), &true);

        Ok(())
    }

    /// Revoke fee waiver (admin only).
    pub fn revoke_fee_waiver(env: Env, caller: Address, target: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage().instance().remove(&DataKey::FeeWaiver(target));

        Ok(())
    }

    /// Collect accumulated fees (admin only).
    pub fn collect_fees(env: Env, caller: Address, recipient: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        let accumulated: i128 = env
            .storage()
            .instance()
            .get(&DataKey::AccumulatedFees)
            .unwrap_or(0);

        if accumulated <= 0 {
            return Err(Error::InvalidFee);
        }

        let fee_config = Self::get_fee_config(&env)?;
        let token_client = token::Client::new(&env, &fee_config.token);

        token_client.transfer(&env.current_contract_address(), &recipient, &accumulated);

        env.storage()
            .instance()
            .set(&DataKey::AccumulatedFees, &0i128);

        Ok(())
    }

    /// Get accumulated fees.
    pub fn get_accumulated_fees(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::AccumulatedFees)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Admin Functions
    // -----------------------------------------------------------------------

    /// Emergency pause (admin only).
    pub fn pause(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage().instance().set(&DataKey::Paused, &true);

        Ok(())
    }

    /// Unpause contract (admin only).
    pub fn unpause(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage().instance().set(&DataKey::Paused, &false);

        Ok(())
    }

    /// Check if contract is paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Add address to blacklist (admin only).
    pub fn add_to_blacklist(env: Env, caller: Address, target: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage()
            .instance()
            .set(&DataKey::Blacklist(target), &true);

        Ok(())
    }

    /// Remove address from blacklist (admin only).
    pub fn remove_from_blacklist(
        env: Env,
        caller: Address,
        target: Address,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage().instance().remove(&DataKey::Blacklist(target));

        Ok(())
    }

    /// Check if address is blacklisted.
    pub fn is_blacklisted(env: &Env, address: &Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Blacklist(address.clone()))
            .unwrap_or(false)
    }

    /// Update governance configuration (admin only).
    pub fn update_governance_config(
        env: Env,
        caller: Address,
        new_config: GovernanceConfig,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin)?;

        env.storage()
            .instance()
            .set(&DataKey::GovernanceConfig, &new_config);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Query Functions
    // -----------------------------------------------------------------------

    /// Get current Merkle root.
    pub fn get_merkle_root(env: &Env) -> Result<BytesN<32>, Error> {
        env.storage()
            .instance()
            .get(&DataKey::MerkleRoot)
            .ok_or(Error::NotInitialized)
    }

    /// Get current root version.
    pub fn get_root_version(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RootVersion)
            .unwrap_or(1)
    }

    /// Get fee configuration.
    pub fn get_fee_config(env: &Env) -> Result<FeeConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::FeeConfig)
            .ok_or(Error::NotInitialized)
    }

    /// Get governance configuration.
    pub fn get_governance_config(env: &Env) -> Result<GovernanceConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::GovernanceConfig)
            .ok_or(Error::NotInitialized)
    }

    /// Get admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Get nonce for an address.
    pub fn get_nonce(env: &Env, address: &Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Nonce(address.clone()))
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal Helpers
    // -----------------------------------------------------------------------

    /// Ensure contract is not paused.
    fn ensure_not_paused(env: &Env) -> Result<(), Error> {
        if Self::is_paused(env.clone()) {
            Err(Error::ContractPaused)
        } else {
            Ok(())
        }
    }

    /// Ensure address has required role.
    fn ensure_role(env: &Env, address: &Address, role: Role) -> Result<(), Error> {
        if Self::has_role(env.clone(), address.clone(), role) {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }

    /// Compute leaf hash: hash(address || nonce || metadata).
    fn compute_leaf_hash(env: &Env, address: &Address, nonce: u64, metadata: &Bytes) -> BytesN<32> {
        let mut buf = Bytes::new(env);

        // Append address bytes
        let addr_str = address.to_string();
        buf.append(&addr_str);

        // Append nonce (8 bytes, big-endian)
        let nonce_bytes = nonce.to_be_bytes();
        for byte in nonce_bytes.iter() {
            buf.push_back(*byte);
        }

        // Append metadata
        buf.append(metadata);

        env.crypto().sha256(&buf).to_bytes()
    }

    /// Compute Merkle root from leaf and proof.
    fn compute_root_from_proof(
        env: &Env,
        leaf: &BytesN<32>,
        proof: &Vec<BytesN<32>>,
    ) -> BytesN<32> {
        let mut computed = leaf.clone();
        for sibling in proof.iter() {
            computed = Self::hash_pair(env, &computed, &sibling);
        }
        computed
    }

    /// Hash two nodes (sorted order for consistency).
    fn hash_pair(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
        let mut buf = Bytes::new(env);
        if a <= b {
            buf.append(&Bytes::from(a.clone()));
            buf.append(&Bytes::from(b.clone()));
        } else {
            buf.append(&Bytes::from(b.clone()));
            buf.append(&Bytes::from(a.clone()));
        }
        env.crypto().sha256(&buf).to_bytes()
    }

    /// Collect registration fee from address.
    fn collect_registration_fee(env: &Env, from: &Address) -> Result<(), Error> {
        // Check if fee waiver granted
        if env
            .storage()
            .instance()
            .get(&DataKey::FeeWaiver(from.clone()))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let fee_config = Self::get_fee_config(env)?;

        if !fee_config.enabled || fee_config.registration_fee <= 0 {
            return Ok(());
        }

        let token_client = token::Client::new(env, &fee_config.token);
        token_client.transfer(from, &env.current_contract_address(), &fee_config.registration_fee);

        // Accumulate fees
        let current: i128 = env
            .storage()
            .instance()
            .get(&DataKey::AccumulatedFees)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::AccumulatedFees, &(current + fee_config.registration_fee));

        Ok(())
    }

    /// Collect dispute fee from address.
    fn collect_dispute_fee(env: &Env, from: &Address) -> Result<(), Error> {
        let fee_config = Self::get_fee_config(env)?;

        if !fee_config.enabled || fee_config.dispute_fee <= 0 {
            return Ok(());
        }

        let token_client = token::Client::new(env, &fee_config.token);
        token_client.transfer(from, &env.current_contract_address(), &fee_config.dispute_fee);

        // Accumulate fees
        let current: i128 = env
            .storage()
            .instance()
            .get(&DataKey::AccumulatedFees)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::AccumulatedFees, &(current + fee_config.dispute_fee));

        Ok(())
    }

    /// Check rate limit for address.
    fn check_rate_limit(env: &Env, address: &Address) -> Result<(), Error> {
        let key = DataKey::RateLimit(address.clone());
        let now = env.ledger().timestamp();
        let window_size = 3600; // 1 hour window
        let max_requests = 10; // 10 requests per hour

        if let Some(mut data) = env.storage().temporary().get::<_, RateLimitData>(&key) {
            // Reset window if expired
            if now > data.window_start + window_size {
                data.count = 1;
                data.window_start = now;
                env.storage().temporary().set(&key, &data);
                return Ok(());
            }

            // Check limit
            if data.count >= max_requests {
                return Err(Error::RateLimitExceeded);
            }

            data.count += 1;
            env.storage().temporary().set(&key, &data);
        } else {
            // First request in window
            let data = RateLimitData {
                count: 1,
                window_start: now,
            };
            env.storage().temporary().set(&key, &data);
        }

        Ok(())
    }

    /// Update rate limit counter.
    fn update_rate_limit(env: &Env, address: &Address) {
        let key = DataKey::RateLimit(address.clone());
        if let Some(mut data) = env.storage().temporary().get::<_, RateLimitData>(&key) {
            data.count += 1;
            env.storage().temporary().set(&key, &data);
        }
    }
}

#[cfg(test)]
mod test;
