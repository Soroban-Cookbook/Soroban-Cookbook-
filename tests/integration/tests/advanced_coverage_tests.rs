//! Advanced-example coverage suite (issue #904).
//!
//! These integration tests exercise the *advanced* Soroban examples that were
//! previously only covered by their own in-crate unit tests.  Every example is
//! registered natively (no Wasm binary required) so the suite runs with any
//! Rust toolchain.
//!
//! Covered examples:
//! - `examples/advanced/04-circuit-breaker`
//! - `examples/advanced/05-rate-limiting`
//! - `examples/advanced/15-reentrancy-guard`
//! - `examples/advanced/30-permit-pattern`
//! - `examples/advanced/21-merkle-proofs`
//! - `examples/advanced/28-merkle-airdrop`
//! - `examples/advanced/29-merkle-whitelist`
//! - `examples/advanced/20-batch-transfer`
//! - `examples/advanced/22-batch-builder`
//! - `examples/advanced/42-multicall`
//! - `examples/advanced/16-hierarchical-access-control`
//! - `examples/advanced/26-data-aggregation-oracle`
//! - `examples/advanced/07-upgrade-patterns`
//! - `examples/advanced/37-beacon-management`

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, testutils::Ledger as _, token,
    Address, Bytes, BytesN, Env, IntoVal, TryFromVal, Vec,
};

// ---------------------------------------------------------------------------
// Shared Merkle helpers
// ---------------------------------------------------------------------------

/// Hash two sibling nodes the same way the on-chain verifiers do:
/// `sha256(min(a, b) || max(a, b))`.
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

/// Build a Merkle root off-chain.  Odd nodes are carried up unchanged.
fn merkle_root(env: &Env, leaves: &[BytesN<32>]) -> BytesN<32> {
    let mut level: std::vec::Vec<BytesN<32>> = leaves.to_vec();
    while level.len() > 1 {
        let mut next: std::vec::Vec<BytesN<32>> = std::vec::Vec::new();
        let mut i = 0;
        while i + 1 < level.len() {
            next.push(hash_pair(env, &level[i], &level[i + 1]));
            i += 2;
        }
        if i < level.len() {
            next.push(level[i].clone());
        }
        level = next;
    }
    level[0].clone()
}

/// Build the sibling path for `index`, ordered from the leaf level upwards.
fn merkle_proof(env: &Env, leaves: &[BytesN<32>], index: usize) -> Vec<BytesN<32>> {
    let mut level: std::vec::Vec<BytesN<32>> = leaves.to_vec();
    let mut idx = index;
    let mut proof: std::vec::Vec<BytesN<32>> = std::vec::Vec::new();
    while level.len() > 1 {
        let sibling = if idx % 2 == 0 {
            if idx + 1 < level.len() {
                Some(level[idx + 1].clone())
            } else {
                None
            }
        } else {
            Some(level[idx - 1].clone())
        };
        if let Some(s) = sibling {
            proof.push(s);
        }
        let mut next: std::vec::Vec<BytesN<32>> = std::vec::Vec::new();
        let mut i = 0;
        while i + 1 < level.len() {
            next.push(hash_pair(env, &level[i], &level[i + 1]));
            i += 2;
        }
        if i < level.len() {
            next.push(level[i].clone());
        }
        level = next;
        idx /= 2;
    }
    let mut out: Vec<BytesN<32>> = Vec::new(env);
    for sibling in proof {
        out.push_back(sibling);
    }
    out
}

/// Leaf hash convention of the Merkle whitelist example:
/// `sha256(address_xdr || nonce_be || metadata)`.
fn whitelist_leaf(env: &Env, address: &Address, nonce: u64, metadata: &Bytes) -> BytesN<32> {
    let mut buf = Bytes::new(env);
    buf.append(&address.to_xdr(env));
    buf.append(&Bytes::from_array(env, &nonce.to_be_bytes()));
    buf.append(metadata);
    env.crypto().sha256(&buf).to_bytes()
}

// ---------------------------------------------------------------------------
// Mock contracts used as cross-contract targets
// ---------------------------------------------------------------------------

/// Minimal receiver used by the reentrancy-guard example.
#[contract]
pub struct FundsReceiver;

#[contractimpl]
impl FundsReceiver {
    pub fn receive_funds(env: Env, to: Address, amount: i128) {
        env.storage()
            .instance()
            .set(&symbol_short!("last"), &(to, amount));
    }
}

/// Counter used as a target of the multicall example.
#[contract]
pub struct MulticallTarget;

#[contractimpl]
impl MulticallTarget {
    pub fn add(env: Env, user: Address, amount: u32) -> u32 {
        let key = (symbol_short!("count"), user);
        let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + amount;
        env.storage().instance().set(&key, &next);
        next
    }

    pub fn explode(env: Env) -> u32 {
        env.storage().instance().set(&symbol_short!("boom"), &true);
        0
    }
}

// ---------------------------------------------------------------------------
// Circuit breaker
// ---------------------------------------------------------------------------

#[test]
fn test_circuit_breaker_opens_after_threshold_failures() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, circuit_breaker::CircuitBreakerContract);
    let client = circuit_breaker::CircuitBreakerContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let caller = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Active);
    assert_eq!(client.get_failure_count(), 0);
    assert_eq!(client.get_failure_threshold(), 3);

    // Two failures are not enough to trip the breaker.
    client.fail(&caller);
    client.fail(&caller);
    assert_eq!(client.get_failure_count(), 2);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Active);

    // The third failure trips it.
    client.fail(&caller);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Paused);
    assert_eq!(client.get_failure_count(), 3);

    // While paused every call is rejected.
    assert_eq!(
        client.try_execute(&caller),
        Err(Ok(circuit_breaker::CircuitError::CircuitPaused))
    );
    assert_eq!(
        client.try_fail(&caller),
        Err(Ok(circuit_breaker::CircuitError::CircuitPaused))
    );
}

#[test]
fn test_circuit_breaker_recovers_after_recovery_window() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, circuit_breaker::CircuitBreakerContract);
    let client = circuit_breaker::CircuitBreakerContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let caller = Address::generate(&env);

    client.initialize(&admin);
    client.configure(&2, &50);
    assert_eq!(client.get_failure_threshold(), 2);
    assert_eq!(client.get_recovery_window(), 50);

    client.fail(&caller);
    client.fail(&caller);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Paused);

    // Not yet past the recovery window.
    env.ledger().with_mut(|l| l.timestamp += 10);
    assert_eq!(
        client.try_execute(&caller),
        Err(Ok(circuit_breaker::CircuitError::CircuitPaused))
    );

    // Past the recovery window the breaker closes again.
    env.ledger().with_mut(|l| l.timestamp += 100);
    client.execute(&caller);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Active);
    assert_eq!(client.get_failure_count(), 0);
}

#[test]
fn test_circuit_breaker_admin_controls_and_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, circuit_breaker::CircuitBreakerContract);
    let client = circuit_breaker::CircuitBreakerContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.initialize(&admin);

    // A contract can only be initialized once.
    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(circuit_breaker::CircuitError::AlreadyInitialized))
    );

    // Invalid configuration is rejected before any state is written.
    assert_eq!(
        client.try_configure(&0, &10),
        Err(Ok(circuit_breaker::CircuitError::InvalidThreshold))
    );
    assert_eq!(
        client.try_configure(&3, &0),
        Err(Ok(circuit_breaker::CircuitError::InvalidRecoveryWindow))
    );

    // The admin can pause and resume the breaker manually.
    client.set_pause(&true);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Paused);
    client.set_pause(&false);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Active);

    // Admin rotation.
    client.set_admin(&new_admin);
    let caller = Address::generate(&env);
    client.fail(&caller);
    client.fail(&caller);
    client.fail(&caller);
    assert_eq!(client.get_state(), circuit_breaker::CircuitState::Paused);
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

#[test]
fn test_rate_limiter_enforces_call_and_amount_limits() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, rate_limiting::RateLimiterContract);
    let client = rate_limiting::RateLimiterContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let limit = rate_limiting::Limit {
        window: 60,
        max_calls: 2,
        max_amount: 100,
    };
    client.initialize(&admin, &limit);

    client.consume(&user, &10);
    client.consume(&user, &20);

    let usage = client.usage_of(&user);
    assert_eq!(usage.calls, 2);
    assert_eq!(usage.amount, 30);
    assert_eq!(client.remaining_calls(&user), 0);
    assert_eq!(client.remaining_amount(&user), 70);

    // Third call in the same window is rejected.
    assert_eq!(
        client.try_consume(&user, &1),
        Err(Ok(rate_limiting::RateLimitError::CallLimitExceeded))
    );

    // Non-positive amounts are rejected.
    assert_eq!(
        client.try_consume(&user, &0),
        Err(Ok(rate_limiting::RateLimitError::InvalidAmount))
    );
}

#[test]
fn test_rate_limiter_window_resets_and_user_overrides() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, rate_limiting::RateLimiterContract);
    let client = rate_limiting::RateLimiterContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let limit = rate_limiting::Limit {
        window: 60,
        max_calls: 1,
        max_amount: 50,
    };
    client.initialize(&admin, &limit);

    client.consume(&user, &50);
    assert_eq!(
        client.try_consume(&user, &1),
        Err(Ok(rate_limiting::RateLimitError::CallLimitExceeded))
    );

    // A per-user limit overrides the default one.
    let user_limit = rate_limiting::Limit {
        window: 60,
        max_calls: 5,
        max_amount: 500,
    };
    client.set_user_limit(&user, &user_limit);
    assert_eq!(client.limit_of(&user).max_calls, 5);
    assert_eq!(client.limit_of(&user).max_amount, 500);

    // Once the window elapses the usage resets.
    env.ledger().with_mut(|l| l.timestamp += 61);
    let usage = client.usage_of(&user);
    assert_eq!(usage.calls, 0);
    assert_eq!(usage.amount, 0);
    client.consume(&user, &50);

    // Explicit reset clears the usage inside the window.
    client.reset(&user);
    let usage = client.usage_of(&user);
    assert_eq!(usage.calls, 0);

    // Clearing the override falls back to the default limit.
    client.clear_user_limit(&user);
    assert_eq!(client.limit_of(&user).max_calls, 1);
}

// ---------------------------------------------------------------------------
// Reentrancy guard
// ---------------------------------------------------------------------------

#[test]
fn test_reentrancy_guard_deposit_and_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let guard_id = env.register_contract(None, reentrancy_guard::ReentrancyGuardContract);
    let target_id = env.register_contract(None, FundsReceiver);
    let guard = reentrancy_guard::ReentrancyGuardContractClient::new(&env, &guard_id);
    let user = Address::generate(&env);
    let receiver = Address::generate(&env);

    guard.init();
    guard.deposit(&user, &1_000);
    assert_eq!(guard.get_balance(&user), 1_000);

    // Withdrawing invokes the target contract, which must expose `receive_funds`.
    guard.withdraw(&receiver, &400, &target_id);
    assert_eq!(guard.get_balance(&user), 600);

    // Withdrawing more than the balance is rejected.
    assert!(guard.try_withdraw(&receiver, &10_000, &target_id).is_err());
}

// ---------------------------------------------------------------------------
// Permit pattern
// ---------------------------------------------------------------------------

#[test]
fn test_permit_pattern_grants_and_spends_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, permit_pattern::PermitPattern);
    let client = permit_pattern::PermitPatternClient::new(&env, &id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(&admin, &1_000);
    assert_eq!(client.balance(&owner), 1_000);

    client.permit(&owner, &spender, &400, &100_000);
    assert_eq!(client.allowance(&owner, &spender), 400);

    client.transfer_from(&spender, &owner, &recipient, &150);
    assert_eq!(client.balance(&owner), 850);
    assert_eq!(client.balance(&recipient), 150);
    assert_eq!(client.allowance(&owner, &spender), 250);

    // Spending more than the remaining allowance fails.
    assert_eq!(
        client.try_transfer_from(&spender, &owner, &recipient, &1_000),
        Err(Ok(permit_pattern::PermitError::InsufficientAllowance))
    );

    // Non-positive transfer amounts are rejected.
    assert_eq!(
        client.try_transfer_from(&spender, &owner, &recipient, &0),
        Err(Ok(permit_pattern::PermitError::InvalidAmount))
    );
}

#[test]
fn test_permit_pattern_expired_permit_has_no_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, permit_pattern::PermitPattern);
    let client = permit_pattern::PermitPatternClient::new(&env, &id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);

    client.initialize(&admin, &1_000);

    // A permit that expires in the past is rejected outright.
    env.ledger().with_mut(|l| l.sequence_number = 10);
    assert_eq!(
        client.try_permit(&owner, &spender, &100, &5),
        Err(Ok(permit_pattern::PermitError::ExpiredPermit))
    );

    // A permit that is valid now but expires later stops being spendable.
    client.permit(&owner, &spender, &100, &20);
    assert_eq!(client.allowance(&owner, &spender), 100);
    env.ledger().with_mut(|l| l.sequence_number = 50);
    assert_eq!(client.allowance(&owner, &spender), 0);
}

// ---------------------------------------------------------------------------
// Merkle proofs
// ---------------------------------------------------------------------------

#[test]
fn test_merkle_proofs_verify_and_claim_leaves() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, merkle_proofs::MerkleProofContract);
    let client = merkle_proofs::MerkleProofContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let leaves: std::vec::Vec<BytesN<32>> = (0u8..4)
        .map(|i| client.hash_leaf(&Bytes::from_slice(&env, &[i, i + 1, i + 2])))
        .collect();
    let root = merkle_root(&env, &leaves);

    client.set_root(&admin, &root, &4);
    let info = client.get_root_info();
    assert_eq!(info.root, root);
    assert_eq!(info.leaf_count, 4);
    assert_eq!(info.generation, 1);

    let proof = merkle_proof(&env, &leaves, 0);
    assert!(client.verify(&leaves[0], &0, &proof));

    // A wrong leaf does not verify.
    let other = client.hash_leaf(&Bytes::from_slice(&env, &[9, 9, 9]));
    assert!(!client.verify(&other, &0, &proof));

    // Claiming is idempotent-protected.
    client.verify_and_claim(&leaves[0], &0, &proof);
    assert!(client.is_claimed(&0));
    assert_eq!(
        client.try_verify_and_claim(&leaves[0], &0, &proof),
        Err(Ok(merkle_proofs::MerkleError::AlreadyClaimed))
    );

    // Out-of-bounds indices are rejected.
    assert_eq!(
        client.try_verify(&leaves[0], &99, &proof),
        Err(Ok(merkle_proofs::MerkleError::IndexOutOfBounds))
    );
}

#[test]
fn test_merkle_proofs_reject_unauthorized_root_update() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, merkle_proofs::MerkleProofContract);
    let client = merkle_proofs::MerkleProofContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.initialize(&admin);

    let root = BytesN::from_array(&env, &[7u8; 32]);
    assert_eq!(
        client.try_set_root(&attacker, &root, &1),
        Err(Ok(merkle_proofs::MerkleError::Unauthorized))
    );
    assert_eq!(
        client.try_set_root(&admin, &root, &0),
        Err(Ok(merkle_proofs::MerkleError::InvalidLeafCount))
    );
}

// ---------------------------------------------------------------------------
// Merkle airdrop
// ---------------------------------------------------------------------------

#[test]
fn test_merkle_airdrop_claims_allocation_once() {
    let env = Env::default();
    env.mock_all_auths();

    let asset_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(asset_admin);
    let asset = token::StellarAssetClient::new(&env, &sac.address());

    let id = env.register_contract(None, merkle_airdrop::MerkleAirdropContract);
    let client = merkle_airdrop::MerkleAirdropContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);

    let leaves: std::vec::Vec<BytesN<32>> = std::vec![
        client.hash_leaf(&claimer, &100),
        client.hash_leaf(&Address::generate(&env), &200),
    ];
    let root = merkle_root(&env, &leaves);

    client.initialize(&admin, &sac.address(), &root);
    assert_eq!(client.get_root(), root);
    assert_eq!(client.get_token(), sac.address());

    // Fund the contract so it can pay out the airdrop.
    asset.mint(&id, &1_000);

    let proof = merkle_proof(&env, &leaves, 0);
    client.claim(&claimer, &100, &proof);
    assert_eq!(asset.balance(&claimer), 100);
    assert!(client.is_claimed(&claimer));

    // A second claim for the same leaf is rejected.
    assert_eq!(
        client.try_claim(&claimer, &100, &proof),
        Err(Ok(merkle_airdrop::Error::AlreadyClaimed))
    );

    // Claiming an amount that is not in the leaf fails the proof check.
    assert_eq!(
        client.try_claim(&claimer, &999, &proof),
        Err(Ok(merkle_airdrop::Error::InvalidProof))
    );
}

// ---------------------------------------------------------------------------
// Merkle whitelist
// ---------------------------------------------------------------------------

#[test]
fn test_merkle_whitelist_registers_members() {
    let env = Env::default();
    env.mock_all_auths();

    let asset_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(asset_admin);

    let id = env.register_contract(None, merkle_whitelist::MerkleWhitelistContract);
    let client = merkle_whitelist::MerkleWhitelistContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    // A zero registration fee keeps the test focused on the Merkle flow.
    client.initialize(
        &admin,
        &sac.address(),
        &0,
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    assert!(client.has_role(&admin, &merkle_whitelist::Role::Admin));
    assert!(!client.is_paused());

    let metadata = Bytes::from_slice(&env, b"member-1");
    let leaf = whitelist_leaf(&env, &member, 0, &metadata);
    let root = merkle_root(&env, &[leaf.clone()]);

    // Publish the new root through governance.  The admin also acts as the
    // governor that sponsors the proposal.
    client.add_role(&admin, &admin, &merkle_whitelist::Role::Governor);
    client.propose_root_update(&admin, &root, &metadata);
    assert_eq!(
        client.get_proposal_status(&1),
        merkle_whitelist::ProposalStatus::Active
    );

    // Three votes are needed for the quorum.
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    client.add_role(&admin, &voter1, &merkle_whitelist::Role::Governor);
    client.add_role(&admin, &voter2, &merkle_whitelist::Role::Governor);
    client.add_role(&admin, &voter3, &merkle_whitelist::Role::Governor);
    client.vote_on_proposal(&voter1, &1, &true);
    client.vote_on_proposal(&voter2, &1, &true);
    client.vote_on_proposal(&voter3, &1, &true);

    // The timelock has to elapse before the proposal can be executed.
    env.ledger().with_mut(|l| l.timestamp += 86_400);
    client.execute_proposal(&admin, &1);
    assert_eq!(client.get_merkle_root(), root);

    // Now the member can register with a valid proof.
    let proof = merkle_proof(&env, &[leaf], 0);
    client.verify_whitelist(&member, &proof, &metadata);
    assert!(client.is_whitelisted(&member));

    // Registering twice is rejected.
    assert_eq!(
        client.try_verify_whitelist(&member, &proof, &metadata),
        Err(Ok(merkle_whitelist::Error::AlreadyRegistered))
    );
}

// ---------------------------------------------------------------------------
// Batch transfer
// ---------------------------------------------------------------------------

#[test]
fn test_batch_transfer_distributes_to_multiple_recipients() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, batch_transfer::BatchTransferContract);
    let client = batch_transfer::BatchTransferContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin, &1_000);
    assert_eq!(client.balance(&admin), 1_000);

    let transfers = Vec::from_array(
        &env,
        [
            batch_transfer::Transfer {
                to: alice.clone(),
                amount: 100,
            },
            batch_transfer::Transfer {
                to: bob.clone(),
                amount: 250,
            },
        ],
    );

    assert_eq!(client.batch_transfer(&admin, &transfers), 2);
    assert_eq!(client.balance(&alice), 100);
    assert_eq!(client.balance(&bob), 250);
    assert_eq!(client.balance(&admin), 650);
}

#[test]
fn test_batch_transfer_validates_input() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, batch_transfer::BatchTransferContract);
    let client = batch_transfer::BatchTransferContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize(&admin, &100);

    // Empty batches are rejected.
    assert_eq!(
        client.try_batch_transfer(&admin, &Vec::new(&env)),
        Err(Ok(batch_transfer::BatchError::EmptyBatch))
    );

    // Non-positive amounts are rejected.
    let bad = Vec::from_array(
        &env,
        [batch_transfer::Transfer {
            to: bob.clone(),
            amount: 0,
        }],
    );
    assert_eq!(
        client.try_batch_transfer(&admin, &bad),
        Err(Ok(batch_transfer::BatchError::InvalidAmount))
    );

    // The batch is all-or-nothing: the total must be covered.
    let too_much = Vec::from_array(
        &env,
        [batch_transfer::Transfer {
            to: bob.clone(),
            amount: 1_000,
        }],
    );
    assert_eq!(
        client.try_batch_transfer(&admin, &too_much),
        Err(Ok(batch_transfer::BatchError::InsufficientBalance))
    );
    assert_eq!(client.balance(&admin), 100);
}

// ---------------------------------------------------------------------------
// Batch builder
// ---------------------------------------------------------------------------

#[test]
fn test_batch_builder_validates_and_executes_batches() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, batch_builder::BatchBuilderContract);
    let client = batch_builder::BatchBuilderContractClient::new(&env, &id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize();
    client.set_balance(&alice, &1_000);

    let batch_id = client.begin_batch();
    client.add_transfer(&batch_id, &alice, &bob, &100);
    client.add_mint(&batch_id, &bob, &25);
    assert_eq!(client.batch_len(&batch_id), 2);

    let estimated = client.estimate_gas(&batch_id);
    assert_eq!(
        estimated,
        batch_builder::BASE_GAS_UNITS
            + batch_builder::GAS_PER_TRANSFER
            + batch_builder::GAS_PER_MINT
    );

    let gas = client.validate_batch(&batch_id);
    assert_eq!(gas, estimated);
    assert!(client.is_validated(&batch_id));

    assert_eq!(client.execute_batch(&batch_id), 2);
    assert_eq!(client.get_balance(&alice), 900);
    assert_eq!(client.get_balance(&bob), 125);

    // A batch can only be executed once.
    assert_eq!(
        client.try_execute_batch(&batch_id),
        Err(Ok(batch_builder::BuilderError::BatchNotFound))
    );
}

#[test]
fn test_batch_builder_rejects_invalid_batches() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, batch_builder::BatchBuilderContract);
    let client = batch_builder::BatchBuilderContractClient::new(&env, &id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.initialize();

    // Executing without validating is rejected.
    let batch_id = client.begin_batch();
    client.add_transfer(&batch_id, &alice, &bob, &10);
    assert_eq!(
        client.try_execute_batch(&batch_id),
        Err(Ok(batch_builder::BuilderError::NotValidated))
    );

    // Duplicate operations are rejected during validation.
    client.add_transfer(&batch_id, &alice, &bob, &10);
    assert_eq!(
        client.try_validate_batch(&batch_id),
        Err(Ok(batch_builder::BuilderError::DuplicateOperation))
    );

    // Unknown batches are rejected.
    assert_eq!(
        client.try_batch_len(&999),
        Err(Ok(batch_builder::BuilderError::BatchNotFound))
    );
}

// ---------------------------------------------------------------------------
// Multicall
// ---------------------------------------------------------------------------

#[test]
fn test_multicall_aggregates_multiple_calls() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, multicall::MulticallContract);
    let target_id = env.register_contract(None, MulticallTarget);
    let client = multicall::MulticallContractClient::new(&env, &id);
    let caller = Address::generate(&env);

    let calls = Vec::from_array(
        &env,
        [
            multicall::Call {
                contract: target_id.clone(),
                function: symbol_short!("add"),
                args: Vec::from_array(&env, [caller.clone().into_val(&env), 5u32.into_val(&env)]),
            },
            multicall::Call {
                contract: target_id,
                function: symbol_short!("add"),
                args: Vec::from_array(&env, [caller.clone().into_val(&env), 7u32.into_val(&env)]),
            },
        ],
    );

    let results = client.aggregate(&caller, &calls);
    assert_eq!(results.len(), 2);
    let first: u32 = u32::try_from_val(&env, &results.get(0).unwrap()).unwrap();
    let second: u32 = u32::try_from_val(&env, &results.get(1).unwrap()).unwrap();
    assert_eq!(first, 5);
    assert_eq!(second, 12);
}

#[test]
fn test_multicall_validates_batches_and_reports_failures() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, multicall::MulticallContract);
    let target_id = env.register_contract(None, MulticallTarget);
    let client = multicall::MulticallContractClient::new(&env, &id);
    let caller = Address::generate(&env);

    // Empty batches are rejected.
    assert_eq!(
        client.try_aggregate(&caller, &Vec::new(&env)),
        Err(Ok(multicall::MulticallError::EmptyBatch))
    );

    // Self-calls are rejected.
    let self_call = Vec::from_array(
        &env,
        [multicall::Call {
            contract: id.clone(),
            function: symbol_short!("aggregate"),
            args: Vec::new(&env),
        }],
    );
    assert_eq!(
        client.try_aggregate(&caller, &self_call),
        Err(Ok(multicall::MulticallError::SelfCall))
    );

    // A failing call is reported per-result when failures are tolerated.
    let failing = Vec::from_array(
        &env,
        [multicall::Call {
            contract: target_id,
            function: symbol_short!("explode"),
            args: Vec::new(&env),
        }],
    );
    let results = client.aggregate_results(&caller, &failing, &false);
    assert_eq!(results.len(), 1);
    assert!(!results.get(0).unwrap().0);

    // ... and aborts the batch when failures are not tolerated.
    assert_eq!(
        client.try_aggregate_results(&caller, &failing, &true),
        Err(Ok(multicall::MulticallError::CallFailed))
    );
}

// ---------------------------------------------------------------------------
// Hierarchical access control
// ---------------------------------------------------------------------------

#[test]
fn test_hierarchical_access_control_roles_and_permissions() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(
        None,
        hierarchical_access_control::HierarchicalAccessControlContract,
    );
    let client =
        hierarchical_access_control::HierarchicalAccessControlContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    assert!(client.has_role(&hierarchical_access_control::ROLE_ADMIN, &admin));
    assert_eq!(
        client
            .get_role_permissions(&hierarchical_access_control::ROLE_ADMIN)
            .len(),
        4
    );
    assert!(client.account_has_permission(&admin, &hierarchical_access_control::PERM_MANAGE_ROLES));

    // The admin can promote other accounts.
    client.grant_role(&admin, &hierarchical_access_control::ROLE_MANAGER, &manager);
    client.grant_role(
        &admin,
        &hierarchical_access_control::ROLE_OPERATOR,
        &operator,
    );
    assert!(client.has_role(&hierarchical_access_control::ROLE_MANAGER, &manager));
    assert!(client.has_role(&hierarchical_access_control::ROLE_OPERATOR, &operator));
    assert_eq!(
        client
            .get_role_members(&hierarchical_access_control::ROLE_OPERATOR)
            .len(),
        1
    );

    // Operators may use resources but not manage them.
    client.use_resource(&operator, &symbol_short!("res1"));
    assert!(
        client.account_has_permission(&operator, &hierarchical_access_control::PERM_USE_RESOURCES)
    );
    assert!(!client.account_has_permission(
        &operator,
        &hierarchical_access_control::PERM_MANAGE_RESOURCES
    ));

    // Roles can be renounced and revoked.
    client.renounce_role(&operator, &hierarchical_access_control::ROLE_OPERATOR);
    assert!(!client.has_role(&hierarchical_access_control::ROLE_OPERATOR, &operator));
    client.revoke_role(&admin, &hierarchical_access_control::ROLE_MANAGER, &manager);
    assert!(!client.has_role(&hierarchical_access_control::ROLE_MANAGER, &manager));
}

// ---------------------------------------------------------------------------
// Data aggregation oracle
// ---------------------------------------------------------------------------

#[test]
fn test_data_aggregation_oracle_aggregates_sources() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, data_aggregation_oracle::DataAggregationOracleContract);
    let client = data_aggregation_oracle::DataAggregationOracleContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let s3 = Address::generate(&env);

    client.initialize(&admin);
    client.add_source(&admin, &s1);
    client.add_source(&admin, &s2);
    client.add_source(&admin, &s3);
    assert_eq!(client.get_sources().len(), 3);

    client.submit_data(&s1, &100);
    client.submit_data(&s2, &102);
    client.submit_data(&s3, &98);

    let result = client.aggregate_data();
    assert_eq!(result.median_value, 100);
    assert_eq!(result.mean_value, 100);
    assert_eq!(result.outliers_removed, 0);

    // The oracle can be paused by the admin.
    client.pause(&admin);
    assert!(client.is_paused());
    client.resume(&admin);
    assert!(!client.is_paused());
}

// ---------------------------------------------------------------------------
// Upgrade patterns
// ---------------------------------------------------------------------------

#[test]
fn test_versioned_upgrade_migrates_storage() {
    use upgrade_patterns::versioned_upgrade::{seed_v1_counter, VersionedUpgradeContract};

    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, VersionedUpgradeContract);
    let client =
        upgrade_patterns::versioned_upgrade::VersionedUpgradeContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.storage_version(), 1);

    // Seed legacy state and migrate it to the v2 layout.
    env.as_contract(&id, || seed_v1_counter(&env, 42));
    client.migrate();
    assert_eq!(client.storage_version(), 2);
    assert_eq!(client.get_counter().val, 42);

    // Migrating twice is rejected.
    assert_eq!(
        client.try_migrate(),
        Err(Ok(
            upgrade_patterns::versioned_upgrade::VersionedError::AlreadyMigrated
        ))
    );

    // The v2 counter behaves like the v1 one.
    let updated = client.increment(&8);
    assert_eq!(updated.val, 50);
}

#[test]
fn test_init_guard_enforces_version_sequence() {
    use upgrade_patterns::init_guard::InitGuardContract;

    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, InitGuardContract);
    let client = upgrade_patterns::init_guard::InitGuardContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    assert!(client.is_initialized());
    assert_eq!(client.setup_version(), 1);
    assert_eq!(client.feature_flag_v2(), None);

    // The post-upgrade initializer must run in sequence.
    assert_eq!(
        client.try_post_upgrade_init(&5),
        Err(Ok(upgrade_patterns::init_guard::InitError::OutOfSequence))
    );
    client.post_upgrade_init(&2);
    assert_eq!(client.setup_version(), 2);
    assert_eq!(client.feature_flag_v2(), Some(false));

    // Running it twice is rejected.
    assert_eq!(
        client.try_post_upgrade_init(&2),
        Err(Ok(upgrade_patterns::init_guard::InitError::AlreadyRan))
    );
}

// ---------------------------------------------------------------------------
// Beacon management
// ---------------------------------------------------------------------------

#[test]
fn test_beacon_management_register_upgrade_and_rollback() {
    let env = Env::default();
    env.mock_all_auths();

    let id = env.register_contract(None, beacon_management::BeaconManagementContract);
    let client = beacon_management::BeaconManagementContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);

    client.initialize(&admin);

    let name = symbol_short!("alpha");
    assert_eq!(client.register_beacon(&admin, &name, &v1), 1);
    assert_eq!(client.list_beacons().len(), 1);
    assert_eq!(client.get_beacon(&name).current_implementation, v1);

    assert_eq!(client.upgrade_beacon(&admin, &name, &v2), 2);
    let state = client.get_beacon(&name);
    assert_eq!(state.current_version, 2);
    assert_eq!(state.current_implementation, v2);
    assert_eq!(state.history.len(), 2);

    // Rolling back restores the previous implementation.
    assert_eq!(client.rollback_beacon(&admin, &name), 1);
    assert_eq!(client.get_beacon(&name).current_implementation, v1);
}
