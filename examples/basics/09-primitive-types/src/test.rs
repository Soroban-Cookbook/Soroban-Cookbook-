//! Tests for the Primitive Types contract.
//!
//! Each test uses the auto-generated `PrimitiveTypesContractClient`, which
//! is the idiomatic pattern across the cookbook (see also `06-soroban-types`).
//! The `#[should_panic]` attribute is used for error paths because the client
//! converts host errors into Rust panics.

use super::*;
use soroban_sdk::Env;

// Section 1 — Unsigned Integer Operations (u32)

#[test]
fn test_u32_add_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.add_u32(&10, &20), 30u32);
    // One away from the boundary — must succeed.
    assert_eq!(client.add_u32(&(u32::MAX - 1), &1), u32::MAX);
}

#[test]
#[should_panic] // OverflowError from the host
fn test_u32_add_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.add_u32(&u32::MAX, &1);
}

#[test]
fn test_u32_sub_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.sub_u32(&20, &10), 10u32);
    assert_eq!(client.sub_u32(&10, &10), 0u32);
}

#[test]
#[should_panic] // UnderflowError — unsigned cannot go below 0
fn test_u32_sub_underflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.sub_u32(&0, &1);
}

#[test]
fn test_u32_mul_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.mul_u32(&5, &6), 30u32);
    assert_eq!(client.mul_u32(&0, &100), 0u32);
    assert_eq!(client.mul_u32(&1, &100), 100u32);
}

#[test]
fn test_u32_div_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.div_u32(&20, &5), 4u32);
    assert_eq!(client.div_u32(&0, &1), 0u32);
    assert_eq!(client.div_u32(&100, &1), 100u32);
}

#[test]
#[should_panic] // DivisionByZero
fn test_u32_div_by_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.div_u32(&20, &0);
}

// Section 1 — Unsigned Integer Operations (u64)

#[test]
fn test_u64_add_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.add_u64(&10, &20), 30u64);
    assert_eq!(client.add_u64(&(u64::MAX - 1), &1), u64::MAX);
}

#[test]
#[should_panic] // OverflowError
fn test_u64_add_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.add_u64(&u64::MAX, &1);
}

#[test]
fn test_u64_mul_and_div() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.mul_u64(&5, &6), 30u64);
    assert_eq!(client.div_u64(&20, &5), 4u64);
}

#[test]
#[should_panic] // DivisionByZero
fn test_u64_div_by_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.div_u64(&20, &0);
}

// Section 1 — Signed Integer Operations (i32)

#[test]
fn test_i32_add_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.add_i32(&10, &20), 30i32);
    // Signed — negative operands are valid.
    assert_eq!(client.add_i32(&-10, &20), 10i32);
    assert_eq!(client.add_i32(&(i32::MAX - 1), &1), i32::MAX);
}

#[test]
#[should_panic] // OverflowError at positive boundary
fn test_i32_add_positive_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.add_i32(&i32::MAX, &1);
}

#[test]
#[should_panic] // OverflowError at negative boundary
fn test_i32_sub_negative_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.sub_i32(&i32::MIN, &1);
}

#[test]
fn test_i32_mul_and_div() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.mul_i32(&5, &6), 30i32);
    assert_eq!(client.div_i32(&-20, &5), -4i32);
}

#[test]
#[should_panic] // DivisionByZero
fn test_i32_div_by_zero() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.div_i32(&20, &0);
}

// Section 1 — Signed Integer Operations (i64)

#[test]
fn test_i64_add_normal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.add_i64(&10, &20), 30i64);
    assert_eq!(client.add_i64(&-10, &20), 10i64);
    assert_eq!(client.add_i64(&(i64::MAX - 1), &1), i64::MAX);
}

#[test]
#[should_panic]
fn test_i64_add_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.add_i64(&i64::MAX, &1);
}

#[test]
fn test_i64_mul_and_div() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.mul_i64(&5, &6), 30i64);
    assert_eq!(client.div_i64(&-20, &5), -4i64);
}

// Section 2 — Boolean Operations

#[test]
fn test_bool_logic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // AND
    assert!(client.bool_and(&true, &true));
    assert!(!client.bool_and(&true, &false));
    assert!(!client.bool_and(&false, &false));

    // OR
    assert!(client.bool_or(&true, &false));
    assert!(!client.bool_or(&false, &false));

    // NOT
    assert!(!client.bool_not(&true));
    assert!(client.bool_not(&false));

    // XOR — true when exactly one input is true
    assert!(client.bool_xor(&true, &false));
    assert!(!client.bool_xor(&true, &true));
    assert!(!client.bool_xor(&false, &false));
}

#[test]
fn test_bool_storage_round_trip() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.set_bool(&true);
    assert!(client.get_bool());

    client.set_bool(&false);
    assert!(!client.get_bool());
}

// Section 3 — Type Conversions

#[test]
fn test_widening_conversions_always_succeed() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // u32 → u64 (zero-extend): every u32 value fits
    assert_eq!(client.u32_to_u64(&0u32), 0u64);
    assert_eq!(client.u32_to_u64(&u32::MAX), u32::MAX as u64);

    // i32 → i64 (sign-extend): every i32 value fits
    assert_eq!(client.i32_to_i64(&i32::MIN), i32::MIN as i64);
    assert_eq!(client.i32_to_i64(&i32::MAX), i32::MAX as i64);
}

#[test]
fn test_narrowing_u64_to_u32_succeeds_in_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.u64_to_u32(&0u64), 0u32);
    assert_eq!(client.u64_to_u32(&(u32::MAX as u64)), u32::MAX);
}

#[test]
#[should_panic] // ConversionError — value > u32::MAX
fn test_narrowing_u64_to_u32_out_of_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.u64_to_u32(&(u32::MAX as u64 + 1));
}

#[test]
fn test_narrowing_i64_to_i32_succeeds_in_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.i64_to_i32(&0i64), 0i32);
    assert_eq!(client.i64_to_i32(&(i32::MAX as i64)), i32::MAX);
    assert_eq!(client.i64_to_i32(&(i32::MIN as i64)), i32::MIN);
}

#[test]
#[should_panic] // ConversionError
fn test_narrowing_i64_to_i32_out_of_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.i64_to_i32(&(i32::MAX as i64 + 1));
}

#[test]
fn test_sign_change_u32_to_i32_succeeds() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.u32_to_i32(&0u32), 0i32);
    assert_eq!(client.u32_to_i32(&(i32::MAX as u32)), i32::MAX);
}

#[test]
#[should_panic] // ConversionError — exceeds i32::MAX
fn test_sign_change_u32_to_i32_out_of_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.u32_to_i32(&(i32::MAX as u32 + 1));
}

#[test]
fn test_sign_change_i32_to_u32_positive() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.i32_to_u32(&0i32), 0u32);
    assert_eq!(client.i32_to_u32(&100i32), 100u32);
}

#[test]
#[should_panic] // NegativeValue — negative has no unsigned representation
fn test_sign_change_i32_to_u32_negative() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.i32_to_u32(&-1i32);
}

#[test]
fn test_u64_i64_round_trips() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.u64_to_i64(&0u64), 0i64);
    assert_eq!(client.u64_to_i64(&(i64::MAX as u64)), i64::MAX);

    assert_eq!(client.i64_to_u64(&0i64), 0u64);
    assert_eq!(client.i64_to_u64(&i64::MAX), i64::MAX as u64);
}

#[test]
#[should_panic] // NegativeValue
fn test_i64_to_u64_negative() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.i64_to_u64(&-1i64);
}

// Section 4 — Overflow Handling

#[test]
fn test_checked_arithmetic_happy_path() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.safe_add(&100, &200), 300u64);
    assert_eq!(client.safe_sub(&200, &100), 100u64);
    assert_eq!(client.safe_mul(&10, &20), 200u64);
}

#[test]
#[should_panic] // OverflowError
fn test_checked_add_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.safe_add(&u64::MAX, &1);
}

#[test]
#[should_panic] // UnderflowError
fn test_checked_sub_underflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.safe_sub(&0, &1);
}

#[test]
#[should_panic] // OverflowError
fn test_checked_mul_overflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.safe_mul(&u64::MAX, &2);
}

#[test]
fn test_saturating_arithmetic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // Normal values behave like ordinary arithmetic.
    assert_eq!(client.saturating_add(&100, &200), 300u64);
    assert_eq!(client.saturating_sub(&200, &100), 100u64);
    assert_eq!(client.saturating_mul(&10, &20), 200u64);

    // Overflow → clamped to MAX (no panic, no error).
    assert_eq!(client.saturating_add(&u64::MAX, &1), u64::MAX);
    assert_eq!(client.saturating_mul(&u64::MAX, &2), u64::MAX);

    // Underflow → clamped to 0.
    assert_eq!(client.saturating_sub(&0, &1), 0u64);
}

#[test]
fn test_wrapping_arithmetic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // (u64::MAX - 1) + 2 = u64::MAX + 1, which wraps modulo 2^64 → 0.
    assert_eq!(client.wrapping_add(&(u64::MAX - 1), &2), 0u64);

    // 0 - 1 wraps to MAX.
    assert_eq!(client.wrapping_sub(&0, &1), u64::MAX);

    // MAX * 2 ≡ MAX - 1 (mod 2^64) because MAX = 2^64 - 1,
    // so (2^64 - 1) * 2 = 2^65 - 2 ≡ -2 ≡ 2^64 - 2 = MAX - 1.
    assert_eq!(
        client.wrapping_mul(&u64::MAX, &2),
        18_446_744_073_709_551_614u64
    );
}

// Section 5a — Financial Calculations (i128)

#[test]
fn test_simple_interest_calculations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // 5 % (500 bps) × 1 period × 1 000 principal = 50
    assert_eq!(client.calculate_interest(&1000, &500, &1), 50i128);
    // 10 % × 2 periods × 1 000 = 200
    assert_eq!(client.calculate_interest(&1000, &1000, &2), 200i128);
    // 100 % × 1 period × 1 000 = 1 000
    assert_eq!(client.calculate_interest(&1000, &10000, &1), 1000i128);
}

#[test]
#[should_panic] // InvalidInput — rate must be in [0, 10_000]
fn test_simple_interest_invalid_rate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.calculate_interest(&1000, &-1, &1);
}

#[test]
fn test_compound_interest() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // 10 % (1 000 bps) compounded 1 period: 1 000 * 1.10 − 1 000 = 100
    assert_eq!(client.compound_interest(&1000, &1000, &1), 100i128);
    // 5 % (500 bps) compounded 2 periods: ≈102 after integer division
    assert_eq!(client.compound_interest(&1000, &500, &2), 102i128);
}

#[test]
fn test_deposit_and_transfer_with_initialized_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.initialize(); // seeds balance at 1 000

    assert_eq!(client.deposit(&500), 1500i128);
    assert_eq!(client.get_balance(), 1500i128);
    assert_eq!(client.transfer(&200), 1300i128);
    assert_eq!(client.get_balance(), 1300i128);
}

#[test]
#[should_panic] // NegativeValue
fn test_deposit_negative_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);
    client.initialize();
    client.deposit(&-100);
}

// Section 5b — Bit Operations

#[test]
fn test_bitwise_logic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.bitwise_and(&0b1010, &0b1100), 0b1000u32);
    assert_eq!(client.bitwise_or(&0b1010, &0b1100), 0b1110u32);
    assert_eq!(client.bitwise_xor(&0b1010, &0b1100), 0b0110u32);
    assert_eq!(client.bitwise_not(&0b1010u32), !0b1010u32);
}

#[test]
fn test_bit_shifts() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.left_shift(&0b1010, &2), 0b101000u32);
    assert_eq!(client.right_shift(&0b1010, &2), 0b0010u32);
}

#[test]
#[should_panic] // InvalidInput — shift ≥ 32 is undefined behaviour
fn test_left_shift_out_of_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.left_shift(&0b1010, &32);
}

#[test]
fn test_bit_manipulation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // 0b1010 has bit 1 set, bit 2 clear.
    assert!(client.is_bit_set(&0b1010, &1));
    assert!(!client.is_bit_set(&0b1010, &2));

    assert_eq!(client.set_bit(&0b1010, &2), 0b1110u32);
    assert_eq!(client.clear_bit(&0b1110, &1), 0b1100u32);
    assert_eq!(client.toggle_bit(&0b1010, &1), 0b1000u32);
}

// Section 5c — Counter and Flag Management

#[test]
fn test_counter_increment_decrement() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);
    client.initialize(); // seeds counter at 0

    assert_eq!(client.get_counter(), 0u64);
    assert_eq!(client.increment_counter(), 1u64);
    assert_eq!(client.increment_counter(), 2u64);
    assert_eq!(client.decrement_counter(), 1u64);
    assert_eq!(client.decrement_counter(), 0u64);
}

#[test]
#[should_panic] // UnderflowError — counter is already 0
fn test_counter_underflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);
    client.initialize();
    client.decrement_counter();
}

#[test]
fn test_flag_set_clear_and_check() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);
    client.initialize(); // seeds flags at 0

    client.set_flag(&0);
    assert!(client.is_flag_set(&0));
    assert!(!client.is_flag_set(&1));

    client.set_flag(&1);
    assert!(client.is_flag_set(&1));

    client.clear_flag(&0);
    assert!(!client.is_flag_set(&0));
    assert!(client.is_flag_set(&1)); // bit 1 unchanged
}

#[test]
#[should_panic] // InvalidInput — only bits 0-31 are valid
fn test_flag_out_of_range() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);
    client.initialize();
    client.is_flag_set(&32);
}

// Section 5d — Comparisons and Clamping

#[test]
fn test_u32_three_way_compare() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.compare_u32(&10, &20), -1i32);
    assert_eq!(client.compare_u32(&20, &10), 1i32);
    assert_eq!(client.compare_u32(&10, &10), 0i32);
}

#[test]
fn test_i32_three_way_compare_with_negatives() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.compare_i32(&-10, &10), -1i32);
    assert_eq!(client.compare_i32(&10, &-10), 1i32);
    assert_eq!(client.compare_i32(&-5, &-5), 0i32);
}

#[test]
fn test_range_check() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert!(client.is_in_range_u32(&10, &5, &15));
    assert!(!client.is_in_range_u32(&4, &5, &15)); // below min
    assert!(!client.is_in_range_u32(&16, &5, &15)); // above max

    assert!(client.is_in_range_i32(&-10, &-15, &-5));
    assert!(!client.is_in_range_i32(&-16, &-15, &-5));
}

#[test]
fn test_clamp() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.clamp_u32(&4, &5, &15), 5u32); // below min
    assert_eq!(client.clamp_u32(&10, &5, &15), 10u32); // in range
    assert_eq!(client.clamp_u32(&16, &5, &15), 15u32); // above max

    assert_eq!(client.clamp_i32(&4, &5, &15), 5i32);
    assert_eq!(client.clamp_i32(&-100, &-50, &-10), -50i32);
}

// Section 5e — Storage Helpers and Initialization

#[test]
fn test_store_and_retrieve_all_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.store_u32(&123u32);
    assert_eq!(client.retrieve_u32(), 123u32);

    client.store_u64(&456u64);
    assert_eq!(client.retrieve_u64(), 456u64);

    client.store_i32(&-789i32);
    assert_eq!(client.retrieve_i32(), -789i32);

    client.store_i64(&-101_112i64);
    assert_eq!(client.retrieve_i64(), -101_112i64);
}

#[test]
fn test_reset_to_defaults() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.store_u32(&999u32);
    client.reset_to_defaults();

    assert_eq!(client.retrieve_u32(), 0u32);
    assert_eq!(client.retrieve_u64(), 0u64);
    assert_eq!(client.retrieve_i32(), 0i32);
    assert_eq!(client.retrieve_i64(), 0i64);
}

#[test]
fn test_initialize_seeds_boundary_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    client.initialize();

    // Each type initialized to its maximum representable value.
    assert_eq!(client.retrieve_u32(), u32::MAX);
    assert_eq!(client.retrieve_u64(), u64::MAX);
    assert_eq!(client.retrieve_i32(), i32::MAX);
    assert_eq!(client.retrieve_i64(), i64::MAX);
    assert!(client.get_bool());
    assert_eq!(client.get_counter(), 0u64); // counter starts at 0
    assert_eq!(client.get_balance(), 1000i128);
}

// ---------------------------------------------------------------------------
// Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_zero_and_identity_values() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    assert_eq!(client.add_u32(&0, &0), 0u32);
    assert_eq!(client.mul_u32(&0, &100), 0u32);
    assert_eq!(client.div_u32(&0, &1), 0u32);
    assert_eq!(client.mul_u32(&1, &100), 100u32);
    assert_eq!(client.div_u32(&100, &1), 100u32);
}

#[test]
fn test_signed_boundary_arithmetic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PrimitiveTypesContract);
    let client = PrimitiveTypesContractClient::new(&env, &contract_id);

    // Adding −1 to MIN+1 reaches MIN exactly (not below).
    assert_eq!(client.add_i32(&(i32::MIN + 1), &-1), i32::MIN);
    assert_eq!(client.add_i64(&(i64::MIN + 1), &-1), i64::MIN);
}
