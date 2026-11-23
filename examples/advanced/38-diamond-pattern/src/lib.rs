//! # Diamond Pattern Base (EIP-2535 Soroban Adaptation)
//!
//! Soroban's adaptation of the EIP-2535 Diamond Standard. Because Soroban
//! lacks `delegatecall`, the pattern is realised as:
//!
//! 1. **Diamond storage** — a single `DataKey` enum whose variant tags act as
//!    per-facet namespace prefixes, guaranteeing storage isolation across all
//!    facets without any key collisions.
//!
//! 2. **Facet registry** — the [`DiamondBase`] contract maintains a dynamic
//!    `selector → facet-address` mapping that can be updated at runtime via
//!    diamond-cut operations.
//!
//! 3. **Function selector mapping** — each logical function is identified by a
//!    [`Symbol`] selector stored in the registry.
//!
//! 4. **Fallback mechanism** — callers resolve the correct facet address via
//!    [`DiamondBase::facet_address`] and dispatch to it directly, mirroring
//!    how EIP-2535 fallback routing works at the router level.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                       DiamondBase                        │
//! │  diamond_cut  │  facets  │  facet_address (fallback)     │
//! └───────────────────────────────┬──────────────────────────┘
//!                                 │ selector → facet lookup
//!               ┌─────────────────┼─────────────────────┐
//!               ▼                 ▼                       ▼
//!         TokenFacet          CounterFacet           (any facet)
//!    TfBalance / TfSupply   CfCount(name)         isolated namespace
//! ```

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Val,
    Vec, xdr::ScValType,
};

// ---------------------------------------------------------------------------
// Diamond cut types
// ---------------------------------------------------------------------------

/// The operation to apply to a set of function selectors during a diamond cut.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FacetCutAction {
    /// Register new selectors mapped to a new facet contract.
    Add,
    /// Re-map existing selectors to a different facet contract.
    Replace,
    /// Remove selectors, erasing their facet mapping entirely.
    Remove,
}

/// Describes one facet change within a [`DiamondBase::diamond_cut`] batch.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FacetCut {
    /// Target facet contract address. Unused (but must be present) for `Remove`.
    pub facet_address: Address,
    /// The operation: Add, Replace, or Remove.
    pub action: FacetCutAction,
    /// The [`Symbol`] function selectors to add, replace, or remove.
    pub selectors: Vec<Symbol>,
}

/// Facet information returned by the diamond-loupe introspection functions.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FacetInfo {
    /// On-chain contract address of this facet.
    pub facet_address: Address,
    /// All function selectors currently routed to this facet.
    pub selectors: Vec<Symbol>,
}

/// The expected Soroban value type of a single function argument.
///
/// Used by the dispatcher's ABI validation layer. Each variant maps to one
/// `ScVal` discriminant, so validation is exact: an `I128` argument will not
/// silently satisfy a `U64` parameter.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgType {
    /// Sentinel used in error reporting when no type is expected.
    None,
    Bool,
    U32,
    I32,
    U64,
    I64,
    Timepoint,
    Duration,
    U128,
    I128,
    U256,
    I256,
    Address,
    Symbol,
    String,
    Bytes,
    Vec,
    Map,
}

impl ArgType {
    /// Returns `true` when `v` carries exactly this argument type.
    fn matches(&self, v: &Val) -> bool {
        let Some(scval_ty) = v.get_tag().get_scval_type() else {
            return false;
        };
        matches!(
            (self, scval_ty),
            (ArgType::Bool, ScValType::Bool)
                | (ArgType::U32, ScValType::U32)
                | (ArgType::I32, ScValType::I32)
                | (ArgType::U64, ScValType::U64)
                | (ArgType::I64, ScValType::I64)
                | (ArgType::Timepoint, ScValType::Timepoint)
                | (ArgType::Duration, ScValType::Duration)
                | (ArgType::U128, ScValType::U128)
                | (ArgType::I128, ScValType::I128)
                | (ArgType::U256, ScValType::U256)
                | (ArgType::I256, ScValType::I256)
                | (ArgType::Address, ScValType::Address)
                | (ArgType::Symbol, ScValType::Symbol)
                | (ArgType::String, ScValType::String)
                | (ArgType::Bytes, ScValType::Bytes)
                | (ArgType::Vec, ScValType::Vec)
                | (ArgType::Map, ScValType::Map)
        )
    }
}

/// Typed errors returned by [`DiamondBase::dispatch`].
///
/// Replaces the host trap that a raw `invoke_contract` produces when a caller
/// supplies malformed arguments, making client-side bugs diagnosable on-chain.
/// Detailed context (argument index, expected type, counts) is published as a
/// `diamond/dsp_err` event alongside the returned variant.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DispatchError {
    /// No facet is registered for this selector.
    UnknownSelector = 1,
    /// No argument signature is known for this selector.
    UnknownSignature = 2,
    /// Wrong number of arguments supplied.
    ArgCountMismatch = 3,
    /// An argument's type does not match the expected signature.
    InvalidArguments = 4,
    /// The facet call itself failed after validation passed.
    FacetInvocationFailed = 5,
}

/// Detail payload published with the `diamond/dsp_err` event when dispatch
/// validation fails, so callers can pinpoint the offending argument.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchErrorInfo {
    /// The selector that failed validation.
    pub selector: Symbol,
    /// Index of the offending argument, or `u32::MAX` for count mismatches.
    pub index: u32,
    /// Expected type at `index`, or `ArgType::None` for count mismatches.
    pub expected: ArgType,
    /// Expected argument count.
    pub expected_count: u32,
    /// Actual argument count supplied.
    pub got_count: u32,
}

// ---------------------------------------------------------------------------
// Diamond storage pattern — namespaced DataKey enum
// ---------------------------------------------------------------------------

/// Unified storage key enum for the diamond and every registered facet.
///
/// The variant tag acts as an implicit namespace: `TfBalance(owner)` can
/// never collide with `CfCount(name)` even though both hold per-address or
/// per-name values. This is the Soroban realisation of the EIP-2535 diamond
/// storage pattern — isolation without hash slots.
#[contracttype]
pub enum DataKey {
    // ── Diamond core storage ──────────────────────────────────────────────
    /// The admin address authorised to invoke `diamond_cut`.
    Admin,
    /// Ordered list of every registered function selector.
    SelectorList,
    /// Ordered list of every unique registered facet address.
    FacetList,
    /// `SelectorFacet(selector)` → Address: which facet handles this selector.
    SelectorFacet(Symbol),
    /// `FacetSelectorList(facet)` → Vec<Symbol>: selectors served by a facet.
    FacetSelectorList(Address),
    /// `SelectorSignature(selector)` → Vec<ArgType>: expected argument types.
    SelectorSignature(Symbol),

    // ── TokenFacet storage namespace (tf_ prefix) ─────────────────────────
    /// ERC-20-style balance per owner.
    TfBalance(Address),
    /// ERC-20-style spend allowance per (owner, spender) pair.
    TfAllowance(Address, Address),
    /// Aggregate token supply.
    TfTotalSupply,

    // ── CounterFacet storage namespace (cf_ prefix) ───────────────────────
    /// Named counter value.
    CfCount(Symbol),
}

// ---------------------------------------------------------------------------
// Event namespaces
// ---------------------------------------------------------------------------

const NS_DIAMOND: Symbol = symbol_short!("diamond");
const NS_TOKEN: Symbol = symbol_short!("token");
const NS_COUNTER: Symbol = symbol_short!("counter");

// ---------------------------------------------------------------------------
// DiamondBase — registry, loupe, and cut
// ---------------------------------------------------------------------------

/// Core diamond contract that owns the selector → facet routing table.
///
/// Supports dynamic reconfiguration (diamond cut) and full introspection
/// (diamond loupe). Concrete facet contracts demonstrate the diamond storage
/// pattern by using isolated `DataKey` namespace prefixes.
#[contract]
pub struct DiamondBase;

#[contractimpl]
impl DiamondBase {
    // ── Initialisation ────────────────────────────────────────────────────

    /// Bootstrap the diamond. Must be called exactly once.
    ///
    /// Sets the admin address that is permitted to invoke [`Self::diamond_cut`].
    pub fn initialize(env: Env, admin: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Admin),
            "already initialized"
        );
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::SelectorList, &Vec::<Symbol>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::FacetList, &Vec::<Address>::new(&env));

        env.events()
            .publish((NS_DIAMOND, symbol_short!("init"), admin), ());
    }

    // ── Diamond cut ───────────────────────────────────────────────────────

    /// Apply a batch of facet cuts to reconfigure the diamond routing table.
    ///
    /// Each [`FacetCut`] entry may add new selectors, replace existing ones,
    /// or remove selectors entirely. Only the diamond admin may call this.
    pub fn diamond_cut(env: Env, admin: Address, cuts: Vec<FacetCut>) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);

        for cut in cuts.iter() {
            match cut.action {
                FacetCutAction::Add => {
                    Self::apply_add(&env, &cut.facet_address, &cut.selectors);
                }
                FacetCutAction::Replace => {
                    Self::apply_replace(&env, &cut.facet_address, &cut.selectors);
                }
                FacetCutAction::Remove => {
                    Self::apply_remove(&env, &cut.selectors);
                }
            }
        }

        env.events()
            .publish((NS_DIAMOND, symbol_short!("cut"), admin), cuts.len());
    }

    // ── Diamond loupe (introspection) ─────────────────────────────────────

    /// Return all registered facets together with their function selectors.
    pub fn facets(env: Env) -> Vec<FacetInfo> {
        let facet_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FacetList)
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for addr in facet_list.iter() {
            let selectors: Vec<Symbol> = env
                .storage()
                .instance()
                .get(&DataKey::FacetSelectorList(addr.clone()))
                .unwrap_or_else(|| Vec::new(&env));
            result.push_back(FacetInfo {
                facet_address: addr.clone(),
                selectors,
            });
        }
        result
    }

    /// Return all unique facet contract addresses registered with the diamond.
    pub fn facet_addresses(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::FacetList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return the function selectors served by `facet`.
    pub fn facet_function_selectors(env: Env, facet: Address) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&DataKey::FacetSelectorList(facet))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return the facet address that handles `selector`, or `None` if unregistered.
    ///
    /// This is the **fallback dispatch entry point**: callers resolve the
    /// correct facet contract address here, then invoke it directly — the
    /// Soroban equivalent of EIP-2535 fallback routing.
    pub fn facet_address(env: Env, selector: Symbol) -> Option<Address> {
        env.storage()
            .instance()
            .get(&DataKey::SelectorFacet(selector))
    }

    /// Return the total count of registered function selectors.
    pub fn selector_count(env: Env) -> u32 {
        let list: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::SelectorList)
            .unwrap_or_else(|| Vec::new(&env));
        list.len()
    }

    // ── Validated dispatch ────────────────────────────────────────────────

    /// Register the expected argument types for `selector`.
    ///
    /// Admin-only. Signatures registered here take precedence over the
    /// built-in signatures of the shipped facets, allowing custom facets to
    /// participate in validated dispatch.
    pub fn register_signature(env: Env, admin: Address, selector: Symbol, arg_types: Vec<ArgType>) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::SelectorSignature(selector), &arg_types);
        env.events()
            .publish((NS_DIAMOND, symbol_short!("sig"), admin), arg_types.len());
    }

    /// Return the registered or built-in argument signature for `selector`.
    pub fn function_signature(env: Env, selector: Symbol) -> Option<Vec<ArgType>> {
        Self::signature_for(&env, &selector)
    }

    /// Resolve `selector` to its facet and invoke it with ABI validation.
    ///
    /// Unlike a raw `env.invoke_contract`, this validates the argument count
    /// and each argument's [`ScValType`] against the function's registered
    /// signature *before* forwarding, returning a typed [`DispatchError`]
    /// instead of letting the facet trap on malformed input.
    pub fn dispatch(env: Env, selector: Symbol, args: Vec<Val>) -> Result<Val, DispatchError> {
        let facet: Address = env
            .storage()
            .instance()
            .get(&DataKey::SelectorFacet(selector.clone()))
            .ok_or(DispatchError::UnknownSelector)?;

        let Some(signature) = Self::signature_for(&env, &selector) else {
            return Err(DispatchError::UnknownSignature);
        };

        if args.len() != signature.len() {
            Self::publish_dispatch_error(
                &env,
                &selector,
                DispatchErrorInfo {
                    selector: selector.clone(),
                    index: u32::MAX,
                    expected: ArgType::None,
                    expected_count: signature.len(),
                    got_count: args.len(),
                },
            );
            return Err(DispatchError::ArgCountMismatch);
        }

        for (i, (arg, expected)) in args.iter().zip(signature.iter()).enumerate() {
            if !expected.matches(&arg) {
                Self::publish_dispatch_error(
                    &env,
                    &selector,
                    DispatchErrorInfo {
                        selector: selector.clone(),
                        index: i as u32,
                        expected,
                        expected_count: signature.len(),
                        got_count: args.len(),
                    },
                );
                return Err(DispatchError::InvalidArguments);
            }
        }

        match env.try_invoke_contract::<Val, DispatchError>(&facet, &selector, args) {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(_)) | Err(Err(_)) => Err(DispatchError::FacetInvocationFailed),
            Err(Ok(err)) => Err(err),
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────

    /// Publish a `diamond/dsp_err` event carrying validation failure details.
    fn publish_dispatch_error(env: &Env, selector: &Symbol, info: DispatchErrorInfo) {
        env.events()
            .publish((NS_DIAMOND, symbol_short!("dsp_err"), selector.clone()), info);
    }

    /// Look up the argument signature for `selector`: on-chain registration
    /// first, then the built-in signatures of the shipped facets.
    fn signature_for(env: &Env, selector: &Symbol) -> Option<Vec<ArgType>> {
        if let Some(sig) = env
            .storage()
            .instance()
            .get(&DataKey::SelectorSignature(selector.clone()))
        {
            return Some(sig);
        }
        builtin_signature(env, selector)
    }

    fn assert_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        assert_eq!(caller, &admin, "caller is not admin");
    }

    /// Register new selectors pointing to `facet`.
    ///
    /// Panics if any selector is already registered — use `Replace` for that.
    fn apply_add(env: &Env, facet: &Address, selectors: &Vec<Symbol>) {
        let mut sel_list: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::SelectorList)
            .unwrap_or_else(|| Vec::new(env));

        let mut facet_sels: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::FacetSelectorList(facet.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut facet_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FacetList)
            .unwrap_or_else(|| Vec::new(env));

        if !addr_in_list(&facet_list, facet) {
            facet_list.push_back(facet.clone());
            env.storage()
                .instance()
                .set(&DataKey::FacetList, &facet_list);
        }

        for sel in selectors.iter() {
            assert!(
                !env.storage()
                    .instance()
                    .has(&DataKey::SelectorFacet(sel.clone())),
                "selector already registered; use Replace"
            );
            env.storage()
                .instance()
                .set(&DataKey::SelectorFacet(sel.clone()), facet);
            sel_list.push_back(sel.clone());
            facet_sels.push_back(sel.clone());
        }

        env.storage()
            .instance()
            .set(&DataKey::SelectorList, &sel_list);
        env.storage()
            .instance()
            .set(&DataKey::FacetSelectorList(facet.clone()), &facet_sels);
    }

    /// Re-map existing selectors to `new_facet`.
    ///
    /// Panics if any selector is not already registered — use `Add` for that.
    fn apply_replace(env: &Env, new_facet: &Address, selectors: &Vec<Symbol>) {
        let mut facet_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FacetList)
            .unwrap_or_else(|| Vec::new(env));

        let mut new_facet_sels: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::FacetSelectorList(new_facet.clone()))
            .unwrap_or_else(|| Vec::new(env));

        for sel in selectors.iter() {
            let old_facet: Address = env
                .storage()
                .instance()
                .get(&DataKey::SelectorFacet(sel.clone()))
                .expect("selector not registered; use Add");

            // Remove selector from old facet's list.
            let old_sels: Vec<Symbol> = env
                .storage()
                .instance()
                .get(&DataKey::FacetSelectorList(old_facet.clone()))
                .unwrap_or_else(|| Vec::new(env));

            let mut filtered_old = Vec::new(env);
            for s in old_sels.iter() {
                if s != sel {
                    filtered_old.push_back(s.clone());
                }
            }
            env.storage()
                .instance()
                .set(&DataKey::FacetSelectorList(old_facet), &filtered_old);

            // Point the selector at the new facet.
            env.storage()
                .instance()
                .set(&DataKey::SelectorFacet(sel.clone()), new_facet);

            if !sym_in_list(&new_facet_sels, &sel) {
                new_facet_sels.push_back(sel.clone());
            }
        }

        if !addr_in_list(&facet_list, new_facet) {
            facet_list.push_back(new_facet.clone());
            env.storage()
                .instance()
                .set(&DataKey::FacetList, &facet_list);
        }

        env.storage().instance().set(
            &DataKey::FacetSelectorList(new_facet.clone()),
            &new_facet_sels,
        );
    }

    /// Remove selectors from the routing table.
    ///
    /// Panics if any selector is not currently registered. Facets with no
    /// remaining selectors are pruned from the global facet list.
    fn apply_remove(env: &Env, selectors: &Vec<Symbol>) {
        let mut sel_list: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::SelectorList)
            .unwrap_or_else(|| Vec::new(env));

        let mut facet_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FacetList)
            .unwrap_or_else(|| Vec::new(env));

        for sel in selectors.iter() {
            let old_facet: Address = env
                .storage()
                .instance()
                .get(&DataKey::SelectorFacet(sel.clone()))
                .expect("selector not registered");

            // Remove from this facet's per-facet selector list.
            let facet_sels: Vec<Symbol> = env
                .storage()
                .instance()
                .get(&DataKey::FacetSelectorList(old_facet.clone()))
                .unwrap_or_else(|| Vec::new(env));

            let mut updated_facet_sels = Vec::new(env);
            for s in facet_sels.iter() {
                if s != sel {
                    updated_facet_sels.push_back(s.clone());
                }
            }
            env.storage().instance().set(
                &DataKey::FacetSelectorList(old_facet.clone()),
                &updated_facet_sels,
            );

            // Prune the facet from the global list when it has no selectors left.
            if updated_facet_sels.is_empty() {
                let mut pruned = Vec::new(env);
                for a in facet_list.iter() {
                    if a != old_facet {
                        pruned.push_back(a.clone());
                    }
                }
                facet_list = pruned;
            }

            // Remove from the global selector list.
            let mut updated_global = Vec::new(env);
            for s in sel_list.iter() {
                if s != sel {
                    updated_global.push_back(s.clone());
                }
            }
            sel_list = updated_global;

            env.storage()
                .instance()
                .remove(&DataKey::SelectorFacet(sel.clone()));
        }

        env.storage()
            .instance()
            .set(&DataKey::SelectorList, &sel_list);
        env.storage()
            .instance()
            .set(&DataKey::FacetList, &facet_list);
    }
}

// ---------------------------------------------------------------------------
// Module-level helpers (pure Rust; no host calls)
// ---------------------------------------------------------------------------

/// Built-in argument signatures for the shipped facet functions.
///
/// On-chain signatures registered via [`DiamondBase::register_signature`]
/// take precedence over these defaults.
fn builtin_signature(env: &Env, selector: &Symbol) -> Option<Vec<ArgType>> {
    use ArgType as A;

    let types: &[ArgType] = if *selector == Symbol::new(env, "tf_mint")
        || *selector == Symbol::new(env, "tf_transfer")
        || *selector == Symbol::new(env, "tf_approve")
    {
        &[A::Address, A::Address, A::I128]
    } else if *selector == Symbol::new(env, "tf_transfer_from") {
        &[A::Address, A::Address, A::Address, A::I128]
    } else if *selector == Symbol::new(env, "tf_balance_of") {
        &[A::Address]
    } else if *selector == Symbol::new(env, "tf_total_supply") {
        &[]
    } else if *selector == Symbol::new(env, "cf_increment")
        || *selector == Symbol::new(env, "cf_reset")
    {
        &[A::Address, A::Symbol]
    } else if *selector == Symbol::new(env, "cf_get_count") {
        &[A::Symbol]
    } else {
        return None;
    };

    let mut sig = Vec::new(env);
    for t in types.iter() {
        sig.push_back(*t);
    }
    Some(sig)
}

fn addr_in_list(list: &Vec<Address>, addr: &Address) -> bool {
    for a in list.iter() {
        if &a == addr {
            return true;
        }
    }
    false
}

fn sym_in_list(list: &Vec<Symbol>, sym: &Symbol) -> bool {
    for s in list.iter() {
        if &s == sym {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// TokenFacet — diamond storage namespace: Tf*
// ---------------------------------------------------------------------------

/// ERC-20-like token facet.
///
/// All state lives exclusively under `TfBalance`, `TfAllowance`, and
/// `TfTotalSupply` DataKey variants. No other facet's keys can ever occupy
/// those variants — this is the diamond storage isolation guarantee.
#[contract]
pub struct TokenFacet;

#[contractimpl]
impl TokenFacet {
    /// Mint `amount` tokens to `to`, authorised by `minter`.
    pub fn tf_mint(env: Env, minter: Address, to: Address, amount: i128) {
        minter.require_auth();
        assert!(amount > 0, "amount must be positive");

        let bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfBalance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TfBalance(to.clone()), &(bal + amount));

        let supply: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfTotalSupply)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TfTotalSupply, &(supply + amount));

        env.events()
            .publish((NS_TOKEN, symbol_short!("mint"), to), amount);
    }

    /// Transfer `amount` tokens from `from` to `to`.
    pub fn tf_transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfBalance(from.clone()))
            .unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfBalance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::TfBalance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::TfBalance(to.clone()), &(to_bal + amount));

        env.events()
            .publish((NS_TOKEN, symbol_short!("transfer"), from, to), amount);
    }

    /// Approve `spender` to spend up to `amount` of `owner`'s tokens.
    pub fn tf_approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        env.storage().persistent().set(
            &DataKey::TfAllowance(owner.clone(), spender.clone()),
            &amount,
        );
        env.events()
            .publish((NS_TOKEN, symbol_short!("approve"), owner, spender), amount);
    }

    /// Transfer `amount` from `from` to `to` using a pre-approved allowance.
    pub fn tf_transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfAllowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        assert!(allowance >= amount, "allowance exceeded");

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfBalance(from.clone()))
            .unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TfBalance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::TfBalance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::TfBalance(to.clone()), &(to_bal + amount));
        env.storage()
            .persistent()
            .set(&DataKey::TfAllowance(from, spender), &(allowance - amount));
    }

    /// Return the token balance of `owner`.
    pub fn tf_balance_of(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TfBalance(owner))
            .unwrap_or(0)
    }

    /// Return the total token supply.
    pub fn tf_total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TfTotalSupply)
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// CounterFacet — diamond storage namespace: Cf*
// ---------------------------------------------------------------------------

/// Named counter facet.
///
/// Demonstrates a second, fully independent storage namespace (`CfCount`)
/// that coexists with `TokenFacet`'s namespace (`Tf*`) in the same
/// `DataKey` enum. Neither facet can accidentally read or overwrite the
/// other's keys — the diamond storage pattern's isolation guarantee in action.
#[contract]
pub struct CounterFacet;

#[contractimpl]
impl CounterFacet {
    /// Increment the named counter by 1.
    pub fn cf_increment(env: Env, caller: Address, name: Symbol) {
        caller.require_auth();
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CfCount(name.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::CfCount(name.clone()), &(count + 1));
        env.events()
            .publish((NS_COUNTER, symbol_short!("inc"), caller, name), count + 1);
    }

    /// Reset the named counter back to zero.
    pub fn cf_reset(env: Env, caller: Address, name: Symbol) {
        caller.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::CfCount(name.clone()), &0u32);
        env.events()
            .publish((NS_COUNTER, symbol_short!("reset"), caller, name), 0u32);
    }

    /// Return the current value of the named counter.
    pub fn cf_get_count(env: Env, name: Symbol) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::CfCount(name))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
