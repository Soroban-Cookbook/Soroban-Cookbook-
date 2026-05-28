#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contract]
pub struct RegistryContract;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,
    Whitelist(Address),
    Registered(Address),
    RemovalRequest(Address),
    WhitelistOnly,
    Fee,
}

#[contractimpl]
impl RegistryContract {
    // Initialize owner, optional whitelist-only flag and fee (in arbitrary units)
    pub fn init(env: Env, owner: Address, whitelist_only: bool, fee: i128) {
        // Only allow first-time init
        let already: Option<Address> = env.storage().instance().get(&DataKey::Owner);
        if already.is_some() {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&DataKey::Owner, &owner.clone());
        env.storage()
            .instance()
            .set(&DataKey::WhitelistOnly, &whitelist_only);
        env.storage().instance().set(&DataKey::Fee, &fee);
    }

    // Owner-only: add an address to whitelist
    pub fn add_whitelist(env: Env, addr: Address) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Whitelist(addr), &true);
    }

    // Owner-only: remove from whitelist
    pub fn remove_whitelist(env: Env, addr: Address) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();
        env.storage().instance().remove(&DataKey::Whitelist(addr));
    }

    // Owner-only: set whitelist-only mode
    pub fn set_whitelist_only(env: Env, whitelist_only: bool) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::WhitelistOnly, &whitelist_only);
    }

    // Owner-only: set fee
    pub fn set_fee(env: Env, fee: i128) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();
        env.storage().instance().set(&DataKey::Fee, &fee);
    }

    // Register an address. Caller must authorize themselves via `require_auth`.
    // `payment` is an abstract numeric amount the caller promises to have paid.
    pub fn register(env: Env, who: Address, payment: i128) {
        who.require_auth();
        // check whitelist-only
        let whitelist_only: bool = env
            .storage()
            .instance()
            .get(&DataKey::WhitelistOnly)
            .unwrap_or(false);

        if whitelist_only {
            let allowed: Option<bool> = env
                .storage()
                .instance()
                .get(&DataKey::Whitelist(who.clone()));
            if allowed != Some(true) {
                panic!("not whitelisted");
            }
        }

        let fee: i128 = env.storage().instance().get(&DataKey::Fee).unwrap_or(0i128);
        if fee > 0 && payment < fee {
            panic!("insufficient fee");
        }

        env.storage()
            .instance()
            .set(&DataKey::Registered(who), &true);
    }

    // Anyone can file a removal request (a dispute) against a registered address.
    pub fn request_removal(env: Env, reporter: Address, target: Address, _reason: Symbol) {
        reporter.require_auth();
        // only record a removal request if the target is currently registered
        let is_reg: Option<bool> = env
            .storage()
            .instance()
            .get(&DataKey::Registered(target.clone()));
        if is_reg == Some(true) {
            env.storage()
                .instance()
                .set(&DataKey::RemovalRequest(target), &reporter);
        }
    }

    // Owner resolves dispute: if `approve` true, the target is removed from registry
    pub fn resolve_removal(env: Env, target: Address, approve: bool) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("not initialized");
        owner.require_auth();

        // must have a pending removal request
        let pending: Option<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RemovalRequest(target.clone()));
        if pending.is_none() {
            panic!("no pending removal");
        }

        env.storage()
            .instance()
            .remove(&DataKey::RemovalRequest(target.clone()));
        if approve {
            env.storage()
                .instance()
                .remove(&DataKey::Registered(target.clone()));
            env.storage().instance().remove(&DataKey::Whitelist(target));
        }
    }

    // Query helper
    pub fn is_registered(env: Env, addr: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Registered(addr))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test;
