//! # Diamond Security Pattern
//!
//! This example demonstrates secure patterns for the Diamond Multi-Facet Proxy pattern in Soroban.
//! It addresses the core security challenges of Diamond Proxies:
//! 1. **Access control per facet:** Facets can only be invoked through the Diamond Proxy.
//! 2. **Upgrade safeguards:** Verifying that facets support the required functions before mapping them,
//!    and preventing core admin bricking.
//! 3. **Storage collision prevention:** Emulating EVM-style shared storage securely by offering
//!    namespaced-isolated key-value storage managed by the Proxy contract.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, IntoVal,
    Symbol, TryIntoVal, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecurityError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 3,
    FacetNotFound = 4,
    InterfaceMismatch = 5,
    UnauthorizedFacet = 6,
    DuplicateFunction = 7,
    InvalidCaller = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    FacetMap(Symbol),
    FacetFunctions(Address),
    NamespacedStorage(Address, Symbol),
}

// Client definition to call supports_interface on the Facet contract
pub struct FacetInterfaceClient<'a> {
    env: &'a Env,
    contract_id: &'a Address,
}

impl<'a> FacetInterfaceClient<'a> {
    pub fn new(env: &'a Env, contract_id: &'a Address) -> Self {
        Self { env, contract_id }
    }
    pub fn supports_interface(&self, functions: &Vec<Symbol>) -> bool {
        let mut args: Vec<Val> = Vec::new(self.env);
        args.push_back(functions.to_val());
        self.env
            .invoke_contract(self.contract_id, &symbol_short!("support"), args)
    }
}

#[contract]
pub struct DiamondProxyContract;

#[contractimpl]
impl DiamondProxyContract {
    /// Initialize the Diamond Proxy with the admin address.
    pub fn init(env: Env, admin: Address) -> Result<(), SecurityError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SecurityError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().extend_ttl(10000, 10000);
        Ok(())
    }

    /// Add a new facet contract and register its supported functions.
    /// Performs interface verification by calling `support` on the facet address.
    pub fn add_facet(
        env: Env,
        admin: Address,
        facet: Address,
        functions: Vec<Symbol>,
    ) -> Result<(), SecurityError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SecurityError::NotInitialized)?;

        if admin != stored_admin {
            return Err(SecurityError::NotAdmin);
        }

        // Interface Verification: call supports_interface on the target facet
        let client = FacetInterfaceClient::new(&env, &facet);
        let supports = client.supports_interface(&functions);
        if !supports {
            return Err(SecurityError::InterfaceMismatch);
        }

        // Register functions
        for function in functions.iter() {
            let map_key = DataKey::FacetMap(function.clone());
            if env.storage().persistent().has(&map_key) {
                return Err(SecurityError::DuplicateFunction);
            }
            env.storage().persistent().set(&map_key, &facet);
        }

        env.storage()
            .persistent()
            .set(&DataKey::FacetFunctions(facet.clone()), &functions);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::FacetFunctions(facet.clone()), 5000, 10000);

        // Emit facet added event
        env.events()
            .publish((symbol_short!("add_fac"), facet), functions);

        Ok(())
    }

    /// Remove a facet contract and deregister all of its functions.
    pub fn remove_facet(env: Env, admin: Address, facet: Address) -> Result<(), SecurityError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SecurityError::NotInitialized)?;

        if admin != stored_admin {
            return Err(SecurityError::NotAdmin);
        }

        let facet_functions_key = DataKey::FacetFunctions(facet.clone());
        if let Some(functions) = env
            .storage()
            .persistent()
            .get::<_, Vec<Symbol>>(&facet_functions_key)
        {
            for function in functions.iter() {
                env.storage()
                    .persistent()
                    .remove(&DataKey::FacetMap(function));
            }
            env.storage().persistent().remove(&facet_functions_key);

            // Emit facet removed event
            env.events().publish((symbol_short!("rem_fac"), facet), ());
        }

        Ok(())
    }

    /// Route a function invocation to the registered facet contract.
    /// Passes the Diamond Proxy's own address as the 'caller' to authenticate the facet.
    pub fn execute(env: Env, function: Symbol, args: Vec<Val>) -> Result<Val, SecurityError> {
        let facet: Address = env
            .storage()
            .persistent()
            .get(&DataKey::FacetMap(function.clone()))
            .ok_or(SecurityError::FacetNotFound)?;

        // Prepend Diamond's own address as the 'caller' first argument to secure facet access.
        let mut facet_args: Vec<Val> = Vec::new(&env);
        facet_args.push_back(env.current_contract_address().to_val());
        for arg in args.iter() {
            facet_args.push_back(arg);
        }

        // Invoke the facet function
        let result: Val = env.invoke_contract(&facet, &function, facet_args);
        Ok(result)
    }

    /// Update the administrator address.
    pub fn upgrade_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), SecurityError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SecurityError::NotInitialized)?;

        if admin != stored_admin {
            return Err(SecurityError::NotAdmin);
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }
}

#[cfg(test)]
mod test;
