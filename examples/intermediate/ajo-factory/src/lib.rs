//! # Factory Templates Example
//!
//! This example demonstrates a versioned factory pattern in Soroban:
//! 1. Template contracts that can be deployed repeatedly.
//! 2. A factory that stores version metadata for each template.
//! 3. Parameter validation before deployment and initialization.
//!
//! This pattern is useful when one factory needs to create multiple contract
//! shapes without hardcoding a single deployment path.

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    Symbol, Vec,
};

pub const TEMPLATE_AJO: Symbol = symbol_short!("ajo");
pub const TEMPLATE_SAVINGS: Symbol = symbol_short!("savings");
pub const TEMPLATE_ESCROW: Symbol = symbol_short!("escrow");
pub const DEFAULT_VERSION: Symbol = symbol_short!("v1");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidMaxMembers = 5,
    InvalidDeadline = 6,
    InvalidTemplateParams = 7,
    TemplateAlreadyRegistered = 8,
    TemplateNotFound = 9,
    InvalidVersion = 10,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateMetadata {
    pub template_id: Symbol,
    pub version: Symbol,
    pub wasm_hash: BytesN<32>,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateParams {
    Ajo(AjoParams),
    Savings(SavingsParams),
    Escrow(EscrowParams),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeployedInstance {
    pub template_id: Symbol,
    pub version: Symbol,
    pub address: Address,
    pub creator: Address,
}

// ---------------------------------------------------------------------------
// Ajo Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct Ajo;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AjoDataKey {
    Amount,
    MaxMembers,
    Creator,
}

#[contractimpl]
impl Ajo {
    pub fn init_ajo(
        env: Env,
        amount: i128,
        max_members: u32,
        creator: Address,
    ) -> Result<(), FactoryError> {
        if env.storage().instance().has(&AjoDataKey::Creator) {
            return Err(FactoryError::AlreadyInitialized);
        }
        validate_ajo_params(amount, max_members)?;

        env.storage().instance().set(&AjoDataKey::Amount, &amount);
        env.storage()
            .instance()
            .set(&AjoDataKey::MaxMembers, &max_members);
        env.storage().instance().set(&AjoDataKey::Creator, &creator);

        Ok(())
    }

    pub fn ajo_creator(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&AjoDataKey::Creator)
            .expect("Not initialized")
    }

    pub fn ajo_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&AjoDataKey::Amount)
            .expect("Not initialized")
    }

    pub fn ajo_max_members(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&AjoDataKey::MaxMembers)
            .expect("Not initialized")
    }

    pub fn init_savings(
        env: Env,
        target_amount: i128,
        deadline: u64,
        owner: Address,
    ) -> Result<(), FactoryError> {
        if env.storage().instance().has(&SavingsDataKey::Owner) {
            return Err(FactoryError::AlreadyInitialized);
        }
        validate_savings_params(target_amount, deadline)?;

        env.storage().instance().set(&SavingsDataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&SavingsDataKey::TargetAmount, &target_amount);
        env.storage()
            .instance()
            .set(&SavingsDataKey::Deadline, &deadline);

        Ok(())
    }

    pub fn savings_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&SavingsDataKey::Owner)
            .expect("Not initialized")
    }

    pub fn savings_target_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&SavingsDataKey::TargetAmount)
            .expect("Not initialized")
    }

    pub fn savings_deadline(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&SavingsDataKey::Deadline)
            .expect("Not initialized")
    }

    pub fn init_escrow(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        amount: i128,
    ) -> Result<(), FactoryError> {
        if env.storage().instance().has(&EscrowDataKey::Depositor) {
            return Err(FactoryError::AlreadyInitialized);
        }
        validate_amount(amount)?;

        env.storage()
            .instance()
            .set(&EscrowDataKey::Depositor, &depositor);
        env.storage()
            .instance()
            .set(&EscrowDataKey::Beneficiary, &beneficiary);
        env.storage()
            .instance()
            .set(&EscrowDataKey::Amount, &amount);

        Ok(())
    }

    pub fn escrow_depositor(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&EscrowDataKey::Depositor)
            .expect("Not initialized")
    }

    pub fn escrow_beneficiary(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&EscrowDataKey::Beneficiary)
            .expect("Not initialized")
    }

    pub fn escrow_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&EscrowDataKey::Amount)
            .expect("Not initialized")
    }
}

// ---------------------------------------------------------------------------
// Savings and Escrow Template Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SavingsDataKey {
    Owner,
    TargetAmount,
    Deadline,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowDataKey {
    Depositor,
    Beneficiary,
    Amount,
}

// ---------------------------------------------------------------------------
// Factory Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AjoFactory;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactoryDataKey {
    WasmHash,
    Template(Symbol),
    TemplateIds,
    DeployedAjos,
    DeployedInstances,
}

#[contractimpl]
impl AjoFactory {
    /// Initialize the factory with the default Ajo template.
    pub fn initialize(env: Env, wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
        if env.storage().instance().has(&FactoryDataKey::WasmHash) {
            return Err(FactoryError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&FactoryDataKey::WasmHash, &wasm_hash);

        let ajos: Vec<Address> = Vec::new(&env);
        let instances: Vec<DeployedInstance> = Vec::new(&env);
        let template_ids: Vec<Symbol> = Vec::new(&env);

        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedInstances, &instances);
        env.storage()
            .instance()
            .set(&FactoryDataKey::TemplateIds, &template_ids);

        register_template_internal(&env, TEMPLATE_AJO, wasm_hash, DEFAULT_VERSION)?;

        Ok(())
    }

    /// Register another deployable template and its version metadata.
    pub fn register_template(
        env: Env,
        template_id: Symbol,
        wasm_hash: BytesN<32>,
        version: Symbol,
    ) -> Result<(), FactoryError> {
        ensure_initialized(&env)?;
        register_template_internal(&env, template_id, wasm_hash, version)
    }

    /// Create any registered template with validated parameters.
    pub fn create_instance(
        env: Env,
        template_id: Symbol,
        params: TemplateParams,
        creator: Address,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let metadata = read_template(&env, &template_id)?;
        validate_template_params(&template_id, &params)?;

        let instances = read_instances(&env);
        let mut salt_input = Bytes::new(&env);
        salt_input.extend_from_slice(&instances.len().to_be_bytes());
        let salt = env.crypto().sha256(&salt_input);
        let deployed_address = env
            .deployer()
            .with_current_contract(salt)
            .deploy(metadata.wasm_hash.clone());

        track_instance(
            &env,
            template_id,
            metadata.version,
            deployed_address.clone(),
            creator,
        );

        Ok(deployed_address)
    }

    /// Backwards-compatible helper for the original single-template example.
    pub fn create_ajo(
        env: Env,
        amount: i128,
        max_members: u32,
        creator: Address,
    ) -> Result<Address, FactoryError> {
        let address = Self::create_instance(
            env.clone(),
            TEMPLATE_AJO,
            TemplateParams::Ajo(AjoParams {
                amount,
                max_members,
            }),
            creator,
        )?;

        let mut ajos: Vec<Address> = env
            .storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env));
        ajos.push_back(address.clone());
        env.storage()
            .instance()
            .set(&FactoryDataKey::DeployedAjos, &ajos);

        Ok(address)
    }

    pub fn get_template(env: Env, template_id: Symbol) -> Result<TemplateMetadata, FactoryError> {
        read_template(&env, &template_id)
    }

    pub fn get_template_ids(env: Env) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::TemplateIds)
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_deployed_instances(env: Env) -> Vec<DeployedInstance> {
        read_instances(&env)
    }

    /// Get all deployed Ajos from the original helper path.
    pub fn get_deployed_ajos(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&FactoryDataKey::DeployedAjos)
            .unwrap_or(Vec::new(&env))
    }
}

fn ensure_initialized(env: &Env) -> Result<(), FactoryError> {
    if !env.storage().instance().has(&FactoryDataKey::WasmHash) {
        return Err(FactoryError::NotInitialized);
    }
    Ok(())
}

fn register_template_internal(
    env: &Env,
    template_id: Symbol,
    wasm_hash: BytesN<32>,
    version: Symbol,
) -> Result<(), FactoryError> {
    let key = FactoryDataKey::Template(template_id.clone());
    if env.storage().instance().has(&key) {
        return Err(FactoryError::TemplateAlreadyRegistered);
    }

    let metadata = TemplateMetadata {
        template_id: template_id.clone(),
        version,
        wasm_hash,
    };
    env.storage().instance().set(&key, &metadata);

    let mut ids: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&FactoryDataKey::TemplateIds)
        .unwrap_or(Vec::new(env));
    ids.push_back(template_id.clone());
    env.storage()
        .instance()
        .set(&FactoryDataKey::TemplateIds, &ids);
    env.events()
        .publish((symbol_short!("tmpl_reg"), template_id), metadata.version);

    Ok(())
}

fn read_template(env: &Env, template_id: &Symbol) -> Result<TemplateMetadata, FactoryError> {
    env.storage()
        .instance()
        .get(&FactoryDataKey::Template(template_id.clone()))
        .ok_or(FactoryError::TemplateNotFound)
}

fn read_instances(env: &Env) -> Vec<DeployedInstance> {
    env.storage()
        .instance()
        .get(&FactoryDataKey::DeployedInstances)
        .unwrap_or(Vec::new(env))
}

fn track_instance(
    env: &Env,
    template_id: Symbol,
    version: Symbol,
    address: Address,
    creator: Address,
) {
    let mut instances = read_instances(env);
    let instance = DeployedInstance {
        template_id: template_id.clone(),
        version,
        address: address.clone(),
        creator: creator.clone(),
    };
    instances.push_back(instance);
    env.storage()
        .instance()
        .set(&FactoryDataKey::DeployedInstances, &instances);

    env.events()
        .publish((symbol_short!("created"), template_id, address), creator);
}

fn validate_template_params(
    template_id: &Symbol,
    params: &TemplateParams,
) -> Result<(), FactoryError> {
    match (template_id.clone(), params.clone()) {
        (id, TemplateParams::Ajo(params)) if id == TEMPLATE_AJO => {
            validate_ajo_params(params.amount, params.max_members)
        }
        (id, TemplateParams::Savings(params)) if id == TEMPLATE_SAVINGS => {
            validate_savings_params(params.target_amount, params.deadline)
        }
        (id, TemplateParams::Escrow(params)) if id == TEMPLATE_ESCROW => {
            validate_amount(params.amount)
        }
        _ => Err(FactoryError::InvalidTemplateParams),
    }
}

fn validate_ajo_params(amount: i128, max_members: u32) -> Result<(), FactoryError> {
    validate_amount(amount)?;
    if max_members < 2 {
        return Err(FactoryError::InvalidMaxMembers);
    }
    Ok(())
}

fn validate_savings_params(target_amount: i128, deadline: u64) -> Result<(), FactoryError> {
    validate_amount(target_amount)?;
    if deadline == 0 {
        return Err(FactoryError::InvalidDeadline);
    }
    Ok(())
}

fn validate_amount(amount: i128) -> Result<(), FactoryError> {
    if amount <= 0 {
        return Err(FactoryError::InvalidAmount);
    }
    Ok(())
}

#[cfg(test)]
mod test;
