use soroban_sdk::{symbol_short, Address, Env};
use soroban_sdk::testutils::{Address as _, Ledger as _};

use crate::{
    AdminRole, ContractState, ProposalStatus, ProxyUpgradeControls, ProxyUpgradeControlsClient,
    UpgradeProposal,
};

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let implementation = Address::generate(&env);
    let timelock = 86400; // 24 hours
    let required_approvals = 2;

    // Initialize contract
    client.initialize(&admin, &implementation, &timelock, &required_approvals);

    // Verify initialization
    assert_eq!(client.get_implementation(), implementation);
    assert_eq!(client.get_state(), ContractState::Active as u32);
    assert_eq!(client.get_default_timelock(), timelock);
    assert_eq!(client.get_required_approvals(), required_approvals);
    assert_eq!(client.get_admin_role(admin), AdminRole::SuperAdmin as u32);

    let admin_list = client.get_admin_list();
    assert_eq!(admin_list.len(), 1);
    assert_eq!(admin_list.get(0), admin);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let implementation = Address::generate(&env);
    let timelock = 86400;
    let required_approvals = 2;

    // Initialize twice should panic
    client.initialize(&admin, &implementation, &timelock, &required_approvals);
    client.initialize(&admin, &implementation, &timelock, &required_approvals);
}

#[test]
fn test_admin_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let guardian = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Add admins
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);
    client.add_admin(&super_admin, &guardian, &AdminRole::Guardian);

    // Verify roles
    assert_eq!(client.get_admin_role(upgrader), AdminRole::Upgrader as u32);
    assert_eq!(client.get_admin_role(guardian), AdminRole::Guardian as u32);

    // Verify admin list
    let admin_list = client.get_admin_list();
    assert_eq!(admin_list.len(), 3);

    // Remove admin
    client.remove_admin(&super_admin, &guardian);
    let admin_list = client.get_admin_list();
    assert_eq!(admin_list.len(), 2);
}

#[test]
fn test_proposal_lifecycle() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let new_implementation = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &3600, &2); // 1 hour timelock
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Create proposal
    let proposal_id = client.create_proposal(
        &upgrader,
        &new_implementation,
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );

    // Check proposal
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.proposer, upgrader);
    assert_eq!(proposal.new_implementation, new_implementation);
    assert_eq!(proposal.status, ProposalStatus::Pending as u32);
    assert_eq!(proposal.approvals.len(), 0);
    assert_eq!(proposal.rejections.len(), 0);

    // Approve proposal (first approval)
    client.approve_proposal(&super_admin, &proposal_id);
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Pending as u32);
    assert_eq!(proposal.approvals.len(), 1);

    // Approve proposal (second approval - should move to approved)
    client.approve_proposal(&upgrader, &proposal_id);
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Approved as u32);
    assert_eq!(proposal.approvals.len(), 2);
    assert!(proposal.approved_at > 0);

    // Fast forward time past timelock
    env.ledger().set_timestamp(env.ledger().timestamp() + 3700);

    // Execute proposal
    client.execute_proposal(&super_admin, &proposal_id);
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Executed as u32);
    assert_eq!(client.get_implementation(), new_implementation);
}

#[test]
fn test_proposal_rejection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let new_implementation = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &3600, &2);
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Create proposal
    let proposal_id = client.create_proposal(
        &upgrader,
        &new_implementation,
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );

    // Reject proposal
    client.reject_proposal(&super_admin, &proposal_id);
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Rejected as u32);
    assert_eq!(proposal.rejections.len(), 1);
}

#[test]
fn test_proposal_cancellation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let new_implementation = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &3600, &2);
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Create proposal
    let proposal_id = client.create_proposal(
        &upgrader,
        &new_implementation,
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );

    // Cancel proposal
    client.cancel_proposal(&upgrader, &proposal_id);
    let proposal = client.get_proposal(proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Cancelled as u32);
}

#[test]
#[should_panic(expected = "Timelock not elapsed")]
fn test_execute_before_timelock() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let new_implementation = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &3600, &2);
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Create and approve proposal
    let proposal_id = client.create_proposal(
        &upgrader,
        &new_implementation,
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );

    client.approve_proposal(&super_admin, &proposal_id);
    client.approve_proposal(&upgrader, &proposal_id);

    // Try to execute before timelock (should panic)
    client.execute_proposal(&super_admin, &proposal_id);
}

#[test]
fn test_emergency_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let guardian = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Add guardian
    client.add_admin(&super_admin, &guardian, &AdminRole::Guardian);

    // Emergency pause
    client.emergency_pause(&guardian, &3600); // 1 hour pause
    assert_eq!(client.get_state(), ContractState::Frozen as u32);
    assert!(client.get_emergency_pause_end() > 0);

    // Unpause
    client.unpause(&super_admin);
    assert_eq!(client.get_state(), ContractState::Active as u32);
    assert_eq!(client.get_emergency_pause_end(), 0);
}

#[test]
fn test_auto_unpause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Emergency pause with short duration
    client.emergency_pause(&super_admin, &10); // 10 seconds
    assert_eq!(client.get_state(), ContractState::Frozen as u32);

    // Fast forward past pause duration
    env.ledger().set_timestamp(env.ledger().timestamp() + 15);

    // Check if auto-unpaused
    assert_eq!(client.get_state(), ContractState::Active as u32);
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_operations_while_paused() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Pause contract
    client.pause(&super_admin);

    // Try to create proposal (should panic)
    client.create_proposal(
        &upgrader,
        &Address::generate(&env),
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );
}

#[test]
fn test_role_based_access() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);
    let guardian = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Add other roles
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);
    client.add_admin(&super_admin, &guardian, &AdminRole::Guardian);

    // Test upgrader can create proposals
    let proposal_id = client.create_proposal(
        &upgrader,
        &Address::generate(&env),
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );
    assert!(proposal_id > 0);

    // Test guardian can emergency pause
    client.emergency_pause(&guardian, &3600);
    assert_eq!(client.get_state(), ContractState::Frozen as u32);

    // Unpause for next test
    client.unpause(&super_admin);

    // Test configuration updates (super admin only)
    client.update_timelock(&super_admin, &7200);
    assert_eq!(client.get_default_timelock(), 7200);

    client.update_required_approvals(&super_admin, &3);
    assert_eq!(client.get_required_approvals(), 3);
}

#[test]
#[should_panic(expected = "Cannot remove the last super admin")]
fn test_cannot_remove_last_super_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Try to remove the only super admin (should panic)
    client.remove_admin(&super_admin, &super_admin);
}

#[test]
fn test_double_vote_prevention() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let upgrader = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Add upgrader
    client.add_admin(&super_admin, &upgrader, &AdminRole::Upgrader);

    // Create proposal
    let proposal_id = client.create_proposal(
        &upgrader,
        &Address::generate(&env),
        &symbol_short!("test_upgrade"),
        &symbol_short!("QmHash123"),
    );

    // Approve once
    client.approve_proposal(&super_admin, &proposal_id);

    // Try to approve again (should panic)
    client.approve_proposal(&super_admin, &proposal_id);
}

#[test]
fn test_configuration_updates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Update timelock
    client.update_timelock(&super_admin, &43200); // 12 hours
    assert_eq!(client.get_default_timelock(), 43200);

    // Update required approvals
    client.update_required_approvals(&super_admin, &3);
    assert_eq!(client.get_required_approvals(), 3);
}

#[test]
#[should_panic(expected = "Timelock must be > 0")]
fn test_invalid_timelock() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &Address::generate(&env), &86400, &2);
    env.mock_all_auths();

    // Try to set zero timelock (should panic)
    client.update_timelock(&super_admin, &0);
}

#[test]
fn test_view_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProxyUpgradeControls);
    let client = ProxyUpgradeControlsClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    let implementation = Address::generate(&env);

    // Initialize
    client.initialize(&super_admin, &implementation, &86400, &2);

    // Test view functions
    assert_eq!(client.get_implementation(), implementation);
    assert_eq!(client.get_state(), ContractState::Active as u32);
    assert_eq!(client.get_admin_role(super_admin), AdminRole::SuperAdmin as u32);
    assert_eq!(client.get_admin_list().len(), 1);
    assert_eq!(client.get_timelock_end(), 0);
    assert_eq!(client.get_emergency_pause_end(), 0);
    assert_eq!(client.get_required_approvals(), 2);
    assert_eq!(client.get_default_timelock(), 86400);
}
