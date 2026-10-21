//! # Ajo Factory — Multi-Template Factory (Issue #95)
//!
//! Factory that supports multiple contract templates with version metadata
//! and parameter validation.  Templates are registered by the factory owner;
//! callers choose a template when deploying a new instance.
//!
//! ## Templates
//! | ID | Contract | Params |
//! |----|----------|--------|
//! `ajo`     | Rotating savings pool | `amount`, `max_members` |
//! `savings` | Goal-based savings    | `target_amount`, `deadline` |
//! `escrow`  | Two-party escrow      | `beneficiary`, `amount` |
//!
//! ## Access control
//! Template registration is open (no admin required) but duplicate IDs are
//! rejected.  Instance creation requires the creator to authorise.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]
use ajo::AjoClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Public template-ID constants
// ---------------------------------------------------------------------------

pub const TEMPLATE_AJO: Symbol = symbol_short!("ajo");
pub const TEMPLATE_SAVINGS: Symbol = symbol_short!("savings");
pub const TEMPLATE_ESCROW: Symbol = symbol_short!("escrow");
pub const DEFAULT_VERSION: Symbol = symbol_short!("v1");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    TemplateNotFound = 4,
    TemplateAlreadyRegistered = 5,
    InvalidTemplateParams = 6,
    InvalidAmount = 7,
    InvalidMaxMembers = 8,
    InvalidDeadline = 9,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactoryDataKey {
    WasmHash,
    DeployedAjos,
    /// Template metadata keyed by template ID symbol.
    Template(Symbol),
    /// Ordered list of registered template IDs.
    TemplateIds,
    /// All deployed instances (across all templates).
    Instances,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Metadata stored for each registered template.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateMetadata {
    pub template_id: Symbol,
    pub wasm_hash: BytesN<32>,
    pub version: Symbol,
}

/// Record of a deployed instance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceRecord {
    pub address: Address,
    pub template_id: Symbol,
    pub creator: Address,
}

// ---------------------------------------------------------------------------
// Per-template constructor parameters
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AjoParams {
    pub amount: i128,
    pub max_members: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingsParams {
    pub target_amount: i128,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowParams {
    pub beneficiary: Address,
    pub amount: i128,
}

/// Discriminated union of all supported parameter sets.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateParams {
    Ajo(AjoParams),
    Savings(SavingsParams),
    Escrow(EscrowParams),
}

// ---------------------------------------------------------------------------
// Salt helper
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
enum DeploySalt {
    Instance(Address, u32),
    /// Separate variant for the legacy `create_ajo` path to avoid salt
    /// collisions with `create_instance`.
    Ajo(Address, u32),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AjoFactory;

#[contractimpl]
impl AjoFactory {
    /// Initialise the factory with the default Ajo WASM hash.
    ///
    /// Automatically registers the `ajo` template so existing callers that
    /// only use `create_ajo` continue to work without calling
    /// `register_template` first.
    pub fn initialize(env: Env, wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
        if env.storage().instance().has(&FactoryDataKey::WasmHash) {
            return Err(FactoryError::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&FactoryDataKey::WasmHash, &wasm_hash);

        // Legacy list kept for backward-compat with `get_deployed_ajos`.
        let ajos: Vec<Address> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        let instances: Vec<InstanceRecord> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&FactoryDataKey::Instances, &instances);

        // Register the default Ajo template.
        let mut ids: Vec<Symbol> = Vec::new(&env);
        ids.push_back(TEMPLATE_AJO);
        env.storage()
            .instance()
            .set(&FactoryDataKey::TemplateIds, &ids);

        let meta = TemplateMetadata {
            template_id: TEMPLATE_AJO,
            wasm_hash,
            version: DEFAULT_VERSION,
        };
        env.storage()
            .instance()
            .set(&FactoryDataKey::Template(TEMPLATE_AJO), &meta);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Template registry
    // -----------------------------------------------------------------------

    /// Register a new template.  Duplicate IDs are rejected.
    pub fn register_template(
        env: Env,
        template_id: Symbol,
        wasm_hash: BytesN<32>,
        version: Symbol,
    ) -> Result<(), FactoryError> {
        let key = FactoryDataKey::Template(template_id.clone());
        if env.storage().instance().has(&key) {
            return Err(FactoryError::TemplateAlreadyRegistered);
        }

        let meta = TemplateMetadata {
            template_id: template_id.clone(),
            wasm_hash,
            version,
        };
        env.storage().instance().set(&key, &meta);

        let mut ids: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::TemplateIds)
            .unwrap_or(Vec::new(&env));
        ids.push_back(template_id.clone());
        env.storage()
            .instance()
            .set(&FactoryDataKey::TemplateIds, &ids);

        env.events()
            .publish((symbol_short!("tmpl_reg"), template_id), ());
        Ok(())
    }

    /// Return metadata for a registered template.
    pub fn get_template(env: Env, template_id: Symbol) -> TemplateMetadata {
        env.storage()
            .instance()
            .get(&FactoryDataKey::Template(template_id))
            .expect("template not found")
    }

    /// Return all registered template IDs.
    pub fn get_template_ids(env: Env) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::TemplateIds)
            .unwrap_or(Vec::new(&env))
    }

    // -----------------------------------------------------------------------
    // Instance creation
    // -----------------------------------------------------------------------

    /// Deploy a new instance of any registered template.
    ///
    /// `params` must match the chosen `template_id`; a mismatch returns
    /// `InvalidTemplateParams`.  Per-template business rules (e.g. positive
    /// amounts) are validated before deployment.
    #[allow(deprecated)]
    pub fn create_instance(
        env: Env,
        template_id: Symbol,
        params: TemplateParams,
        creator: Address,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let meta: TemplateMetadata = env
            .storage()
            .instance()
            .get(&FactoryDataKey::Template(template_id.clone()))
            .ok_or(FactoryError::TemplateNotFound)?;

        // Validate params match the template and satisfy business rules.
        match (&template_id, &params) {
            (id, TemplateParams::Ajo(p)) if *id == TEMPLATE_AJO => {
                if p.amount <= 0 {
                    return Err(FactoryError::InvalidAmount);
                }
                if p.max_members < 2 {
                    return Err(FactoryError::InvalidMaxMembers);
                }
            }
            (id, TemplateParams::Savings(p)) if *id == TEMPLATE_SAVINGS => {
                if p.target_amount <= 0 {
                    return Err(FactoryError::InvalidAmount);
                }
                if p.deadline == 0 {
                    return Err(FactoryError::InvalidDeadline);
                }
            }
            (id, TemplateParams::Escrow(p)) if *id == TEMPLATE_ESCROW => {
                if p.amount <= 0 {
                    return Err(FactoryError::InvalidAmount);
                }
            }
            _ => return Err(FactoryError::InvalidTemplateParams),
        }

        let mut instances: Vec<InstanceRecord> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::Instances)
            .unwrap_or(Vec::new(&env));

        let nonce = instances.len();
        let salt_preimage = DeploySalt::Instance(creator.clone(), nonce);
        let salt = env.crypto().sha256(&salt_preimage.to_xdr(&env));

        let deployed_address = env
            .deployer()
            .with_current_contract(salt)
            .deploy(meta.wasm_hash);

        // Initialise the deployed contract for Ajo templates.
        if template_id == TEMPLATE_AJO {
            if let TemplateParams::Ajo(ref p) = params {
                let ajo_client = AjoClient::new(&env, &deployed_address);
                ajo_client.initialize(&p.amount, &p.max_members, &creator);
            }
        }

        let record = InstanceRecord {
            address: deployed_address.clone(),
            template_id: template_id.clone(),
            creator: creator.clone(),
        };
        instances.push_back(record);
        env.storage()
            .instance()
            .set(&FactoryDataKey::Instances, &instances);

        env.events().publish(
            (
                symbol_short!("Created"),
                template_id,
                deployed_address.clone(),
            ),
            creator,
        );

        Ok(deployed_address)
    }

    /// Convenience wrapper — deploy an Ajo instance (backward-compatible).
    #[allow(deprecated)]
    pub fn create_ajo(
        env: Env,
        amount: i128,
        max_members: u32,
        creator: Address,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::WasmHash)
            .ok_or(FactoryError::NotInitialized)?;

        let mut ajos: Vec<Address> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env));

        let mut instances: Vec<InstanceRecord> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::Instances)
            .unwrap_or(Vec::new(&env));

        let nonce = ajos.len();
        let salt_preimage = DeploySalt::Ajo(creator.clone(), nonce);
        let salt = env.crypto().sha256(&salt_preimage.to_xdr(&env));

        let deployed_address = env.deployer().with_current_contract(salt).deploy(wasm_hash);

        let ajo_client = AjoClient::new(&env, &deployed_address);
        ajo_client.initialize(&amount, &max_members, &creator);

        ajos.push_back(deployed_address.clone());
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        let record = InstanceRecord {
            address: deployed_address.clone(),
            template_id: TEMPLATE_AJO,
            creator: creator.clone(),
        };
        instances.push_back(record);
        env.storage()
            .instance()
            .set(&FactoryDataKey::Instances, &instances);

        env.events().publish(
            (symbol_short!("Created"), deployed_address.clone()),
            creator,
        );

        Ok(deployed_address)
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// All deployed Ajo addresses (legacy).
    pub fn get_deployed_ajos(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env))
    }

    /// All deployed instances across all templates.
    pub fn get_deployed_instances(env: Env) -> Vec<InstanceRecord> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::Instances)
            .unwrap_or(Vec::new(&env))
    }
}

// Re-export the template contract for WASM upload in tests.
pub use ajo::{Ajo, AjoError};

#[cfg(test)]
mod test;
