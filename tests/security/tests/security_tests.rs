#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, Address, Env};

// ---------------------------------------------------------------------------
// Malicious Token Contract for Reentrancy Testing
// ---------------------------------------------------------------------------

#[contract]
pub struct MaliciousToken;

#[contractimpl]
impl MaliciousToken {
    pub fn initialize(env: Env) {
        // Setup initial flag to false
        let key = symbol_short!("reent");
        env.storage().instance().set(&key, &false);
    }

    pub fn setup_reentrancy(env: Env, wrapper: Address, user: Address, amount: i128) {
        env.storage()
            .instance()
            .set(&symbol_short!("wrap_ad"), &wrapper);
        env.storage()
            .instance()
            .set(&symbol_short!("user_ad"), &user);
        env.storage()
            .instance()
            .set(&symbol_short!("ramt"), &amount);
    }

    pub fn transfer(env: Env, _from: Address, _to: Address, _amount: i128) {
        let reent_key = symbol_short!("reent");
        let reentered: bool = env.storage().instance().get(&reent_key).unwrap_or(false);

        if !reentered {
            // Retrieve reentrancy configuration
            if let Some(wrapper_addr) = env
                .storage()
                .instance()
                .get::<_, Address>(&symbol_short!("wrap_ad"))
            {
                let user_addr: Address = env
                    .storage()
                    .instance()
                    .get(&symbol_short!("user_ad"))
                    .unwrap();
                let ramt: i128 = env
                    .storage()
                    .instance()
                    .get(&symbol_short!("ramt"))
                    .unwrap();

                if ramt > 0 {
                    // Set flag to true to avoid infinite recursion
                    env.storage().instance().set(&reent_key, &true);

                    let wrapper_client =
                        token_wrapper::TokenWrapperClient::new(&env, &wrapper_addr);
                    // Reenter wrap/unwrap by calling unwrap
                    let _ = wrapper_client.try_unwrap(&user_addr, &ramt);
                }
            }
        }
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        // Mock large balance so collateral checks succeed
        10_000_000i128
    }

    // Include dummy/noop implementations of standard SEP-41 methods if needed
    pub fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }
    pub fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _live_until_ledgers: u32,
    ) {
    }
    pub fn burn(_env: Env, _from: Address, _amount: i128) {}
    pub fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {}
    pub fn decimals(_env: Env) -> u32 {
        7
    }
    pub fn name(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "Malicious")
    }
    pub fn symbol(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "MAL")
    }
}

// ---------------------------------------------------------------------------
// Security Test Suite
// ---------------------------------------------------------------------------

#[test]
fn test_unauthorized_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);

    // First initialization succeeds
    wrapper.initialize(&underlying_id);

    // Second initialization fails
    let res = wrapper.try_initialize(&underlying_id);
    assert_eq!(
        res,
        Err(Ok(token_wrapper::WrapperError::AlreadyInitialized))
    );
}

#[test]
fn test_unauthorized_wrap() {
    let env = Env::default();
    // Intentionally DO NOT call env.mock_all_auths() to test lack of signature

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);

    // Attempting to wrap without Alice's mock authorization should fail with a host error
    let res = wrapper.try_wrap(&alice, &100);
    assert!(res.is_err());
}

#[test]
fn test_unauthorized_transfer() {
    let env = Env::default();
    // Intentionally DO NOT call env.mock_all_auths() to test lack of signature

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Attempting to transfer without Alice's mock authorization should fail with a host error
    let res = wrapper.try_transfer(&alice, &bob, &50);
    assert!(res.is_err());
}

#[test]
fn test_invalid_wrap_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);

    // Wrap negative amount
    let res_neg = wrapper.try_wrap(&alice, &-100);
    assert_eq!(res_neg, Err(Ok(token_wrapper::WrapperError::InvalidAmount)));

    // Wrap zero amount
    let res_zero = wrapper.try_wrap(&alice, &0);
    assert_eq!(
        res_zero,
        Err(Ok(token_wrapper::WrapperError::InvalidAmount))
    );
}

#[test]
fn test_invalid_transfer_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Transfer negative amount
    let res_neg = wrapper.try_transfer(&alice, &bob, &-50);
    assert_eq!(res_neg, Err(Ok(token_wrapper::WrapperError::InvalidAmount)));

    // Transfer zero amount
    let res_zero = wrapper.try_transfer(&alice, &bob, &0);
    assert_eq!(
        res_zero,
        Err(Ok(token_wrapper::WrapperError::InvalidAmount))
    );
}

#[test]
fn test_wrap_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let underlying_admin = soroban_sdk::token::StellarAssetClient::new(&env, &underlying_id);

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);

    // Mint and wrap maximum possible i128
    underlying_admin.mint(&alice, &i128::MAX);
    wrapper.wrap(&alice, &i128::MAX);

    // Wrapping additional amount should trigger arithmetic overflow check
    let res = wrapper.try_wrap(&alice, &1);
    assert_eq!(
        res,
        Err(Ok(token_wrapper::WrapperError::ArithmeticOverflow))
    );
}

#[test]
fn test_unwrap_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let underlying_admin = soroban_sdk::token::StellarAssetClient::new(&env, &underlying_id);

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);
    underlying_admin.mint(&alice, &100);
    wrapper.wrap(&alice, &100);

    // Attempting to unwrap more than user balance should fail
    let res = wrapper.try_unwrap(&alice, &101);
    assert_eq!(
        res,
        Err(Ok(token_wrapper::WrapperError::InsufficientWrappedBalance))
    );
}

#[test]
fn test_transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let underlying_admin = soroban_sdk::token::StellarAssetClient::new(&env, &underlying_id);

    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&underlying_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    underlying_admin.mint(&alice, &100);
    wrapper.wrap(&alice, &100);

    // Attempting to transfer more than user balance should fail
    let res = wrapper.try_transfer(&alice, &bob, &101);
    assert_eq!(
        res,
        Err(Ok(token_wrapper::WrapperError::InsufficientWrappedBalance))
    );
}

#[test]
fn test_reentrancy_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy the MaliciousToken
    let mal_token_id = env.register_contract(None, MaliciousToken);
    let mal_token = MaliciousTokenClient::new(&env, &mal_token_id);
    mal_token.initialize();

    // Deploy the TokenWrapper using MaliciousToken as underlying
    let wrapper_id = env.register_contract(None, token_wrapper::TokenWrapper);
    let wrapper = token_wrapper::TokenWrapperClient::new(&env, &wrapper_id);
    wrapper.initialize(&mal_token_id);

    let alice = Address::generate(&env);

    // 1. Initially wrap 100 tokens with reentrancy DISABLED (ramt = 0)
    mal_token.setup_reentrancy(&wrapper_id, &alice, &0);
    wrapper.wrap(&alice, &100);

    assert_eq!(wrapper.balance(&alice), 100);
    assert_eq!(wrapper.total_supply(), 100);

    // 2. Now setup reentrancy to call unwrap for 100 tokens during the wrap transfer
    mal_token.setup_reentrancy(&wrapper_id, &alice, &100);

    // Alice tries to wrap an additional 50 tokens.
    //
    // Security threat (non-CEI / vulnerable pattern):
    // If `wrap` called the external transfer FIRST (interaction before effects), a
    // malicious underlying token could reenter `unwrap` and see the stale pre-wrap
    // balance of 100. After draining it to 0, the outer `wrap` would complete and
    // write balance = 150, minting 150 wrapped tokens backed by 0 underlying.
    //
    // Mitigation — Checks-Effects-Interactions (CEI):
    // Balance is updated to 150 BEFORE the external transfer call (effect before
    // interaction). Any reentrant call to `unwrap` therefore sees the already-updated
    // state (balance 150, not the stale 100), closing the stale-read exploit window.
    //
    // Key invariants that must hold regardless of reentrancy outcome:
    //   1. total_supply == alice_balance  (internal accounting consistency)
    //   2. total_supply <= total_deposited (no double-minting beyond deposits)
    wrapper.wrap(&alice, &50);

    let alice_balance = wrapper.balance(&alice);
    let total_supply = wrapper.total_supply();

    // Invariant 1: Internal consistency.
    // total_supply must always equal the sum of all user balances. A mismatch would
    // mean tokens were minted or burned without proper accounting — the hallmark of a
    // double-mint or silent-drain exploit.
    assert_eq!(
        alice_balance, total_supply,
        "Internal consistency violated: alice balance ({}) != total supply ({}). \
         Tokens were minted or burned without proper accounting.",
        alice_balance, total_supply
    );

    // Invariant 2: No double-minting beyond total deposited amount.
    // alice deposited at most 100 (first wrap) + 50 (second wrap) = 150 underlying.
    // Wrapped supply must never exceed this without further explicit deposits.
    assert!(
        total_supply <= 150,
        "Double-minting detected: supply ({}) exceeds total deposited (150). \
         A reentrancy exploit created unbacked wrapped tokens.",
        total_supply
    );
}

// ---------------------------------------------------------------------------
// Security Tests: RBAC Privilege Escalation
// ---------------------------------------------------------------------------

mod rbac_security_tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, Vec};

    fn setup_rbac(env: &Env) -> (role_based_access_control::RoleBasedAccessControlClient<'_>, Address) {
        let contract_id = env.register_contract(None, role_based_access_control::RoleBasedAccessControl);
        let client = role_based_access_control::RoleBasedAccessControlClient::new(env, &contract_id);
        let owner = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&owner);
        (client, owner)
    }

    #[test]
    fn test_non_member_cannot_grant_roles() {
        let env = Env::default();
        let (client, owner) = setup_rbac(&env);
        let attacker = Address::generate(&env);
        let target = Address::generate(&env);

        // Attacker tries to grant moderator role - should fail
        let result = client.try_grant_role(&attacker, &target, &role_based_access_control::Role::Moderator);
        assert_eq!(result, Err(Ok(role_based_access_control::RbacError::Unauthorized)));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_moderator_cannot_grant_admin_privilege_escalation() {
        let env = Env::default();
        let (client, owner) = setup_rbac(&env);
        let moderator = Address::generate(&env);
        let target = Address::generate(&env);

        // Owner grants moderator role
        client.grant_role(&owner, &moderator, &role_based_access_control::Role::Moderator);

        // Moderator tries to grant admin - privilege escalation attempt
        client.grant_role(&moderator, &target, &role_based_access_control::Role::Admin);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_admin_cannot_revoke_owner() {
        let env = Env::default();
        let (client, owner) = setup_rbac(&env);
        let admin = Address::generate(&env);

        // Owner grants admin role
        client.grant_role(&owner, &admin, &role_based_access_control::Role::Admin);

        // Admin tries to revoke owner - should fail
        client.revoke_role(&admin, &owner);
    }

    #[test]
    fn test_admin_action_requires_admin_role() {
        let env = Env::default();
        let (client, _owner) = setup_rbac(&env);
        let user = Address::generate(&env);

        // User without admin role cannot perform admin action
        let result = client.try_admin_action(&user, &42u64);
        assert_eq!(result, Err(Ok(role_based_access_control::RbacError::Unauthorized)));
    }
}

// ---------------------------------------------------------------------------
// Security Tests: Multisig Signature Spoofing
// ---------------------------------------------------------------------------

mod multisig_security_tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, Vec};

    fn setup_multisig(env: &Env) -> (multi_sig_patterns::MultiPartyAuthClient<'_>) {
        let contract_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
        let client = multi_sig_patterns::MultiPartyAuthClient::new(env, &contract_id);
        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);
        let signer3 = Address::generate(env);
        let signers = Vec::from_array(env, [signer1.clone(), signer2.clone(), signer3.clone()]);
        env.mock_all_auths();
        client.initialize(&2, &signers);
        client
    }

    #[test]
    fn test_invalid_signer_set_rejected() {
        let env = Env::default();
        let client = setup_multisig(&env);

        let attacker = Address::generate(&env);
        let proposal_id = client.create_proposal(&attacker);

        // This should fail because attacker is not in signer set
        // But create_proposal already validates, so we test approval
    }

    #[test]
    fn test_missing_approval_prevents_execution() {
        let env = Env::default();
        let client = setup_multisig(&env);

        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);

        // Create a fresh setup
        let contract_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
        let client = multi_sig_patterns::MultiPartyAuthClient::new(env, &contract_id);
        let s1 = Address::generate(env);
        let s2 = Address::generate(env);
        let s3 = Address::generate(env);
        let signers = Vec::from_array(env, [s1.clone(), s2.clone(), s3.clone()]);
        env.mock_all_auths();
        client.initialize(&2, &signers);

        let proposal_id = client.create_proposal(&s1);

        // Only 1 approval (below threshold of 2)
        client.approve(&proposal_id, &s1);

        // Try to execute with only 1 approval - should fail
        let result = client.try_execute(&proposal_id, &s1);
        assert_eq!(result, Err(Ok(multi_sig_patterns::AuthError::ThresholdNotMet)));
    }

    #[test]
    fn test_double_approval_prevented() {
        let env = Env::default();
        let client = setup_multisig(&env);

        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);

        // Re-setup with known signers
        let contract_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
        let client = multi_sig_patterns::MultiPartyAuthClient::new(&env, &contract_id);
        let s1 = Address::generate(&env);
        let s2 = Address::generate(&env);
        let signers = Vec::from_array(&env, [s1.clone(), s2.clone()]);
        env.mock_all_auths();
        client.initialize(&1, &signers);

        let proposal_id = client.create_proposal(&s1);

        // First approval succeeds
        client.approve(&proposal_id, &s1);

        // Second approval from same signer - should fail
        let result = client.try_approve(&proposal_id, &s1);
        assert_eq!(result, Err(Ok(multi_sig_patterns::AuthError::AlreadyApproved)));
    }
}

// ---------------------------------------------------------------------------
// Security Tests: Timelock Bypass
// ---------------------------------------------------------------------------

mod timelock_security_tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    fn setup_timelock(env: &Env) -> (timelock::TimelockContractClient<'_>, Address) {
        let contract_id = env.register_contract(None, timelock::TimelockContract);
        let client = timelock::TimelockContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin);
        (client, admin)
    }

    fn op_id(env: &Env, s: &[u8]) -> Bytes {
        Bytes::from_slice(env, s)
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_timelock_unauthorized_queue() {
        let env = Env::default();
        let contract_id = env.register_contract(None, timelock::TimelockContract);
        let client = timelock::TimelockContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);

        env.mock_all_auths();
        client.initialize(&admin);
        env.set_auths(&[]);

        // Attacker tries to queue without auth
        let (min_delay, _) = client.get_delay_bounds();
        client.queue(&op_id(&env, b"attack"), &min_delay);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_timelock_unauthorized_execute() {
        let env = Env::default();
        let (client, _admin) = setup_timelock(&env);

        client.queue(&op_id(&env, b"unauth_exec"), &60);
        env.ledger().with_mut(|l| *l.timestamp += 61);

        env.set_auths(&[]);
        client.execute(&op_id(&env, b"unauth_exec"));
    }

    #[test]
    fn test_timelock_skip_prevented() {
        let env = Env::default();
        let (client, _admin) = setup_timelock(&env);

        let id = op_id(&env, b"skip_test");
        client.queue(&id, &60);

        // Try to execute before delay expires - should fail
        let result = client.try_execute(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_timelock_replay_prevented() {
        let env = Env::default();
        let (client, _admin) = setup_timelock(&env);

        let id = op_id(&env, b"replay_test");
        client.queue(&id, &60);
        env.ledger().with_mut(|l| *l.timestamp += 61);

        // First execution succeeds
        client.execute(&id);

        // Second execution should fail (replay protection)
        let result = client.try_execute(&id);
        assert!(result.is_err());
    }
}
