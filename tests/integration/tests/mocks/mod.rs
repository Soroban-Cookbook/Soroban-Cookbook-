//! # Mock Contracts
//!
//! Lightweight mock contracts for testing cross-contract interactions without
//! depending on full contract implementations. These are useful for isolating
//! specific behaviors in integration tests.

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Mock Token Contract
// ---------------------------------------------------------------------------

/// Minimal mock token for testing token-dependent contracts.
/// Implements basic mint/transfer/balance without full SEP-41 compliance.
#[contracttype]
#[derive(Clone)]
pub enum MockTokenKey {
    Balance(Address),
    Supply,
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn mint(env: Env, to: Address, amount: i128) {
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&MockTokenKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&MockTokenKey::Balance(to), &(balance + amount));

        let supply: i128 = env
            .storage()
            .instance()
            .get(&MockTokenKey::Supply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&MockTokenKey::Supply, &(supply + amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&MockTokenKey::Balance(from.clone()))
            .unwrap_or(0);
        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&MockTokenKey::Balance(to.clone()))
            .unwrap_or(0);

        assert!(from_bal >= amount, "insufficient balance");

        env.storage()
            .persistent()
            .set(&MockTokenKey::Balance(from), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&MockTokenKey::Balance(to), &(to_bal + amount));
    }

    pub fn balance(env: Env, addr: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&MockTokenKey::Balance(addr))
            .unwrap_or(0)
    }

    pub fn supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&MockTokenKey::Supply)
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Mock Oracle Contract
// ---------------------------------------------------------------------------

/// Minimal mock oracle for testing price-feed consumers.
/// Admin sets a price; anyone can read it.
#[contracttype]
#[derive(Clone)]
pub enum OracleKey {
    Admin,
    Price(Symbol),
}

#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&OracleKey::Admin, &admin);
    }

    pub fn set_price(env: Env, admin: Address, asset: Symbol, price: i128) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&OracleKey::Admin).unwrap();
        assert!(admin == stored, "not admin");
        env.storage()
            .persistent()
            .set(&OracleKey::Price(asset.clone()), &price);
        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("price")),
            (asset, price),
        );
    }

    pub fn get_price(env: Env, asset: Symbol) -> Option<i128> {
        env.storage().persistent().get(&OracleKey::Price(asset))
    }
}

// ---------------------------------------------------------------------------
// Mock Timelock Contract
// ---------------------------------------------------------------------------

/// Minimal mock timelock for testing time-dependent interactions.
#[contracttype]
#[derive(Clone)]
pub enum TimelockKey {
    Locked(Symbol),
    UnlockAt(Symbol),
}

#[contract]
pub struct MockTimelock;

#[contractimpl]
impl MockTimelock {
    pub fn lock(env: Env, key: Symbol, unlock_at: u64) {
        env.storage()
            .persistent()
            .set(&TimelockKey::Locked(key.clone()), &true);
        env.storage()
            .persistent()
            .set(&TimelockKey::UnlockAt(key), &unlock_at);
    }

    pub fn is_locked(env: Env, key: Symbol) -> bool {
        let locked: bool = env
            .storage()
            .persistent()
            .get(&TimelockKey::Locked(key.clone()))
            .unwrap_or(false);
        if !locked {
            return false;
        }
        let unlock_at: u64 = env
            .storage()
            .persistent()
            .get(&TimelockKey::UnlockAt(key))
            .unwrap_or(0);
        env.ledger().timestamp() < unlock_at
    }

    pub fn unlock(env: Env, key: Symbol) -> bool {
        let unlock_at: u64 = env
            .storage()
            .persistent()
            .get(&TimelockKey::UnlockAt(key.clone()))
            .unwrap_or(0);
        if env.ledger().timestamp() >= unlock_at {
            env.storage()
                .persistent()
                .set(&TimelockKey::Locked(key), &false);
            true
        } else {
            false
        }
    }
}
