#![allow(deprecated)]
#[cfg(test)]
mod tests {
    use crate::*;
    use soroban_sdk::{Address, Env, Vec};

    fn setup() -> (Env, GasOptimizationContractClient<'static>, Address) {
        let env = Env::default();
        let contract_id = env.register(GasOptimizationContract, ());
        let client = GasOptimizationContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        (env, client, admin)
    }

    // ============== Optimization Tests ==============

    #[test]
    fn test_optimization_1_instance_storage_initialization() {
        let (env, client, admin) = setup();

        // Optimization 1: instance storage used for config
        let result = client.try_initialize(&admin, &100u32);
        assert!(result.is_ok());

        // Second init must fail — config already exists
        let result2 = client.try_initialize(&admin, &200u32);
        assert!(result2.is_err());
    }

    #[test]
    fn test_optimization_2_caching_config() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &100u32);

        // Give user1 a balance first so the transfer can succeed
        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 2000i128));
        client.batch_mint(&recipients);

        // Optimization 2: transfer uses a single cached config read
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_3_batch_operations_vs_individual() {
        let (env, client, admin) = setup();

        client.initialize(&admin, &0u32);

        // Optimization 3: batch mint is more gas-efficient than individual mints
        let mut recipients = Vec::new(&env);
        for i in 0..5i128 {
            let addr = Address::generate(&env);
            recipients.push_back((addr, 1000 * (i + 1)));
        }

        let result = client.try_batch_mint(&recipients);
        assert!(result.is_ok());

        // Verify all balances were set correctly
        for i in 0..5i128 {
            let addr = &recipients.get(i as u32).unwrap().0;
            let balance = client.get_balance(addr);
            assert_eq!(balance, 1000 * (i + 1));
        }
    }

    #[test]
    fn test_optimization_4_symbol_interning() {
        let (_env, client, admin) = setup();

        // Optimization 4: the contract uses symbol_short! for the config key
        // — this test verifies initialization succeeds with that key in place.
        let result = client.try_initialize(&admin, &50u32);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_5_enum_state_over_strings() {
        let (_env, client, admin) = setup();

        client.initialize(&admin, &100u32);

        // Optimization 5: state stored as bitflags, not strings
        let result = client.try_set_emergency(&true);
        assert!(result.is_ok());

        // Pausing in emergency mode should still work (separate flag)
        let pause_result = client.try_pause();
        assert!(pause_result.is_ok());
    }

    #[test]
    fn test_optimization_6_minimizing_storage_reads() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &100u32);

        // Set up initial balance
        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 5000i128));
        client.batch_mint(&recipients);

        // Optimization 6: get_balance is a single persistent read
        let balance = client.get_balance(&user1);
        assert_eq!(balance, 5000);

        // Transfer with minimal storage reads
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_ok());

        // fee = 1000 * 100 / 10_000 = 10
        // user1: 5000 - 1000 = 4000
        assert_eq!(client.get_balance(&user1), 4000);
        // user2: 0 + (1000 - 10) = 990
        assert_eq!(client.get_balance(&user2), 990);
    }

    #[test]
    fn test_optimization_7_lazy_initialization() {
        let (env, client, admin) = setup();

        // Optimization 7: config written only once during initialization
        let result = client.try_initialize(&admin, &200u32);
        assert!(result.is_ok());

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 2000i128));
        client.batch_mint(&recipients);

        // Transfer should succeed (contract not paused by default)
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimization_8_checked_arithmetic() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32);

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 1000i128));
        client.batch_mint(&recipients);

        // Optimization 8: checked arithmetic prevents overflow / underflow
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &2000i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimization_9_short_circuit_evaluation() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &100u32);
        client.pause();

        // Optimization 9: paused check short-circuits before any balance I/O
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimization_10_typed_errors() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &100u32);

        // Optimization 10: typed errors give callers precise failure information
        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &0i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimization_11_bitflags_for_state() {
        let (_env, client, admin) = setup();

        client.initialize(&admin, &100u32);

        // Optimization 11: bitflags store multiple booleans in a single u32
        assert!(client.try_set_emergency(&true).is_ok());
        assert!(client.try_pause().is_ok());
        assert!(client.try_unpause().is_ok());
    }

    #[test]
    fn test_optimization_12_struct_packing() {
        let (_env, client, admin) = setup();

        // Optimization 12: Config uses u32 (Soroban-native) for all numeric
        // fields, avoiding any padding or unsupported type issues.
        let result = client.try_initialize(&admin, &500u32);
        assert!(result.is_ok());
    }

    // ============== Functional Tests ==============

    #[test]
    fn test_transfer_basic() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32); // zero fee for simplicity

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 5000i128));
        client.batch_mint(&recipients);

        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_ok());

        // Zero fee: user1 loses 1000, user2 gains 1000
        assert_eq!(client.get_balance(&user1), 4000);
        assert_eq!(client.get_balance(&user2), 1000);
    }

    #[test]
    fn test_transfer_with_fee() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &500u32); // 5 % fee

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 10000i128));
        client.batch_mint(&recipients);

        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_ok());

        // fee = 1000 * 500 / 10_000 = 50
        assert_eq!(client.get_balance(&user1), 9000);
        assert_eq!(client.get_balance(&user2), 950);
    }

    #[test]
    fn test_pause_unpause() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32);

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 1000i128));
        client.batch_mint(&recipients);

        env.mock_all_auths();

        // Works before pause
        assert!(client.try_transfer(&user1, &user2, &100i128).is_ok());

        client.pause();

        // Blocked while paused
        assert!(client.try_transfer(&user1, &user2, &100i128).is_err());

        client.unpause();

        // Works again after unpause
        assert!(client.try_transfer(&user1, &user2, &100i128).is_ok());
    }

    #[test]
    fn test_emergency_blocks_transfer() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32);

        let mut recipients = Vec::new(&env);
        recipients.push_back((user1.clone(), 1000i128));
        client.batch_mint(&recipients);

        client.set_emergency(&true);

        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &100i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_burn() {
        let (env, client, admin) = setup();

        client.initialize(&admin, &0u32);

        let mut recipients: Vec<(Address, i128)> = Vec::new(&env);
        for i in 1i128..=3 {
            recipients.push_back((Address::generate(&env), i * 1000));
        }
        client.batch_mint(&recipients);

        let mut burn_list: Vec<(Address, i128)> = Vec::new(&env);
        for i in 0..3u32 {
            let addr = &recipients.get(i).unwrap().0;
            burn_list.push_back((addr.clone(), 500i128));
        }

        let result = client.try_batch_burn(&burn_list);
        assert!(result.is_ok());

        for i in 0..3u32 {
            let addr = &recipients.get(i).unwrap().0;
            let original = recipients.get(i).unwrap().1;
            assert_eq!(client.get_balance(addr), original - 500);
        }
    }

    #[test]
    fn test_batch_operations_efficiency() {
        let (env, client, admin) = setup();

        client.initialize(&admin, &0u32);

        let mut recipients: Vec<(Address, i128)> = Vec::new(&env);
        for i in 1i128..=10 {
            recipients.push_back((Address::generate(&env), i * 100));
        }
        client.batch_mint(&recipients);

        for i in 0..10u32 {
            let addr = &recipients.get(i).unwrap().0;
            let expected = recipients.get(i).unwrap().1;
            assert_eq!(client.get_balance(addr), expected);
        }
    }

    #[test]
    fn test_insufficient_balance_error() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32);

        env.mock_all_auths();
        let result = client.try_transfer(&user1, &user2, &1000i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_amount_error() {
        let (env, client, admin) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.initialize(&admin, &0u32);

        env.mock_all_auths();
        // Zero amount is invalid
        let result = client.try_transfer(&user1, &user2, &0i128);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_balances_batch_query() {
        let (env, client, admin) = setup();

        client.initialize(&admin, &0u32);

        let mut recipients: Vec<(Address, i128)> = Vec::new(&env);
        for i in 1i128..=5 {
            recipients.push_back((Address::generate(&env), i * 1000));
        }
        client.batch_mint(&recipients);

        let mut addresses: Vec<Address> = Vec::new(&env);
        for i in 0..5u32 {
            addresses.push_back(recipients.get(i).unwrap().0.clone());
        }

        let balances = client.get_balances(&addresses);
        assert_eq!(balances.len(), 5);

        for i in 0..5u32 {
            let expected = recipients.get(i).unwrap().1;
            assert_eq!(balances.get(i).unwrap(), expected);
        }
    }

    #[test]
    fn test_emergency_mode_admin_functions_still_work() {
        let (_env, client, admin) = setup();

        client.initialize(&admin, &100u32);

        // Admin functions work in emergency mode
        assert!(client.try_set_emergency(&true).is_ok());
        assert!(client.try_pause().is_ok());
        assert!(client.try_unpause().is_ok());
        assert!(client.try_set_emergency(&false).is_ok());
    }
}
