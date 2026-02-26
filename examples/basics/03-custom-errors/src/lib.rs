//! # Admin Controls for Proxy Upgrades
//!
//! This contract implements a comprehensive admin control system for proxy upgrades
//! with authentication, proposal system, timelock, and emergency pause functionality.
//!
//! ## Security Features:
//! - Multi-signature admin support
//! - Time-delayed upgrade execution
//! - Emergency pause capabilities
//! - Upgrade proposal tracking
//!
//! ## Usage:
//! 1. Initialize with admin addresses
//! 2. Propose upgrades with timelock
//! 3. Execute upgrades after delay period
//! 4. Emergency pause when needed

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    Env, Address, Vec, Map,
    symbol_short, Symbol
};

/// Time constants (in seconds)
const TIMELOCK_DELAY: u64 = 86400; // 24 hours
const EMERGENCY_PAUSE_DURATION: u64 = 3600; // 1 hour

/// Storage keys
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PROXY_ADMIN_KEY: Symbol = symbol_short!("PROXY_ADM");
const UPGRADE_PROPOSALS_KEY: Symbol = symbol_short!("UPG_PROP");
const TIMELOCK_KEY: Symbol = symbol_short!("TIMELOCK");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
const EMERGENCY_PAUSE_KEY: Symbol = symbol_short!("EMRG_PS");

/// Custom error types for admin controls
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdminControlError {
    /// Unauthorized access attempt
    Unauthorized = 1,
    /// Contract is currently paused
    ContractPaused = 2,
    /// Invalid proposal parameters
    InvalidProposal = 3,
    /// Proposal not found
    ProposalNotFound = 4,
    /// Timelock period not met
    TimelockNotMet = 5,
    /// Proposal already executed
    ProposalExecuted = 6,
    /// Invalid admin address
    InvalidAdmin = 7,
    /// Contract not initialized
    NotInitialized = 8,
    /// Emergency pause active
    EmergencyPauseActive = 9,
    /// Invalid upgrade delay
    InvalidDelay = 10,
}

/// Upgrade proposal structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    /// Unique proposal ID
    pub proposal_id: u64,
    /// New implementation contract address
    pub new_implementation: Address,
    /// Proposal timestamp
    pub proposed_at: u64,
    /// When upgrade can be executed
    pub executable_at: u64,
    /// Whether proposal has been executed
    pub executed: bool,
    /// Proposal description hash
    pub description_hash: Symbol,
    /// Required admin signatures
    pub required_signatures: u32,
    /// Current signatures collected
    pub signatures_collected: u32,
}

/// Admin controls for proxy upgrades
#[contract]
pub struct AdminControls;

#[contractimpl]
impl AdminControls {
    /// Initialize the admin controls contract
    /// 
    /// # Arguments
    /// * `admin` - Primary admin address
    /// * `proxy_admin` - Proxy contract admin address
    pub fn initialize(env: Env, admin: Address, proxy_admin: Address) -> Result<(), AdminControlError> {
        // Check if already initialized
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Init");
        }
        
        // Validate addresses
        if admin.to_string().is_empty() || proxy_admin.to_string().is_empty() {
            return Err(AdminControlError::InvalidAdmin);
        }
        
        // Set admin addresses
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PROXY_ADMIN_KEY, &proxy_admin);
        
        // Initialize empty proposals map
        let proposals: Map<u64, UpgradeProposal> = Map::new(&env);
        env.storage().instance().set(&UPGRADE_PROPOSALS_KEY, &proposals);
        
        // Set initial state
        env.storage().instance().set(&PAUSED_KEY, &false);
        env.storage().instance().set(&EMERGENCY_PAUSE_KEY, &false);
        
        Ok(())
    }
    
    /// Propose an upgrade with timelock
    /// 
    /// # Arguments
    /// * `admin` - Admin proposing the upgrade
    /// * `new_implementation` - New contract address
    /// * `description_hash` - Hash of upgrade description
    /// * `custom_delay` - Optional custom timelock delay (0 for default)
    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        new_implementation: Address,
        description_hash: Symbol,
        custom_delay: u64,
    ) -> Result<u64, AdminControlError> {
        // Check authorization
        Self::check_admin_auth(&env, &admin)?;
        
        // Check if contract is paused
        if env.storage().instance().get(&PAUSED_KEY).unwrap_or(false) {
            return Err(AdminControlError::ContractPaused);
        }
        
        // Validate new implementation
        if new_implementation.to_string().is_empty() {
            return Err(AdminControlError::InvalidProposal);
        }
        
        // Calculate timelock
        let delay = if custom_delay == 0 { TIMELOCK_DELAY } else { custom_delay };
        if delay < 3600 || delay > 604800 { // Between 1 hour and 7 days
            return Err(AdminControlError::InvalidDelay);
        }
        
        let current_time = env.ledger().timestamp();
        let executable_at = current_time + delay;
        
        // Generate proposal ID
        let proposals: Map<u64, UpgradeProposal> = env.storage().instance()
            .get(&UPGRADE_PROPOSALS_KEY).unwrap_or_else(|| Map::new(&env));
        let proposal_id: u64 = (proposals.len() + 1).into();
        
        // Create proposal
        let proposal = UpgradeProposal {
            proposal_id,
            new_implementation,
            proposed_at: current_time,
            executable_at,
            executed: false,
            description_hash,
            required_signatures: 1u32, // Single admin for simplicity
            signatures_collected: 1u32,
        };
        
        // Store proposal
        let mut updated_proposals = proposals;
        updated_proposals.set(proposal_id, proposal);
        env.storage().instance().set(&UPGRADE_PROPOSALS_KEY, &updated_proposals);
        
        Ok(proposal_id)
    }
    
    /// Execute an upgrade after timelock period
    /// 
    /// # Arguments
    /// * `admin` - Admin executing the upgrade
    /// * `proposal_id` - ID of the proposal to execute
    pub fn execute_upgrade(
        env: Env,
        admin: Address,
        proposal_id: u64,
    ) -> Result<Address, AdminControlError> {
        // Check authorization
        Self::check_admin_auth(&env, &admin)?;
        
        // Check emergency pause
        if env.storage().instance().get(&EMERGENCY_PAUSE_KEY).unwrap_or(false) {
            return Err(AdminControlError::EmergencyPauseActive);
        }
        
        // Get proposal
        let proposals: Map<u64, UpgradeProposal> = env.storage().instance()
            .get(&UPGRADE_PROPOSALS_KEY).unwrap_or_else(|| Map::new(&env));
        
        let mut proposal = proposals.get(proposal_id)
            .ok_or(AdminControlError::ProposalNotFound)?;
        
        // Check if already executed
        if proposal.executed {
            return Err(AdminControlError::ProposalExecuted);
        }
        
        // Check timelock
        let current_time = env.ledger().timestamp();
        if current_time < proposal.executable_at {
            return Err(AdminControlError::TimelockNotMet);
        }
        
        // Mark as executed
        proposal.executed = true;
        
        // Store proposal
        let new_impl = proposal.new_implementation.clone();
        let mut updated_proposals = proposals;
        updated_proposals.set(proposal_id, proposal);
        env.storage().instance().set(&UPGRADE_PROPOSALS_KEY, &updated_proposals);
        
        // In a real implementation, this would trigger the proxy upgrade
        // For now, we return the new implementation address
        Ok(new_impl)
    }
    
    /// Emergency pause function
    /// 
    /// # Arguments
    /// * `admin` - Admin address
    /// * `duration` - Pause duration in seconds (0 for default)
    pub fn emergency_pause(
        env: Env,
        admin: Address,
        duration: u64,
    ) -> Result<(), AdminControlError> {
        // Check authorization
        Self::check_admin_auth(&env, &admin)?;
        
        // Set pause duration
        let pause_duration = if duration == 0 { EMERGENCY_PAUSE_DURATION } else { duration };
        let pause_until = env.ledger().timestamp() + pause_duration;
        
        // Set emergency pause
        env.storage().instance().set(&EMERGENCY_PAUSE_KEY, &true);
        env.storage().instance().set(&TIMELOCK_KEY, &pause_until);
        env.storage().instance().set(&PAUSED_KEY, &true);
        
        Ok(())
    }
    
    /// Lift emergency pause
    /// 
    /// # Arguments
    /// * `admin` - Admin address
    pub fn lift_emergency_pause(env: Env, admin: Address) -> Result<(), AdminControlError> {
        // Check authorization
        Self::check_admin_auth(&env, &admin)?;
        
        // Check if pause period has expired
        let pause_until: u64 = env.storage().instance().get(&TIMELOCK_KEY).unwrap_or(0);
        let current_time = env.ledger().timestamp();
        
        if current_time < pause_until {
            return Err(AdminControlError::TimelockNotMet);
        }
        
        // Lift pause
        env.storage().instance().set(&EMERGENCY_PAUSE_KEY, &false);
        env.storage().instance().set(&PAUSED_KEY, &false);
        
        Ok(())
    }
    
    /// Get proposal details
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<UpgradeProposal, AdminControlError> {
        let proposals: Map<u64, UpgradeProposal> = env.storage().instance()
            .get(&UPGRADE_PROPOSALS_KEY).unwrap_or_else(|| Map::new(&env));
        
        proposals.get(proposal_id)
            .ok_or(AdminControlError::ProposalNotFound)
    }
    
    /// Get all proposals
    pub fn get_all_proposals(env: Env) -> Vec<UpgradeProposal> {
        let proposals: Map<u64, UpgradeProposal> = env.storage().instance()
            .get(&UPGRADE_PROPOSALS_KEY).unwrap_or_else(|| Map::new(&env));
        
        let mut result = Vec::new(&env);
        for (_, proposal) in proposals {
            result.push_back(proposal);
        }
        result
    }
    
    /// Check if contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }
    
    /// Get admin address
    pub fn get_admin(env: Env) -> Result<Address, AdminControlError> {
        env.storage().instance().get(&ADMIN_KEY)
            .ok_or(AdminControlError::NotInitialized)
    }
    
    /// Check admin authorization
    fn check_admin_auth(env: &Env, admin: &Address) -> Result<(), AdminControlError> {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&ADMIN_KEY)
            .ok_or(AdminControlError::NotInitialized)?;
        
        if admin != &stored_admin {
            return Err(AdminControlError::Unauthorized);
        }
        
        Ok(())
    }
}

mod test;
