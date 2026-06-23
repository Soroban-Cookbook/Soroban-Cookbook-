//! # Primitive Types in Soroban
//!
//! This example demonstrates the usage of primitive types in Soroban smart
//! contracts.  It is intentionally minimal so that each concept stands on its
//! own — no auth, no complex storage hierarchies, just the type system.
//!
//! ## What's Covered
//!
//! ### 1. Integer Types
//! - **Unsigned**: `u32`, `u64` — for values that are always ≥ 0.
//! - **Signed**: `i32`, `i64` — for values that may be negative.
//! - **Large**: `i128` — the preferred type for financial amounts (balances,
//!   fees, interest) because Soroban's token interface uses it natively.
//!
//! ### 2. Boolean Type
//! - `bool` in conditional logic, logical operators, and contract storage.
//!
//! ### 3. Type Conversions
//! - *Widening* (always safe): `u32 → u64`, `i32 → i64`.
//! - *Narrowing* (may fail): range-checked before casting; returns
//!   `Err(ConversionError)` instead of silently truncating.
//! - *Sign change* (may fail): unsigned ↔ signed guards against negative or
//!   out-of-range values.
//!
//! ### 4. Overflow Handling
//! - **Checked**: returns `Err` on overflow — safest for contract logic.
//! - **Saturating**: clamps to `MAX`/`MIN` — useful when you prefer a
//!   bounded result over an error.
//! - **Wrapping**: wraps modulo 2^N — only appropriate when intentional
//!   (e.g. hash mixing, nonces).
//!
//! ### 5. Additional Patterns (bonus)
//! - Bit manipulation (flags, masks).
//! - Comparison and clamping utilities.
//! - Counter and balance storage using instance storage.
//! - Financial calculations using `i128` for precision.

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Env};

// Contract Errors

/// All error codes the contract can return.
///
/// Using a `#[contracterror]` enum (backed by `u32`) is the idiomatic Soroban
/// way to return typed errors.  The host encodes them as `SCError::Contract`
/// values so callers can pattern-match on them.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// General errors (1000-1099)
    InvalidInput = 1000,
    Unauthorized = 1001,
    NotFound = 1002,
    AlreadyExists = 1003,

    /// Type conversion errors (1100-1199)
    ConversionError = 1100,
    OverflowError = 1101,
    UnderflowError = 1102,
    DivisionByZero = 1103,
    NegativeValue = 1104,

    /// Arithmetic errors (1200-1299)
    ArithmeticError = 1200,
    InvalidOperation = 1201,
    InsufficientBalance = 1202,
}

// Storage Keys

/// Keys used to address values in instance storage.
///
/// `#[contracttype]` makes this enum serialisable by the Soroban host so it
/// can be used as a storage key.  Using an enum (rather than bare strings)
/// makes key management explicit and refactor-safe.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    U32Value = 0,
    U64Value = 1,
    I32Value = 2,
    I64Value = 3,
    BoolValue = 4,
    Counter = 5,
    Balance = 6,
    Flags = 7,
}

// Contract

#[contract]
pub struct PrimitiveTypesContract;

#[contractimpl]
impl PrimitiveTypesContract {
    /// Seed the contract's instance storage with demonstration values.
    ///
    /// Each value is set to the type's maximum/representative constant so that
    /// callers can immediately observe boundary behaviour with `retrieve_*`.
    pub fn initialize(env: Env) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U32Value, &u32::MAX); // 4_294_967_295
        env.storage().instance().set(&DataKey::U64Value, &u64::MAX); // 18_446_744_073_709_551_615
        env.storage().instance().set(&DataKey::I32Value, &i32::MAX); // 2_147_483_647
        env.storage().instance().set(&DataKey::I64Value, &i64::MAX); // 9_223_372_036_854_775_807
        env.storage().instance().set(&DataKey::BoolValue, &true);
        env.storage().instance().set(&DataKey::Counter, &0u64);
        env.storage().instance().set(&DataKey::Balance, &1000i128);
        env.storage().instance().set(&DataKey::Flags, &0u32);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Section 1 — Unsigned Integer Operations (u32)
    //
    // u32 is a 32-bit unsigned integer (0 … 4_294_967_295).
    // Good for: small counters, array indices, flag bitmasks, ledger sequence
    // numbers that fit in 32 bits.
    // -----------------------------------------------------------------------

    /// Add two `u32` values.
    ///
    /// `checked_add` returns `None` instead of wrapping on overflow, so we
    /// convert `None → Err(OverflowError)`.  This is the pattern used
    /// throughout this contract for safe arithmetic.
    pub fn add_u32(_env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract `b` from `a`.
    ///
    /// For unsigned types, subtraction can *underflow* (go below 0), which is
    /// reported as `UnderflowError` so the caller can distinguish it from
    /// overflow.
    pub fn sub_u32(_env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_sub(b).ok_or(ContractError::UnderflowError)
    }

    /// Multiply two `u32` values.
    pub fn mul_u32(_env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide `a` by `b`.
    ///
    /// Division by zero is explicitly checked *before* the division operator
    /// because Soroban's Wasm sandbox propagates a host panic (not a graceful
    /// error) on integer division-by-zero.
    pub fn div_u32(_env: Env, a: u32, b: u32) -> Result<u32, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // -----------------------------------------------------------------------
    // Section 1 — Unsigned Integer Operations (u64)
    //
    // u64 is a 64-bit unsigned integer (0 … 18_446_744_073_709_551_615).
    // Good for: timestamps (`env.ledger().timestamp()` returns u64), large
    // counters, token IDs, nonces.
    // -----------------------------------------------------------------------

    /// Add two `u64` values.  Identical pattern to `add_u32`; bigger range.
    pub fn add_u64(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two `u64` values.
    pub fn sub_u64(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_sub(b).ok_or(ContractError::UnderflowError)
    }

    /// Multiply two `u64` values.
    pub fn mul_u64(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two `u64` values.
    pub fn div_u64(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // -----------------------------------------------------------------------
    // Section 1 — Signed Integer Operations (i32)
    //
    // i32 is a 32-bit signed integer (-2_147_483_648 … 2_147_483_647).
    // Good for: differences, temperatures, relative positions, error codes.
    // Note: overflow can occur at *both ends* — checked_add / checked_sub
    // guard both overflow AND underflow, so we unify the error.
    // -----------------------------------------------------------------------

    /// Add two `i32` values.  Overflow is possible at both `i32::MAX + 1`
    /// and `i32::MIN - 1`, so a single `OverflowError` covers both.
    pub fn add_i32(_env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two `i32` values.
    pub fn sub_i32(_env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_sub(b).ok_or(ContractError::OverflowError)
    }

    /// Multiply two `i32` values.
    pub fn mul_i32(_env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two `i32` values.
    pub fn div_i32(_env: Env, a: i32, b: i32) -> Result<i32, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // -----------------------------------------------------------------------
    // Section 1 — Signed Integer Operations (i64)
    //
    // i64 is a 64-bit signed integer.
    // Good for: large signed counters, price deltas, signed timestamps.
    // -----------------------------------------------------------------------

    /// Add two `i64` values.
    pub fn add_i64(_env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_add(b).ok_or(ContractError::OverflowError)
    }

    /// Subtract two `i64` values.
    pub fn sub_i64(_env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_sub(b).ok_or(ContractError::OverflowError)
    }

    /// Multiply two `i64` values.
    pub fn mul_i64(_env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        a.checked_mul(b).ok_or(ContractError::OverflowError)
    }

    /// Divide two `i64` values.
    pub fn div_i64(_env: Env, a: i64, b: i64) -> Result<i64, ContractError> {
        if b == 0 {
            return Err(ContractError::DivisionByZero);
        }
        Ok(a / b)
    }

    // -----------------------------------------------------------------------
    // Section 2 — Boolean Operations
    //
    // `bool` is the simplest type in the set: it can only be `true` or
    // `false`.  Soroban serialises it natively, so booleans cost no extra
    // overhead compared to a u32 flag.
    // -----------------------------------------------------------------------

    /// Logical AND — both operands must be `true`.
    pub fn bool_and(_env: Env, a: bool, b: bool) -> bool {
        a && b
    }

    /// Logical OR — at least one operand must be `true`.
    pub fn bool_or(_env: Env, a: bool, b: bool) -> bool {
        a || b
    }

    /// Logical NOT — inverts the operand.
    pub fn bool_not(_env: Env, a: bool) -> bool {
        !a
    }

    /// Logical XOR — true when exactly one operand is `true`.
    ///
    /// Rust has no `^^` operator for booleans; the idiomatic spelling is `!=`.
    pub fn bool_xor(_env: Env, a: bool, b: bool) -> bool {
        a != b
    }

    /// Persist a boolean in instance storage.
    ///
    /// A common pattern: use a `bool` flag to track whether the contract has
    /// been initialised, whether a feature is enabled, or whether an action
    /// has been taken.
    pub fn set_bool(env: Env, value: bool) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::BoolValue, &value);
        Ok(())
    }

    /// Retrieve the stored boolean.  Returns `NotFound` before `set_bool` is
    /// called.
    pub fn get_bool(env: Env) -> Result<bool, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::BoolValue)
            .ok_or(ContractError::NotFound)
    }

    // -----------------------------------------------------------------------
    // Section 3 — Type Conversions
    //
    // Rust requires *explicit* casts between integer types.  The rules are:
    //
    //  Widening (always safe — no data loss)
    //    u32 → u64     value fits; just zero-extend.
    //    i32 → i64     value fits; sign-extend.
    //
    //  Narrowing (may truncate — must range-check first)
    //    u64 → u32     values > u32::MAX are rejected.
    //    i64 → i32     values outside [i32::MIN, i32::MAX] are rejected.
    //
    //  Sign change (may fail on negative or out-of-range)
    //    u32 → i32     values > i32::MAX cannot be represented as i32.
    //    i32 → u32     negative values have no unsigned representation.
    //    u64 → i64     values > i64::MAX cannot be represented.
    //    i64 → u64     negative values have no unsigned representation.
    // -----------------------------------------------------------------------

    /// Widen `u32` to `u64`.  Always succeeds — every `u32` value fits in
    /// a `u64`.
    pub fn u32_to_u64(_env: Env, value: u32) -> u64 {
        // `as u64` is safe here; Rust guarantees no truncation when the
        // destination type is wider.
        value as u64
    }

    /// Narrow `u64` to `u32`.
    ///
    /// Values that exceed `u32::MAX` (4_294_967_295) are rejected.
    pub fn u64_to_u32(_env: Env, value: u64) -> Result<u32, ContractError> {
        if value > u32::MAX as u64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as u32)
    }

    /// Widen `i32` to `i64`.  Always succeeds.
    pub fn i32_to_i64(_env: Env, value: i32) -> i64 {
        value as i64
    }

    /// Narrow `i64` to `i32`.
    ///
    /// Values outside [-2_147_483_648, 2_147_483_647] are rejected.
    pub fn i64_to_i32(_env: Env, value: i64) -> Result<i32, ContractError> {
        if value > i32::MAX as i64 || value < i32::MIN as i64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i32)
    }

    /// Convert unsigned `u32` to signed `i32`.
    ///
    /// Values above `i32::MAX` (2_147_483_647) cannot be represented as
    /// positive `i32` values, so they are rejected.
    pub fn u32_to_i32(_env: Env, value: u32) -> Result<i32, ContractError> {
        if value > i32::MAX as u32 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i32)
    }

    /// Convert signed `i32` to unsigned `u32`.
    ///
    /// Negative values have no representation in an unsigned type, so they
    /// are explicitly rejected with `NegativeValue`.
    pub fn i32_to_u32(_env: Env, value: i32) -> Result<u32, ContractError> {
        if value < 0 {
            return Err(ContractError::NegativeValue);
        }
        Ok(value as u32)
    }

    /// Convert negative-safe `i64` to unsigned `u64`.
    pub fn i64_to_u64(_env: Env, value: i64) -> Result<u64, ContractError> {
        if value < 0 {
            return Err(ContractError::NegativeValue);
        }
        Ok(value as u64)
    }

    /// Convert unsigned `u64` to signed `i64`.
    ///
    /// Values above `i64::MAX` (9_223_372_036_854_775_807) are rejected.
    pub fn u64_to_i64(_env: Env, value: u64) -> Result<i64, ContractError> {
        if value > i64::MAX as u64 {
            return Err(ContractError::ConversionError);
        }
        Ok(value as i64)
    }

    // -----------------------------------------------------------------------
    // Section 4 — Overflow Handling
    //
    // Rust natively offers three families of overflow-aware operations:
    //
    //  checked_*   → Option<T>: None on overflow.  Convert to Result<T, E>
    //                with .ok_or(…).  Best for contract logic — explicit
    //                errors are better than silent corruption.
    //
    //  saturating_* → T: clamps to MAX/MIN instead of wrapping.  Use when a
    //                bounded output is acceptable (e.g. approximate counters,
    //                display values).
    //
    //  wrapping_*  → T: wraps modulo 2^N.  Use only when overflow is
    //                intentional (e.g. hash functions, nonce generation).
    //
    // Note: In Soroban's release profile `overflow-checks = true` is set
    // (see workspace Cargo.toml).  That means plain `a + b` *panics* on
    // overflow in release builds.  Always use one of the three families above.
    // -----------------------------------------------------------------------

    // —— Checked arithmetic ——

    /// Checked add — returns `Err(OverflowError)` if the result exceeds
    /// `u64::MAX`.
    pub fn safe_add(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_add(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::OverflowError),
        }
    }

    /// Checked subtract — returns `Err(UnderflowError)` if `b > a`.
    pub fn safe_sub(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_sub(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::UnderflowError),
        }
    }

    /// Checked multiply — returns `Err(OverflowError)` if the product exceeds
    /// `u64::MAX`.
    pub fn safe_mul(_env: Env, a: u64, b: u64) -> Result<u64, ContractError> {
        match a.checked_mul(b) {
            Some(result) => Ok(result),
            None => Err(ContractError::OverflowError),
        }
    }

    // —— Saturating arithmetic ——

    /// Saturating add — clamps to `u64::MAX` instead of overflowing.
    ///
    /// ```text
    /// saturating_add(u64::MAX, 1) == u64::MAX   // clamped, no error
    /// ```
    pub fn saturating_add(_env: Env, a: u64, b: u64) -> u64 {
        a.saturating_add(b)
    }

    /// Saturating subtract — clamps to `0` instead of underflowing.
    ///
    /// ```text
    /// saturating_sub(0, 1) == 0   // clamped, no error
    /// ```
    pub fn saturating_sub(_env: Env, a: u64, b: u64) -> u64 {
        a.saturating_sub(b)
    }

    /// Saturating multiply — clamps to `u64::MAX` on overflow.
    pub fn saturating_mul(_env: Env, a: u64, b: u64) -> u64 {
        a.saturating_mul(b)
    }

    // —— Wrapping arithmetic ——

    /// Wrapping add — wraps modulo 2^64 on overflow.
    ///
    /// ```text
    /// wrapping_add(u64::MAX, 1) == 0   // wraps to 0
    /// ```
    pub fn wrapping_add(_env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_add(b)
    }

    /// Wrapping subtract — wraps modulo 2^64 on underflow.
    ///
    /// ```text
    /// wrapping_sub(0, 1) == u64::MAX   // wraps to max
    /// ```
    pub fn wrapping_sub(_env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_sub(b)
    }

    /// Wrapping multiply — wraps modulo 2^64 on overflow.
    pub fn wrapping_mul(_env: Env, a: u64, b: u64) -> u64 {
        a.wrapping_mul(b)
    }

    // -----------------------------------------------------------------------
    // Section 5a — Financial Calculations (i128)
    //
    // Soroban's token interface (`soroban_sdk::token`) expresses all amounts
    // as `i128`.  This avoids the ambiguity of using two u64 fields or
    // floating-point arithmetic (which is unavailable in Wasm).
    //
    // Rule of thumb: use `i128` whenever you store or transfer value.
    // -----------------------------------------------------------------------

    /// Compute simple interest.
    ///
    /// Formula: `interest = principal × rate × periods / 10_000`
    ///
    /// `rate` is expressed in *basis points* (bps): 10_000 bps = 100%.
    pub fn calculate_interest(
        _env: Env,
        principal: i128,
        rate: i32, // basis points; 10_000 == 100 %
        periods: u32,
    ) -> Result<i128, ContractError> {
        // Validate: rate must be in [0, 10_000].
        if !(0..=10000).contains(&rate) {
            return Err(ContractError::InvalidInput);
        }

        let rate_i128 = rate as i128;
        let periods_i128 = periods as i128;

        // Both intermediate multiplications can overflow if inputs are large.
        match principal.checked_mul(rate_i128) {
            Some(interest_rate_product) => match interest_rate_product.checked_mul(periods_i128) {
                Some(total_product) => Ok(total_product / 10000i128),
                None => Err(ContractError::OverflowError),
            },
            None => Err(ContractError::OverflowError),
        }
    }

    /// Compute compound interest earned over `periods`.
    ///
    /// Formula: `interest = principal × (1 + rate/10_000)^periods − principal`
    ///
    /// Uses integer arithmetic by iterating multiplication.  Only practical
    /// for small `periods`; real contracts would use a logarithmic approach.
    pub fn compound_interest(
        _env: Env,
        principal: i128,
        rate: i32, // basis points
        periods: u32,
    ) -> Result<i128, ContractError> {
        if !(0..=10000).contains(&rate) {
            return Err(ContractError::InvalidInput);
        }

        let mut amount = principal;
        let rate_factor = 10000i128 + rate as i128; // e.g. 10500 for 5 %

        for _ in 0..periods {
            match amount.checked_mul(rate_factor) {
                Some(product) => {
                    amount = product / 10000i128;
                }
                None => return Err(ContractError::OverflowError),
            }
        }

        Ok(amount - principal)
    }

    /// Debit `amount` from the stored balance.
    pub fn transfer(env: Env, amount: i128) -> Result<i128, ContractError> {
        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)?;

        if amount < 0 {
            return Err(ContractError::NegativeValue);
        }

        match current_balance.checked_sub(amount) {
            Some(new_balance) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Balance, &new_balance);
                Ok(new_balance)
            }
            None => Err(ContractError::InsufficientBalance),
        }
    }

    /// Credit `amount` to the stored balance.
    pub fn deposit(env: Env, amount: i128) -> Result<i128, ContractError> {
        let current_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)?;

        if amount < 0 {
            return Err(ContractError::NegativeValue);
        }

        match current_balance.checked_add(amount) {
            Some(new_balance) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Balance, &new_balance);
                Ok(new_balance)
            }
            None => Err(ContractError::OverflowError),
        }
    }

    // -----------------------------------------------------------------------
    // Section 5b — Bit Operations
    //
    // Integers double as compact flag stores.  A single `u32` can hold 32
    // independent boolean flags at 1 bit each.
    // -----------------------------------------------------------------------

    /// Bitwise AND — a bit is 1 only if both inputs have it set.
    pub fn bitwise_and(_env: Env, a: u32, b: u32) -> u32 {
        a & b
    }

    /// Bitwise OR — a bit is 1 if at least one input has it set.
    pub fn bitwise_or(_env: Env, a: u32, b: u32) -> u32 {
        a | b
    }

    /// Bitwise XOR — a bit is 1 if exactly one input has it set.
    pub fn bitwise_xor(_env: Env, a: u32, b: u32) -> u32 {
        a ^ b
    }

    /// Bitwise NOT — inverts every bit.
    pub fn bitwise_not(_env: Env, a: u32) -> u32 {
        !a
    }

    /// Left-shift `a` by `shift` positions.
    ///
    /// Shifting by ≥ 32 is undefined behaviour in Rust; we guard against it
    /// explicitly.
    pub fn left_shift(_env: Env, a: u32, shift: u32) -> Result<u32, ContractError> {
        if shift >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(a << shift)
    }

    /// Right-shift `a` by `shift` positions (logical, zero-fill).
    pub fn right_shift(_env: Env, a: u32, shift: u32) -> Result<u32, ContractError> {
        if shift >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(a >> shift)
    }

    /// Test whether bit `bit` is set in `value`.
    pub fn is_bit_set(_env: Env, value: u32, bit: u32) -> Result<bool, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok((value & (1u32 << bit)) != 0)
    }

    /// Set bit `bit` in `value`, returning the updated value.
    pub fn set_bit(_env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value | (1u32 << bit))
    }

    /// Clear bit `bit` in `value`, returning the updated value.
    pub fn clear_bit(_env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value & !(1u32 << bit))
    }

    /// Toggle bit `bit` in `value`, returning the updated value.
    pub fn toggle_bit(_env: Env, value: u32, bit: u32) -> Result<u32, ContractError> {
        if bit >= 32 {
            return Err(ContractError::InvalidInput);
        }
        Ok(value ^ (1u32 << bit))
    }

    // -----------------------------------------------------------------------
    // Section 5c — Counter and Flag Management (stored state)
    // -----------------------------------------------------------------------

    /// Increment the stored counter by 1.  Returns `OverflowError` if the
    /// counter has already reached `u64::MAX`.
    pub fn increment_counter(env: Env) -> Result<u64, ContractError> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)?;

        match counter.checked_add(1) {
            Some(new_counter) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Counter, &new_counter);
                Ok(new_counter)
            }
            None => Err(ContractError::OverflowError),
        }
    }

    /// Decrement the stored counter by 1.  Returns `UnderflowError` if
    /// counter is already 0 (u64 is unsigned).
    pub fn decrement_counter(env: Env) -> Result<u64, ContractError> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)?;

        match counter.checked_sub(1) {
            Some(new_counter) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Counter, &new_counter);
                Ok(new_counter)
            }
            None => Err(ContractError::UnderflowError),
        }
    }

    /// Read the stored counter.
    pub fn get_counter(env: Env) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Counter)
            .ok_or(ContractError::NotFound)
    }

    /// Set bit `flag_bit` in the stored flags word.
    pub fn set_flag(env: Env, flag_bit: u32) -> Result<(), ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        let new_flags = flags | (1u32 << flag_bit);
        env.storage().instance().set(&DataKey::Flags, &new_flags);
        Ok(())
    }

    /// Clear bit `flag_bit` in the stored flags word.
    pub fn clear_flag(env: Env, flag_bit: u32) -> Result<(), ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        let new_flags = flags & !(1u32 << flag_bit);
        env.storage().instance().set(&DataKey::Flags, &new_flags);
        Ok(())
    }

    /// Test whether bit `flag_bit` is set in the stored flags word.
    pub fn is_flag_set(env: Env, flag_bit: u32) -> Result<bool, ContractError> {
        if flag_bit >= 32 {
            return Err(ContractError::InvalidInput);
        }

        let flags: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Flags)
            .ok_or(ContractError::NotFound)?;

        Ok((flags & (1u32 << flag_bit)) != 0)
    }

    // -----------------------------------------------------------------------
    // Section 5d — Comparison and Clamping
    // -----------------------------------------------------------------------

    /// Three-way comparison of two `u32` values.
    ///
    /// Returns `1` if `a > b`, `-1` if `a < b`, `0` if equal.
    /// This mirrors the semantics of C's `memcmp` / `strcmp` and Rust's
    /// `Ordering` (without needing to expose the `Ordering` enum across the
    /// ABI boundary).
    pub fn compare_u32(_env: Env, a: u32, b: u32) -> i32 {
        if a > b {
            1
        } else if a < b {
            -1
        } else {
            0
        }
    }

    /// Three-way comparison of two `i32` values — works for negative numbers.
    pub fn compare_i32(_env: Env, a: i32, b: i32) -> i32 {
        if a > b {
            1
        } else if a < b {
            -1
        } else {
            0
        }
    }

    /// Inclusive range check for `u32`.
    pub fn is_in_range_u32(_env: Env, value: u32, min: u32, max: u32) -> bool {
        value >= min && value <= max
    }

    /// Inclusive range check for `i32` (supports negative bounds).
    pub fn is_in_range_i32(_env: Env, value: i32, min: i32, max: i32) -> bool {
        value >= min && value <= max
    }

    /// Clamp a `u32` to `[min, max]`.
    ///
    /// Rust's `clamp` method is equivalent to `min.max(value).min(max)`.
    pub fn clamp_u32(_env: Env, value: u32, min: u32, max: u32) -> u32 {
        value.clamp(min, max)
    }

    /// Clamp an `i32` to `[min, max]`.
    pub fn clamp_i32(_env: Env, value: i32, min: i32, max: i32) -> i32 {
        value.clamp(min, max)
    }

    // -----------------------------------------------------------------------
    // Section 5e — Storage Helpers
    // -----------------------------------------------------------------------

    /// Persist a `u32` value in instance storage.
    pub fn store_u32(env: Env, value: u32) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U32Value, &value);
        Ok(())
    }

    /// Retrieve the stored `u32` value.
    pub fn retrieve_u32(env: Env) -> Result<u32, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::U32Value)
            .ok_or(ContractError::NotFound)
    }

    /// Persist a `u64` value in instance storage.
    pub fn store_u64(env: Env, value: u64) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U64Value, &value);
        Ok(())
    }

    /// Retrieve the stored `u64` value.
    pub fn retrieve_u64(env: Env) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::U64Value)
            .ok_or(ContractError::NotFound)
    }

    /// Persist an `i32` value in instance storage.
    pub fn store_i32(env: Env, value: i32) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::I32Value, &value);
        Ok(())
    }

    /// Retrieve the stored `i32` value.
    pub fn retrieve_i32(env: Env) -> Result<i32, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::I32Value)
            .ok_or(ContractError::NotFound)
    }

    /// Persist an `i64` value in instance storage.
    pub fn store_i64(env: Env, value: i64) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::I64Value, &value);
        Ok(())
    }

    /// Retrieve the stored `i64` value.
    pub fn retrieve_i64(env: Env) -> Result<i64, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::I64Value)
            .ok_or(ContractError::NotFound)
    }

    /// Read the current balance.
    pub fn get_balance(env: Env) -> Result<i128, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Balance)
            .ok_or(ContractError::NotFound)
    }

    /// Reset all stored primitives to zero / false.
    pub fn reset_to_defaults(env: Env) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::U32Value, &0u32);
        env.storage().instance().set(&DataKey::U64Value, &0u64);
        env.storage().instance().set(&DataKey::I32Value, &0i32);
        env.storage().instance().set(&DataKey::I64Value, &0i64);
        env.storage().instance().set(&DataKey::BoolValue, &false);
        env.storage().instance().set(&DataKey::Counter, &0u64);
        env.storage().instance().set(&DataKey::Balance, &0i128);
        env.storage().instance().set(&DataKey::Flags, &0u32);
        Ok(())
    }
}

// Pull in the dedicated test module.
#[cfg(test)]
mod test;
