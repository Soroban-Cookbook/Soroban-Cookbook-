#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Balance(Address),
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchOperation {
    Credit(Address, i128),
    Debit(Address, i128),
    Transfer(Address, Address, i128),
    SetPaused(bool),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BatchError {
    InvalidAmount = 1,
    InsufficientBalance = 2,
    ArithmeticOverflow = 3,
    ContractPaused = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpStatus {
    Applied,
    Skipped(BatchError),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchExecution {
    pub applied: u32,
    pub failed: u32,
    pub statuses: Vec<OpStatus>,
}

#[contract]
pub struct BatchOperations;

#[contractimpl]
impl BatchOperations {
    pub fn set_balance(env: Env, user: Address, amount: i128) -> Result<(), BatchError> {
        if amount < 0 {
            return Err(BatchError::InvalidAmount);
        }
        env.storage().instance().set(&DataKey::Balance(user), &amount);
        Ok(())
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    pub fn execute_batch_atomic(
        env: Env,
        operations: Vec<BatchOperation>,
    ) -> Result<(), BatchError> {
        let (tracked_accounts, snapshot_balances) = Self::snapshot_balances(&env, &operations);
        let paused_before = Self::is_paused(env.clone());

        for op in operations.iter() {
            if let Err(err) = Self::apply_operation(&env, op) {
                Self::restore_state(&env, tracked_accounts, snapshot_balances, paused_before);
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn execute_batch_partial(env: Env, operations: Vec<BatchOperation>) -> BatchExecution {
        let mut statuses = Vec::new(&env);
        let mut applied = 0u32;
        let mut failed = 0u32;

        for op in operations.iter() {
            match Self::apply_operation(&env, op) {
                Ok(()) => {
                    statuses.push_back(OpStatus::Applied);
                    applied += 1;
                }
                Err(err) => {
                    statuses.push_back(OpStatus::Skipped(err));
                    failed += 1;
                }
            }
        }

        BatchExecution {
            applied,
            failed,
            statuses,
        }
    }

    fn apply_operation(env: &Env, op: BatchOperation) -> Result<(), BatchError> {
        match op {
            BatchOperation::Credit(user, amount) => Self::credit(env, user, amount),
            BatchOperation::Debit(user, amount) => Self::debit(env, user, amount),
            BatchOperation::Transfer(from, to, amount) => Self::transfer(env, from, to, amount),
            BatchOperation::SetPaused(paused) => {
                env.storage().instance().set(&DataKey::Paused, &paused);
                Ok(())
            }
        }
    }

    fn credit(env: &Env, user: Address, amount: i128) -> Result<(), BatchError> {
        Self::validate_financial_op(env, amount)?;

        let current = Self::get_balance(env.clone(), user.clone());
        let updated = current
            .checked_add(amount)
            .ok_or(BatchError::ArithmeticOverflow)?;

        env.storage().instance().set(&DataKey::Balance(user), &updated);
        Ok(())
    }

    fn debit(env: &Env, user: Address, amount: i128) -> Result<(), BatchError> {
        Self::validate_financial_op(env, amount)?;

        let current = Self::get_balance(env.clone(), user.clone());
        if current < amount {
            return Err(BatchError::InsufficientBalance);
        }

        let updated = current
            .checked_sub(amount)
            .ok_or(BatchError::ArithmeticOverflow)?;

        env.storage().instance().set(&DataKey::Balance(user), &updated);
        Ok(())
    }

    fn transfer(env: &Env, from: Address, to: Address, amount: i128) -> Result<(), BatchError> {
        Self::validate_financial_op(env, amount)?;

        let from_balance = Self::get_balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(BatchError::InsufficientBalance);
        }

        let to_balance = Self::get_balance(env.clone(), to.clone());
        let next_from = from_balance
            .checked_sub(amount)
            .ok_or(BatchError::ArithmeticOverflow)?;
        let next_to = to_balance
            .checked_add(amount)
            .ok_or(BatchError::ArithmeticOverflow)?;

        env.storage().instance().set(&DataKey::Balance(from), &next_from);
        env.storage().instance().set(&DataKey::Balance(to), &next_to);
        Ok(())
    }

    fn validate_financial_op(env: &Env, amount: i128) -> Result<(), BatchError> {
        if amount <= 0 {
            return Err(BatchError::InvalidAmount);
        }

        if Self::is_paused(env.clone()) {
            return Err(BatchError::ContractPaused);
        }

        Ok(())
    }

    fn snapshot_balances(env: &Env, operations: &Vec<BatchOperation>) -> (Vec<Address>, Vec<i128>) {
        let mut tracked_accounts = Vec::new(env);
        let mut snapshot_balances = Vec::new(env);

        for op in operations.iter() {
            match op {
                BatchOperation::Credit(user, _) | BatchOperation::Debit(user, _) => {
                    Self::track_account(env, &mut tracked_accounts, &mut snapshot_balances, user)
                }
                BatchOperation::Transfer(from, to, _) => {
                    Self::track_account(env, &mut tracked_accounts, &mut snapshot_balances, from);
                    Self::track_account(env, &mut tracked_accounts, &mut snapshot_balances, to);
                }
                BatchOperation::SetPaused(_) => {}
            }
        }

        (tracked_accounts, snapshot_balances)
    }

    fn track_account(
        env: &Env,
        tracked_accounts: &mut Vec<Address>,
        snapshot_balances: &mut Vec<i128>,
        account: Address,
    ) {
        if tracked_accounts.contains(&account) {
            return;
        }

        tracked_accounts.push_back(account.clone());
        snapshot_balances.push_back(Self::get_balance(env.clone(), account));
    }

    fn restore_state(
        env: &Env,
        tracked_accounts: Vec<Address>,
        snapshot_balances: Vec<i128>,
        paused_before: bool,
    ) {
        env.storage().instance().set(&DataKey::Paused, &paused_before);

        let len = tracked_accounts.len();
        let mut i = 0;
        while i < len {
            let account = tracked_accounts.get(i).unwrap();
            let balance = snapshot_balances.get(i).unwrap();
            env.storage().instance().set(&DataKey::Balance(account), &balance);
            i += 1;
        }
    }
}

#[cfg(test)]
mod test;
