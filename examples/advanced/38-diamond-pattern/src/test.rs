#![cfg(test)]
#![allow(deprecated)]

extern crate std;

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, IntoVal, Val, Vec};

use crate::{
    ArgType, CounterFacet, CounterFacetClient, DiamondBase, DiamondBaseClient, DispatchError,
    FacetCut, FacetCutAction, TokenFacet, TokenFacetClient,
};

// ---------------------------------------------------------------------------
// Helper: register the three contracts and initialise the diamond
// ---------------------------------------------------------------------------

fn setup() -> (
    Env,
    Address,
    DiamondBaseClient<'static>,
    Address,
    TokenFacetClient<'static>,
    Address,
    CounterFacetClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let diamond_id = env.register(DiamondBase, ());
    let token_id = env.register(TokenFacet, ());
    let counter_id = env.register(CounterFacet, ());

    let diamond = DiamondBaseClient::new(&env, &diamond_id);
    let token = TokenFacetClient::new(&env, &token_id);
    let counter = CounterFacetClient::new(&env, &counter_id);

    let admin = Address::generate(&env);
    diamond.initialize(&admin);

    (env, admin, diamond, token_id, token, counter_id, counter)
}

// ---------------------------------------------------------------------------
// DiamondBase — initialisation
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_creates_empty_registry() {
    let (_env, _admin, diamond, _tid, _token, _cid, _counter) = setup();
    assert_eq!(diamond.selector_count(), 0);
    assert_eq!(diamond.facet_addresses().len(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let (env, _admin, diamond, _tid, _token, _cid, _counter) = setup();
    let other = Address::generate(&env);
    diamond.initialize(&other);
}

// ---------------------------------------------------------------------------
// Diamond cut — Add
// ---------------------------------------------------------------------------

#[test]
fn test_diamond_cut_add_registers_selectors() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();

    let mut cuts = Vec::new(&env);
    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("tf_mint"));
    sels.push_back(symbol_short!("tf_bal"));
    cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    assert_eq!(diamond.selector_count(), 2);
    assert_eq!(
        diamond.facet_address(&symbol_short!("tf_mint")),
        Some(token_id.clone())
    );
    assert_eq!(
        diamond.facet_address(&symbol_short!("tf_bal")),
        Some(token_id)
    );
}

#[test]
fn test_diamond_cut_add_multiple_facets() {
    let (env, admin, diamond, token_id, _token, counter_id, _counter) = setup();

    let mut token_sels = Vec::new(&env);
    token_sels.push_back(symbol_short!("tf_mint"));
    token_sels.push_back(symbol_short!("tf_trsf"));

    let mut counter_sels = Vec::new(&env);
    counter_sels.push_back(symbol_short!("cf_inc"));
    counter_sels.push_back(symbol_short!("cf_cnt"));

    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id,
        action: FacetCutAction::Add,
        selectors: token_sels,
    });
    cuts.push_back(FacetCut {
        facet_address: counter_id,
        action: FacetCutAction::Add,
        selectors: counter_sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    assert_eq!(diamond.selector_count(), 4);
    assert_eq!(diamond.facet_addresses().len(), 2);
}

#[test]
#[should_panic(expected = "selector already registered")]
fn test_add_duplicate_selector_panics() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();

    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("tf_mint"));

    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels.clone(),
    });
    diamond.diamond_cut(&admin, &cuts);

    // Adding the same selector again must panic.
    let mut cuts2 = Vec::new(&env);
    cuts2.push_back(FacetCut {
        facet_address: token_id,
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &cuts2);
}

// ---------------------------------------------------------------------------
// Diamond cut — Replace
// ---------------------------------------------------------------------------

#[test]
fn test_diamond_cut_replace_remaps_selector() {
    let (env, admin, diamond, token_id, _token, counter_id, _counter) = setup();

    // Register under token facet first.
    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("shared"));

    let mut add_cuts = Vec::new(&env);
    add_cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels.clone(),
    });
    diamond.diamond_cut(&admin, &add_cuts);
    assert_eq!(
        diamond.facet_address(&symbol_short!("shared")),
        Some(token_id)
    );

    // Replace: now points to counter facet.
    let mut replace_cuts = Vec::new(&env);
    replace_cuts.push_back(FacetCut {
        facet_address: counter_id.clone(),
        action: FacetCutAction::Replace,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &replace_cuts);
    assert_eq!(
        diamond.facet_address(&symbol_short!("shared")),
        Some(counter_id)
    );
    // Total selector count stays the same.
    assert_eq!(diamond.selector_count(), 1);
}

// ---------------------------------------------------------------------------
// Diamond cut — Remove
// ---------------------------------------------------------------------------

#[test]
fn test_diamond_cut_remove_deregisters_selector() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();

    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("tf_mint"));
    sels.push_back(symbol_short!("tf_bal"));

    let mut add_cuts = Vec::new(&env);
    add_cuts.push_back(FacetCut {
        facet_address: token_id,
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &add_cuts);
    assert_eq!(diamond.selector_count(), 2);

    // Remove one selector.
    let mut rm_sels = Vec::new(&env);
    rm_sels.push_back(symbol_short!("tf_mint"));
    let mut rm_cuts = Vec::new(&env);
    rm_cuts.push_back(FacetCut {
        facet_address: Address::generate(&env),
        action: FacetCutAction::Remove,
        selectors: rm_sels,
    });
    diamond.diamond_cut(&admin, &rm_cuts);

    assert_eq!(diamond.selector_count(), 1);
    assert_eq!(diamond.facet_address(&symbol_short!("tf_mint")), None);
    assert!(diamond.facet_address(&symbol_short!("tf_bal")).is_some());
}

#[test]
fn test_remove_last_selector_prunes_facet_from_list() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();

    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("only_sel"));

    let mut add_cuts = Vec::new(&env);
    add_cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels.clone(),
    });
    diamond.diamond_cut(&admin, &add_cuts);
    assert_eq!(diamond.facet_addresses().len(), 1);

    // Removing the only selector should prune the facet from the list.
    let mut rm_cuts = Vec::new(&env);
    rm_cuts.push_back(FacetCut {
        facet_address: Address::generate(&env),
        action: FacetCutAction::Remove,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &rm_cuts);

    assert_eq!(diamond.facet_addresses().len(), 0);
    assert_eq!(diamond.selector_count(), 0);
}

// ---------------------------------------------------------------------------
// Diamond loupe — introspection
// ---------------------------------------------------------------------------

#[test]
fn test_loupe_facet_addresses() {
    let (env, admin, diamond, token_id, _token, counter_id, _counter) = setup();

    let mut token_sels = Vec::new(&env);
    token_sels.push_back(symbol_short!("tf_mint"));
    let mut counter_sels = Vec::new(&env);
    counter_sels.push_back(symbol_short!("cf_inc"));

    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: token_sels,
    });
    cuts.push_back(FacetCut {
        facet_address: counter_id.clone(),
        action: FacetCutAction::Add,
        selectors: counter_sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    let addrs = diamond.facet_addresses();
    assert_eq!(addrs.len(), 2);
    assert!(addrs.contains(&token_id));
    assert!(addrs.contains(&counter_id));
}

#[test]
fn test_loupe_facet_function_selectors() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();

    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("tf_mint"));
    sels.push_back(symbol_short!("tf_trsf"));

    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    let facet_sels = diamond.facet_function_selectors(&token_id);
    assert_eq!(facet_sels.len(), 2);
    assert!(facet_sels.contains(&symbol_short!("tf_mint")));
    assert!(facet_sels.contains(&symbol_short!("tf_trsf")));
}

#[test]
fn test_loupe_facets_returns_complete_registry() {
    let (env, admin, diamond, token_id, _token, counter_id, _counter) = setup();

    let mut token_sels = Vec::new(&env);
    token_sels.push_back(symbol_short!("tf_mint"));
    let mut counter_sels = Vec::new(&env);
    counter_sels.push_back(symbol_short!("cf_inc"));

    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id,
        action: FacetCutAction::Add,
        selectors: token_sels,
    });
    cuts.push_back(FacetCut {
        facet_address: counter_id,
        action: FacetCutAction::Add,
        selectors: counter_sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    let all_facets = diamond.facets();
    assert_eq!(all_facets.len(), 2);
}

#[test]
fn test_loupe_unknown_selector_returns_none() {
    let (_env, _admin, diamond, _tid, _token, _cid, _counter) = setup();
    assert_eq!(diamond.facet_address(&symbol_short!("nope")), None);
}

// ---------------------------------------------------------------------------
// Fallback dispatch — resolving a facet and calling it directly
// ---------------------------------------------------------------------------

#[test]
fn test_fallback_dispatch_via_facet_address() {
    let (env, admin, diamond, token_id, token, _cid, _counter) = setup();

    // Register the TokenFacet for the "tf_mint" selector.
    let mut sels = Vec::new(&env);
    sels.push_back(symbol_short!("tf_mint"));
    let mut cuts = Vec::new(&env);
    cuts.push_back(FacetCut {
        facet_address: token_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(&admin, &cuts);

    // Fallback mechanism: resolve the facet for "tf_mint".
    let resolved = diamond
        .facet_address(&symbol_short!("tf_mint"))
        .expect("selector not registered");
    assert_eq!(resolved, token_id);

    // Dispatch to the resolved facet.
    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);
    token.tf_mint(&minter, &recipient, &750i128);

    assert_eq!(token.tf_balance_of(&recipient), 750);
    assert_eq!(token.tf_total_supply(), 750);
}

// ---------------------------------------------------------------------------
// Validated dispatch — ABI checking before facet invocation
// ---------------------------------------------------------------------------

/// Register `selector` → `facet_id` on the diamond.
fn register_selector(
    env: &Env,
    diamond: &DiamondBaseClient,
    admin: &Address,
    facet_id: &Address,
    selector: &soroban_sdk::Symbol,
) {
    let mut sels = Vec::new(env);
    sels.push_back(selector.clone());
    let mut cuts = Vec::new(env);
    cuts.push_back(FacetCut {
        facet_address: facet_id.clone(),
        action: FacetCutAction::Add,
        selectors: sels,
    });
    diamond.diamond_cut(admin, &cuts);
}

fn tf_mint_args(env: &Env, minter: &Address, to: &Address, amount: i128) -> Vec<Val> {
    let mut args = Vec::new(env);
    args.push_back(minter.into_val(env));
    args.push_back(to.into_val(env));
    args.push_back(amount.into_val(env));
    args
}

#[test]
fn test_dispatch_valid_args_invokes_facet() {
    let (env, admin, diamond, token_id, token, _cid, _counter) = setup();
    // tf_mint's require_auth happens in the facet (non-root) invocation.
    env.mock_all_auths_allowing_non_root_auth();
    register_selector(&env, &diamond, &admin, &token_id, &symbol_short!("tf_mint"));

    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);
    let args = tf_mint_args(&env, &minter, &recipient, 750i128);

    let result = diamond.try_dispatch(&symbol_short!("tf_mint"), &args);
    assert!(matches!(result, Ok(Ok(_))));
    assert_eq!(token.tf_balance_of(&recipient), 750);
}

#[test]
fn test_dispatch_wrong_arg_type_returns_typed_error() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();
    register_selector(&env, &diamond, &admin, &token_id, &symbol_short!("tf_mint"));

    // Third arg should be i128; pass a Symbol instead.
    let mut args = Vec::new(&env);
    args.push_back(Address::generate(&env).into_val(&env));
    args.push_back(Address::generate(&env).into_val(&env));
    args.push_back(symbol_short!("oops").into_val(&env));

    let result = diamond.try_dispatch(&symbol_short!("tf_mint"), &args);
    assert!(matches!(result, Err(Ok(DispatchError::InvalidArguments))));
}

#[test]
fn test_dispatch_wrong_arg_count_returns_typed_error() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();
    register_selector(&env, &diamond, &admin, &token_id, &symbol_short!("tf_mint"));

    // tf_mint expects 3 args; supply only 2.
    let mut args = Vec::new(&env);
    args.push_back(Address::generate(&env).into_val(&env));
    args.push_back(Address::generate(&env).into_val(&env));

    let result = diamond.try_dispatch(&symbol_short!("tf_mint"), &args);
    assert!(matches!(result, Err(Ok(DispatchError::ArgCountMismatch))));
}

#[test]
fn test_dispatch_unregistered_selector_returns_typed_error() {
    let (env, _admin, diamond, _tid, _token, _cid, _counter) = setup();

    let args = Vec::new(&env);
    let result = diamond.try_dispatch(&symbol_short!("nope"), &args);
    assert!(matches!(result, Err(Ok(DispatchError::UnknownSelector))));
}

#[test]
fn test_dispatch_unknown_signature_returns_typed_error() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();
    // "mystery" is registered to a facet but has no known signature.
    register_selector(&env, &diamond, &admin, &token_id, &symbol_short!("mystery"));

    let args = Vec::new(&env);
    let result = diamond.try_dispatch(&symbol_short!("mystery"), &args);
    assert!(matches!(result, Err(Ok(DispatchError::UnknownSignature))));
}

#[test]
fn test_register_signature_overrides_builtin() {
    let (env, admin, diamond, token_id, _token, _cid, _counter) = setup();
    register_selector(&env, &diamond, &admin, &token_id, &symbol_short!("tf_mint"));

    // Built-in signature for tf_mint is [Address, Address, I128].
    let builtin = diamond
        .function_signature(&symbol_short!("tf_mint"))
        .expect("builtin signature missing");
    assert_eq!(builtin.len(), 3);

    // Override with a different signature: now expects a single Address.
    let mut arg_types = Vec::new(&env);
    arg_types.push_back(ArgType::Address);
    diamond.register_signature(&admin, &symbol_short!("tf_mint"), &arg_types);

    let sig = diamond
        .function_signature(&symbol_short!("tf_mint"))
        .expect("registered signature missing");
    assert_eq!(sig.len(), 1);

    // Dispatching the real 3-arg call now fails count validation.
    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);
    let args = tf_mint_args(&env, &minter, &recipient, 10i128);
    let result = diamond.try_dispatch(&symbol_short!("tf_mint"), &args);
    assert!(matches!(result, Err(Ok(DispatchError::ArgCountMismatch))));
}

#[test]
fn test_dispatch_counter_facet_via_builtin_signature() {
    let (env, admin, diamond, _tid, _token, counter_id, counter) = setup();

    // "cf_get_count" is 12 chars — too long for symbol_short!.
    let selector = soroban_sdk::Symbol::new(&env, "cf_get_count");
    register_selector(&env, &diamond, &admin, &counter_id, &selector);

    counter.cf_increment(&Address::generate(&env), &symbol_short!("hits"));
    counter.cf_increment(&Address::generate(&env), &symbol_short!("hits"));

    let mut args = Vec::new(&env);
    args.push_back(symbol_short!("hits").into_val(&env));

    let result = diamond.try_dispatch(&selector, &args);
    assert!(matches!(result, Ok(Ok(_))));
}

// ---------------------------------------------------------------------------
// TokenFacet
// ---------------------------------------------------------------------------

#[test]
fn test_token_mint_and_balance() {
    let (env, _admin, _diamond, _tid, token, _cid, _counter) = setup();
    let minter = Address::generate(&env);
    let recipient = Address::generate(&env);

    token.tf_mint(&minter, &recipient, &1_000i128);

    assert_eq!(token.tf_balance_of(&recipient), 1_000);
    assert_eq!(token.tf_total_supply(), 1_000);
}

#[test]
fn test_token_transfer_updates_balances() {
    let (env, _admin, _diamond, _tid, token, _cid, _counter) = setup();
    let minter = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    token.tf_mint(&minter, &alice, &500i128);
    token.tf_transfer(&alice, &bob, &200i128);

    assert_eq!(token.tf_balance_of(&alice), 300);
    assert_eq!(token.tf_balance_of(&bob), 200);
    assert_eq!(token.tf_total_supply(), 500);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_token_transfer_insufficient_balance_panics() {
    let (env, _admin, _diamond, _tid, token, _cid, _counter) = setup();
    let minter = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    token.tf_mint(&minter, &alice, &100i128);
    token.tf_transfer(&alice, &bob, &200i128);
}

#[test]
fn test_token_approve_and_transfer_from() {
    let (env, _admin, _diamond, _tid, token, _cid, _counter) = setup();
    let minter = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    token.tf_mint(&minter, &owner, &1_000i128);
    token.tf_approve(&owner, &spender, &400i128);
    token.tf_transfer_from(&spender, &owner, &recipient, &300i128);

    assert_eq!(token.tf_balance_of(&owner), 700);
    assert_eq!(token.tf_balance_of(&recipient), 300);
}

#[test]
#[should_panic(expected = "allowance exceeded")]
fn test_token_transfer_from_exceeds_allowance_panics() {
    let (env, _admin, _diamond, _tid, token, _cid, _counter) = setup();
    let minter = Address::generate(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    token.tf_mint(&minter, &owner, &1_000i128);
    token.tf_approve(&owner, &spender, &100i128);
    token.tf_transfer_from(&spender, &owner, &recipient, &500i128);
}

// ---------------------------------------------------------------------------
// CounterFacet
// ---------------------------------------------------------------------------

#[test]
fn test_counter_increment_and_get() {
    let (env, _admin, _diamond, _tid, _token, _cid, counter) = setup();
    let caller = Address::generate(&env);
    let name = symbol_short!("hits");

    counter.cf_increment(&caller, &name);
    counter.cf_increment(&caller, &name);
    counter.cf_increment(&caller, &name);

    assert_eq!(counter.cf_get_count(&name), 3);
}

#[test]
fn test_counter_reset_returns_to_zero() {
    let (env, _admin, _diamond, _tid, _token, _cid, counter) = setup();
    let caller = Address::generate(&env);
    let name = symbol_short!("clicks");

    counter.cf_increment(&caller, &name);
    counter.cf_increment(&caller, &name);
    assert_eq!(counter.cf_get_count(&name), 2);

    counter.cf_reset(&caller, &name);
    assert_eq!(counter.cf_get_count(&name), 0);
}

#[test]
fn test_counter_independent_names_do_not_interfere() {
    let (env, _admin, _diamond, _tid, _token, _cid, counter) = setup();
    let caller = Address::generate(&env);

    counter.cf_increment(&caller, &symbol_short!("pageA"));
    counter.cf_increment(&caller, &symbol_short!("pageA"));
    counter.cf_increment(&caller, &symbol_short!("pageB"));

    assert_eq!(counter.cf_get_count(&symbol_short!("pageA")), 2);
    assert_eq!(counter.cf_get_count(&symbol_short!("pageB")), 1);
}

// ---------------------------------------------------------------------------
// Diamond storage pattern — cross-facet namespace isolation
// ---------------------------------------------------------------------------

#[test]
fn test_storage_namespaces_do_not_collide() {
    // TfBalance(address) and CfCount(symbol) live in different DataKey variants;
    // writes to one can never bleed into the other.
    let (env, _admin, _diamond, _tid, token, _cid, counter) = setup();
    let minter = Address::generate(&env);
    let user = Address::generate(&env);
    let name = symbol_short!("views");

    token.tf_mint(&minter, &user, &5_000i128);
    counter.cf_increment(&user, &name);
    counter.cf_increment(&user, &name);

    assert_eq!(token.tf_balance_of(&user), 5_000);
    assert_eq!(counter.cf_get_count(&name), 2);

    // Resetting the counter must not affect the token balance.
    counter.cf_reset(&user, &name);
    assert_eq!(counter.cf_get_count(&name), 0);
    assert_eq!(token.tf_balance_of(&user), 5_000);

    // Transferring tokens must not affect the counter.
    let recipient = Address::generate(&env);
    token.tf_transfer(&user, &recipient, &1_000i128);
    assert_eq!(counter.cf_get_count(&name), 0);
    assert_eq!(token.tf_balance_of(&user), 4_000);
}
