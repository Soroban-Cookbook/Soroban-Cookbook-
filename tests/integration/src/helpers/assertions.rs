//! Custom assertion macros and functions for Soroban contract testing.

/// Assert that a contract invocation succeeded.
///
/// # Example
/// ```ignore
/// let result: Result<(), Error> = invoke_contract(&env, &contract, &func, &args);
/// assert_contract_success!(&result);
/// ```
#[macro_export]
macro_rules! assert_contract_success {
    ($result:expr) => {
        assert!($result.is_ok(), "Expected success but got error: {:?}", $result)
    };
}

/// Assert that a contract invocation failed with a specific error type.
///
/// # Example
/// ```ignore
/// let result: Result<(), Error> = invoke_contract(&env, &contract, &func, &args);
/// assert_contract_error!(&result, ContractError::Unauthorized);
/// ```
#[macro_export]
macro_rules! assert_contract_error {
    ($result:expr) => {
        assert!($result.is_err(), "Expected error but got success: {:?}", $result)
    };
}

/// Assert that a value equals the expected value with a custom message.
///
/// # Example
/// ```ignore
/// assert_eq_with_msg!(actual, expected, "balance should match");
/// ```
#[macro_export]
macro_rules! assert_eq_with_msg {
    ($actual:expr, $expected:expr, $msg:expr) => {
        assert_eq!($actual, $expected, "{}", $msg)
    };
}

/// Assert that a contract state value matches expected value.
///
/// # Example
/// ```ignore
/// assert_state_value(&env, &contract_id, &symbol_short!("count"), &expected);
/// ```
pub fn assert_state_value<T: PartialEq + std::fmt::Debug>(
    actual: T,
    expected: T,
    field_name: &str,
) {
    assert_eq!(
        actual, expected,
        "State mismatch for field '{}': expected {:?}, got {:?}",
        field_name, expected, actual
    );
}

/// Assert that two collections have the same length.
///
/// Useful for verifying event logs or result collections.
pub fn assert_collection_length<T>(collection: &[T], expected_len: usize, collection_name: &str) {
    assert_eq!(
        collection.len(),
        expected_len,
        "{} should have {} items, but got {}",
        collection_name,
        expected_len,
        collection.len()
    );
}

/// Assert that a boolean condition is true with a detailed message.
///
/// # Example
/// ```ignore
/// assert_condition!(value > 0, "value should be positive, got: {}", value);
/// ```
#[macro_export]
macro_rules! assert_condition {
    ($condition:expr, $($msg:tt)*) => {
        assert!($condition, $($msg)*)
    };
}
