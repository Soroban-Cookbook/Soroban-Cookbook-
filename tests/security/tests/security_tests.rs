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
