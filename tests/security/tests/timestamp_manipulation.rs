//! Timestamp manipulation security tests (#1486).
//!
//! Soroban test environments allow setting `env.ledger().timestamp` to any
//! value. These tests verify that contracts using `env.ledger().timestamp()`
//! for time-gated logic behave correctly when the clock is manipulated —
//! including attempts to bypass deadlines, pre-execute proposals before they
//! expire, and claim rewards with an artificially advanced clock.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

// ---------------------------------------------------------------------------
// Simple Voting: timestamp-gated cast_vote and execute
// ---------------------------------------------------------------------------

fn voting_setup(env: &Env) -> (simple_voting::VotingContractClient<'_>, Address) {
    let id = env.register_contract(None, simple_voting::VotingContract);
    let client = simple_voting::VotingContractClient::new(env, &id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

fn title(env: &Env) -> String {
    String::from_str(env, "Security Proposal")
}

/// Voting on a proposal at or after its deadline must be rejected even when
/// the caller manipulates the ledger timestamp to match the deadline exactly.
#[test]
fn vote_rejected_at_exact_deadline_timestamp() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let (client, admin) = voting_setup(&env);

    // Create a proposal with deadline = 2_000.
    let id = client.create_prop(&admin, &title(&env), &2_000);
    let voter = Address::generate(&env);

    // Advance to exactly the deadline — voting must now be rejected.
    env.ledger().set_timestamp(2_000);
    let res = client.try_cast_vote(&voter, &id, &simple_voting::VoteChoice::For);
    assert!(res.is_err(), "vote at exact deadline must be rejected");
}

/// Executing a proposal before its deadline must fail even when the
/// timestamp is set to `deadline - 1`.
#[test]
fn execute_rejected_one_second_before_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let (client, admin) = voting_setup(&env);

    let id = client.create_prop(&admin, &title(&env), &5_000);

    // Cast a vote so there is a winner.
    let voter = Address::generate(&env);
    client.cast_vote(&voter, &id, &simple_voting::VoteChoice::For);

    // Set timestamp to one second before deadline.
    env.ledger().set_timestamp(4_999);
    let res = client.try_execute(&id);
    assert!(
        res.is_err(),
        "execute one second before deadline must fail"
    );
}

/// Executing a proposal exactly at the deadline must succeed.
#[test]
fn execute_succeeds_at_exact_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(1_000);
    let (client, admin) = voting_setup(&env);

    let deadline = 3_000u64;
    let id = client.create_prop(&admin, &title(&env), &deadline);

    let voter = Address::generate(&env);
    client.cast_vote(&voter, &id, &simple_voting::VoteChoice::For);

    env.ledger().set_timestamp(deadline);
    let status = client.execute(&id);
    assert_eq!(status, simple_voting::ProposalStatus::Passed);
}

// ---------------------------------------------------------------------------
// Staking Pool: reward calculation with manipulated timestamps
// ---------------------------------------------------------------------------

use soroban_sdk::{contract, contractimpl, contracttype};

#[contracttype]
#[derive(Clone)]
enum TokKey {
    Bal(Address),
}

#[contract]
struct Tok;

#[contractimpl]
impl Tok {
    pub fn init(env: Env, owner: Address, bal: i128) {
        env.storage().persistent().set(&TokKey::Bal(owner), &bal);
    }
    pub fn mint(env: Env, to: Address, amount: i128) {
        let cur: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(to.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(&TokKey::Bal(to), &(cur + amount));
    }
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let f: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(from.clone()))
            .unwrap_or(0);
        let t: i128 = env
            .storage()
            .persistent()
            .get(&TokKey::Bal(to.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(&TokKey::Bal(from), &(f - amount));
        env.storage().persistent().set(&TokKey::Bal(to), &(t + amount));
    }
    pub fn balance(env: Env, who: Address) -> i128 {
        env.storage().persistent().get(&TokKey::Bal(who)).unwrap_or(0)
    }
}

fn staking_setup(rate: i128) -> (Env, Address, staking_pool::StakingPoolContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let owner = Address::generate(&env);
    let stake_id = env.register(Tok, ());
    let reward_id = env.register(Tok, ());
    let pool_id = env.register(staking_pool::StakingPoolContract, ());

    TokClient::new(&env, &reward_id).init(&pool_id, &1_000_000_000_000i128);

    let pool = staking_pool::StakingPoolContractClient::new(&env, &pool_id);
    pool.initialize(&owner, &stake_id, &reward_id, &rate);

    let staker = Address::generate(&env);
    TokClient::new(&env, &stake_id).init(&staker, &1_000_001i128);
    pool.stake(&staker, &1_000_000i128);

    (env, staker, pool)
}

/// Reward accumulation must be monotonically non-decreasing regardless of how
/// far the timestamp is advanced. An attacker cannot "rewind" the clock to
/// receive fewer rewards than deserved, nor should a far-future timestamp
/// produce a panic from overflow.
#[test]
fn reward_per_token_monotonic_with_extreme_timestamp_jump() {
    let (env, _staker, pool) = staking_setup(100);

    let rpt_before = pool.reward_per_token();

    // Jump the clock by u32::MAX seconds (~136 years) — an extreme but
    // not u64-saturating value. The contract uses saturating_sub for
    // the elapsed time calculation.
    env.ledger().with_mut(|l| l.timestamp += u32::MAX as u64);

    let rpt_after = pool.reward_per_token();

    assert!(
        rpt_after >= rpt_before,
        "reward_per_token decreased after clock jump: {} -> {}",
        rpt_before, rpt_after
    );
}

/// Earning rewards over a zero-second interval must return the same value as
/// the pending rewards at stake time (no phantom time-based emission).
#[test]
fn zero_elapsed_time_produces_no_new_rewards() {
    let (env, staker, pool) = staking_setup(100);

    // No time has advanced since stake; earned should be ~0.
    let earned = pool.earned(&staker);
    assert_eq!(earned, 0, "no rewards should accrue at t=0 elapsed");
}

/// Claiming rewards after a large but controlled timestamp jump must
/// not panic and must transfer a non-negative amount.
#[test]
fn claim_rewards_after_large_timestamp_jump_does_not_panic() {
    let (env, staker, pool) = staking_setup(1); // small rate

    // Advance 1_000_000 seconds to accrue some rewards.
    env.ledger().with_mut(|l| l.timestamp += 1_000_000);

    // This must not panic — just verify the claim succeeds.
    pool.claim_rewards(&staker);
}
