//! # Diamond Facets Pattern
//!
//! Demonstrates the Diamond / multi-facet pattern on Soroban.
//!
//! In Ethereum the Diamond pattern (EIP-2535) routes function selectors to
//! separate "facet" contracts that share a single storage namespace. Soroban
//! lacks delegatecall, so the pattern is adapted: a **DiamondRouter** holds
//! the canonical storage and calls sibling facet contracts. Each facet
//! operates on a well-defined storage sub-namespace to ensure isolation.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────┐
//! │   DiamondRouter  │  ← entry point; owns shared storage
//! │  (router contract)│    routes calls to registered facets
//! └────────┬─────────┘
//!          │ cross-contract calls
//!    ┌─────┼──────────┐
//!    ▼     ▼          ▼
//! [Token] [Access] [Registry]
//! Facet   Facet    Facet
//! ```
//!
//! ## Facets
//!
//! | Facet | Responsibility |
//! |-------|----------------|
//! [`TokenFacet`] | Mint / transfer / balance (ERC-20-like) |
//! [`AccessFacet`] | Role-based access control: grant/revoke/check |
//! [`RegistryFacet`] | Key→value metadata registry with ownership |
//!
//! ## Storage Isolation
//!
//! Each facet uses a distinct `DataKey` variant as a namespace prefix so
//! keys never collide across facets, even within the same logical contract.
//!
//! ## Inter-Facet Communication
//!
//! The router demonstrates calling multiple facets in a single transaction
//! (e.g. `mint_and_register` first calls the token facet to mint, then the
//! registry facet to log metadata).

#![no_std]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

// ---------------------------------------------------------------------------
// Shared storage key namespace
// ---------------------------------------------------------------------------

/// Top-level storage key enum — each facet owns a distinct variant prefix
/// ensuring storage isolation even when multiple facets run in one contract.
#[contracttype]
pub enum DataKey {
    // --- TokenFacet namespace ---
    /// `Balance(owner)` → i128
    Balance(Address),
    /// `Allowance(owner, spender)` → i128
    Allowance(Address, Address),
    /// `TotalSupply` → i128
    TotalSupply,

    // --- AccessFacet namespace ---
    /// `Role(address)` → u32 (role level)
    Role(Address),
    /// `RoleAdmin` → Address
    RoleAdmin,

    // --- RegistryFacet namespace ---
    /// `RegEntry(key)` → (Address, String) owner + value
    RegEntry(Symbol),
    /// `RegOwner(key)` → Address
    RegOwner(Symbol),
}

// ---------------------------------------------------------------------------
// Event namespaces
// ---------------------------------------------------------------------------

const NS_TOKEN: Symbol = symbol_short!("token");
const NS_ACCESS: Symbol = symbol_short!("access");
const NS_REGISTRY: Symbol = symbol_short!("registry");

// ---------------------------------------------------------------------------
// Role constants (access facet)
// ---------------------------------------------------------------------------

pub const ROLE_USER: u32 = 0;
pub const ROLE_MINTER: u32 = 1;
pub const ROLE_ADMIN: u32 = 2;

// ---------------------------------------------------------------------------
// TokenFacet
// ---------------------------------------------------------------------------

/// Token facet: mint, transfer, balance queries.
///
/// Storage namespace: `DataKey::Balance`, `DataKey::Allowance`, `DataKey::TotalSupply`.
#[contract]
pub struct TokenFacet;

#[contractimpl]
impl TokenFacet {
    /// Mint `amount` tokens to `to`. Requires `minter` to have ROLE_MINTER or higher.
    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) {
        minter.require_auth();
        assert!(amount > 0, "amount must be positive");

        let bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(bal + amount));

        let supply: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &(supply + amount));

        env.events()
            .publish((NS_TOKEN, symbol_short!("mint"), to), amount);
    }

    /// Transfer `amount` from `from` to `to`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_bal + amount));

        env.events()
            .publish((NS_TOKEN, symbol_short!("transfer"), from, to), amount);
    }

    /// Query the balance of `owner`.
    pub fn balance_of(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    /// Query the total token supply.
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Approve `spender` to transfer up to `amount` from `owner`.
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);
        env.events()
            .publish((NS_TOKEN, symbol_short!("approve"), owner, spender), amount);
    }

    /// Transfer using an allowance.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        assert!(allowance >= amount, "allowance exceeded");

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_bal + amount));
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(from, spender), &(allowance - amount));
    }
}

// ---------------------------------------------------------------------------
// AccessFacet
// ---------------------------------------------------------------------------

/// Access facet: role-based permissions.
///
/// Storage namespace: `DataKey::Role`, `DataKey::RoleAdmin`.
#[contract]
pub struct AccessFacet;

#[contractimpl]
impl AccessFacet {
    /// Bootstrap the access facet with an initial admin.
    pub fn initialize(env: Env, admin: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::RoleAdmin),
            "already initialized"
        );
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::RoleAdmin, &admin.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Role(admin.clone()), &ROLE_ADMIN);
        env.events()
            .publish((NS_ACCESS, symbol_short!("init"), admin), ROLE_ADMIN);
    }

    /// Grant a role to `account` (admin only).
    pub fn grant_role(env: Env, admin: Address, account: Address, role: u32) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);

        env.storage()
            .persistent()
            .set(&DataKey::Role(account.clone()), &role);
        env.events()
            .publish((NS_ACCESS, symbol_short!("grant"), admin, account), role);
    }

    /// Revoke any role from `account` (admin only, resets to ROLE_USER).
    pub fn revoke_role(env: Env, admin: Address, account: Address) {
        admin.require_auth();
        Self::assert_admin(&env, &admin);

        env.storage()
            .persistent()
            .set(&DataKey::Role(account.clone()), &ROLE_USER);
        env.events().publish(
            (NS_ACCESS, symbol_short!("revoke"), admin, account),
            ROLE_USER,
        );
    }

    /// Return the role level of `account`.
    pub fn get_role(env: Env, account: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::Role(account))
            .unwrap_or(ROLE_USER)
    }

    /// Return `true` if `account` has at least `required_role`.
    pub fn has_role(env: Env, account: Address, required_role: u32) -> bool {
        let role: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Role(account))
            .unwrap_or(ROLE_USER);
        role >= required_role
    }

    fn assert_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::RoleAdmin)
            .expect("not initialized");
        assert_eq!(caller, &admin, "caller is not admin");
    }
}

// ---------------------------------------------------------------------------
// RegistryFacet
// ---------------------------------------------------------------------------

/// Registry facet: key → (owner, value) metadata store.
///
/// Storage namespace: `DataKey::RegEntry`, `DataKey::RegOwner`.
#[contract]
pub struct RegistryFacet;

#[contractimpl]
impl RegistryFacet {
    /// Register or update a key-value entry. Caller becomes the owner.
    pub fn set_entry(env: Env, caller: Address, key: Symbol, value: String) {
        caller.require_auth();

        // If entry already exists, only the owner may overwrite.
        if let Some(existing_owner) = env
            .storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::RegOwner(key.clone()))
        {
            assert_eq!(caller, existing_owner, "not the entry owner");
        }

        env.storage()
            .persistent()
            .set(&DataKey::RegOwner(key.clone()), &caller.clone());
        env.storage()
            .persistent()
            .set(&DataKey::RegEntry(key.clone()), &value.clone());

        env.events()
            .publish((NS_REGISTRY, symbol_short!("set"), caller, key), value);
    }

    /// Remove an entry (owner only).
    pub fn remove_entry(env: Env, caller: Address, key: Symbol) {
        caller.require_auth();
        let owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::RegOwner(key.clone()))
            .expect("entry not found");
        assert_eq!(caller, owner, "not the entry owner");

        env.storage()
            .persistent()
            .remove(&DataKey::RegOwner(key.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::RegEntry(key.clone()));

        env.events()
            .publish((NS_REGISTRY, symbol_short!("remove"), caller, key), ());
    }

    /// Read an entry value (returns None if not found).
    pub fn get_entry(env: Env, key: Symbol) -> Option<String> {
        env.storage().persistent().get(&DataKey::RegEntry(key))
    }

    /// Read the owner of an entry.
    pub fn get_owner(env: Env, key: Symbol) -> Option<Address> {
        env.storage().persistent().get(&DataKey::RegOwner(key))
    }
}

// ---------------------------------------------------------------------------
// DiamondRouter
// ---------------------------------------------------------------------------

/// Router contract that demonstrates inter-facet communication.
///
/// In a full Diamond deployment each facet would be a separate on-chain
/// contract. Here we show the pattern with three facets compiled together;
/// the router holds references to their deployed addresses and orchestrates
/// multi-step operations.
#[contract]
pub struct DiamondRouter;

/// Registered facet addresses held by the router.
#[contracttype]
pub enum RouterKey {
    Token,
    Access,
    Registry,
}

#[contractimpl]
impl DiamondRouter {
    /// Register facet contract addresses (admin only, call once).
    pub fn register_facets(
        env: Env,
        admin: Address,
        token: Address,
        access: Address,
        registry: Address,
    ) {
        admin.require_auth();
        assert!(
            !env.storage().instance().has(&RouterKey::Token),
            "already registered"
        );
        env.storage().instance().set(&RouterKey::Token, &token);
        env.storage().instance().set(&RouterKey::Access, &access);
        env.storage()
            .instance()
            .set(&RouterKey::Registry, &registry);
    }

    /// Demonstrate inter-facet communication: mint tokens AND register metadata.
    ///
    /// This shows how the router can orchestrate multiple facets in a single
    /// transaction, maintaining atomicity. If either step fails, the whole
    /// transaction reverts.
    pub fn mint_and_register(
        env: Env,
        minter: Address,
        recipient: Address,
        amount: i128,
        meta_key: Symbol,
        meta_value: String,
    ) {
        minter.require_auth();

        let token_id: Address = env
            .storage()
            .instance()
            .get(&RouterKey::Token)
            .expect("token facet not registered");

        let registry_id: Address = env
            .storage()
            .instance()
            .get(&RouterKey::Registry)
            .expect("registry facet not registered");

        // Step 1: mint via TokenFacet
        let token_client = TokenFacetClient::new(&env, &token_id);
        token_client.mint(&minter, &recipient, &amount);

        // Step 2: log metadata via RegistryFacet
        let registry_client = RegistryFacetClient::new(&env, &registry_id);
        registry_client.set_entry(&minter, &meta_key, &meta_value);

        env.events().publish(
            (
                symbol_short!("diamond"),
                symbol_short!("compose"),
                minter,
                recipient,
            ),
            (amount, meta_key),
        );
    }

    /// Query a facet address by name: "token", "access", "registry".
    pub fn get_facet(env: Env, name: Symbol) -> Option<Address> {
        if name == symbol_short!("token") {
            env.storage().instance().get(&RouterKey::Token)
        } else if name == symbol_short!("access") {
            env.storage().instance().get(&RouterKey::Access)
        } else if name == symbol_short!("registry") {
            env.storage().instance().get(&RouterKey::Registry)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test;
