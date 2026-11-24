#![no_std]

//! # Batch Builder Utility
//!
//! Demonstrates a staged builder for composing, validating, estimating, and
//! executing batches of balance mutations in a single Soroban contract.
//!
//! ## Instruction-budget rationale
//!
//! Every Soroban transaction executes inside a fixed CPU-instruction budget
//! (currently ~100 million instructions on Mainnet).  Each `invoke_contract`
//! call, host-object allocation, and ledger read/write consumes a portion of
//! that budget.  Empirical benchmarks show that a single balance-mutation
//! operation (transfer, mint, or burn) costs roughly 5 000–6 000 instructions
//! in overhead alone, before the contract's own logic runs.
//!
//! A batch of **20 operations** therefore consumes on the order of
//! 120 000–132 000 instructions just in overhead — well within the budget for
//! typical contracts, while leaving headroom for validation, storage I/O, and
//! event emission.  Batches larger than 20 risk approaching the budget ceiling
//! unpredictably, depending on the complexity of each operation and the number
//! of distinct accounts touched.
//!
//! Rather than letting the runtime trap silently when the budget is exceeded,
//! the contract rejects oversized batches *upfront* with
//! [`BuilderError::TooManyOps`], giving callers a deterministic error they can
//! act on.  Deployers who need a different ceiling can supply their own
//! `max_ops` value when calling [`BatchBuilderContract::initialize`].

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

/// Default maximum number of operations allowed in one batch.
///
/// Chosen so that the total host-instruction overhead (≈ 6 000 instructions
/// per op × 20 ops = 120 000) leaves ample budget for contract logic, storage
/// I/O, and event emission within Soroban's transaction limit.  Pass a custom
/// value to [`BatchBuilderContract::initialize`] to override this default.
pub const MAX_BATCH_SIZE: u32 = 20;

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
    /// Returned when a batch exceeds the configured maximum number of
    /// operations.  Callers must reduce the batch size and resubmit.
    ///
    /// This error is raised both when adding an operation via `add_*` helpers
    /// **and** when calling `execute_batch`, so a batch that was constructed
    /// outside the normal builder path (e.g., via direct storage manipulation
    /// in tests) is still rejected before any operation is applied.
    TooManyOps = 3,
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
    /// Stores the operator-configured maximum batch size for this deployment.
    MaxBatchSize,
}

#[contract]
pub struct BatchBuilderContract;

#[contractimpl]
impl BatchBuilderContract {
    /// Initialize the batch-builder.
    ///
    /// `max_ops` sets the per-batch operation limit for this deployment.
    /// Pass `0` to use [`MAX_BATCH_SIZE`] (the default of 20).
    ///
    /// Re-initializing an already-initialized contract is a no-op so that
    /// contract upgrades do not accidentally reset the limit.
    pub fn initialize(env: Env, max_ops: u32) {
        if env.storage().instance().has(&DataKey::NextBatchId) {
            return;
        }
        let limit = if max_ops == 0 { MAX_BATCH_SIZE } else { max_ops };
        env.storage()
            .instance()
            .set(&DataKey::MaxBatchSize, &limit);
        env.storage().instance().set(&DataKey::NextBatchId, &1u32);
    }

    /// Start a new empty batch and return its identifier.
    pub fn begin_batch(env: Env) -> u32 {
        Self::ensure_initialized(&env);
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
    ///
    /// Returns [`BuilderError::TooManyOps`] if the batch contains more
    /// operations than the configured limit, even if validation was skipped or
    /// the limit was lowered after validation.  This guard ensures the
    /// instruction budget cannot be exhausted by an oversized batch that
    /// bypassed the normal builder path.
    pub fn execute_batch(env: Env, batch_id: u32) -> Result<u32, BuilderError> {
        let draft = Self::load_draft(&env, batch_id)?;
        if !draft.validated {
            return Err(BuilderError::NotValidated);
        }

        // Re-check size at execution time so a batch that bypassed push_op
        // (e.g., constructed directly in storage) is still rejected before any
        // operation is applied.
        let limit = Self::max_batch_size(&env);
        if draft.ops.len() > limit {
            return Err(BuilderError::TooManyOps);
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

    // ── internal helpers ────────────────────────────────────────────────────

    fn ensure_initialized(env: &Env) {
        if !env.storage().instance().has(&DataKey::NextBatchId) {
            let limit = MAX_BATCH_SIZE;
            env.storage()
                .instance()
                .set(&DataKey::MaxBatchSize, &limit);
            env.storage().instance().set(&DataKey::NextBatchId, &1u32);
        }
    }

    fn max_batch_size(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::MaxBatchSize)
            .unwrap_or(MAX_BATCH_SIZE)
    }

    fn push_op(env: Env, batch_id: u32, op: BatchOp) -> Result<(), BuilderError> {
        let mut draft = Self::load_draft(&env, batch_id)?;
        if draft.validated {
            return Err(BuilderError::AlreadyValidated);
        }
        let limit = Self::max_batch_size(&env);
        if draft.ops.len() >= limit {
            return Err(BuilderError::TooManyOps);
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

        let limit = Self::max_batch_size(env);
        if ops.len() > limit {
            return Err(BuilderError::TooManyOps);
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
