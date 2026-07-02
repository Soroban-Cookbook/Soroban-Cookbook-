#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol, Vec,
};

const ADMIN_KEY: Symbol = symbol_short!("admin");
const BEACONS_KEY: Symbol = symbol_short!("beacons");
const BEACON_NAMES_KEY: Symbol = symbol_short!("names");

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BeaconVersion {
    pub version: u32,
    pub implementation: Address,
    pub activated_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BeaconState {
    pub current_version: u32,
    pub current_implementation: Address,
    pub history: Vec<BeaconVersion>,
}

#[contract]
pub struct BeaconManagementContract;

#[contractimpl]
impl BeaconManagementContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialized");
        }

        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage()
            .instance()
            .set(&BEACONS_KEY, &Map::<Symbol, BeaconState>::new(&env));
        env.storage()
            .instance()
            .set(&BEACON_NAMES_KEY, &Vec::<Symbol>::new(&env));
    }

    pub fn register_beacon(env: Env, admin: Address, name: Symbol, implementation: Address) -> u32 {
        Self::require_admin(&env, &admin);

        let mut beacons: Map<Symbol, BeaconState> = env
            .storage()
            .instance()
            .get(&BEACONS_KEY)
            .unwrap_or_else(|| Map::new(&env));
        if beacons.contains_key(name.clone()) {
            panic!("beacon already exists");
        }

        let mut history = Vec::<BeaconVersion>::new(&env);
        history.push_back(BeaconVersion {
            version: 1,
            implementation: implementation.clone(),
            activated_at: env.ledger().timestamp(),
        });

        let state = BeaconState {
            current_version: 1,
            current_implementation: implementation.clone(),
            history,
        };
        beacons.set(name.clone(), state);
        env.storage().instance().set(&BEACONS_KEY, &beacons);

        let mut names: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&BEACON_NAMES_KEY)
            .unwrap_or_else(|| Vec::<Symbol>::new(&env));
        names.push_back(name.clone());
        env.storage().instance().set(&BEACON_NAMES_KEY, &names);

        1
    }

    pub fn upgrade_beacon(env: Env, admin: Address, name: Symbol, implementation: Address) -> u32 {
        Self::require_admin(&env, &admin);

        let mut beacons: Map<Symbol, BeaconState> = env
            .storage()
            .instance()
            .get(&BEACONS_KEY)
            .unwrap_or_else(|| Map::new(&env));
        let mut state = beacons
            .get(name.clone())
            .unwrap_or_else(|| panic!("beacon not found"));

        let new_version = state.current_version + 1;
        let mut history = state.history.clone();
        history.push_back(BeaconVersion {
            version: new_version,
            implementation: implementation.clone(),
            activated_at: env.ledger().timestamp(),
        });

        state.current_version = new_version;
        state.current_implementation = implementation.clone();
        state.history = history;
        beacons.set(name.clone(), state);
        env.storage().instance().set(&BEACONS_KEY, &beacons);

        new_version
    }

    pub fn rollback_beacon(env: Env, admin: Address, name: Symbol) -> u32 {
        Self::require_admin(&env, &admin);

        let mut beacons: Map<Symbol, BeaconState> = env
            .storage()
            .instance()
            .get(&BEACONS_KEY)
            .unwrap_or_else(|| Map::new(&env));
        let mut state = beacons
            .get(name.clone())
            .unwrap_or_else(|| panic!("beacon not found"));

        if state.history.len() < 2 {
            panic!("no previous version");
        }

        let previous = state
            .history
            .get(state.history.len() - 2)
            .unwrap_or_else(|| panic!("no previous version"));
        state.current_version = previous.version;
        state.current_implementation = previous.implementation.clone();
        beacons.set(name.clone(), state);
        env.storage().instance().set(&BEACONS_KEY, &beacons);

        previous.version
    }

    pub fn get_beacon(env: Env, name: Symbol) -> BeaconState {
        let beacons: Map<Symbol, BeaconState> = env
            .storage()
            .instance()
            .get(&BEACONS_KEY)
            .unwrap_or_else(|| Map::new(&env));
        beacons
            .get(name)
            .unwrap_or_else(|| panic!("beacon not found"))
    }

    pub fn list_beacons(env: Env) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&BEACON_NAMES_KEY)
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn require_admin(env: &Env, admin: &Address) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("not initialized"));
        if &stored_admin != admin {
            panic!("not authorized");
        }
        admin.require_auth();
    }
}

#[cfg(test)]
mod test;
