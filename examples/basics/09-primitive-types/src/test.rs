#![cfg(test)]
use super::*;
use soroban_sdk::Env;

#[test]
fn test_u32_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test addition
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), 10, 20), Ok(30));
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), u32::MAX - 1, 1), Ok(u32::MAX));
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), u32::MAX, 1), Err(ContractError::OverflowError));

        // Test subtraction
        assert_eq!(PrimitiveTypesContract::sub_u32(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_u32(env.clone(), 10, 10), Ok(0));
        assert_eq!(PrimitiveTypesContract::sub_u32(env.clone(), 0, 1), Err(ContractError::UnderflowError));

        // Test multiplication
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), 5, 6), Ok(30));
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), u32::MAX / 2, 2), Ok(u32::MAX - 1));
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), u32::MAX / 2, 3), Err(ContractError::OverflowError));

        // Test division
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 20, 5), Ok(4));
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 20, 0), Err(ContractError::DivisionByZero));
    });
}

#[test]
fn test_u64_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test addition
        assert_eq!(PrimitiveTypesContract::add_u64(env.clone(), 10, 20), Ok(30));
        assert_eq!(PrimitiveTypesContract::add_u64(env.clone(), u64::MAX - 1, 1), Ok(u64::MAX));
        assert_eq!(PrimitiveTypesContract::add_u64(env.clone(), u64::MAX, 1), Err(ContractError::OverflowError));

        // Test subtraction
        assert_eq!(PrimitiveTypesContract::sub_u64(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_u64(env.clone(), 10, 10), Ok(0));
        assert_eq!(PrimitiveTypesContract::sub_u64(env.clone(), 0, 1), Err(ContractError::UnderflowError));

        // Test multiplication
        assert_eq!(PrimitiveTypesContract::mul_u64(env.clone(), 5, 6), Ok(30));
        assert_eq!(PrimitiveTypesContract::mul_u64(env.clone(), u64::MAX / 2, 2), Ok(u64::MAX - 1));
        assert_eq!(PrimitiveTypesContract::mul_u64(env.clone(), u64::MAX / 2, 3), Err(ContractError::OverflowError));

        // Test division
        assert_eq!(PrimitiveTypesContract::div_u64(env.clone(), 20, 5), Ok(4));
        assert_eq!(PrimitiveTypesContract::div_u64(env.clone(), 20, 0), Err(ContractError::DivisionByZero));
    });
}

#[test]
fn test_i32_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test addition
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), 10, 20), Ok(30));
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), i32::MAX - 1, 1), Ok(i32::MAX));
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), i32::MAX, 1), Err(ContractError::OverflowError));

        // Test subtraction
        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), 10, 10), Ok(0));
        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), i32::MIN + 1, 2), Err(ContractError::OverflowError));
        assert_eq!(PrimitiveTypesContract::sub_i32(env.clone(), i32::MIN, 1), Err(ContractError::OverflowError));

        // Test multiplication
        assert_eq!(PrimitiveTypesContract::mul_i32(env.clone(), 5, 6), Ok(30));
        assert_eq!(PrimitiveTypesContract::mul_i32(env.clone(), i32::MAX / 2, 2), Ok(i32::MAX - 1));
        assert_eq!(PrimitiveTypesContract::mul_i32(env.clone(), i32::MAX / 2, 3), Err(ContractError::OverflowError));

        // Test division
        assert_eq!(PrimitiveTypesContract::div_i32(env.clone(), 20, 5), Ok(4));
        assert_eq!(PrimitiveTypesContract::div_i32(env.clone(), 20, 0), Err(ContractError::DivisionByZero));
        assert_eq!(PrimitiveTypesContract::div_i32(env.clone(), -20, 5), Ok(-4));
    });
}

#[test]
fn test_i64_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test addition
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), 10, 20), Ok(30));
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), i64::MAX - 1, 1), Ok(i64::MAX));
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), i64::MAX, 1), Err(ContractError::OverflowError));

        // Test subtraction
        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), 20, 10), Ok(10));
        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), 10, 10), Ok(0));
        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), i64::MIN + 1, 2), Err(ContractError::OverflowError));
        assert_eq!(PrimitiveTypesContract::sub_i64(env.clone(), i64::MIN, 1), Err(ContractError::OverflowError));

        // Test multiplication
        assert_eq!(PrimitiveTypesContract::mul_i64(env.clone(), 5, 6), Ok(30));
        assert_eq!(PrimitiveTypesContract::mul_i64(env.clone(), i64::MAX / 2, 2), Ok(i64::MAX - 1));
        assert_eq!(PrimitiveTypesContract::mul_i64(env.clone(), i64::MAX / 2, 3), Err(ContractError::OverflowError));

        // Test division
        assert_eq!(PrimitiveTypesContract::div_i64(env.clone(), 20, 5), Ok(4));
        assert_eq!(PrimitiveTypesContract::div_i64(env.clone(), 20, 0), Err(ContractError::DivisionByZero));
        assert_eq!(PrimitiveTypesContract::div_i64(env.clone(), -20, 5), Ok(-4));
    });
}

#[test]
fn test_boolean_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test logical operations
        assert_eq!(PrimitiveTypesContract::bool_and(env.clone(), true, true), true);
        assert_eq!(PrimitiveTypesContract::bool_and(env.clone(), true, false), false);
        assert_eq!(PrimitiveTypesContract::bool_and(env.clone(), false, false), false);

        assert_eq!(PrimitiveTypesContract::bool_or(env.clone(), true, false), true);
        assert_eq!(PrimitiveTypesContract::bool_or(env.clone(), false, false), false);
        assert_eq!(PrimitiveTypesContract::bool_or(env.clone(), true, true), true);

        assert_eq!(PrimitiveTypesContract::bool_not(env.clone(), true), false);
        assert_eq!(PrimitiveTypesContract::bool_not(env.clone(), false), true);

        assert_eq!(PrimitiveTypesContract::bool_xor(env.clone(), true, false), true);
        assert_eq!(PrimitiveTypesContract::bool_xor(env.clone(), true, true), false);
        assert_eq!(PrimitiveTypesContract::bool_xor(env.clone(), false, false), false);

        // Test storage
        assert_eq!(PrimitiveTypesContract::set_bool(env.clone(), true), Ok(()));
        assert_eq!(PrimitiveTypesContract::get_bool(env.clone()), Ok(true));
        assert_eq!(PrimitiveTypesContract::set_bool(env.clone(), false), Ok(()));
        assert_eq!(PrimitiveTypesContract::get_bool(env.clone()), Ok(false));
    });
}

#[test]
fn test_type_conversions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test safe conversions (always succeed)
        assert_eq!(PrimitiveTypesContract::u32_to_u64(env.clone(), 100), 100);
        assert_eq!(PrimitiveTypesContract::i32_to_i64(env.clone(), 100), 100);

        // Test u64 to u32 conversion
        assert_eq!(PrimitiveTypesContract::u64_to_u32(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::u64_to_u32(env.clone(), u32::MAX as u64), Ok(u32::MAX));
        assert_eq!(PrimitiveTypesContract::u64_to_u32(env.clone(), u32::MAX as u64 + 1), Err(ContractError::ConversionError));

        // Test i64 to i32 conversion
        assert_eq!(PrimitiveTypesContract::i64_to_i32(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MAX as i64), Ok(i32::MAX));
        assert_eq!(PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MAX as i64 + 1), Err(ContractError::ConversionError));
        assert_eq!(PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MIN as i64), Ok(i32::MIN));
        assert_eq!(PrimitiveTypesContract::i64_to_i32(env.clone(), i32::MIN as i64 - 1), Err(ContractError::ConversionError));

        // Test u32 to i32 conversion
        assert_eq!(PrimitiveTypesContract::u32_to_i32(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::u32_to_i32(env.clone(), i32::MAX as u32), Ok(i32::MAX));
        assert_eq!(PrimitiveTypesContract::u32_to_i32(env.clone(), i32::MAX as u32 + 1), Err(ContractError::ConversionError));

        // Test i32 to u32 conversion
        assert_eq!(PrimitiveTypesContract::i32_to_u32(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::i32_to_u32(env.clone(), 0), Ok(0));
        assert_eq!(PrimitiveTypesContract::i32_to_u32(env.clone(), -1), Err(ContractError::NegativeValue));

        // Test i64 to u64 conversion
        assert_eq!(PrimitiveTypesContract::i64_to_u64(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::i64_to_u64(env.clone(), 0), Ok(0));
        assert_eq!(PrimitiveTypesContract::i64_to_u64(env.clone(), -1), Err(ContractError::NegativeValue));

        // Test u64 to i64 conversion
        assert_eq!(PrimitiveTypesContract::u64_to_i64(env.clone(), 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::u64_to_i64(env.clone(), i64::MAX as u64), Ok(i64::MAX));
        assert_eq!(PrimitiveTypesContract::u64_to_i64(env.clone(), i64::MAX as u64 + 1), Err(ContractError::ConversionError));
    });
}

#[test]
fn test_overflow_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test safe operations
        assert_eq!(PrimitiveTypesContract::safe_add(env.clone(), 100, 200), Ok(300));
        assert_eq!(PrimitiveTypesContract::safe_add(env.clone(), u64::MAX - 1, 1), Ok(u64::MAX));
        assert_eq!(PrimitiveTypesContract::safe_add(env.clone(), u64::MAX, 1), Err(ContractError::OverflowError));

        assert_eq!(PrimitiveTypesContract::safe_sub(env.clone(), 200, 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::safe_sub(env.clone(), 100, 100), Ok(0));
        assert_eq!(PrimitiveTypesContract::safe_sub(env.clone(), 0, 1), Err(ContractError::UnderflowError));

        assert_eq!(PrimitiveTypesContract::safe_mul(env.clone(), 10, 20), Ok(200));
        assert_eq!(PrimitiveTypesContract::safe_mul(env.clone(), u64::MAX / 2, 2), Ok(u64::MAX - 1));
        assert_eq!(PrimitiveTypesContract::safe_mul(env.clone(), u64::MAX / 2, 3), Err(ContractError::OverflowError));

        // Test saturating operations
        assert_eq!(PrimitiveTypesContract::saturating_add(env.clone(), 100, 200), 300);
        assert_eq!(PrimitiveTypesContract::saturating_add(env.clone(), u64::MAX, 1), u64::MAX);

        assert_eq!(PrimitiveTypesContract::saturating_sub(env.clone(), 200, 100), 100);
        assert_eq!(PrimitiveTypesContract::saturating_sub(env.clone(), 0, 1), 0);

        assert_eq!(PrimitiveTypesContract::saturating_mul(env.clone(), 10, 20), 200);
        assert_eq!(PrimitiveTypesContract::saturating_mul(env.clone(), u64::MAX, 2), u64::MAX);

        // Test wrapping operations
        assert_eq!(PrimitiveTypesContract::wrapping_add(env.clone(), u64::MAX - 1, 2), 0);
        assert_eq!(PrimitiveTypesContract::wrapping_sub(env.clone(), 0, 1), u64::MAX);
        assert_eq!(PrimitiveTypesContract::wrapping_mul(env.clone(), u64::MAX, 2), 18446744073709551614);
    });
}

#[test]
fn test_financial_calculations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Initialize contract first
        PrimitiveTypesContract::initialize(env.clone()).unwrap();
        
        // Test simple interest
        assert_eq!(PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 500, 1), Ok(50)); // 5% for 1 period
        assert_eq!(PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 1000, 2), Ok(200)); // 10% for 2 periods
        assert_eq!(PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 10000, 1), Ok(1000)); // 100% for 1 period
        assert_eq!(PrimitiveTypesContract::calculate_interest(env.clone(), 1000, -1, 1), Err(ContractError::InvalidInput));
        assert_eq!(PrimitiveTypesContract::calculate_interest(env.clone(), 1000, 10001, 1), Err(ContractError::InvalidInput));

        // Test compound interest
        assert_eq!(PrimitiveTypesContract::compound_interest(env.clone(), 1000, 1000, 1), Ok(100)); // 100% for 1 period (interest earned)
        assert_eq!(PrimitiveTypesContract::compound_interest(env.clone(), 1000, 500, 2), Ok(102)); // 50% compounded for 2 periods (interest earned)

        // Test transfer operations
        assert_eq!(PrimitiveTypesContract::deposit(env.clone(), 500), Ok(1500));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1500));
        assert_eq!(PrimitiveTypesContract::transfer(env.clone(), 200), Ok(1300));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1300));
        assert_eq!(PrimitiveTypesContract::transfer(env.clone(), 2000), Ok(-700)); // Transfer succeeds, balance goes negative
        assert_eq!(PrimitiveTypesContract::transfer(env.clone(), -100), Err(ContractError::NegativeValue));
        assert_eq!(PrimitiveTypesContract::deposit(env.clone(), -100), Err(ContractError::NegativeValue));
    });
}

#[test]
fn test_bit_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test bitwise operations
        assert_eq!(PrimitiveTypesContract::bitwise_and(env.clone(), 0b1010, 0b1100), 0b1000);
        assert_eq!(PrimitiveTypesContract::bitwise_or(env.clone(), 0b1010, 0b1100), 0b1110);
        assert_eq!(PrimitiveTypesContract::bitwise_xor(env.clone(), 0b1010, 0b1100), 0b0110);
        assert_eq!(PrimitiveTypesContract::bitwise_not(env.clone(), 0b1010), !0b1010);

        // Test shift operations
        assert_eq!(PrimitiveTypesContract::left_shift(env.clone(), 0b1010, 2), Ok(0b101000));
        assert_eq!(PrimitiveTypesContract::right_shift(env.clone(), 0b1010, 2), Ok(0b0010));
        assert_eq!(PrimitiveTypesContract::left_shift(env.clone(), 0b1010, 32), Err(ContractError::InvalidInput));
        assert_eq!(PrimitiveTypesContract::right_shift(env.clone(), 0b1010, 32), Err(ContractError::InvalidInput));

        // Test bit manipulation
        assert_eq!(PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 1), Ok(true));
        assert_eq!(PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 2), Ok(false));
        assert_eq!(PrimitiveTypesContract::is_bit_set(env.clone(), 0b1010, 32), Err(ContractError::InvalidInput));

        assert_eq!(PrimitiveTypesContract::set_bit(env.clone(), 0b1010, 2), Ok(0b1110));
        assert_eq!(PrimitiveTypesContract::clear_bit(env.clone(), 0b1110, 1), Ok(0b1100));
        assert_eq!(PrimitiveTypesContract::toggle_bit(env.clone(), 0b1010, 1), Ok(0b1000));
    });
}

#[test]
fn test_counter_and_flags() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Initialize contract first
        PrimitiveTypesContract::initialize(env.clone()).unwrap();
        
        // Test counter operations
        assert_eq!(PrimitiveTypesContract::get_counter(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::increment_counter(env.clone()), Ok(1));
        assert_eq!(PrimitiveTypesContract::increment_counter(env.clone()), Ok(2));
        assert_eq!(PrimitiveTypesContract::decrement_counter(env.clone()), Ok(1));
        assert_eq!(PrimitiveTypesContract::decrement_counter(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::decrement_counter(env.clone()), Err(ContractError::UnderflowError));

        // Test flag operations
        assert_eq!(PrimitiveTypesContract::set_flag(env.clone(), 0), Ok(()));
        assert_eq!(PrimitiveTypesContract::is_flag_set(env.clone(), 0), Ok(true));
        assert_eq!(PrimitiveTypesContract::is_flag_set(env.clone(), 1), Ok(false));
        assert_eq!(PrimitiveTypesContract::set_flag(env.clone(), 1), Ok(()));
        assert_eq!(PrimitiveTypesContract::is_flag_set(env.clone(), 1), Ok(true));
        assert_eq!(PrimitiveTypesContract::clear_flag(env.clone(), 0), Ok(()));
        assert_eq!(PrimitiveTypesContract::is_flag_set(env.clone(), 0), Ok(false));
        assert_eq!(PrimitiveTypesContract::is_flag_set(env.clone(), 32), Err(ContractError::InvalidInput));
        assert_eq!(PrimitiveTypesContract::set_flag(env.clone(), 32), Err(ContractError::InvalidInput));
    });
}

#[test]
fn test_comparisons() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test comparisons
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 10, 20), -1);
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 20, 10), 1);
        assert_eq!(PrimitiveTypesContract::compare_u32(env.clone(), 10, 10), 0);

        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, 20), -1);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 20, 10), 1);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, 10), 0);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), -10, 10), -1);
        assert_eq!(PrimitiveTypesContract::compare_i32(env.clone(), 10, -10), 1);

        // Test range checking
        assert_eq!(PrimitiveTypesContract::is_in_range_u32(env.clone(), 10, 5, 15), true);
        assert_eq!(PrimitiveTypesContract::is_in_range_u32(env.clone(), 4, 5, 15), false);
        assert_eq!(PrimitiveTypesContract::is_in_range_u32(env.clone(), 16, 5, 15), false);

        assert_eq!(PrimitiveTypesContract::is_in_range_i32(env.clone(), 10, 5, 15), true);
        assert_eq!(PrimitiveTypesContract::is_in_range_i32(env.clone(), -10, -15, -5), true);
        assert_eq!(PrimitiveTypesContract::is_in_range_i32(env.clone(), -16, -15, -5), false);

        // Test clamping
        assert_eq!(PrimitiveTypesContract::clamp_u32(env.clone(), 10, 5, 15), 10);
        assert_eq!(PrimitiveTypesContract::clamp_u32(env.clone(), 4, 5, 15), 5);
        assert_eq!(PrimitiveTypesContract::clamp_u32(env.clone(), 16, 5, 15), 15);

        assert_eq!(PrimitiveTypesContract::clamp_i32(env.clone(), 10, 5, 15), 10);
        assert_eq!(PrimitiveTypesContract::clamp_i32(env.clone(), 4, 5, 15), 5);
        assert_eq!(PrimitiveTypesContract::clamp_i32(env.clone(), 16, 5, 15), 15);
    });
}

#[test]
fn test_storage_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test storage and retrieval
        assert_eq!(PrimitiveTypesContract::store_u32(env.clone(), 123), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_u32(env.clone()), Ok(123));

        assert_eq!(PrimitiveTypesContract::store_u64(env.clone(), 456), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_u64(env.clone()), Ok(456));

        assert_eq!(PrimitiveTypesContract::store_i32(env.clone(), -789), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_i32(env.clone()), Ok(-789));

        assert_eq!(PrimitiveTypesContract::store_i64(env.clone(), -101112), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_i64(env.clone()), Ok(-101112));

        // Test reset
        assert_eq!(PrimitiveTypesContract::reset_to_defaults(env.clone()), Ok(()));
        assert_eq!(PrimitiveTypesContract::retrieve_u32(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_u64(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_i32(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::retrieve_i64(env.clone()), Ok(0));
    });
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test initialization
        assert_eq!(PrimitiveTypesContract::initialize(env.clone()), Ok(()));
        
        // Verify initialized values
        assert_eq!(PrimitiveTypesContract::retrieve_u32(env.clone()), Ok(u32::MAX));
        assert_eq!(PrimitiveTypesContract::retrieve_u64(env.clone()), Ok(u64::MAX));
        assert_eq!(PrimitiveTypesContract::retrieve_i32(env.clone()), Ok(i32::MAX));
        assert_eq!(PrimitiveTypesContract::retrieve_i64(env.clone()), Ok(i64::MAX));
        assert_eq!(PrimitiveTypesContract::get_bool(env.clone()), Ok(true));
        assert_eq!(PrimitiveTypesContract::get_counter(env.clone()), Ok(0));
        assert_eq!(PrimitiveTypesContract::get_balance(env.clone()), Ok(1000));
    });
}

#[test]
fn test_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);

    env.as_contract(&contract_id, || {
        // Test zero values
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), 0, 0), Ok(0));
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), 0, 100), Ok(0));
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 0, 1), Ok(0));

        // Test one values
        assert_eq!(PrimitiveTypesContract::mul_u32(env.clone(), 1, 100), Ok(100));
        assert_eq!(PrimitiveTypesContract::div_u32(env.clone(), 100, 1), Ok(100));

        // Test maximum values
        assert_eq!(PrimitiveTypesContract::add_u32(env.clone(), u32::MAX - 1, 1), Ok(u32::MAX));
        assert_eq!(PrimitiveTypesContract::add_u64(env.clone(), u64::MAX - 1, 1), Ok(u64::MAX));
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), i32::MAX - 1, 1), Ok(i32::MAX));
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), i64::MAX - 1, 1), Ok(i64::MAX));

        // Test minimum values
        assert_eq!(PrimitiveTypesContract::add_i32(env.clone(), i32::MIN + 1, -1), Ok(i32::MIN));
        assert_eq!(PrimitiveTypesContract::add_i64(env.clone(), i64::MIN + 1, -1), Ok(i64::MIN));
    });
}
