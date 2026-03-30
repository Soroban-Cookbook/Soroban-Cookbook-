//! Tests for panic-vs-errors patterns.
//!
//! Each test is labelled to show *which* failure mode it exercises.

#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        Address, Env, IntoVal, Symbol,
    };

    use crate::{ContractError, ErrorDemoContract, ErrorDemoContractClient};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Deploy a fresh, initialised contract and return (env, client, admin).
    fn setup() -> (Env, ErrorDemoContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ErrorDemoContract);
        let client = ErrorDemoContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        client.initialize(&admin);

        (env, client, admin)
    }

    // =======================================================================
    // Typed error tests
    // =======================================================================

    /// `deposit` rejects zero amounts with a typed error.
    ///
    /// **Pattern:** expected user mistake → `Err(ContractError::ZeroAmount)`.
    #[test]
    fn test_error_zero_amount_deposit() {
        let (_, client, _) = setup();
        let user = Address::generate(&client.env);

        let result = client.try_deposit(&user, &0);
        assert_eq!(result, Err(Ok(ContractError::ZeroAmount)));
    }

    /// `withdraw` rejects zero amounts with a typed error.
    #[test]
    fn test_error_zero_amount_withdraw() {
        let (_, client, _) = setup();
        let user = Address::generate(&client.env);

        let result = client.try_withdraw(&user, &0);
        assert_eq!(result, Err(Ok(ContractError::ZeroAmount)));
    }

    /// `withdraw` returns `InsufficientBalance` when the account has less than requested.
    ///
    /// **Pattern:** business-logic constraint → typed error the client can handle.
    #[test]
    fn test_error_insufficient_balance() {
        let (_, client, _) = setup();
        let user = Address::generate(&client.env);

        // Never deposited — balance is 0.
        let result = client.try_withdraw(&user, &100);
        assert_eq!(result, Err(Ok(ContractError::InsufficientBalance)));
    }

    /// `deposit` returns `ContractPaused` when the contract is paused.
    ///
    /// **Pattern:** state-based rejection → typed error.
    #[test]
    fn test_error_contract_paused_deposit() {
        let (_, client, admin) = setup();
        client.pause(&admin);

        let user = Address::generate(&client.env);
        let result = client.try_deposit(&user, &50);
        assert_eq!(result, Err(Ok(ContractError::ContractPaused)));
    }

    /// `withdraw` also returns `ContractPaused` when paused.
    #[test]
    fn test_error_contract_paused_withdraw() {
        let (_, client, admin) = setup();
        client.pause(&admin);

        let user = Address::generate(&client.env);
        let result = client.try_withdraw(&user, &50);
        assert_eq!(result, Err(Ok(ContractError::ContractPaused)));
    }

    // =======================================================================
    // Panic tests
    // =======================================================================

    /// Calling `initialize` twice panics — it is an invariant violation, not
    /// a user-facing error.
    ///
    /// **Pattern:** contract invariant → `panic!`.  `try_initialize` returns
    /// `Err` at the SDK level (the transaction reverted), but there is no
    /// `ContractError` variant for this — it is a hard abort.
    #[test]
    fn test_panic_double_initialise() {
        let (_, client, admin) = setup();

        // Second initialisation must abort the transaction.
        let result = client.try_initialize(&admin);
        assert!(result.is_err());
    }

    /// `pause` called by a non-admin panics via `panic_with_error!`.
    ///
    /// **Pattern:** auth invariant → `panic_with_error!(env, Unauthorized)`.
    /// The error code is visible in the SDK result even though the tx reverted.
    #[test]
    fn test_panic_with_error_unauthorized_pause() {
        let (_, client, _) = setup();
        let non_admin = Address::generate(&client.env);

        let result = client.try_pause(&non_admin);
        // The transaction reverted with an error value, not a clean Ok.
        assert!(result.is_err());
    }

    /// `status_label` panics on an unknown code — defensive impossible-branch guard.
    ///
    /// **Pattern:** programmer error → `panic!("this is a bug")`.
    #[test]
    fn test_panic_impossible_branch() {
        let (_, client, _) = setup();

        // Codes 0–2 are valid.
        assert_eq!(client.status_label(&0), Symbol::new(&client.env, "ok"));
        assert_eq!(client.status_label(&1), Symbol::new(&client.env, "paused"));
        assert_eq!(client.status_label(&2), Symbol::new(&client.env, "error"));

        // Code 99 is out of range — must panic (transaction reverts).
        let result = client.try_status_label(&99);
        assert!(result.is_err());
    }

    // =======================================================================
    // Happy-path tests (for completeness)
    // =======================================================================

    /// Successful deposit/withdraw round-trip.
    #[test]
    fn test_happy_path_deposit_withdraw() {
        let (_, client, _) = setup();
        let user = Address::generate(&client.env);

        let after_deposit = client.deposit(&user, &200);
        assert_eq!(after_deposit, 200);

        let after_withdraw = client.withdraw(&user, &75);
        assert_eq!(after_withdraw, 125);

        assert_eq!(client.balance(&user), 125);
    }

    /// Pause → operations blocked → unpause → operations resume.
    #[test]
    fn test_pause_unpause_cycle() {
        let (_, client, admin) = setup();
        let user = Address::generate(&client.env);

        client.deposit(&user, &100);

        client.pause(&admin);
        assert!(client.is_paused());

        // Both operations rejected while paused.
        assert_eq!(
            client.try_deposit(&user, &50),
            Err(Ok(ContractError::ContractPaused))
        );
        assert_eq!(
            client.try_withdraw(&user, &50),
            Err(Ok(ContractError::ContractPaused))
        );

        client.unpause(&admin);
        assert!(!client.is_paused());

        // Operations succeed again.
        assert_eq!(client.deposit(&user, &50), 150);
        assert_eq!(client.withdraw(&user, &150), 0);
    }
}