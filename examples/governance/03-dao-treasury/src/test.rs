#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

// ---------------------------------------------------------------------------
// Test helper
// ---------------------------------------------------------------------------

struct Ctx {
    env: Env,
    admin: Address,
    contract_id: Address,
}

impl Ctx {
    fn new(min_quorum: i128, voting_duration: u32, exec_duration: u32) -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(DaoContract, ());
        let client = DaoContractClient::new(&env, &contract_id);
        client.initialize(&admin, &min_quorum, &voting_duration, &exec_duration);

        Ctx {
            env,
            admin,
            contract_id,
        }
    }

    fn client(&self) -> DaoContractClient<'_> {
        DaoContractClient::new(&self.env, &self.contract_id)
    }

    fn advance(&self, delta: u32) {
        self.env.ledger().with_mut(|l| l.sequence_number += delta);
    }
}

// ---------------------------------------------------------------------------
// 1. Initialization tests
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_success() {
    let ctx = Ctx::new(100, 50, 100);
    let client = ctx.client();

    assert_eq!(client.admin(), Some(ctx.admin.clone()));
    assert_eq!(client.proposal_count(), 0);
    assert_eq!(client.treasury_balance(), 0);
}

#[test]
fn test_initialize_duplicate_fails() {
    let ctx = Ctx::new(100, 50, 100);
    let client = ctx.client();

    let res = client.try_initialize(&ctx.admin, &100i128, &50u32, &100u32);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 2. Treasury tests
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_increases_balance() {
    let ctx = Ctx::new(10, 10, 20);
    let client = ctx.client();
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    assert_eq!(client.treasury_balance(), 1000);

    client.deposit(&depositor, &500i128);
    assert_eq!(client.treasury_balance(), 1500);
}

#[test]
fn test_deposit_zero_fails() {
    let ctx = Ctx::new(10, 10, 20);
    let client = ctx.client();
    let depositor = Address::generate(&ctx.env);

    let res = client.try_deposit(&depositor, &0i128);
    assert!(res.is_err());
}

#[test]
fn test_deposit_negative_fails() {
    let ctx = Ctx::new(10, 10, 20);
    let client = ctx.client();
    let depositor = Address::generate(&ctx.env);

    let res = client.try_deposit(&depositor, &-100i128);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 3. Proposal submission tests
// ---------------------------------------------------------------------------

#[test]
fn test_propose_transfer_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &500i128);

    assert_eq!(id, 0);
    assert_eq!(client.proposal_count(), 1);

    let prop = client.get_proposal(&0);
    assert_eq!(prop.proposer, proposer);
    assert_eq!(prop.kind, ProposalKind::Transfer);
    assert_eq!(prop.state, ProposalState::Active);
    assert_eq!(prop.votes_yes, 0);
    assert_eq!(prop.votes_no, 0);
    assert_eq!(prop.transfer_amount, 500);
}

#[test]
fn test_propose_upgrade_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let hash = BytesN::from_array(&ctx.env, &[1u8; 32]);

    let id = client.propose_upgrade(&proposer, &hash);

    assert_eq!(id, 0);
    let prop = client.get_proposal(&0);
    assert_eq!(prop.kind, ProposalKind::Upgrade);
    assert_eq!(prop.state, ProposalState::Active);
}

#[test]
fn test_multiple_proposals_have_sequential_ids() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id0 = client.propose_transfer(&proposer, &recipient, &100i128);
    let id1 = client.propose_transfer(&proposer, &recipient, &200i128);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(client.proposal_count(), 2);
}

#[test]
fn test_propose_transfer_zero_amount_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let res = client.try_propose_transfer(&proposer, &recipient, &0i128);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 4. Voting tests
// ---------------------------------------------------------------------------

#[test]
fn test_vote_approve_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &50i128);

    let prop = client.get_proposal(&id);
    assert_eq!(prop.votes_yes, 50);
    assert_eq!(prop.votes_no, 0);
    assert!(client.has_voted(&id, &voter));
}

#[test]
fn test_vote_reject_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &false, &30i128);

    let prop = client.get_proposal(&id);
    assert_eq!(prop.votes_yes, 0);
    assert_eq!(prop.votes_no, 30);
}

#[test]
fn test_double_vote_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &50i128);

    let res = client.try_vote(&voter, &id, &true, &50i128);
    assert!(res.is_err());
}

#[test]
fn test_vote_after_window_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);

    // Advance past the voting window
    ctx.advance(60);

    let res = client.try_vote(&voter, &id, &true, &50i128);
    assert!(res.is_err());
}

#[test]
fn test_vote_zero_weight_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    let res = client.try_vote(&voter, &id, &true, &0i128);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 5. Proposal state resolution tests
// ---------------------------------------------------------------------------

#[test]
fn test_proposal_passes_with_quorum() {
    let ctx = Ctx::new(50, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    // Vote yes with weight 100 (above quorum of 50)
    client.vote(&voter, &id, &true, &100i128);

    // Advance past voting window
    ctx.advance(51);

    assert_eq!(client.proposal_state(&id), ProposalState::Passed);
}

#[test]
fn test_proposal_fails_below_quorum() {
    let ctx = Ctx::new(200, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    // Only 50 votes cast – below quorum of 200
    client.vote(&voter, &id, &true, &50i128);

    ctx.advance(51);

    assert_eq!(client.proposal_state(&id), ProposalState::Failed);
}

#[test]
fn test_proposal_fails_when_no_votes_outweigh_yes() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter_yes = Address::generate(&ctx.env);
    let voter_no = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter_yes, &id, &true, &40i128);
    client.vote(&voter_no, &id, &false, &60i128);

    ctx.advance(51);

    assert_eq!(client.proposal_state(&id), ProposalState::Failed);
}

#[test]
fn test_proposal_expires_after_exec_window() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &100i128);

    // Advance past voting_end + exec_duration
    ctx.advance(200);

    assert_eq!(client.proposal_state(&id), ProposalState::Expired);
}

// ---------------------------------------------------------------------------
// 6. Execution tests
// ---------------------------------------------------------------------------

#[test]
fn test_execute_transfer_proposal_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    // Fund the treasury
    client.deposit(&depositor, &1000i128);

    let id = client.propose_transfer(&proposer, &recipient, &300i128);
    client.vote(&voter, &id, &true, &100i128);

    // Advance past voting window
    ctx.advance(51);

    client.execute(&executor, &id);

    assert_eq!(client.treasury_balance(), 700);
    assert_eq!(client.proposal_state(&id), ProposalState::Executed);
}

#[test]
fn test_execute_fails_when_voting_not_ended() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &100i128);

    // Voting still active – execution should fail
    let res = client.try_execute(&executor, &id);
    assert!(res.is_err());
}

#[test]
fn test_execute_fails_insufficient_treasury() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    // Fund treasury with only 50
    client.deposit(&depositor, &50i128);

    let id = client.propose_transfer(&proposer, &recipient, &200i128);
    client.vote(&voter, &id, &true, &100i128);

    ctx.advance(51);

    let res = client.try_execute(&executor, &id);
    assert!(res.is_err());
}

#[test]
fn test_execute_fails_when_proposal_failed() {
    let ctx = Ctx::new(1000, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    // No votes → quorum not met → Failed
    ctx.advance(51);

    let res = client.try_execute(&executor, &id);
    assert!(res.is_err());
}

#[test]
fn test_execute_fails_when_already_executed() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &100i128);
    ctx.advance(51);
    client.execute(&executor, &id);

    // Attempt to execute again
    let res = client.try_execute(&executor, &id);
    assert!(res.is_err());
}

#[test]
fn test_execute_fails_after_exec_window_expires() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &100i128);

    // Advance past both voting AND execution window
    ctx.advance(200);

    let res = client.try_execute(&executor, &id);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 7. Cancellation tests
// ---------------------------------------------------------------------------

#[test]
fn test_cancel_by_proposer_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.cancel(&proposer, &id);

    assert_eq!(client.proposal_state(&id), ProposalState::Cancelled);
}

#[test]
fn test_cancel_by_admin_success() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.cancel(&ctx.admin, &id);

    assert_eq!(client.proposal_state(&id), ProposalState::Cancelled);
}

#[test]
fn test_cancel_by_unauthorized_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let random = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    let res = client.try_cancel(&random, &id);
    assert!(res.is_err());
}

#[test]
fn test_cancel_executed_proposal_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter, &id, &true, &100i128);
    ctx.advance(51);
    client.execute(&executor, &id);

    let res = client.try_cancel(&proposer, &id);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 8. Not-initialized guard
// ---------------------------------------------------------------------------

#[test]
fn test_actions_fail_without_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DaoContract, ());
    let client = DaoContractClient::new(&env, &contract_id);
    let proposer = Address::generate(&env);
    let recipient = Address::generate(&env);

    let res = client.try_propose_transfer(&proposer, &recipient, &100i128);
    assert!(res.is_err());
}

// ---------------------------------------------------------------------------
// 9. Edge case – equal yes/no votes (ties should fail)
// ---------------------------------------------------------------------------

#[test]
fn test_tied_votes_fail() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter_yes = Address::generate(&ctx.env);
    let voter_no = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);

    let id = client.propose_transfer(&proposer, &recipient, &100i128);
    client.vote(&voter_yes, &id, &true, &50i128);
    client.vote(&voter_no, &id, &false, &50i128);

    ctx.advance(51);

    // Tie → votes_yes NOT strictly > votes_no → Failed
    assert_eq!(client.proposal_state(&id), ProposalState::Failed);
}

// ---------------------------------------------------------------------------
// 10. Multiple voters weighted correctly
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_voters_weighted_correctly() {
    let ctx = Ctx::new(100, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let v1 = Address::generate(&ctx.env);
    let v2 = Address::generate(&ctx.env);
    let v3 = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);
    let id = client.propose_transfer(&proposer, &recipient, &50i128);

    client.vote(&v1, &id, &true, &60i128);
    client.vote(&v2, &id, &false, &20i128);
    client.vote(&v3, &id, &true, &50i128);

    let prop = client.get_proposal(&id);
    assert_eq!(prop.votes_yes, 110);
    assert_eq!(prop.votes_no, 20);

    ctx.advance(51);
    assert_eq!(client.proposal_state(&id), ProposalState::Passed);
}

// ---------------------------------------------------------------------------
// 11. Treasury balance after multiple executions
// ---------------------------------------------------------------------------

#[test]
fn test_treasury_balance_after_multiple_transfers() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();
    let proposer = Address::generate(&ctx.env);
    let voter = Address::generate(&ctx.env);
    let executor = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let depositor = Address::generate(&ctx.env);

    client.deposit(&depositor, &1000i128);

    // First proposal: transfer 200
    let id0 = client.propose_transfer(&proposer, &recipient, &200i128);
    client.vote(&voter, &id0, &true, &100i128);
    ctx.advance(51);
    client.execute(&executor, &id0);

    assert_eq!(client.treasury_balance(), 800);

    // Second proposal: transfer 300
    ctx.advance(1); // new ledger for next proposal window
    let id1 = client.propose_transfer(&proposer, &recipient, &300i128);
    let voter2 = Address::generate(&ctx.env);
    client.vote(&voter2, &id1, &true, &100i128);
    ctx.advance(51);
    client.execute(&executor, &id1);

    assert_eq!(client.treasury_balance(), 500);
}

// ---------------------------------------------------------------------------
// 12. Proposal not found error
// ---------------------------------------------------------------------------

#[test]
fn test_get_nonexistent_proposal_fails() {
    let ctx = Ctx::new(10, 50, 100);
    let client = ctx.client();

    let res = client.try_get_proposal(&999u32);
    assert!(res.is_err());
}
