//! Use-case coverage suite (issue #901).
//!
//! Integration tests for the use-case examples of the cookbook: DeFi, NFT,
//! governance and token contracts.  Every example is registered natively (no
//! Wasm binary required) so the suite runs with any Rust toolchain.
//!
//! Covered examples:
//! - `examples/defi/02-constant-product-amm`
//! - `examples/defi/14-automatic-snapshot-triggers`
//! - `examples/defi/15-reward-strategies`
//! - `examples/nfts/02-nft-metadata`
//! - `examples/nfts/03-nft-metadata-standards`
//! - `examples/nfts/05-automatic-snapshot-triggers`
//! - `examples/governance/06-vote-delegation`
//! - `examples/governance/09-timelock-governance`
//! - `examples/governance/10-dao-treasury`
//! - `examples/governance/11-automatic-snapshot-triggers`
//! - `examples/tokens/01-vesting-management`
//! - `examples/tokens/02-minting-strategies`
//! - `examples/tokens/05-allowance-pattern`
//! - `examples/tokens/06-reward-token`
//! - `examples/tokens/07-token-metadata`
//! - `examples/tokens/08-multi-token-balance-manager`
//! - `examples/tokens/10-custom-token`
//! - `examples/tokens/token-lock`

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, testutils::Ledger as _, token,
    Address, Env, IntoVal, String, Symbol, Vec,
};

/// Target contract used by the timelock-governance example.
#[contract]
pub struct TimelockTarget;

#[contractimpl]
impl TimelockTarget {
    pub fn run(env: Env, value: u32) -> u32 {
        env.storage().instance().set(&symbol_short!("ran"), &value);
        value
    }
}

// ---------------------------------------------------------------------------
// DeFi — constant product AMM
// ---------------------------------------------------------------------------

#[test]
fn test_constant_product_amm_add_liquidity_and_swap() {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let token_x = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_y = env.register_stellar_asset_contract_v2(issuer.clone());
    let asset_x = token::StellarAssetClient::new(&env, &token_x.address());
    let asset_y = token::StellarAssetClient::new(&env, &token_y.address());

    let id = env.register_contract(None, constant_product_amm::ConstantProductAmm);
    let amm = constant_product_amm::ConstantProductAmmClient::new(&env, &id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    asset_x.mint(&alice, &10_000);
    asset_y.mint(&alice, &10_000);
    asset_x.mint(&bob, &10_000);

    amm.initialize(&token_x.address(), &token_y.address());

    // The first provider sets the price ratio and receives the LP tokens.
    let liquidity = amm.add_liquidity(&alice, &10_000, &10_000);
    assert_eq!(liquidity, 10_000);
    assert_eq!(amm.lp_balance(&alice), 10_000);
    assert_eq!(amm.total_supply(), 10_000);
    assert_eq!(amm.reserves(), (10_000, 10_000));

    // Swapping applies the 0.3% pool fee.
    let out = amm.swap(&bob, &token_x.address(), &1_000, &0);
    let expected = (997i128 * 1_000 * 10_000) / (10_000 * 1_000 + 997 * 1_000);
    assert_eq!(out, expected);
    assert!(out < 1_000);
    assert_eq!(amm.reserves(), (11_000, 10_000 - out));
}

#[test]
fn test_constant_product_amm_rejects_bad_swaps() {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let token_x = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_y = env.register_stellar_asset_contract_v2(issuer.clone());
    let other = env.register_stellar_asset_contract_v2(issuer.clone());
    let asset_x = token::StellarAssetClient::new(&env, &token_x.address());
    let asset_y = token::StellarAssetClient::new(&env, &token_y.address());

    let id = env.register_contract(None, constant_product_amm::ConstantProductAmm);
    let amm = constant_product_amm::ConstantProductAmmClient::new(&env, &id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    asset_x.mint(&alice, &10_000);
    asset_y.mint(&alice, &10_000);
    asset_x.mint(&bob, &10_000);

    amm.initialize(&token_x.address(), &token_y.address());
    amm.add_liquidity(&alice, &10_000, &10_000);

    // Swapping an unknown token is rejected.
    assert_eq!(
        amm.try_swap(&bob, &other.address(), &100, &0),
        Err(Ok(constant_product_amm::AmmError::InvalidTokenPair))
    );

    // A slippage guard that cannot be met is rejected.
    assert_eq!(
        amm.try_swap(&bob, &token_x.address(), &1_000, &10_000),
        Err(Ok(constant_product_amm::AmmError::InsufficientOutputAmount))
    );

    // Non-positive amounts are rejected.
    assert_eq!(
        amm.try_swap(&bob, &token_x.address(), &0, &0),
        Err(Ok(constant_product_amm::AmmError::InvalidAmount))
    );

    // Adding liquidity with a different ratio is rejected.
    assert_eq!(
        amm.try_add_liquidity(&alice, &1_000, &2_000),
        Err(Ok(constant_product_amm::AmmError::RatioMismatch))
    );
}

// ---------------------------------------------------------------------------
// DeFi — automatic snapshot triggers
// ---------------------------------------------------------------------------

#[test]
fn test_snapshot_trigger_records_and_prunes_history() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, snapshot_triggers_defi::SnapshotTrigger);
    let client = snapshot_triggers_defi::SnapshotTriggerClient::new(&env, &id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &10);
    client.record_value(&user, &1_000);
    assert_eq!(client.snapshot_count(&user), 1);
    assert_eq!(client.get_latest(&user).value, 1_000);

    // The snapshot is only due once the configured frequency elapsed.
    assert_eq!(
        client.try_auto_snapshot(&user),
        Err(Ok(snapshot_triggers_defi::SnapshotError::NoSnapshotDue))
    );

    env.ledger().with_mut(|l| l.sequence_number = 10);
    client.auto_snapshot(&user);
    assert_eq!(client.snapshot_count(&user), 2);

    // A second automatic snapshot in the same window is not due.
    assert_eq!(
        client.try_auto_snapshot(&user),
        Err(Ok(snapshot_triggers_defi::SnapshotError::NoSnapshotDue))
    );

    // History can be pruned.
    let pruned = client.prune(&admin, &user, &5);
    assert_eq!(pruned, 1);
    assert_eq!(client.snapshot_count(&user), 1);
}

#[test]
fn test_snapshot_trigger_can_be_disabled() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, snapshot_triggers_nfts::SnapshotTrigger);
    let client = snapshot_triggers_nfts::SnapshotTriggerClient::new(&env, &id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &5);
    let (frequency, enabled, _last, _total) = client.get_config();
    assert_eq!(frequency, 5);
    assert!(enabled);

    client.set_enabled(&admin, &false);
    let (_frequency, enabled, _last, _total) = client.get_config();
    assert!(!enabled);

    // A non-admin cannot change the configuration.
    assert_eq!(
        client.try_set_frequency(&user, &7),
        Err(Ok(snapshot_triggers_nfts::SnapshotError::Unauthorized))
    );
}

#[test]
fn test_governance_snapshot_trigger_tracks_history() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, snapshot_triggers_governance::SnapshotTrigger);
    let client = snapshot_triggers_governance::SnapshotTriggerClient::new(&env, &id);
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin, &4);
    client.record_value(&voter, &42);
    client.record_value(&voter, &84);

    assert_eq!(client.snapshot_count(&voter), 2);
    let snapshots = client.get_all_snapshots(&voter);
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots.get(1).unwrap().value, 84);

    // The first snapshot can be addressed by index.
    let first = client.get_snapshot(&voter, &0);
    assert_eq!(first.value, 42);
    assert_eq!(
        client.try_get_snapshot(&voter, &5),
        Err(Ok(
            snapshot_triggers_governance::SnapshotError::IndexOutOfBounds
        ))
    );
}

// ---------------------------------------------------------------------------
// DeFi — reward strategies
// ---------------------------------------------------------------------------

#[test]
fn test_reward_strategies_distribute_over_time() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, reward_strategies::RewardDistributor);
    let client = reward_strategies::RewardDistributorClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(
        &admin,
        &reward_strategies::Strategy::Linear,
        &1_000,
        &0,
        &100,
        &0,
        &0,
    );

    client.register(&admin, &alice, &1);
    client.register(&admin, &bob, &3);

    // Halfway through the window half of the reward is unlocked.
    assert_eq!(client.claimable(&alice, &50), 500);
    assert_eq!(client.claimable(&bob, &50), 1_500);

    // The entitlement is capped at the total reward.
    assert_eq!(client.claimable(&alice, &1_000), 1_000);

    let claimed = client.claim(&alice, &100);
    assert_eq!(claimed, 1_000);
    assert_eq!(client.claimable(&alice, &100), 0);

    // Nothing left to claim.
    assert_eq!(
        client.try_claim(&alice, &100),
        Err(Ok(reward_strategies::RewardError::NothingToClaim))
    );
}

#[test]
fn test_reward_strategies_validate_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, reward_strategies::RewardDistributor);
    let client = reward_strategies::RewardDistributorClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.initialize(
        &admin,
        &reward_strategies::Strategy::Linear,
        &1_000,
        &0,
        &100,
        &0,
        &0,
    );

    // Only the admin can register participants.
    assert_eq!(
        client.try_register(&stranger, &alice, &1),
        Err(Ok(reward_strategies::RewardError::Unauthorized))
    );

    // A zero weight is invalid.
    assert_eq!(
        client.try_register(&admin, &alice, &0),
        Err(Ok(reward_strategies::RewardError::InvalidAmount))
    );

    client.register(&admin, &alice, &1);

    // Registering twice is rejected.
    assert_eq!(
        client.try_register(&admin, &alice, &1),
        Err(Ok(reward_strategies::RewardError::AlreadyRegistered))
    );

    // Unknown participants have no claimable amount.
    assert_eq!(
        client.try_claimable(&stranger, &10),
        Err(Ok(reward_strategies::RewardError::NotRegistered))
    );
}

// ---------------------------------------------------------------------------
// NFT — metadata
// ---------------------------------------------------------------------------

#[test]
fn test_nft_metadata_off_chain_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, nft_metadata_advanced::NFTMetadataContract);
    let client = nft_metadata_advanced::NFTMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "https://meta.example/"),
        &false,
    );
    assert!(!client.is_on_chain_metadata());

    let uri = Some(String::from_str(&env, "ipfs://token-1"));
    client.mint(&owner, &1, &uri, &None);
    assert_eq!(client.owner_of(&1), owner);
    assert_eq!(client.token_uri(&1), uri.clone().unwrap());
    assert_eq!(
        client.base_uri(),
        String::from_str(&env, "https://meta.example/")
    );

    // Minting the same token twice is rejected.
    assert_eq!(
        client.try_mint(&owner, &1, &uri, &None),
        Err(Ok(nft_metadata_advanced::NFTError::TokenAlreadyMinted))
    );

    // Token id zero is invalid.
    assert_eq!(
        client.try_mint(&owner, &0, &uri, &None),
        Err(Ok(nft_metadata_advanced::NFTError::InvalidTokenId))
    );
}

#[test]
fn test_nft_metadata_on_chain_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, nft_metadata_advanced::NFTMetadataContract);
    let client = nft_metadata_advanced::NFTMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);

    client.initialize(&admin, &String::from_str(&env, "https://meta/"), &true);
    assert!(client.is_on_chain_metadata());

    let metadata = nft_metadata_advanced::NFTMetadata {
        name: String::from_str(&env, "Cookie NFT"),
        description: String::from_str(&env, "A very rare cookie"),
        image: String::from_str(&env, "ipfs://cookie"),
        external_url: None,
        attributes: Vec::new(&env),
    };

    client.mint(&owner, &7, &None, &Some(metadata.clone()));
    let stored = client.metadata(&7).unwrap();
    assert_eq!(stored.name, metadata.name);
    assert_eq!(stored.image, metadata.image);

    // On-chain metadata requires a metadata payload.
    assert_eq!(
        client.try_mint(&owner, &8, &None, &None),
        Err(Ok(nft_metadata_advanced::NFTError::MetadataNotEnabled))
    );

    // The admin can update the metadata afterwards.
    let updated = nft_metadata_advanced::NFTMetadata {
        name: String::from_str(&env, "Cookie NFT v2"),
        ..metadata
    };
    client.set_metadata(&7, &updated);
    assert_eq!(
        client.metadata(&7).unwrap().name,
        String::from_str(&env, "Cookie NFT v2")
    );
}

#[test]
fn test_nft_metadata_standards_validate_and_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, nft_metadata_standards::NftMetadataContract);
    let client = nft_metadata_standards::NftMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Cookie Collection"),
        &String::from_str(&env, "COOKIE"),
        &String::from_str(&env, "ipfs://cookies/"),
    );

    let metadata = nft_metadata_standards::TokenMetadata {
        name: String::from_str(&env, "Cookie #1"),
        description: String::from_str(&env, "Freshly baked"),
        image: String::from_str(&env, "ipfs://cookies/1.png"),
        external_url: String::from_str(&env, "https://cookies.example/1"),
        animation_url: String::from_str(&env, ""),
        background_color: String::from_str(&env, "FFAACC"),
        attributes: Vec::new(&env),
    };

    client.mint(&admin, &alice, &1, &metadata);
    assert_eq!(client.owner_of(&1), alice);
    assert_eq!(client.balance_of(&alice), 1);
    assert_eq!(client.total_supply(), 1);
    assert_eq!(
        client.token_uri(&1),
        String::from_str(&env, "ipfs://cookies/1")
    );

    // Only the admin may mint.
    assert_eq!(
        client.try_mint(&alice, &alice, &2, &metadata),
        Err(Ok(nft_metadata_standards::NftError::NotAdmin))
    );

    // Approvals gate transfers.
    assert_eq!(
        client.try_transfer(&alice, &bob, &1),
        Err(Ok(nft_metadata_standards::NftError::NotApproved))
    );
    client.approve(&alice, &bob, &1);
    assert_eq!(client.get_approved(&1), Some(bob.clone()));
    client.transfer(&alice, &bob, &1);
    assert_eq!(client.owner_of(&1), bob);
    assert_eq!(client.balance_of(&alice), 0);
    assert_eq!(client.balance_of(&bob), 1);
}

#[test]
fn test_nft_metadata_standards_reject_invalid_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, nft_metadata_standards::NftMetadataContract);
    let client = nft_metadata_standards::NftMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Cookie Collection"),
        &String::from_str(&env, "COOKIE"),
        &String::from_str(&env, "ipfs://cookies/"),
    );

    let bad_image = nft_metadata_standards::TokenMetadata {
        name: String::from_str(&env, "Cookie #1"),
        description: String::from_str(&env, "Freshly baked"),
        image: String::from_str(&env, "not-a-uri"),
        external_url: String::from_str(&env, ""),
        animation_url: String::from_str(&env, ""),
        background_color: String::from_str(&env, ""),
        attributes: Vec::new(&env),
    };

    assert_eq!(
        client.try_mint(&admin, &alice, &1, &bad_image),
        Err(Ok(nft_metadata_standards::NftError::InvalidImageUri))
    );

    let empty_name = nft_metadata_standards::TokenMetadata {
        name: String::from_str(&env, ""),
        ..bad_image.clone()
    };
    assert_eq!(
        client.try_mint(&admin, &alice, &1, &empty_name),
        Err(Ok(nft_metadata_standards::NftError::MetadataFieldEmpty))
    );
}

// ---------------------------------------------------------------------------
// Governance — vote delegation
// ---------------------------------------------------------------------------

#[test]
fn test_vote_delegation_moves_voting_power() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, vote_delegation::VoteDelegationContract);
    let client = vote_delegation::VoteDelegationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.init(&admin);
    client.set_balance(&admin, &alice, &100);
    client.set_balance(&admin, &bob, &50);

    assert_eq!(client.get_voting_power(&alice), 100);
    assert_eq!(client.get_voting_power(&bob), 50);

    // Delegating moves the voting power to the delegate.
    client.delegate(&alice, &bob);
    assert_eq!(client.get_delegate(&alice), Some(bob.clone()));
    assert_eq!(client.get_voting_power(&alice), 0);
    assert_eq!(client.get_voting_power(&bob), 150);

    // Un-delegating restores it.
    client.undelegate(&alice);
    assert_eq!(client.get_delegate(&alice), None);
    assert_eq!(client.get_voting_power(&alice), 100);
    assert_eq!(client.get_voting_power(&bob), 50);

    // Self-delegation is rejected.
    assert_eq!(
        client.try_delegate(&alice, &alice),
        Err(Ok(vote_delegation::DelegationError::SelfDelegation))
    );
}

// ---------------------------------------------------------------------------
// Governance — timelock
// ---------------------------------------------------------------------------

#[test]
fn test_timelock_governance_queue_and_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let target_id = env.register_contract(None, TimelockTarget);
    let id = env.register_contract(None, timelock_governance::TimelockGovernance);
    let client = timelock_governance::TimelockGovernanceClient::new(&env, &id);
    let admin = Address::generate(&env);

    client.init(&admin, &100);

    let args = Vec::from_array(&env, [7u32.into_val(&env)]);
    let proposal_id = client.queue(&target_id, &symbol_short!("run"), &args, &100);
    assert_eq!(proposal_id, 1);
    assert_eq!(
        client.get_proposal(&proposal_id).status,
        timelock_governance::ProposalStatus::Queued
    );

    // A delay below the minimum is rejected.
    assert_eq!(
        client.try_queue(&target_id, &symbol_short!("run"), &args, &10),
        Err(Ok(timelock_governance::Error::InvalidDelay))
    );

    // The delay has to elapse before execution.
    assert!(matches!(
        client.try_execute(&proposal_id),
        Err(Ok(timelock_governance::Error::DelayNotMet))
    ));

    env.ledger().with_mut(|l| l.timestamp = 100);
    client.execute(&proposal_id);
    assert_eq!(
        client.get_proposal(&proposal_id).status,
        timelock_governance::ProposalStatus::Executed
    );
}

#[test]
fn test_timelock_governance_can_cancel_a_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let target_id = env.register_contract(None, TimelockTarget);
    let id = env.register_contract(None, timelock_governance::TimelockGovernance);
    let client = timelock_governance::TimelockGovernanceClient::new(&env, &id);
    let admin = Address::generate(&env);

    client.init(&admin, &10);
    let proposal_id = client.queue(&target_id, &symbol_short!("run"), &Vec::new(&env), &10);
    client.cancel(&proposal_id);
    assert_eq!(
        client.get_proposal(&proposal_id).status,
        timelock_governance::ProposalStatus::Canceled
    );
}

// ---------------------------------------------------------------------------
// Governance — DAO treasury
// ---------------------------------------------------------------------------

#[test]
fn test_dao_treasury_proposal_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(&admin, &100, &10, &20);
    client.deposit(&proposer, &1_000);
    assert_eq!(client.treasury_balance(), 1_000);

    let proposal_id = client.propose_transfer(&proposer, &recipient, &500);
    assert_eq!(proposal_id, 0);
    assert_eq!(client.proposal_count(), 1);

    // A proposal cannot be executed while voting is still open.
    assert_eq!(
        client.try_execute(&voter, &proposal_id),
        Err(Ok(dao_treasury::DaoError::InvalidState))
    );

    client.vote(&voter, &proposal_id, &true, &100);
    assert!(client.has_voted(&proposal_id, &voter));

    // Voting twice is rejected.
    assert_eq!(
        client.try_vote(&voter, &proposal_id, &true, &100),
        Err(Ok(dao_treasury::DaoError::AlreadyVoted))
    );

    // Once voting closes the proposal passes and can be executed.
    env.ledger().with_mut(|l| l.sequence_number = 11);
    assert_eq!(
        client.proposal_state(&proposal_id),
        dao_treasury::ProposalState::Passed
    );
    client.execute(&voter, &proposal_id);
    assert_eq!(client.treasury_balance(), 500);
    assert_eq!(
        client.proposal_state(&proposal_id),
        dao_treasury::ProposalState::Executed
    );

    // Executing twice is rejected.
    assert_eq!(
        client.try_execute(&voter, &proposal_id),
        Err(Ok(dao_treasury::DaoError::AlreadyExecuted))
    );
}

#[test]
fn test_dao_treasury_requires_quorum() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, dao_treasury::DaoContract);
    let client = dao_treasury::DaoContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin, &100, &10, &20);
    let proposal_id = client.propose_transfer(&proposer, &proposer, &100);
    client.vote(&voter, &proposal_id, &true, &10);

    // The quorum is not met, so the proposal fails.
    env.ledger().with_mut(|l| l.sequence_number = 11);
    assert_eq!(
        client.proposal_state(&proposal_id),
        dao_treasury::ProposalState::Failed
    );
    assert_eq!(
        client.try_execute(&voter, &proposal_id),
        Err(Ok(dao_treasury::DaoError::InvalidState))
    );
}

// ---------------------------------------------------------------------------
// Tokens — vesting
// ---------------------------------------------------------------------------

#[test]
fn test_vesting_management_releases_tokens_over_time() {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = token::StellarAssetClient::new(&env, &sac.address());

    let id = env.register_contract(None, vesting_management::VestingContract);
    let client = vesting_management::VestingContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    client.initialize(&admin, &Some(sac.address()));

    // Fund the vesting contract so it can pay out.
    asset.mint(&client.address, &1_000);

    // A cliff of 50 means nothing unlocks before that point.
    client.create_schedule(&admin, &beneficiary, &1_000, &0, &50, &100, &None);

    // Nothing is claimable before the cliff is reached.
    assert_eq!(
        client.try_claim(&beneficiary),
        Err(Ok(vesting_management::VestingError::CliffNotReached))
    );

    // Half way through the vesting period half of the tokens are unlocked.
    env.ledger().with_mut(|l| l.timestamp = 50);
    assert_eq!(client.vested_amount(&beneficiary), 500);
    assert_eq!(client.releasable_amount(&beneficiary), 500);
    assert_eq!(client.claim(&beneficiary), 500);
    assert_eq!(asset.balance(&beneficiary), 500);

    // The remaining tokens unlock at the end of the schedule.
    env.ledger().with_mut(|l| l.timestamp = 100);
    assert_eq!(client.claim(&beneficiary), 500);
    assert_eq!(asset.balance(&beneficiary), 1_000);
}

#[test]
fn test_vesting_management_validates_schedules() {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);

    let id = env.register_contract(None, vesting_management::VestingContract);
    let client = vesting_management::VestingContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin, &Some(sac.address()));

    // A cliff longer than the vesting duration is invalid.
    assert_eq!(
        client.try_create_schedule(&admin, &beneficiary, &100, &0, &200, &100, &None),
        Err(Ok(vesting_management::VestingError::InvalidSchedule))
    );

    // Only the admin can create schedules.
    assert_eq!(
        client.try_create_schedule(&stranger, &beneficiary, &100, &0, &0, &100, &None),
        Err(Ok(vesting_management::VestingError::Unauthorized))
    );

    client.create_schedule(&admin, &beneficiary, &100, &0, &0, &100, &None);

    // A second schedule for the same beneficiary is rejected.
    assert_eq!(
        client.try_create_schedule(&admin, &beneficiary, &100, &0, &0, &100, &None),
        Err(Ok(vesting_management::VestingError::ScheduleAlreadyExists))
    );
}

// ---------------------------------------------------------------------------
// Tokens — minting strategies
// ---------------------------------------------------------------------------

#[test]
fn test_minting_strategies_enforce_cap_and_schedule() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, minting_strategies_token::MintingStrategiesToken);
    let client = minting_strategies_token::MintingStrategiesTokenClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin, &1_000, &0, &100, &10);

    // Minting with a cap respects the cap.
    assert_eq!(client.mint_with_cap(&admin, &alice, &100), 100);
    assert_eq!(
        client.try_mint_with_cap(&admin, &alice, &1_000),
        Err(Ok(
            minting_strategies_token::MintingError::SupplyCapExceeded
        ))
    );

    // The unlimited strategy ignores the cap.
    assert_eq!(client.mint_unlimited(&admin, &alice, &1), 101);

    // Scheduled minting follows the release schedule.
    assert_eq!(client.scheduled_available(), 0);
    env.ledger().with_mut(|l| l.timestamp = 250);
    assert_eq!(client.scheduled_available(), 20);
    assert_eq!(client.mint_scheduled(&admin, &bob, &20), 20);

    // Minting more than the schedule allows is rejected.
    assert_eq!(
        client.try_mint_scheduled(&admin, &bob, &1_000),
        Err(Ok(
            minting_strategies_token::MintingError::ScheduleUnavailable
        ))
    );

    // Only the admin can mint.
    assert_eq!(
        client.try_mint_with_cap(&alice, &alice, &1),
        Err(Ok(minting_strategies_token::MintingError::Unauthorized))
    );
}

// ---------------------------------------------------------------------------
// Tokens — allowance pattern
// ---------------------------------------------------------------------------

#[test]
fn test_allowance_pattern_approve_and_spend() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, allowance_pattern::AllowancePattern);
    let client = allowance_pattern::AllowancePatternClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.initialize(&admin, &1_000_000);
    assert_eq!(client.balance(&admin), 1_000_000);

    client.approve(&alice, &bob, &100, &100_000);
    assert_eq!(client.allowance(&alice, &bob), 100);

    client.transfer_from(&bob, &alice, &carol, &40);
    assert_eq!(client.balance(&carol), 40);
    assert_eq!(client.allowance(&alice, &bob), 60);

    // Spending more than the allowance is rejected.
    assert_eq!(
        client.try_transfer_from(&bob, &alice, &carol, &1_000),
        Err(Ok(allowance_pattern::AllowanceError::InsufficientAllowance))
    );

    // The allowance can be revoked.
    client.revoke(&alice, &bob);
    assert_eq!(client.allowance(&alice, &bob), 0);
}

#[test]
fn test_allowance_pattern_expires() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, allowance_pattern::AllowancePattern);
    let client = allowance_pattern::AllowancePatternClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin, &1_000);

    // An expiration in the past is rejected.
    env.ledger().with_mut(|l| l.sequence_number = 100);
    assert_eq!(
        client.try_approve(&alice, &bob, &10, &50),
        Err(Ok(allowance_pattern::AllowanceError::InvalidExpiration))
    );

    client.approve(&alice, &bob, &10, &200);
    assert_eq!(client.allowance(&alice, &bob), 10);

    // Once the expiration ledger passed the allowance is worthless.
    env.ledger().with_mut(|l| l.sequence_number = 300);
    assert_eq!(client.allowance(&alice, &bob), 0);
    assert_eq!(
        client.try_transfer_from(&bob, &alice, &bob, &10),
        Err(Ok(allowance_pattern::AllowanceError::InsufficientAllowance))
    );
}

// ---------------------------------------------------------------------------
// Tokens — metadata
// ---------------------------------------------------------------------------

#[test]
fn test_token_metadata_mint_transfer_and_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, token_metadata::TokenMetadataContract);
    let client = token_metadata::TokenMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Cookie Token"),
        &String::from_str(&env, "COOK"),
        &7,
        &String::from_str(&env, "https://tokens.example/cookie.json"),
    );

    let metadata = client.metadata();
    assert_eq!(metadata.name, String::from_str(&env, "Cookie Token"));
    assert_eq!(metadata.symbol, String::from_str(&env, "COOK"));
    assert_eq!(metadata.decimals, 7);

    client.mint(&alice, &100);
    assert_eq!(client.balance(&alice), 100);
    assert_eq!(client.total_supply(), 100);

    client.transfer(&alice, &bob, &30);
    assert_eq!(client.balance(&alice), 70);
    assert_eq!(client.balance(&bob), 30);

    client.burn(&alice, &20);
    assert_eq!(client.balance(&alice), 50);
    assert_eq!(client.total_supply(), 80);

    // Burning more than the balance is rejected.
    assert_eq!(
        client.try_burn(&alice, &1_000),
        Err(Ok(token_metadata::MetadataError::InsufficientBalance))
    );
}

#[test]
fn test_token_metadata_rejects_empty_names() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, token_metadata::TokenMetadataContract);
    let client = token_metadata::TokenMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    assert_eq!(
        client.try_initialize(
            &admin,
            &String::from_str(&env, ""),
            &String::from_str(&env, "TK"),
            &7,
            &String::from_str(&env, ""),
        ),
        Err(Ok(token_metadata::MetadataError::EmptyString))
    );
}

// ---------------------------------------------------------------------------
// Tokens — multi-token balance manager
// ---------------------------------------------------------------------------

#[test]
fn test_multi_token_balance_manager_batches_balances_and_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_b = env.register_stellar_asset_contract_v2(issuer.clone());
    let asset_a = token::StellarAssetClient::new(&env, &token_a.address());

    let id = env.register_contract(None, multi_token_balance_manager::MultiTokenBalanceManager);
    let client = multi_token_balance_manager::MultiTokenBalanceManagerClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin);

    let metadata_a = multi_token_balance_manager::TokenMetadata {
        name: String::from_str(&env, "Token A"),
        symbol: String::from_str(&env, "TKA"),
        decimals: 7,
        standard: multi_token_balance_manager::MetadataStandard::Sep41,
    };
    let metadata_b = multi_token_balance_manager::TokenMetadata {
        name: String::from_str(&env, "Token B"),
        symbol: String::from_str(&env, "TKB"),
        decimals: 4,
        standard: multi_token_balance_manager::MetadataStandard::StellarAsset,
    };

    client.register_token(&admin, &token_a.address(), &metadata_a);
    client.register_token(&admin, &token_b.address(), &metadata_b);
    assert_eq!(
        client.metadata(&token_a.address()).symbol,
        String::from_str(&env, "TKA")
    );

    asset_a.mint(&alice, &500);

    // Batch balances.
    let balances = client.batch_balance(
        &alice,
        &Vec::from_array(&env, [token_a.address(), token_b.address()]),
    );
    assert_eq!(balances.len(), 2);
    assert_eq!(balances.get(0).unwrap().balance, 500);
    assert_eq!(balances.get(1).unwrap().balance, 0);

    // Batch transfers.
    let transfers = Vec::from_array(
        &env,
        [multi_token_balance_manager::TransferRequest {
            token: token_a.address(),
            to: bob.clone(),
            amount: 100,
        }],
    );
    client.batch_transfer(&alice, &transfers);
    assert_eq!(asset_a.balance(&bob), 100);

    // Empty batches are rejected.
    assert_eq!(
        client.try_batch_transfer(&alice, &Vec::new(&env)),
        Err(Ok(
            multi_token_balance_manager::BalanceManagerError::EmptyBatch
        ))
    );

    // Unregistered tokens are rejected.
    let unknown = env.register_stellar_asset_contract_v2(issuer);
    assert_eq!(
        client.try_batch_balance(&alice, &Vec::from_array(&env, [unknown.address()])),
        Err(Ok(
            multi_token_balance_manager::BalanceManagerError::NotRegistered
        ))
    );
}

// ---------------------------------------------------------------------------
// Tokens — lock
// ---------------------------------------------------------------------------

#[test]
fn test_token_lock_releases_funds_after_unlock_time() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, token_lock::TokenLockContract);
    let client = token_lock::TokenLockContractClient::new(&env, &id);
    let user = Address::generate(&env);

    client.lock(&user, &100, &1_000);
    client.lock(&user, &50, &2_000);
    assert_eq!(client.locked_balance(&user), 150);
    assert_eq!(client.unlockable_balance(&user), 0);
    assert_eq!(client.lock_schedule(&user).len(), 2);

    // Only the matured entry can be released.
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    assert_eq!(client.unlockable_balance(&user), 100);
    assert_eq!(client.unlock(&user), 100);
    assert_eq!(client.locked_balance(&user), 50);

    // The remaining entry unlocks later on.
    env.ledger().with_mut(|l| l.timestamp = 2_000);
    assert_eq!(client.unlock(&user), 50);
    assert_eq!(client.locked_balance(&user), 0);
}

// ---------------------------------------------------------------------------
// Tokens — reward token
// ---------------------------------------------------------------------------

#[test]
fn test_reward_token_pools_and_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, reward_token::RewardToken);
    let client = reward_token::RewardTokenClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Reward Token"),
        &String::from_str(&env, "RWD"),
        &7,
    );

    client.mint(&admin, &1_000);
    let pool_id = client.create_pool(&1_000_000);
    assert_eq!(pool_id, 0);
    client.deposit_to_pool(&pool_id, &1_000);
    assert_eq!(client.pool_info(&pool_id).total_deposited, 1_000);

    client.mint(&alice, &10);
    assert_eq!(client.claimable_rewards(&alice, &pool_id), 10);
    assert_eq!(client.claim_rewards(&alice, &pool_id), 10);
    assert_eq!(client.balance(&alice), 20);

    // A second claim has nothing left to pay out.
    assert_eq!(
        client.try_claim_rewards(&alice, &pool_id),
        Err(Ok(reward_token::RewardError::NothingToClaim))
    );

    // Unknown pools are rejected.
    assert_eq!(
        client.try_pool_info(&99),
        Err(Ok(reward_token::RewardError::PoolNotFound))
    );
}

// ---------------------------------------------------------------------------
// Tokens — custom token with multi-sig and pause
// ---------------------------------------------------------------------------

#[test]
fn test_custom_token_multi_sig_and_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, custom_token::CustomToken);
    let client = custom_token::CustomTokenClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);

    let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);
    client.initialize(
        &admin,
        &String::from_str(&env, "Custom"),
        &Symbol::new(&env, "CST"),
        &6,
        &1_000,
        &2,
        &signers,
    );

    assert_eq!(client.balance(&admin), 1_000);
    assert_eq!(client.decimals(), 6);
    assert_eq!(client.symbol(), Symbol::new(&env, "CST"));

    // Regular transfers work while the token is not paused.
    client.transfer(&admin, &alice, &100);
    assert_eq!(client.balance(&alice), 100);

    // Pausing blocks every value-moving operation.
    client.set_pause(&admin, &true);
    assert!(client.is_paused());
    assert_eq!(
        client.try_transfer(&alice, &bob, &1),
        Err(Ok(custom_token::TokenError::Paused))
    );
    client.set_pause(&admin, &false);

    // A multi-sig transfer needs the configured number of signers.
    client.mint(&admin, &id, &500);
    let one_signer = Vec::from_array(&env, [signer1.clone()]);
    assert_eq!(
        client.try_multi_sig_transfer(&one_signer, &bob, &50),
        Err(Ok(custom_token::TokenError::InsufficientApprovals))
    );
    client.multi_sig_transfer(&signers, &bob, &50);
    assert_eq!(client.balance(&bob), 50);

    // The signers can be rotated.
    let new_signers = Vec::from_array(&env, [alice.clone()]);
    client.update_signers(&admin, &1, &new_signers);
    let (threshold, current) = client.get_signers();
    assert_eq!(threshold, 1);
    assert_eq!(current.len(), 1);
}
