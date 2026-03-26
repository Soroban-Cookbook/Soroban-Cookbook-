//! Unit tests for the Primitive Types contract.
//!
//! Uses `env.as_contract` + direct function calls — the correct pattern for
//! contracts whose functions return `Result<T, ContractError>`. The generated
//! client unwraps results (panicking on `Err`), so it cannot be used to assert
//! error cases without the verbose `try_*` variants.

use super::*;
use soroban_sdk::Env;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Create a default environment, register the contract, and return both.
fn setup() -> (Env, soroban_sdk::Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    (env, contract_id)
}

// ---------------------------------------------------------------------------
// Unsigned Integer Operations (u32)
// ---------------------------------------------------------------------------

#[test]
fn test_u32_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Addition
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), 10, 20), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::add_u32(env.clone(), u32::MAX - 1, 1),
            Ok(u32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_u32(env.clone(), u32::MAX, 1),
            Err(ContractError::OverflowError)
        );

        // Subtraction
        assert_eq!(PrimitiveTypesContract::sub_u32(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_u32(env.clone(), 10, 10), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::sub_u32(env.clone(), 0, 1),
            Err(ContractError::UnderflowError)
        );

        // Multiplication
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), 5, 6), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::mul_u32(env.clone(), u32::MAX / 2, 2),
            Ok(u32::MAX - 1)
        );
        assert_eq!(
            PrimitiveTypesContract::mul_u32(env.clone(), u32::MAX / 2, 3),
            Err(ContractError::OverflowError)
        );

        // Division
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 20, 5), Ok(4));
        assert_eq!(
            PrimitiveTypesContract::div_u32(env.clone(), 20, 0),
            Err(ContractError::DivisionByZero)
        );
    });
}

// ---------------------------------------------------------------------------
// Unsigned Integer Operations (u64)
// ---------------------------------------------------------------------------

#[test]
fn test_u64_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        assert_eq!(PrimitiveTypesContract::add_u64(env.clone(), 10, 20), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::add_u64(env.clone(), u64::MAX - 1, 1),
            Ok(u64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_u64(env.clone(), u64::MAX, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::sub_u64(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_u64(env.clone(), 10, 10), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::sub_u64(env.clone(), 0, 1),
            Err(ContractError::UnderflowError)
        );

        assert_eq!(PrimitiveTypesContract::mul_u64(env.clone(), 5, 6), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::mul_u64(env.clone(), u64::MAX / 2, 2),
            Ok(u64::MAX - 1)
        );
        assert_eq!(
            PrimitiveTypesContract::mul_u64(env.clone(), u64::MAX / 2, 3),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::div_u64(env.clone(), 20, 5), Ok(4));
        assert_eq!(
            PrimitiveTypesContract::div_u64(env.clone(), 20, 0),
            Err(ContractError::DivisionByZero)
        );
    });
}

// ---------------------------------------------------------------------------
// Signed Integer Operations (i32)
// ---------------------------------------------------------------------------

#[test]
fn test_i32_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), 10, 20), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::add_i32(env.clone(), i32::MAX - 1, 1),
            Ok(i32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_i32(env.clone(), i32::MAX, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), 10, 10), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::sub_i32(env.clone(), i32::MIN + 1, 2),
            Err(ContractError::OverflowError)
        );
        assert_eq!(
            PrimitiveTypesContract::sub_i32(env.clone(), i32::MIN, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::mul_i32(env.clone(), 5, 6), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::mul_i32(env.clone(), i32::MAX / 2, 2),
            Ok(i32::MAX - 1)
        );
        assert_eq!(
            PrimitiveTypesContract::mul_i32(env.clone(), i32::MAX / 2, 3),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::div_i32(env.clone(), 20, 5), Ok(4));
        assert_eq!(
            PrimitiveTypesContract::div_i32(env.clone(), 20, 0),
            Err(ContractError::DivisionByZero)
        );
        // Signed division: negative numerator
        assert_eq!(PrimitiveTypesContract::div_i32(env.clone(), -20, 5), Ok(-4));
    });
}

// ---------------------------------------------------------------------------
// Signed Integer Operations (i64)
// ---------------------------------------------------------------------------

#[test]
fn test_i64_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), 10, 20), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::add_i64(env.clone(), i64::MAX - 1, 1),
            Ok(i64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_i64(env.clone(), i64::MAX, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), 10, 10), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::sub_i64(env.clone(), i64::MIN + 1, 2),
            Err(ContractError::OverflowError)
        );
        assert_eq!(
            PrimitiveTypesContract::sub_i64(env.clone(), i64::MIN, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::mul_i64(env.clone(), 5, 6), Ok(30));
        assert_eq!(
            PrimitiveTypesContract::mul_i64(env.clone(), i64::MAX / 2, 2),
            Ok(i64::MAX - 1)
        );
        assert_eq!(
            PrimitiveTypesContract::mul_i64(env.clone(), i64::MAX / 2, 3),
            Err(ContractError::OverflowError)
        );

        assert_eq!(PrimitiveTypesContract::div_i64(env.clone(), 20, 5), Ok(4));
        assert_eq!(
            PrimitiveTypesContract::div_i64(env.clone(), 20, 0),
            Err(ContractError::DivisionByZero)
        );
        assert_eq!(PrimitiveTypesContract::div_i64(env.clone(), -20, 5), Ok(-4));
    });
}

// ---------------------------------------------------------------------------
// Boolean Operations
// ---------------------------------------------------------------------------

#[test]
fn test_boolean_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Logical operations
        assert!(PrimitiveTypesContract::bool_and(env.clone(), true, true));
        assert!(!PrimitiveTypesContract::bool_and(env.clone(), true, false));
        assert!(!PrimitiveTypesContract::bool_and(env.clone(), false, false));

        assert!(PrimitiveTypesContract::bool_or(env.clone(), true, false));
        assert!(!PrimitiveTypesContract::bool_or(env.clone(), false, false));
        assert!(PrimitiveTypesContract::bool_or(env.clone(), true, true));

        assert!(!PrimitiveTypesContract::bool_not(env.clone(), true));
        assert!(PrimitiveTypesContract::bool_not(env.clone(), false));

        // XOR: true only when operands differ
        assert!(PrimitiveTypesContract::bool_xor(env.clone(), true, false));
        assert!(!PrimitiveTypesContract::bool_xor(env.clone(), true, true));
        assert!(!PrimitiveTypesContract::bool_xor(env.clone(), false, false));

        // Store and retrieve boolean
        assert_eq!(PrimitiveTypesContract::set_bool(env.clone(), true), Ok(()));
        assert!(PrimitiveTypesContract::get_bool(env.clone()).unwrap());
        assert_eq!(PrimitiveTypesContract::set_bool(env.clone(), false), Ok(()));
        assert!(!PrimitiveTypesContract::get_bool(env.clone()).unwrap());
    });
}

// ---------------------------------------------------------------------------
// Type Conversions
// ---------------------------------------------------------------------------

#[test]
fn test_type_conversions() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Lossless widenings — always succeed
        assert_eq!(PrimitiveTypesContract::u32_to_u64(env.clone(), 100), 100);
        assert_eq!(PrimitiveTypesContract::i32_to_i64(env.clone(), 100), 100);

        // u64 → u32 narrowing
        assert_eq!(
            PrimitiveTypesContract::u64_to_u32(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::u64_to_u32(env.clone(), u32::MAX as u64),
            Ok(u32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::u64_to_u32(env.clone(), u32::MAX as u64 + 1),
            Err(ContractError::ConversionError)
        );

        // i64 → i32 narrowing
        assert_eq!(
            PrimitiveTypesContract::i64_to_i32(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MAX as i64),
            Ok(i32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MAX as i64 + 1),
            Err(ContractError::ConversionError)
        );
        assert_eq!(
            PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MIN as i64),
            Ok(i32::MIN)
        );
        assert_eq!(
            PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MIN as i64 - 1),
            Err(ContractError::ConversionError)
        );

        // u32 → i32 (loses top bit range)
        assert_eq!(
            PrimitiveTypesContract::u32_to_i32(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::u32_to_i32(env.clone(), i32::MAX as u32),
            Ok(i32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::u32_to_i32(env.clone(), i32::MAX as u32 + 1),
            Err(ContractError::ConversionError)
        );

        // i32 → u32 (negative values rejected)
        assert_eq!(
            PrimitiveTypesContract::i32_to_u32(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(PrimitiveTypesContract::i32_to_u32(env.clone(), 0), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::i32_to_u32(env.clone(), -1),
            Err(ContractError::NegativeValue)
        );

        // i64 → u64 (negative values rejected)
        assert_eq!(
            PrimitiveTypesContract::i64_to_u64(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(PrimitiveTypesContract::i64_to_u64(env.clone(), 0), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::i64_to_u64(env.clone(), -1),
            Err(ContractError::NegativeValue)
        );

        // u64 → i64 (values above i64::MAX rejected)
        assert_eq!(
            PrimitiveTypesContract::u64_to_i64(env.clone(), 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::u64_to_i64(env.clone(), i64::MAX as u64),
            Ok(i64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::u64_to_i64(env.clone(), i64::MAX as u64 + 1),
            Err(ContractError::ConversionError)
        );
    });
}

// ---------------------------------------------------------------------------
// Overflow Handling
// ---------------------------------------------------------------------------

#[test]
fn test_overflow_handling() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Checked (returns Err on overflow / underflow)
        assert_eq!(
            PrimitiveTypesContract::safe_add(env.clone(), 100, 200),
            Ok(300)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_add(env.clone(), u64::MAX - 1, 1),
            Ok(u64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_add(env.clone(), u64::MAX, 1),
            Err(ContractError::OverflowError)
        );

        assert_eq!(
            PrimitiveTypesContract::safe_sub(env.clone(), 200, 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_sub(env.clone(), 100, 100),
            Ok(0)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_sub(env.clone(), 0, 1),
            Err(ContractError::UnderflowError)
        );

        assert_eq!(
            PrimitiveTypesContract::safe_mul(env.clone(), 10, 20),
            Ok(200)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_mul(env.clone(), u64::MAX / 2, 2),
            Ok(u64::MAX - 1)
        );
        assert_eq!(
            PrimitiveTypesContract::safe_mul(env.clone(), u64::MAX / 2, 3),
            Err(ContractError::OverflowError)
        );

        // Saturating (clamps to numeric boundaries — never errors)
        assert_eq!(
            PrimitiveTypesContract::saturating_add(env.clone(), 100, 200),
            300
        );
        assert_eq!(
            PrimitiveTypesContract::saturating_add(env.clone(), u64::MAX, 1),
            u64::MAX
        );

        assert_eq!(
            PrimitiveTypesContract::saturating_sub(env.clone(), 200, 100),
            100
        );
        assert_eq!(PrimitiveTypesContract::saturating_sub(env.clone(), 0, 1), 0);

        assert_eq!(
            PrimitiveTypesContract::saturating_mul(env.clone(), 10, 20),
            200
        );
        assert_eq!(
            PrimitiveTypesContract::saturating_mul(env.clone(), u64::MAX, 2),
            u64::MAX
        );

        // Wrapping (modular arithmetic — never errors)
        assert_eq!(
            PrimitiveTypesContract::wrapping_add(env.clone(), u64::MAX - 1, 2),
            0
        );
        assert_eq!(
            PrimitiveTypesContract::wrapping_sub(env.clone(), 0, 1),
            u64::MAX
        );
        assert_eq!(
            PrimitiveTypesContract::wrapping_mul(env.clone(), u64::MAX, 2),
            18446744073709551614
        );
    });
}

// ---------------------------------------------------------------------------
// Financial Calculations
// ---------------------------------------------------------------------------

#[test]
fn test_financial_calculations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        PrimitiveTypesContract::initialize(env.clone()).unwrap();

        // Simple interest: principal * rate_bps * periods / 10_000
        assert_eq!(
            PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 500, 1),
            Ok(50) // 5% × 1 period
        );
        assert_eq!(
            PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 1000, 2),
            Ok(200) // 10% × 2 periods
        );
        assert_eq!(
            PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 10000, 1),
            Ok(1000) // 100% × 1 period
        );
        assert_eq!(
            PrimitiveTypesContract::calculate_interest(env.clone(), 1000, -1, 1),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(
            PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 10001, 1),
            Err(ContractError::InvalidInput)
        );

        // Compound interest
        assert_eq!(
            PrimitiveTypesContract::compound_interest(env.clone(), 1000, 1000, 1),
            Ok(100) // 100% for 1 period → interest = 100
        );
        assert_eq!(
            PrimitiveTypesContract::compound_interest(env.clone(), 1000, 500, 2),
            Ok(102) // ~5% compounded 2× → 102 (integer division)
        );

        // Balance management
        assert_eq!(PrimitiveTypesContract::deposit(env.clone(), 500), Ok(1500));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1500));
        assert_eq!(PrimitiveTypesContract::transfer(env.clone(), 200), Ok(1300));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1300));
        // Transferring more than balance is allowed (goes negative by design)
        assert_eq!(
            PrimitiveTypesContract::transfer(env.clone(), 2000),
            Ok(-700)
        );
        assert_eq!(
            PrimitiveTypesContract::transfer(env.clone(), -100),
            Err(ContractError::NegativeValue)
        );
        assert_eq!(
            PrimitiveTypesContract::deposit(env.clone(), -100),
            Err(ContractError::NegativeValue)
        );
    });
}

// ---------------------------------------------------------------------------
// Bit Operations
// ---------------------------------------------------------------------------

#[test]
fn test_bit_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Bitwise boolean operations
        assert_eq!(
            PrimitiveTypesContract::bitwise_and(env.clone(), 0b1010, 0b1100),
            0b1000
        );
        assert_eq!(
            PrimitiveTypesContract::bitwise_or(env.clone(), 0b1010, 0b1100),
            0b1110
        );
        assert_eq!(
            PrimitiveTypesContract::bitwise_xor(env.clone(), 0b1010, 0b1100),
            0b0110
        );
        assert_eq!(
            PrimitiveTypesContract::bitwise_not(env.clone(), 0b1010),
            !0b1010
        );

        // Shift operations
        assert_eq!(
            PrimitiveTypesContract::left_shift(env.clone(), 0b1010, 2),
            Ok(0b101000)
        );
        assert_eq!(
            PrimitiveTypesContract::right_shift(env.clone(), 0b1010, 2),
            Ok(0b0010)
        );
        assert_eq!(
            PrimitiveTypesContract::left_shift(env.clone(), 0b1010, 32),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(
            PrimitiveTypesContract::right_shift(env.clone(), 0b1010, 32),
            Err(ContractError::InvalidInput)
        );

        // Bit inspection and manipulation
        assert_eq!(
            PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 1),
            Ok(true)
        );
        assert_eq!(
            PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 2),
            Ok(false)
        );
        assert_eq!(
            PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 32),
            Err(ContractError::InvalidInput)
        );

        assert_eq!(
            PrimitiveTypesContract::set_bit(env.clone(), 0b1010, 2),
            Ok(0b1110)
        );
        assert_eq!(
            PrimitiveTypesContract::clear_bit(env.clone(), 0b1110, 1),
            Ok(0b1100)
        );
        assert_eq!(
            PrimitiveTypesContract::toggle_bit(env.clone(), 0b1010, 1),
            Ok(0b1000)
        );
    });
}

// ---------------------------------------------------------------------------
// Counter and Flag Management
// ---------------------------------------------------------------------------

#[test]
fn test_counter_and_flags() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        PrimitiveTypesContract::initialize(env.clone()).unwrap();

        // Counter increment / decrement with overflow guard
        assert_eq!(PrimitiveTypesContract::get_counter(env.clone()), Ok(0));
        assert_eq!(
            PrimitiveTypesContract::increment_counter(env.clone()),
            Ok(1)
        );
        assert_eq!(
            PrimitiveTypesContract::increment_counter(env.clone()),
            Ok(2)
        );
        assert_eq!(
            PrimitiveTypesContract::decrement_counter(env.clone()),
            Ok(1)
        );
        assert_eq!(
            PrimitiveTypesContract::decrement_counter(env.clone()),
            Ok(0)
        );
        assert_eq!(
            PrimitiveTypesContract::decrement_counter(env.clone()),
            Err(ContractError::UnderflowError)
        );

        // Bit-packed flags
        assert_eq!(PrimitiveTypesContract::set_flag(env.clone(), 0), Ok(()));
        assert_eq!(
            PrimitiveTypesContract::is_flag_set(env.clone(), 0),
            Ok(true)
        );
        assert_eq!(
            PrimitiveTypesContract::is_flag_set(env.clone(), 1),
            Ok(false)
        );
        assert_eq!(PrimitiveTypesContract::set_flag(env.clone(), 1), Ok(()));
        assert_eq!(
            PrimitiveTypesContract::is_flag_set(env.clone(), 1),
            Ok(true)
        );
        assert_eq!(PrimitiveTypesContract::clear_flag(env.clone(), 0), Ok(()));
        assert_eq!(
            PrimitiveTypesContract::is_flag_set(env.clone(), 0),
            Ok(false)
        );
        assert_eq!(
            PrimitiveTypesContract::is_flag_set(env.clone(), 32),
            Err(ContractError::InvalidInput)
        );
        assert_eq!(
            PrimitiveTypesContract::set_flag(env.clone(), 32),
            Err(ContractError::InvalidInput)
        );
    });
}

// ---------------------------------------------------------------------------
// Comparison and Range Operations
// ---------------------------------------------------------------------------

#[test]
fn test_comparisons() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Returns -1 / 0 / 1
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 10, 20), -1);
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 20, 10), 1);
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 10, 10), 0);

        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, 20), -1);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 20, 10), 1);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, 10), 0);
        assert_eq!(
            PrimitiveTypesContract::compare_i32(env.clone(), -10, 10),
            -1
        );
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, -10), 1);

        // Range checking
        assert!(PrimitiveTypesContract::is_in_range_u32(
            env.clone(),
            10,
            5,
            15
        ));
        assert!(!PrimitiveTypesContract::is_in_range_u32(
            env.clone(),
            4,
            5,
            15
        ));
        assert!(!PrimitiveTypesContract::is_in_range_u32(
            env.clone(),
            16,
            5,
            15
        ));

        assert!(PrimitiveTypesContract::is_in_range_i32(
            env.clone(),
            10,
            5,
            15
        ));
        assert!(PrimitiveTypesContract::is_in_range_i32(
            env.clone(),
            -10,
            -15,
            -5
        ));
        assert!(!PrimitiveTypesContract::is_in_range_i32(
            env.clone(),
            -16,
            -15,
            -5
        ));

        // Clamping
        assert_eq!(
            PrimitiveTypesContract::clamp_u32(env.clone(), 10, 5, 15),
            10
        );
        assert_eq!(PrimitiveTypesContract::clamp_u32(env.clone(), 4, 5, 15), 5);
        assert_eq!(
            PrimitiveTypesContract::clamp_u32(env.clone(), 16, 5, 15),
            15
        );

        assert_eq!(
            PrimitiveTypesContract::clamp_i32(env.clone(), 10, 5, 15),
            10
        );
        assert_eq!(PrimitiveTypesContract::clamp_i32(env.clone(), 4, 5, 15), 5);
        assert_eq!(
            PrimitiveTypesContract::clamp_i32(env.clone(), 16, 5, 15),
            15
        );
    });
}

// ---------------------------------------------------------------------------
// Storage Round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_storage_operations() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        assert_eq!(PrimitiveTypesContract::store_u32(env.clone(), 123), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_u32(env.clone()), Ok(123));

        assert_eq!(PrimitiveTypesContract::store_u64(env.clone(), 456), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_u64(env.clone()), Ok(456));

        assert_eq!(
            PrimitiveTypesContract::store_i32(env.clone(), -789),
            Ok(())
        );
        assert_eq!(PrimitiveTypesContract::retrieve_i32(env.clone()), Ok(-789));

        assert_eq!(
            PrimitiveTypesContract::store_i64(env.clone(), -101112),
            Ok(())
        );
        assert_eq!(
            PrimitiveTypesContract::retrieve_i64(env.clone()),
            Ok(-101112)
        );

        // Reset restores everything to 0 / false
        assert_eq!(
            PrimitiveTypesContract::reset_to_defaults(env.clone()),
            Ok(())
        );
        assert_eq!(PrimitiveTypesContract::retrieve_u32(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_u64(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_i32(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_i64(env.clone()), Ok(0));
    });
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialization() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        assert_eq!(PrimitiveTypesContract::initialize(env.clone()), Ok(()));

        // Verify the well-known defaults set by initialize()
        assert_eq!(
            PrimitiveTypesContract::retrieve_u32(env.clone()),
            Ok(u32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::retrieve_u64(env.clone()),
            Ok(u64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::retrieve_i32(env.clone()),
            Ok(i32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::retrieve_i64(env.clone()),
            Ok(i64::MAX)
        );
        assert_eq!(PrimitiveTypesContract::get_bool(env.clone()), Ok(true));
        assert_eq!(PrimitiveTypesContract::get_counter(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1000));
    });
}

// ---------------------------------------------------------------------------
// Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_edge_cases() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // Zero operands
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), 0, 0), Ok(0));
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), 0, 100), Ok(0));
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 0, 1), Ok(0));

        // Identity operands
        assert_eq!(
            PrimitiveTypesContract::mul_u32(env.clone(), 1, 100),
            Ok(100)
        );
        assert_eq!(
            PrimitiveTypesContract::div_u32(env.clone(), 100, 1),
            Ok(100)
        );

        // One step below overflow
        assert_eq!(
            PrimitiveTypesContract::add_u32(env.clone(), u32::MAX - 1, 1),
            Ok(u32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_u64(env.clone(), u64::MAX - 1, 1),
            Ok(u64::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_i32(env.clone(), i32::MAX - 1, 1),
            Ok(i32::MAX)
        );
        assert_eq!(
            PrimitiveTypesContract::add_i64(env.clone(), i64::MAX - 1, 1),
            Ok(i64::MAX)
        );

        // One step above the minimum for signed types
        assert_eq!(
            PrimitiveTypesContract::add_i32(env.clone(), i32::MIN + 1, -1),
            Ok(i32::MIN)
        );
        assert_eq!(
            PrimitiveTypesContract::add_i64(env.clone(), i64::MIN + 1, -1),
            Ok(i64::MIN)
        );
    });
}
