//! # Admin Controls for Proxy Upgrades
//!
//! This contract demonstrates a comprehensive admin control system for managing proxy upgrades
//! in Soroban smart contracts. It implements security best practices including:
//!
//! - **Admin Authentication**: Multi-signature admin controls with role-based access
//! - **Upgrade Proposal System**: Structured proposal workflow with voting mechanisms
//! - **Timelock Protection**: Mandatory delay periods for security review
//! - **Emergency Controls**: Immediate pause capabilities for crisis response
//! - **Audit Trail**: Complete event logging for transparency
//!
//! ## Security Architecture
//!
//! 1. **Defense in Depth**: Multiple layers of authentication and authorization
//! 2. **Time-Based Controls**: Prevents rushed or malicious upgrades
//! 3. **Emergency Response**: Quick containment capabilities
//! 4. **Transparent Governance**: All actions are logged and auditable

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env,
    Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Proposal status tracking the lifecycle of upgrade proposals
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Proposal created and waiting for admin review
    Pending = 0,
    /// Approved by admins and in timelock period
    Approved = 1,
    /// Timelock completed, ready for execution
    Ready = 2,
    /// Successfully executed
    Executed = 3,
    /// Rejected by admins
    Rejected = 4,
    /// Cancelled by proposer
    Cancelled = 5,
    /// Execution failed
    Failed = 6,
}

/// Contract operational states for emergency controls
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractState {
    /// Normal operation
    Active = 0,
    /// No new proposals, but existing ones can complete
    Paused = 1,
    /// All operations frozen except emergency controls
    Frozen = 2,
}

/// Admin role levels for granular permissions
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdminRole {
    /// Full administrative privileges
    SuperAdmin = 0,
    /// Can propose and approve upgrades
    Upgrader = 1,
    /// Emergency controls only
    Guardian = 2,
}

/// Upgrade proposal structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    /// Unique proposal identifier
    pub id: u64,
    /// Address of the new implementation contract
    pub new_implementation: Address,
    /// Proposal creator
    pub proposer: Address,
    /// Current status
    pub status: ProposalStatus,
    /// Timestamp when created
    pub created_at: u64,
    /// Timestamp when approved (if applicable)
    pub approved_at: u64,
    /// Timestamp when ready for execution
    pub ready_at: u64,
    /// Admin approvals received
    pub approvals: Vec<Address>,
    /// Admin rejections received
    pub rejections: Vec<Address>,
    /// Description of the upgrade
    pub description: Symbol,
    /// IPFS hash of detailed upgrade documentation
    pub documentation_hash: Symbol,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin address
    Admin,
    /// Current contract state
    State,
    /// Current implementation address
    Implementation,
    /// Pending timelock end time
    TimelockEnd,
    /// Default timelock duration in seconds
    DefaultTimelock,
    /// Emergency pause end time
    EmergencyPauseEnd,
    /// Minimum number of admin approvals required
    RequiredApprovals,
    /// Proposal by ID
    Proposal(u64),
    /// Next proposal ID counter
    NextProposalId,
    /// Admin role assignments
    AdminRole(Address),
    /// List of all admin addresses
    AdminList,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProxyUpgradeError {
    /// Unauthorized access attempt
    Unauthorized = 1,
    /// Insufficient admin role
    InsufficientRole = 2,
    /// Contract is paused or frozen
    ContractPaused = 3,
    /// Contract is frozen
    ContractFrozen = 4,
    /// Proposal not found
    ProposalNotFound = 5,
    /// Invalid proposal status for operation
    InvalidProposalStatus = 6,
    /// Timelock period not elapsed
    TimelockNotElapsed = 7,
    /// Insufficient approvals
    InsufficientApprovals = 8,
    /// Already voted on this proposal
    AlreadyVoted = 9,
    /// Invalid implementation address
    InvalidImplementation = 10,
    /// Proposal execution failed
    ExecutionFailed = 11,
    /// Emergency pause active
    EmergencyPauseActive = 12,
    /// Not initialized
    NotInitialized = 13,
    /// Already initialized
    AlreadyInitialized = 14,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ProxyUpgradeControls;

#[contractimpl]
impl ProxyUpgradeControls {
    // ==================== INITIALIZATION ====================

    /// Initialize the contract with the first super admin
    /// 
    /// # Arguments
    /// * `admin` - Initial super admin address
    /// * `implementation` - Initial implementation contract address
    /// * `default_timelock` - Default timelock duration in seconds
    /// * `required_approvals` - Number of admin approvals required
    pub fn initialize(
        env: Env,
        admin: Address,
        implementation: Address,
        default_timelock: u64,
        required_approvals: u32,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        // Validate inputs
        if default_timelock == 0 {
            panic!("Timelock must be > 0");
        }
        if required_approvals == 0 {
            panic!("Required approvals must be > 0");
        }

        // Set initial state
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::State, &ContractState::Active);
        env.storage().instance().set(&DataKey::Implementation, &implementation);
        env.storage().instance().set(&DataKey::DefaultTimelock, &default_timelock);
        env.storage().instance().set(&DataKey::RequiredApprovals, &required_approvals);
        env.storage().instance().set(&DataKey::NextProposalId, &1u64);

        // Set up initial admin
        env.storage()
            .persistent()
            .set(&DataKey::AdminRole(admin.clone()), &AdminRole::SuperAdmin);
        
        // Initialize admin list
        let mut admin_list = Vec::<Address>::new(&env);
        admin_list.push_back(admin.clone());
        env.storage().persistent().set(&DataKey::AdminList, &admin_list);

        // Extend storage TTL
        env.storage().instance().extend_ttl(100, 100);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::AdminRole(admin.clone()), 100, 100);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::AdminList, 100, 100);

        // Log initialization
        env.events().publish(
            (symbol_short!("init"),),
            (admin.clone(), implementation, default_timelock, required_approvals),
        );
    }

    // ==================== ADMIN MANAGEMENT ====================

    /// Add a new admin with specified role
    /// 
    /// # Arguments
    /// * `caller` - Current super admin
    /// * `new_admin` - Address to add as admin
    /// * `role` - Role to assign
    pub fn add_admin(env: Env, caller: Address, new_admin: Address, role: AdminRole) {
        caller.require_auth();
        Self::require_role(&env, &caller, &[AdminRole::SuperAdmin]);
        Self::require_active_state(&env);

        // Check if already admin
        if env
            .storage()
            .persistent()
            .has(&DataKey::AdminRole(new_admin.clone()))
        {
            panic!("Address is already an admin");
        }

        // Add admin role
        env.storage()
            .persistent()
            .set(&DataKey::AdminRole(new_admin.clone()), &role);

        // Update admin list
        let mut admin_list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AdminList)
            .unwrap_or_else(|| Vec::new(&env));
        admin_list.push_back(new_admin.clone());
        env.storage().persistent().set(&DataKey::AdminList, &admin_list);

        // Extend TTL
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::AdminRole(new_admin.clone()), 100, 100);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::AdminList, 100, 100);

        env.events().publish(
            (symbol_short!("admin_add"),),
            (caller, new_admin.clone(), role as u32),
        );
    }

    /// Remove an admin
    /// 
    /// # Arguments
    /// * `caller` - Current super admin
    /// * `admin_to_remove` - Admin address to remove
    pub fn remove_admin(env: Env, caller: Address, admin_to_remove: Address) {
        caller.require_auth();
        Self::require_role(&env, &caller, &[AdminRole::SuperAdmin]);
        Self::require_active_state(&env);

        // Cannot remove the last super admin
        let admin_role: AdminRole = env
            .storage()
            .persistent()
            .get(&DataKey::AdminRole(admin_to_remove.clone()))
            .unwrap_or_else(|| panic!("Address is not an admin"));

        if admin_role == AdminRole::SuperAdmin {
            let admin_list: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::AdminList)
                .unwrap_or_else(|| Vec::new(&env));
            
            let super_admin_count = admin_list.iter().filter(|addr| {
                let role: AdminRole = env
                    .storage()
                    .persistent()
                    .get(&DataKey::AdminRole(addr.clone()))
                    .unwrap_or(AdminRole::Guardian);
                role == AdminRole::SuperAdmin
            }).count();

            if super_admin_count <= 1 {
                panic!("Cannot remove the last super admin");
            }
        }

        // Remove admin role
        env.storage()
            .persistent()
            .remove(&DataKey::AdminRole(admin_to_remove.clone()));

        // Update admin list
        let admin_list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AdminList)
            .unwrap_or_else(|| Vec::new(&env));
        
        let mut new_list = Vec::<Address>::new(&env);
        for addr in admin_list.iter() {
            if addr.clone() != admin_to_remove {
                new_list.push_back(addr.clone());
            }
        }
        env.storage().persistent().set(&DataKey::AdminList, &new_list);

        env.events().publish(
            (symbol_short!("admin_rem"),),
            (caller, admin_to_remove),
        );
    }

    // ==================== UPGRADE PROPOSAL SYSTEM ====================

    /// Create a new upgrade proposal
    /// 
    /// # Arguments
    /// * `proposer` - Address creating the proposal
    /// * `new_implementation` - New implementation contract address
    /// * `description` - Brief description of the upgrade
    /// * `documentation_hash` - IPFS hash of detailed documentation
    /// 
    /// # Returns
    /// Proposal ID
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        new_implementation: Address,
        description: Symbol,
        documentation_hash: Symbol,
    ) -> u64 {
        proposer.require_auth();
        Self::require_role(&env, &proposer, &[AdminRole::SuperAdmin, AdminRole::Upgrader]);
        Self::require_active_state(&env);

        // Validate implementation address
        // Simple validation - ensure it's not the current contract address
        if new_implementation == env.current_contract_address() {
            panic!("Invalid implementation address");
        }

        // Get next proposal ID
        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(1);

        // Create proposal
        let proposal = UpgradeProposal {
            id: proposal_id,
            new_implementation: new_implementation.clone(),
            proposer: proposer.clone(),
            status: ProposalStatus::Pending,
            created_at: env.ledger().timestamp(),
            approved_at: 0,
            ready_at: 0,
            approvals: Vec::new(&env),
            rejections: Vec::new(&env),
            description: description.clone(),
            documentation_hash: documentation_hash.clone(),
        };

        // Store proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // Increment proposal ID counter
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(proposal_id + 1));

        // Extend TTL
        env.storage().instance().extend_ttl(100, 100);

        // Log proposal creation
        env.events().publish(
            (symbol_short!("prop_cr"),),
            (
                proposal_id,
                proposer,
                new_implementation,
                description,
                documentation_hash,
            ),
        );

        proposal_id
    }

    /// Approve a proposal
    /// 
    /// # Arguments
    /// * `admin` - Admin approving the proposal
    /// * `proposal_id` - ID of the proposal to approve
    pub fn approve_proposal(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::require_active_state(&env);

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"));

        // Check proposal status
        if proposal.status != ProposalStatus::Pending {
            panic!("Proposal is not pending");
        }

        // Check if already voted
        if proposal.approvals.iter().any(|addr| addr == admin) {
            panic!("Already voted on this proposal");
        }

        // Add approval
        proposal.approvals.push_back(admin.clone());

        // Check if sufficient approvals
        let required_approvals: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RequiredApprovals)
            .unwrap_or(1);

        if (proposal.approvals.len() as u32) >= required_approvals {
            // Move to approved status and start timelock
            proposal.status = ProposalStatus::Approved;
            proposal.approved_at = env.ledger().timestamp();
            
            let default_timelock: u64 = env
                .storage()
                .instance()
                .get(&DataKey::DefaultTimelock)
                .unwrap_or(86400); // 24 hours default
            
            proposal.ready_at = proposal.approved_at + default_timelock;

            // Set global timelock
            env.storage()
                .instance()
                .set(&DataKey::TimelockEnd, &proposal.ready_at);
        }

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("prop_ap"),),
            (proposal_id, admin, proposal.approvals.len()),
        );
    }

    /// Reject a proposal
    /// 
    /// # Arguments
    /// * `admin` - Admin rejecting the proposal
    /// * `proposal_id` - ID of the proposal to reject
    pub fn reject_proposal(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::require_active_state(&env);

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"));

        // Check proposal status
        if proposal.status != ProposalStatus::Pending {
            panic!("Proposal is not pending");
        }

        // Check if already voted
        if proposal.rejections.iter().any(|addr| addr == admin) {
            panic!("Already voted on this proposal");
        }

        // Add rejection
        proposal.rejections.push_back(admin.clone());
        proposal.status = ProposalStatus::Rejected;

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("prop_rj"),),
            (proposal_id, admin, proposal.rejections.len()),
        );
    }

    /// Execute an approved proposal after timelock
    /// 
    /// # Arguments
    /// * `caller` - Address executing the proposal
    /// * `proposal_id` - ID of the proposal to execute
    pub fn execute_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"));

        // Check proposal status
        if proposal.status != ProposalStatus::Approved {
            panic!("Proposal is not approved");
        }

        // Check timelock
        let now = env.ledger().timestamp();
        if now < proposal.ready_at {
            panic!("Timelock not elapsed");
        }

        // Execute upgrade (in a real implementation, this would update the proxy)
        env.storage()
            .instance()
            .set(&DataKey::Implementation, &proposal.new_implementation);

        // Update proposal status
        proposal.status = ProposalStatus::Executed;

        // Clear global timelock
        env.storage().instance().remove(&DataKey::TimelockEnd);

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("prop_exec"),),
            (proposal_id, caller, proposal.new_implementation),
        );
    }

    /// Cancel a proposal (only proposer can cancel pending proposals)
    /// 
    /// # Arguments
    /// * `proposer` - Original proposal creator
    /// * `proposal_id` - ID of the proposal to cancel
    pub fn cancel_proposal(env: Env, proposer: Address, proposal_id: u64) {
        proposer.require_auth();

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"));

        // Check if proposer
        if proposal.proposer != proposer {
            panic!("Only proposer can cancel");
        }

        // Check status
        if proposal.status != ProposalStatus::Pending {
            panic!("Cannot cancel non-pending proposal");
        }

        // Cancel proposal
        proposal.status = ProposalStatus::Cancelled;

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("prop_cn"),),
            (proposal_id, proposer),
        );
    }

    // ==================== EMERGENCY CONTROLS ====================

    /// Emergency pause - halts all operations except emergency controls
    /// 
    /// # Arguments
    /// * `admin` - Admin initiating the pause
    /// * `duration` - Duration in seconds (0 for indefinite)
    pub fn emergency_pause(env: Env, admin: Address, duration: u64) {
        admin.require_auth();
        Self::require_role(&env, &admin, &[AdminRole::SuperAdmin, AdminRole::Guardian]);

        let now = env.ledger().timestamp();
        let pause_end = if duration > 0 { now + duration } else { 0 };

        env.storage().instance().set(&DataKey::State, &ContractState::Frozen);
        env.storage().instance().set(&DataKey::EmergencyPauseEnd, &pause_end);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("em_pause"),),
            (admin, duration, pause_end),
        );
    }

    /// Unpause contract (super admin only)
    /// 
    /// # Arguments
    /// * `admin` - Super admin unpausing
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        Self::require_role(&env, &admin, &[AdminRole::SuperAdmin]);

        env.storage().instance().set(&DataKey::State, &ContractState::Active);
        env.storage().instance().remove(&DataKey::EmergencyPauseEnd);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish((symbol_short!("unpause"),), admin);
    }

    /// Set contract to paused state (no new proposals)
    /// 
    /// # Arguments
    /// * `admin` - Admin pausing the contract
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        Self::require_role(&env, &admin, &[AdminRole::SuperAdmin, AdminRole::Guardian]);

        env.storage().instance().set(&DataKey::State, &ContractState::Paused);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish((symbol_short!("pause"),), admin);
    }

    // ==================== CONFIGURATION ====================

    /// Update default timelock duration
    /// 
    /// # Arguments
    /// * `admin` - Super admin
    /// * `duration` - New timelock duration in seconds
    pub fn update_timelock(env: Env, admin: Address, duration: u64) {
        admin.require_auth();
        Self::require_role(&env, &admin, &[AdminRole::SuperAdmin]);

        if duration == 0 {
            panic!("Timelock must be > 0");
        }

        env.storage().instance().set(&DataKey::DefaultTimelock, &duration);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("time_up"),),
            (admin, duration),
        );
    }

    /// Update required approvals
    /// 
    /// # Arguments
    /// * `admin` - Super admin
    /// * `required` - New number of required approvals
    pub fn update_required_approvals(env: Env, admin: Address, required: u32) {
        admin.require_auth();
        Self::require_role(&env, &admin, &[AdminRole::SuperAdmin]);

        if required == 0 {
            panic!("Required approvals must be > 0");
        }

        env.storage()
            .instance()
            .set(&DataKey::RequiredApprovals, &required);
        env.storage().instance().extend_ttl(100, 100);

        env.events().publish(
            (symbol_short!("app_up"),),
            (admin, required),
        );
    }

    // ==================== VIEW FUNCTIONS ====================

    /// Get current implementation address
    pub fn get_implementation(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Implementation)
            .unwrap_or_else(|| panic!("No implementation set"))
    }

    /// Get proposal details
    pub fn get_proposal(env: Env, proposal_id: u64) -> UpgradeProposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"))
    }

    /// Get contract state (with auto-unpause check)
    pub fn get_state(env: Env) -> u32 {
        // Trigger auto-unpause check
        Self::check_auto_unpause(&env);
        
        env.storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active) as u32
    }

    /// Internal function to check and auto-unpause if needed
    fn check_auto_unpause(env: &Env) {
        if let Some(state) = env.storage().instance().get::<DataKey, ContractState>(&DataKey::State) {
            if let ContractState::Frozen = state {
                if let Some(pause_end) = env.storage().instance().get::<DataKey, u64>(&DataKey::EmergencyPauseEnd) {
                    if pause_end > 0 && env.ledger().timestamp() >= pause_end {
                        // Auto-unpause if expired
                        env.storage().instance().set(&DataKey::State, &ContractState::Active);
                        env.storage().instance().remove(&DataKey::EmergencyPauseEnd);
                    }
                }
            }
        }
    }

    /// Get admin role
    pub fn get_admin_role(env: Env, admin: Address) -> u32 {
        let role: AdminRole = env
            .storage()
            .persistent()
            .get(&DataKey::AdminRole(admin))
            .unwrap_or_else(|| panic!("Address is not an admin"));
        role as u32
    }

    /// Get list of all admins
    pub fn get_admin_list(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AdminList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get current timelock end time
    pub fn get_timelock_end(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TimelockEnd)
            .unwrap_or(0)
    }

    /// Get emergency pause end time
    pub fn get_emergency_pause_end(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::EmergencyPauseEnd)
            .unwrap_or(0)
    }

    /// Get required approvals count
    pub fn get_required_approvals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::RequiredApprovals)
            .unwrap_or(1)
    }

    /// Get default timelock duration
    pub fn get_default_timelock(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::DefaultTimelock)
            .unwrap_or(86400) // 24 hours default
    }

    // ==================== INTERNAL HELPERS ====================

    fn require_admin(env: &Env, caller: &Address) {
        if !env
            .storage()
            .persistent()
            .has(&DataKey::AdminRole(caller.clone()))
        {
            panic!("Not an admin");
        }
    }

    fn require_role(env: &Env, caller: &Address, allowed: &[AdminRole]) {
        let role: AdminRole = env
            .storage()
            .persistent()
            .get(&DataKey::AdminRole(caller.clone()))
            .unwrap_or_else(|| panic!("Not an admin"));
        
        if !allowed.contains(&role) {
            panic!("Insufficient role");
        }
    }

    fn require_active_state(env: &Env) {
        let mut state: ContractState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(ContractState::Active);

        // Check for auto-unpause first
        if let ContractState::Frozen = state {
            if let Some(pause_end) = env.storage().instance().get::<DataKey, u64>(&DataKey::EmergencyPauseEnd) {
                if pause_end > 0 && env.ledger().timestamp() >= pause_end {
                    // Auto-unpause if expired
                    env.storage().instance().set(&DataKey::State, &ContractState::Active);
                    env.storage().instance().remove(&DataKey::EmergencyPauseEnd);
                    state = ContractState::Active;
                }
            }
        }

        match state {
            ContractState::Active => {}, // OK
            ContractState::Paused => {
                panic!("Contract is paused");
            },
            ContractState::Frozen => {
                panic!("Contract is frozen");
            },
        }
    }
}

mod test;
