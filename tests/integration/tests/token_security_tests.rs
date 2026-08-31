//! Security tests for token examples (Issue #795).
//!
//! Covers the three vulnerability classes the issue calls out — reentrancy,
//! arithmetic issues, and authorization bypass attempts — against real
//! example crates rather than synthetic contracts:
//!
//! - **Authorization bypass** (`sep41_token`, `examples/tokens/01-sep41-token`):
//!   that crate's own `src/test.rs` calls `env.mock_all_auths()` in every
//!   fixture, which approves every `require_auth()` call unconditionally —
//!   so while it thoroughly covers business logic, it never actually
//!   exercises Soroban's authorization enforcement itself. The tests below
//!   disable the blanket mock (`env.mock_auths(&[])`, the same technique
//!   already used in `examples/basics/03-authentication/src/test.rs` and
//!   `tests/integration/tests/basic_security_tests.rs`) and confirm every
//!   state-mutating entry point genuinely rejects a call lacking its
//!   required signer's real authorization.
//! - **Arithmetic issues** (`sep41_token`): boundary tests at `i128::MAX`
//!   confirming the `checked_add` guards in `mint`/`transfer` return a
//!   clean `TokenError::ArithmeticOverflow` rather than panicking or
//!   silently wrapping, plus a full mint-to-max/burn-to-zero cycle
//!   confirming `total_supply` bookkeeping stays consistent with the sum of
//!   balances throughout.
//! - **Reentrancy** (`token_wrapper`, `examples/tokens/06-token-wrapper`):
//!   a `MaliciousUnderlyingToken` test double (mirroring the
//!   `MaliciousContract` pattern in
//!   `examples/advanced/05-reentrancy-guard/src/test.rs`) whose `transfer`
//!   calls back into `TokenWrapper::wrap` before the outer call returns.
//!   Demonstrated and fixed as part of this issue — see
//!   `SECURITY_REVIEW_TOKEN_EXAMPLES.md` for the finding and
//!   `examples/tokens/06-token-wrapper/src/lib.rs`'s `DataKey::Entered`
//!   guard for the remediation these tests protect against regressing.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    testutils::Address as _,
    vec, Address, Env, IntoVal, String, Symbol,
};

use sep41_token::{Sep41Token, Sep41TokenClient, TokenError};
use token_wrapper::{TokenWrapper, TokenWrapperClient};

fn setup_sep41(env: &Env) -> (Sep41TokenClient<'static>, Address, Address, Address) {
    let admin = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);

    let token_id = env.register_contract(None, Sep41Token);
    let token = Sep41TokenClient::new(env, &token_id);
    token
        .initialize(
            &admin,
            &String::from_str(env, "Cookbook USD"),
            &symbol_short!("CUSD"),
            &2u32,
            &1_000_000i128,
        )
        .unwrap();

    (token, admin, alice, bob)
}

// ---------------------------------------------------------------------------
// Authorization bypass
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError")]
fn transfer_without_sender_auth_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, _bob) = setup_sep41(&env);

    // Disable the blanket mock: only real, declared authorizations pass now.
    env.mock_auths(&[]);
    token.transfer(&admin, &alice, &100);
}

#[test]
#[should_panic(expected = "HostError")]
fn approve_without_owner_auth_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, _bob) = setup_sep41(&env);

    env.mock_auths(&[]);
    token.approve(&admin, &alice, &100);
}

#[test]
#[should_panic(expected = "HostError")]
fn transfer_from_without_spender_auth_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, bob) = setup_sep41(&env);
    token.approve(&admin, &alice, &500);

    // Alice (the spender) never authorized this specific transfer_from call.
    env.mock_auths(&[]);
    token.transfer_from(&alice, &admin, &bob, &100);
}

#[test]
#[should_panic(expected = "HostError")]
fn mint_without_admin_auth_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, _bob) = setup_sep41(&env);

    env.mock_auths(&[]);
    token.mint(&admin, &alice, &100);
}

#[test]
#[should_panic(expected = "HostError")]
fn burn_without_owner_auth_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, _alice, _bob) = setup_sep41(&env);

    env.mock_auths(&[]);
    token.burn(&admin, &100);
}

/// `require_auth()` alone isn't sufficient authorization for admin-gated
/// actions: an attacker who is a genuine, self-authorizing address (not an
/// impersonation) must still be rejected by `mint`'s own admin-identity
/// check. Unlike the tests above, this one keeps `mock_all_auths()` on
/// (Alice's self-authorization is real and would pass host-level auth) to
/// isolate that this is the *application-level* admin check doing the
/// rejecting, not auth enforcement.
#[test]
fn mint_rejects_a_real_but_non_admin_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, _admin, alice, bob) = setup_sep41(&env);

    assert_eq!(
        token.try_mint(&alice, &bob, &100),
        Err(Ok(TokenError::Unauthorized))
    );
}

// ---------------------------------------------------------------------------
// Arithmetic issues
// ---------------------------------------------------------------------------

#[test]
fn mint_at_i128_max_then_overflow_is_rejected_cleanly() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, _bob) = setup_sep41(&env);

    // setup_sep41 gives admin an initial_supply of 1_000_000, so this single
    // mint brings total_supply to exactly i128::MAX.
    token.mint(&admin, &alice, &(i128::MAX - 1_000_000)).unwrap();
    assert_eq!(token.balance(&alice), i128::MAX - 1_000_000);
    assert_eq!(token.total_supply().unwrap(), i128::MAX);

    // Any further mint must fail cleanly, not panic or silently wrap negative.
    assert_eq!(
        token.try_mint(&admin, &alice, &1),
        Err(Ok(TokenError::ArithmeticOverflow))
    );
    assert_eq!(token.balance(&alice), i128::MAX - 1_000_000);
    assert_eq!(token.total_supply().unwrap(), i128::MAX);
}

#[test]
fn transfer_of_exact_full_balance_succeeds_at_the_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, bob) = setup_sep41(&env);

    let balance = token.balance(&admin);
    token.transfer(&admin, &alice, &balance);
    assert_eq!(token.balance(&admin), 0);
    assert_eq!(token.balance(&alice), balance);

    // Transferring even 1 more from the now-empty account must fail, not
    // underflow.
    assert_eq!(
        token.try_transfer(&admin, &bob, &1),
        Err(Ok(TokenError::InsufficientBalance))
    );
}

#[test]
fn burn_of_exact_full_balance_zeroes_out_without_underflow() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, _alice, _bob) = setup_sep41(&env);

    let balance = token.balance(&admin);
    let remaining = token.burn(&admin, &balance).unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(token.balance(&admin), 0);
    assert_eq!(token.total_supply().unwrap(), 0);

    assert_eq!(
        token.try_burn(&admin, &1),
        Err(Ok(TokenError::InsufficientBalance))
    );
}

#[test]
fn total_supply_stays_consistent_with_balances_across_a_mint_burn_cycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (token, admin, alice, bob) = setup_sep41(&env);

    token.mint(&admin, &alice, &(i128::MAX / 2));
    token.mint(&admin, &bob, &(i128::MAX / 4));
    token.transfer(&admin, &alice, &500_000);
    token.burn(&alice, &250_000);

    let total_supply = token.total_supply().unwrap();
    let sum_of_balances = token.balance(&admin) + token.balance(&alice) + token.balance(&bob);
    assert_eq!(
        total_supply, sum_of_balances,
        "total_supply must always equal the sum of all balances"
    );
}

// ---------------------------------------------------------------------------
// Reentrancy
// ---------------------------------------------------------------------------

#[contracttype]
pub enum MaliciousKey {
    Wrapper,
    Attacking,
    RealBalance(Address),
}

/// A hostile "underlying token" whose `transfer` calls back into
/// `TokenWrapper::wrap` before returning — the attack `06-token-wrapper`'s
/// `DataKey::Entered` guard exists to block. Mirrors the `MaliciousContract`
/// pattern in `examples/advanced/05-reentrancy-guard/src/test.rs`.
#[contract]
pub struct MaliciousUnderlyingToken;

#[contractimpl]
impl MaliciousUnderlyingToken {
    /// `attacking`: whether `transfer` should try to reenter `wrap` (matches
    /// the `attack_type` parameter convention in
    /// `05-reentrancy-guard/src/test.rs`'s `MaliciousContract::init`).
    pub fn init(env: Env, wrapper: Address, attacking: bool) {
        env.storage().instance().set(&MaliciousKey::Wrapper, &wrapper);
        env.storage().instance().set(&MaliciousKey::Attacking, &attacking);
    }

    pub fn set_balance(env: Env, id: Address, amount: i128) {
        env.storage()
            .persistent()
            .set(&MaliciousKey::RealBalance(id), &amount);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&MaliciousKey::RealBalance(id))
            .unwrap_or(0)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        // Move the "real" ledger like a normal token would, so a
        // successful (non-reentrant) call still leaves believable balances.
        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&MaliciousKey::RealBalance(from.clone()))
            .unwrap_or(0);
        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&MaliciousKey::RealBalance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&MaliciousKey::RealBalance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&MaliciousKey::RealBalance(to), &(to_bal + amount));

        let attacking: bool = env
            .storage()
            .instance()
            .get(&MaliciousKey::Attacking)
            .unwrap_or(false);
        if attacking {
            // Disable further attacks first so the reentrant `wrap` call
            // below doesn't recurse forever if it (wrongly) succeeds.
            env.storage().instance().set(&MaliciousKey::Attacking, &false);
            let wrapper: Address = env
                .storage()
                .instance()
                .get(&MaliciousKey::Wrapper)
                .unwrap();
            // Try to mint a second batch of wrapped shares against the same
            // real transfer that's still in flight.
            let _: i128 = env.invoke_contract(
                &wrapper,
                &Symbol::new(&env, "wrap"),
                vec![&env, from.into_val(&env), amount.into_val(&env)],
            );
        }
    }
}

#[test]
fn wrap_succeeds_normally_against_a_non_reentrant_token() {
    let env = Env::default();
    env.mock_all_auths();

    let wrapper_id = env.register_contract(None, TokenWrapper);
    let wrapper = TokenWrapperClient::new(&env, &wrapper_id);

    let underlying_id = env.register_contract(None, MaliciousUnderlyingToken);
    let underlying = MaliciousUnderlyingTokenClient::new(&env, &underlying_id);
    underlying.init(&wrapper_id, &false); // attacking = false: behaves like a normal token

    wrapper.initialize(&underlying_id).unwrap();

    let alice = Address::generate(&env);
    underlying.set_balance(&alice, &1_000);

    assert_eq!(wrapper.wrap(&alice, &400).unwrap(), 400);
    assert_eq!(wrapper.balance(&alice), 400);
    assert_eq!(underlying.balance(&wrapper_id), 400);
    assert_eq!(underlying.balance(&alice), 600);
}

#[test]
#[should_panic]
fn wrap_reentrancy_attack_is_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let wrapper_id = env.register_contract(None, TokenWrapper);
    let wrapper = TokenWrapperClient::new(&env, &wrapper_id);

    let underlying_id = env.register_contract(None, MaliciousUnderlyingToken);
    let underlying = MaliciousUnderlyingTokenClient::new(&env, &underlying_id);
    underlying.init(&wrapper_id, &true); // attacking = true

    wrapper.initialize(&underlying_id).unwrap();

    let alice = Address::generate(&env);
    underlying.set_balance(&alice, &1_000);

    // A single real deposit of 400. The malicious token's `transfer` will
    // try to call `wrap` a second time for the same 400 before this call
    // returns — the reentrancy guard must reject that inner call and abort
    // the whole transaction, so it must never leave `alice` credited with
    // 800 wrapped shares for a single 400-token deposit.
    wrapper.wrap(&alice, &400);
}
