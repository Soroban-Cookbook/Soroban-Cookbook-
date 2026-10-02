//! # Multicall
//!
//! Batch several cross-contract calls into a single transaction.
//!
//! Two entry points cover the two common needs:
//!
//! | Function        | Failure semantics                                              |
//! |-----------------|----------------------------------------------------------------|
//! | `aggregate`     | **Atomic.** Any failing call aborts and reverts the whole batch. |
//! | `aggregate_results` | **Best effort.** Each call reports success/failure individually; a failed call's state changes are rolled back, the others are kept. Pass `require_success = true` to get atomic behaviour with per-call results. |
//!
//! ## Security notes
//!
//! * `caller.require_auth()` is enforced on every batch. The caller's
//!   signature covers the multicall invocation *and* the nested sub-calls in
//!   its authorization tree, so a target that checks `caller.require_auth()`
//!   is satisfied only when the caller really signed for it.
//! * The multicall contract never acts on its own behalf: it holds no funds and
//!   no privileges, so it cannot be used to escalate authority.
//! * Calls back into the multicall contract itself are rejected to prevent
//!   nested batches / re-entrancy through the aggregator.
//! * Batch size is capped by [`MAX_CALLS`] so a single transaction cannot
//!   request unbounded work.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Error, Symbol,
    Val, Vec,
};

/// Maximum number of calls accepted in a single batch.
pub const MAX_CALLS: u32 = 32;

/// A single call to be executed as part of a batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Call {
    /// Contract to invoke.
    pub contract: Address,
    /// Function name on the target contract.
    pub function: Symbol,
    /// Arguments passed to the function.
    pub args: Vec<Val>,
}

/// Outcome of one call inside `aggregate_results`: `(success, data)`.
///
/// `data` is the return value on success and `()` (void) on failure. A tuple
/// is used because `Val` cannot be a `#[contracttype]` struct field.
pub type CallResult = (bool, Val);

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MulticallError {
    /// The batch contained no calls.
    EmptyBatch = 1,
    /// The batch exceeded [`MAX_CALLS`].
    TooManyCalls = 2,
    /// A call targeted the multicall contract itself.
    SelfCall = 3,
    /// A call failed while `require_success` was set.
    CallFailed = 4,
}

#[contract]
pub struct MulticallContract;

#[contractimpl]
impl MulticallContract {
    /// Execute every call in order and return their results.
    ///
    /// Atomic: if any call fails, the whole transaction reverts.
    pub fn aggregate(
        env: Env,
        caller: Address,
        calls: Vec<Call>,
    ) -> Result<Vec<Val>, MulticallError> {
        caller.require_auth();
        validate(&env, &calls)?;

        let mut results = Vec::new(&env);
        for call in calls.iter() {
            let value: Val = env.invoke_contract(&call.contract, &call.function, call.args);
            results.push_back(value);
        }

        publish_batch(&env, &caller, results.len());
        Ok(results)
    }

    /// Execute every call in order, capturing failures instead of aborting.
    ///
    /// When `require_success` is `true`, the first failing call aborts the
    /// batch with [`MulticallError::CallFailed`] and reverts all prior calls.
    pub fn aggregate_results(
        env: Env,
        caller: Address,
        calls: Vec<Call>,
        require_success: bool,
    ) -> Result<Vec<CallResult>, MulticallError> {
        caller.require_auth();
        validate(&env, &calls)?;

        let mut results = Vec::new(&env);
        for call in calls.iter() {
            let outcome =
                env.try_invoke_contract::<Val, Error>(&call.contract, &call.function, call.args);

            let result = match outcome {
                Ok(Ok(data)) => (true, data),
                _ => {
                    if require_success {
                        return Err(MulticallError::CallFailed);
                    }
                    (false, Val::VOID.into())
                }
            };
            results.push_back(result);
        }

        publish_batch(&env, &caller, results.len());
        Ok(results)
    }
}

/// Reject empty, oversized, or self-targeting batches.
fn validate(env: &Env, calls: &Vec<Call>) -> Result<(), MulticallError> {
    let len = calls.len();
    if len == 0 {
        return Err(MulticallError::EmptyBatch);
    }
    if len > MAX_CALLS {
        return Err(MulticallError::TooManyCalls);
    }

    let this = env.current_contract_address();
    for call in calls.iter() {
        if call.contract == this {
            return Err(MulticallError::SelfCall);
        }
    }
    Ok(())
}

/// Emit `("multicall", caller) -> call_count` after a batch completes.
fn publish_batch(env: &Env, caller: &Address, count: u32) {
    #[allow(deprecated)]
    env.events()
        .publish((symbol_short!("multicall"), caller.clone()), count);
}

#[cfg(test)]
mod test;
