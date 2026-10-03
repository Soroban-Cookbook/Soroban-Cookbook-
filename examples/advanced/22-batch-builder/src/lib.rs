#![no_std]

//! # Batch Builder Utility
//!
//! Demonstrates a staged builder for composing, validating, estimating, and
//! executing batches of balance mutations in a single Soroban contract.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

/// Maximum number of operations allowed in one batch.
pub const MAX_BATCH_SIZE: u32 = 32;

/// Fixed overhead applied to every batch plan.
pub const BASE_GAS_UNITS: u64 = 12_000;
/// Estimated host cost per transfer operation.
pub const GAS_PER_TRANSFER: u64 = 6_000;
/// Estimated host cost per mint operation.
pub const GAS_PER_MINT: u64 = 5_000;
/// Estimated host cost per burn operation.
pub const GAS_PER_BURN: u64 = 5_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchOp {
    Transfer(Address, Address, i128),
    Mint(Address, i128),
    Burn(Address, i128),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BuilderError {
    BatchNotFound = 1,
    EmptyBatch = 2,
    BatchTooLarge = 3,
    InvalidAmount = 4,
    NotValidated = 5,
    AlreadyValidated = 6,
    InsufficientBalance = 7,
    DuplicateOperation = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchDraft {
    pub ops: Vec<BatchOp>,
    pub validated: bool,
    pub estimated_gas: u64,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    NextBatchId,
    Batch(u32),
    Balance(Address),
}

#[contract]
pub struct BatchBuilderContract;

#[contractimpl]
impl BatchBuilderContract {
    /// Initialize batch-builder storage.
    pub fn initialize(env: Env) {
        if env.storage().instance().has(&DataKey::NextBatchId) {
            return;
        }
        env.storage().instance().set(&DataKey::NextBatchId, &1u32);
    }

    /// Start a new empty batch and return its identifier.
    pub fn begin_batch(env: Env) -> u32 {
        Self::initialize(env.clone());
        let id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::NextBatchId)
            .unwrap_or(1);
        let draft = BatchDraft {
            ops: Vec::new(&env),
            validated: false,
            estimated_gas: BASE_GAS_UNITS,
        };
        env.storage().instance().set(&DataKey::Batch(id), &draft);
        env.storage()
            .instance()
            .set(&DataKey::NextBatchId, &(id + 1));
        id
    }

    /// Append a transfer operation to the batch.
    pub fn add_transfer(
        env: Env,
        batch_id: u32,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BuilderError> {
        Self::push_op(env, batch_id, BatchOp::Transfer(from, to, amount))
    }

    /// Append a mint operation to the batch.
    pub fn add_mint(
        env: Env,
        batch_id: u32,
        account: Address,
        amount: i128,
    ) -> Result<(), BuilderError> {
        Self::push_op(env, batch_id, BatchOp::Mint(account, amount))
    }

    /// Append a burn operation to the batch.
    pub fn add_burn(
        env: Env,
        batch_id: u32,
        account: Address,
        amount: i128,
    ) -> Result<(), BuilderError> {
        Self::push_op(env, batch_id, BatchOp::Burn(account, amount))
    }

    /// Validate the batch and return the estimated gas cost.
    pub fn validate_batch(env: Env, batch_id: u32) -> Result<u64, BuilderError> {
        let mut draft = Self::load_draft(&env, batch_id)?;
        if draft.validated {
            return Err(BuilderError::AlreadyValidated);
        }
        Self::validate_ops(&env, &draft.ops)?;
        let gas = Self::compute_gas(&draft.ops);
        draft.validated = true;
        draft.estimated_gas = gas;
        env.storage()
            .instance()
            .set(&DataKey::Batch(batch_id), &draft);
        Ok(gas)
    }

    /// Estimate gas for the current batch without marking it validated.
    pub fn estimate_gas(env: Env, batch_id: u32) -> Result<u64, BuilderError> {
        let draft = Self::load_draft(&env, batch_id)?;
        if draft.ops.is_empty() {
            return Err(BuilderError::EmptyBatch);
        }
        Ok(Self::compute_gas(&draft.ops))
    }

    /// Return the number of queued operations.
    pub fn batch_len(env: Env, batch_id: u32) -> Result<u32, BuilderError> {
        let draft = Self::load_draft(&env, batch_id)?;
        Ok(draft.ops.len())
    }

    /// Return whether the batch passed validation.
    pub fn is_validated(env: Env, batch_id: u32) -> Result<bool, BuilderError> {
        let draft = Self::load_draft(&env, batch_id)?;
        Ok(draft.validated)
    }

    /// Execute a validated batch and return the number of applied operations.
    pub fn execute_batch(env: Env, batch_id: u32) -> Result<u32, BuilderError> {
        let draft = Self::load_draft(&env, batch_id)?;
        if !draft.validated {
            return Err(BuilderError::NotValidated);
        }

        let mut executed = 0u32;
        for op in draft.ops.iter() {
            Self::apply_op(&env, op)?;
            executed += 1;
        }

        env.storage().instance().remove(&DataKey::Batch(batch_id));
        Ok(executed)
    }

    /// Seed an account balance (used by tests and demos).
    pub fn set_balance(env: Env, account: Address, amount: i128) {
        env.storage()
            .instance()
            .set(&DataKey::Balance(account), &amount);
    }

    /// Read an account balance.
    pub fn get_balance(env: Env, account: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }

    fn push_op(env: Env, batch_id: u32, op: BatchOp) -> Result<(), BuilderError> {
        let mut draft = Self::load_draft(&env, batch_id)?;
        if draft.validated {
            return Err(BuilderError::AlreadyValidated);
        }
        if draft.ops.len() >= MAX_BATCH_SIZE {
            return Err(BuilderError::BatchTooLarge);
        }
        draft.ops.push_back(op);
        draft.estimated_gas = Self::compute_gas(&draft.ops);
        env.storage()
            .instance()
            .set(&DataKey::Batch(batch_id), &draft);
        Ok(())
    }

    fn load_draft(env: &Env, batch_id: u32) -> Result<BatchDraft, BuilderError> {
        env.storage()
            .instance()
            .get(&DataKey::Batch(batch_id))
            .ok_or(BuilderError::BatchNotFound)
    }

    fn validate_ops(env: &Env, ops: &Vec<BatchOp>) -> Result<(), BuilderError> {
        if ops.is_empty() {
            return Err(BuilderError::EmptyBatch);
        }
        if ops.len() > MAX_BATCH_SIZE {
            return Err(BuilderError::BatchTooLarge);
        }

        let mut i = 0u32;
        while i < ops.len() {
            let mut j = i + 1;
            while j < ops.len() {
                if ops.get(i).unwrap() == ops.get(j).unwrap() {
                    return Err(BuilderError::DuplicateOperation);
                }
                j += 1;
            }
            i += 1;
        }

        let mut simulated: Vec<(Address, i128)> = Vec::new(env);

        for op in ops.iter() {
            match op {
                BatchOp::Transfer(from, to, amount) => {
                    if amount <= 0 {
                        return Err(BuilderError::InvalidAmount);
                    }
                    let from_balance = Self::simulated_balance(env, &simulated, from.clone());
                    if from_balance < amount {
                        return Err(BuilderError::InsufficientBalance);
                    }
                    Self::adjust_simulated(&mut simulated, from.clone(), from_balance - amount);
                    let to_balance = Self::simulated_balance(env, &simulated, to.clone());
                    Self::adjust_simulated(&mut simulated, to.clone(), to_balance + amount);
                }
                BatchOp::Mint(account, amount) => {
                    if amount <= 0 {
                        return Err(BuilderError::InvalidAmount);
                    }
                    let balance = Self::simulated_balance(env, &simulated, account.clone());
                    Self::adjust_simulated(&mut simulated, account.clone(), balance + amount);
                }
                BatchOp::Burn(account, amount) => {
                    if amount <= 0 {
                        return Err(BuilderError::InvalidAmount);
                    }
                    let balance = Self::simulated_balance(env, &simulated, account.clone());
                    if balance < amount {
                        return Err(BuilderError::InsufficientBalance);
                    }
                    Self::adjust_simulated(&mut simulated, account.clone(), balance - amount);
                }
            }
        }

        Ok(())
    }

    fn compute_gas(ops: &Vec<BatchOp>) -> u64 {
        let mut gas = BASE_GAS_UNITS;
        for op in ops.iter() {
            gas = gas.saturating_add(Self::gas_for_op(&op));
        }
        gas
    }

    fn apply_op(env: &Env, op: BatchOp) -> Result<(), BuilderError> {
        match op {
            BatchOp::Transfer(from, to, amount) => {
                let from_balance = Self::get_balance(env.clone(), from.clone());
                let to_balance = Self::get_balance(env.clone(), to.clone());
                env.storage()
                    .instance()
                    .set(&DataKey::Balance(from), &(from_balance - amount));
                env.storage()
                    .instance()
                    .set(&DataKey::Balance(to), &(to_balance + amount));
                Ok(())
            }
            BatchOp::Mint(account, amount) => {
                let balance = Self::get_balance(env.clone(), account.clone());
                env.storage()
                    .instance()
                    .set(&DataKey::Balance(account), &(balance + amount));
                Ok(())
            }
            BatchOp::Burn(account, amount) => {
                let balance = Self::get_balance(env.clone(), account.clone());
                env.storage()
                    .instance()
                    .set(&DataKey::Balance(account), &(balance - amount));
                Ok(())
            }
        }
    }

    fn gas_for_op(op: &BatchOp) -> u64 {
        match op {
            BatchOp::Transfer(_, _, _) => GAS_PER_TRANSFER,
            BatchOp::Mint(_, _) => GAS_PER_MINT,
            BatchOp::Burn(_, _) => GAS_PER_BURN,
        }
    }

    fn simulated_balance(env: &Env, simulated: &Vec<(Address, i128)>, account: Address) -> i128 {
        let mut idx = 0u32;
        while idx < simulated.len() {
            let (addr, balance) = simulated.get(idx).unwrap();
            if addr == account {
                return balance;
            }
            idx += 1;
        }
        Self::get_balance(env.clone(), account)
    }

    fn adjust_simulated(simulated: &mut Vec<(Address, i128)>, account: Address, balance: i128) {
        let mut idx = 0u32;
        while idx < simulated.len() {
            let (addr, _) = simulated.get(idx).unwrap();
            if addr == account {
                simulated.set(idx, (account, balance));
                return;
            }
            idx += 1;
        }
        simulated.push_back((account, balance));
    }
}

#[cfg(test)]
mod test;
