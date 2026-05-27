//! # Panic vs. Errors in Soroban
//!
//! Soroban contracts have two failure modes: **panics** (unrecoverable aborts)
//! and **errors** (typed, recoverable values the caller can inspect).
//!
//! ## Decision Rule
//!
//! | Situation | Mechanism | Why |
//! |-----------|-----------|-----|
//! | Invariant that should never be false | `panic!` / `panic_with_error!` | Signals a bug; no recovery makes sense |
//! | Auth failure (`require_auth`) | Soroban panics internally | Unauthorized callers must be rejected hard |
//! | Expected bad input from caller | `Err(ContractError::…)` | Caller can handle and retry |
//! | Business-logic constraint violated | `Err(ContractError::…)` | Predictable; documentable; testable |
//! | Reached truly impossible branch | `panic!("unreachable: …")` | Defensive; keeps the type system happy |
//!
//! ## Performance Note
//!
//! Both aborts consume the submitted fee; there is no gas "refund" for a
//! cleaner error path. Prefer typed errors for *user-facing* failures because
//! they allow the client to react without re-submitting a doomed transaction.
//!
//! ## Anatomy
//!
//! ```text
//! ┌────────────────────┬────────────────────────────────────────────┐
//! │  Panic             │  Typed Error                               │
//! ├────────────────────┼────────────────────────────────────────────┤
//! │  panic!("msg")     │  #[contracterror] enum + Result<T, E>      │
//! │  panic_with_error! │  ? operator / map_err                      │
//! │  Immediate abort   │  Caller sees u32 discriminant via XDR      │
//! │  No info to caller │  Documentable, testable variants           │
//! └────────────────────┴────────────────────────────────────────────┘
//! ```

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Error enum
// ---------------------------------------------------------------------------

/// All expected failure modes for this contract.
///
/// `#[contracterror]` serialises each variant as its `u32` discriminant
/// in the XDR result so the caller (SDK or horizon) can identify the reason.
///
/// ### Numbering convention
/// - Start at 1 (0 is reserved for "no error" in some XDR tooling).
/// - Leave gaps between categories so new variants can be inserted without
///   breaking existing clients.
/// - Document every variant; these are part of your public API.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    // ---- input validation (1xx) -------------------------------------------
    /// Caller supplied an amount of zero, which is never meaningful.
    ZeroAmount = 100,
    /// Requested withdrawal exceeds the recorded balance.
    InsufficientBalance = 101,
    /// A string/symbol argument exceeded the maximum allowed length.
    InputTooLong = 102,

    // ---- state / business logic (2xx) -------------------------------------
    /// The contract has been administratively paused; operations are blocked.
    ContractPaused = 200,
    /// The caller is not the registered admin.
    Unauthorized = 201,
    /// Arithmetic overflow detected during a safe-math operation.
    Overflow = 202,
}

// ---------------------------------------------------------------------------
// Storage key type
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Admin,
    Paused,
}

// ---------------------------------------------------------------------------
// Audit event payload
// ---------------------------------------------------------------------------

/// Emitted whenever a deposit or withdrawal succeeds.
#[contracttype]
pub struct LedgerEventData {
    pub amount: i128,
    pub action: Symbol,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ErrorDemoContract;

#[contractimpl]
impl ErrorDemoContract {
    // =======================================================================
    // Initialisation
    // =======================================================================

    /// Initialise the contract.  Panics if called a second time.
    ///
    /// ### Why panic?
    /// Re-initialisation is a *contract-level invariant violation*: it must
    /// never happen.  There is nothing the caller can do to "fix" it — the
    /// call should never have been made at all.  A panic signals that
    /// clearly and costs less code than an error path no one should reach.
    pub fn initialize(env: Env, admin: Address) {
        // Guard: panic if already initialised — this is an invariant, not
        // a user error.
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("contract already initialised");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    // =======================================================================
    // Example A — typed errors for expected input/business failures
    // =======================================================================

    /// Deposit `amount` into the caller's balance.
    ///
    /// ### Error returns (typed)
    /// - [`ContractError::ZeroAmount`]    – `amount == 0`
    /// - [`ContractError::ContractPaused`] – contract is paused
    /// - [`ContractError::Overflow`]       – balance would overflow `i128`
    ///
    /// These are *expected* failure modes: the client can read the error code
    /// and present a meaningful message to the end user or retry with
    /// corrected parameters.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<i128, ContractError> {
        // ── Guard: predictable user mistakes → typed errors ─────────────────
        if amount == 0 {
            return Err(ContractError::ZeroAmount);
        }

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }

        // ── Auth: Soroban panics internally if signature is missing ──────────
        // `require_auth` is itself a hard abort — unauthorised callers are
        // rejected unconditionally.  This is intentional: there is no
        // "recover from being someone else."
        from.require_auth();

        // ── Business logic ───────────────────────────────────────────────────
        let key = DataKey::Balance(from.clone());
        let old_balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);

        // Safe addition: overflow is possible if balances are huge — return a
        // typed error rather than silently wrapping.
        let new_balance = old_balance
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?; // `?` propagates the Err variant

        env.storage().persistent().set(&key, &new_balance);

        env.events().publish(
            (symbol_short!("errdemo"), symbol_short!("deposit"), from),
            LedgerEventData {
                amount,
                action: symbol_short!("deposit"),
            },
        );

        Ok(new_balance)
    }

    /// Withdraw `amount` from the caller's balance.
    ///
    /// ### Error returns (typed)
    /// - [`ContractError::ZeroAmount`]         – `amount == 0`
    /// - [`ContractError::ContractPaused`]      – contract is paused
    /// - [`ContractError::InsufficientBalance`] – balance < amount
    pub fn withdraw(env: Env, from: Address, amount: i128) -> Result<i128, ContractError> {
        if amount == 0 {
            return Err(ContractError::ZeroAmount);
        }

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }

        from.require_auth();

        let key = DataKey::Balance(from.clone());
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);

        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        let new_balance = balance - amount; // safe: we checked above
        env.storage().persistent().set(&key, &new_balance);

        env.events().publish(
            (symbol_short!("errdemo"), symbol_short!("withdraw"), from),
            LedgerEventData {
                amount,
                action: symbol_short!("withdraw"),
            },
        );

        Ok(new_balance)
    }

    // =======================================================================
    // Example B — panic_with_error! for hard invariant violations
    // =======================================================================

    /// Pause the contract.  Only the registered admin may do this.
    ///
    /// ### Why `panic_with_error!` instead of `Err(Unauthorized)`?
    ///
    /// This is an *administrative invariant*: if your code allows a non-admin
    /// to reach this branch, you have a bug in your auth logic, not a user
    /// input error.  `panic_with_error!` lets you attach the error code for
    /// off-chain diagnostics while still aborting the transaction hard.
    ///
    /// Compare: returning `Err(Unauthorized)` would imply the caller could
    /// "try again differently" — but there is nothing to retry.
    pub fn pause(env: Env, caller: Address) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            // Panic: if Admin key is missing the contract was never initialised.
            // This should be impossible after `initialize` runs; treat as a bug.
            .unwrap_or_else(|| panic!("uninitialised: admin key missing"));

        if caller != admin {
            // Hard abort with a typed code.  Off-chain tools can read
            // `ContractError::Unauthorized` (discriminant 201) from the
            // transaction result even though the transaction reverted.
            panic_with_error!(env, ContractError::Unauthorized);
        }

        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env, caller: Address) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("uninitialised: admin key missing"));

        if caller != admin {
            panic_with_error!(env, ContractError::Unauthorized);
        }

        env.storage().instance().set(&DataKey::Paused, &false);
    }

    // =======================================================================
    // Example C — defensive panic on impossible branch
    // =======================================================================

    /// Returns a human-readable label for an internal status code.
    ///
    /// The match is exhaustive over a closed enum, but Rust requires a
    /// wildcard arm.  The `panic!` here documents that reaching `_` would
    /// mean a programming error (someone added a variant and forgot to update
    /// this function), not a user mistake.
    pub fn status_label(_env: Env, code: u32) -> Symbol {
        match code {
            0 => symbol_short!("ok"),
            1 => symbol_short!("paused"),
            2 => symbol_short!("error"),
            _ => panic!("unknown status code: this is a bug"),
            // ↑  Do NOT return an Err here — callers passing arbitrary codes
            //    should use validated input; treat anything else as a contract
            //    invariant violation.
        }
    }

    // =======================================================================
    // Read helpers
    // =======================================================================

    /// Returns the balance for `account`, or 0 if never deposited.
    pub fn balance(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Converts lower-level math errors into contract-level errors.
    pub fn divide_with_conversion(a: i128, b: i128) -> Result<i128, Error> {
        Ok(Self::divide_checked(a, b).map_err(Error::from)?)
    }
}

#[cfg(test)]
mod test;