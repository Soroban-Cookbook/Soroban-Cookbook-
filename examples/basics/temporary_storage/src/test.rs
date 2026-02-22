#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Ledger, Env};

#[test]
fn test_temporary_storage_behavior() {
    let env = Env::default();

    // Initial ledger setup
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 1000,
        protocol_version: 20,
        sequence_number: 1, // Start at ledger 1
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6312000,
    });

    let contract_id = env.register_contract(None, TemporaryStorageContract);
    let client = TemporaryStorageContractClient::new(&env, &contract_id);

    // 1. Test immediate retrieval (within the same ledger)
    client.start_calculation(&500);
    assert_eq!(client.get_temp_value(), 500);

    // 2. Simulate the passage of time (Jump forward 50 ledgers)
    // Since our TTL was set to 32, the data should be deleted by ledger 51.
    env.ledger().with_mut(|li| {
        li.sequence_number += 50;
    });

    // 3. Verify it's gone.
    // In Soroban, expired temporary storage is physically removed.
    assert_eq!(client.get_temp_value(), 0); // Returns 0 because of unwrap_or(0)
}

#[test]
#[should_panic(expected = "Reentrancy forbidden")]
fn test_reentrancy_guard() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TemporaryStorageContract);
    let client = TemporaryStorageContractClient::new(&env, &contract_id);

    // We manually set the reentrancy flag in the contract's temporary storage
    env.as_contract(&contract_id, || {
        env.storage()
            .temporary()
            .set(&TempKey::ReentrancyGuard, &true);
    });

    // This should panic because the "guard" is active
    client.guarded_function();
}
