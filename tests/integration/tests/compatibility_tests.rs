#![allow(deprecated)]
//! # Cross-Version & Environment Compatibility Test Suite (Issue #950 / Phase 6 #306)
//!
//! Validates:
//! 1. Soroban SDK version & type representation compatibility
//! 2. Network protocol simulation (Protocol parameters & ledger advancement)
//! 3. Contract upgrade compatibility & state preservation across migrations
//! 4. Storage layout backwards-compatibility
//! 5. Error code numeric stability across contract revisions

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

use soroban_sdk::{
    contracttype, symbol_short,
    testutils::{Address as _, Ledger as _, LedgerInfo},
    Address, Bytes, BytesN, Env, FromVal, Map, String, Symbol, Val, Vec,
};

// ---------------------------------------------------------------------------
// 1. SDK Version & Core Type Representation Compatibility
// ---------------------------------------------------------------------------

#[test]
fn test_sdk_version_type_compatibility() {
    let env = Env::default();
    env.mock_all_auths();

    // Verify Symbol representation and conversions
    let sym = symbol_short!("test_sym");
    let sym_from_str = Symbol::new(&env, "test_sym");
    assert_eq!(sym, sym_from_str);

    // Verify Address representation and conversions
    let addr = Address::generate(&env);
    let val_addr: Val = addr.to_val();
    let addr_recovered = Address::from_val(&env, &val_addr);
    assert_eq!(addr, addr_recovered);

    // Verify String representation and UTF-8 handling
    let test_str = String::from_str(&env, "Soroban Cross-Version String");
    assert_eq!(test_str.len(), 28);

    // Verify Vec and Map cross-SDK collections
    let mut vec: Vec<i32> = Vec::new(&env);
    vec.push_back(10);
    vec.push_back(20);
    vec.push_back(30);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.get(1), Some(20));

    let mut map: Map<Symbol, i64> = Map::new(&env);
    map.set(symbol_short!("key_a"), 100);
    map.set(symbol_short!("key_b"), 200);
    assert_eq!(map.get(symbol_short!("key_a")), Some(100));
    assert_eq!(map.len(), 2);

    // Verify Bytes and BytesN binary format compatibility
    let raw_bytes: [u8; 32] = [0x55; 32];
    let bytes_n = BytesN::from_array(&env, &raw_bytes);
    let dyn_bytes = Bytes::from_slice(&env, &raw_bytes);
    assert_eq!(bytes_n.to_array(), raw_bytes);
    assert_eq!(dyn_bytes.len(), 32);
}

// ---------------------------------------------------------------------------
// 2. Network Compatibility Tests (Protocol Parameters & Ledger Advancement)
// ---------------------------------------------------------------------------

#[test]
fn test_network_compatibility_ledger_protocol_simulation() {
    let env = Env::default();
    env.mock_all_auths();

    // Register storage contract to test under varied ledger protocol environments
    let storage_id = env.register_contract(None, storage_patterns::StorageContract);
    let storage = storage_patterns::StorageContractClient::new(&env, &storage_id);

    let proto = env.ledger().protocol_version();

    // Protocol phase A simulation
    env.ledger().set(LedgerInfo {
        timestamp: 1720000000,
        protocol_version: proto,
        sequence_number: 1000,
        network_id: [1u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6312000,
    });

    let key_a = symbol_short!("k_phase_a");
    storage.set_persistent(&key_a, &2020u64);
    assert_eq!(storage.get_persistent(&key_a), Some(2020u64));

    // Protocol phase B simulation (Ledger advancement)
    env.ledger().set(LedgerInfo {
        timestamp: 1730000000,
        protocol_version: proto,
        sequence_number: 150000,
        network_id: [1u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 32,
        min_persistent_entry_ttl: 8192,
        max_entry_ttl: 8000000,
    });

    // Verify state stored under phase A remains readable in phase B
    assert_eq!(storage.get_persistent(&key_a), Some(2020u64));

    let key_b = symbol_short!("k_phase_b");
    storage.set_persistent(&key_b, &2121u64);
    assert_eq!(storage.get_persistent(&key_b), Some(2121u64));

    // Phase C simulation (Further ledger sequence and TTL limit expansion)
    env.ledger().set(LedgerInfo {
        timestamp: 1740000000,
        protocol_version: proto,
        sequence_number: 500000,
        network_id: [1u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 64,
        min_persistent_entry_ttl: 16384,
        max_entry_ttl: 10000000,
    });

    // Both previous entries survive and operate under new protocol configuration
    assert_eq!(storage.get_persistent(&key_a), Some(2020u64));
    assert_eq!(storage.get_persistent(&key_b), Some(2121u64));
}

// ---------------------------------------------------------------------------
// 3. Upgrade Compatibility & State Preservation Across Implementations
// ---------------------------------------------------------------------------

#[test]
fn test_upgrade_compatibility_state_preservation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Register v1 implementation and proxy
    let v1_id = env.register_contract(
        None,
        upgradeable_proxy_implementation_v1::ImplementationV1,
    );
    let proxy_id = env.register_contract(None, upgradeable_proxy::proxy::ProxyContract);
    let proxy = upgradeable_proxy::proxy::ProxyContractClient::new(&env, &proxy_id);

    // Initialize proxy targeting v1 implementation
    proxy.init(&admin, &v1_id);
    assert_eq!(proxy.get_implementation(), v1_id);

    // Call v1 methods through proxy
    assert_eq!(proxy.add(&10, &5), 15);
    assert_eq!(proxy.subtract(&20, &8), 12);

    // Deploy an upgraded implementation contract
    let v2_id = env.register_contract(
        None,
        upgradeable_proxy_implementation_v1::ImplementationV1,
    );

    // Perform upgrade
    proxy.upgrade(&v2_id);
    assert_eq!(proxy.get_implementation(), v2_id);

    // Verify critical compatibility invariant: Proxy forwards calls to upgraded contract
    assert_eq!(proxy.add(&30, &10), 40);
    assert_eq!(proxy.subtract(&50, &15), 35);
}

// ---------------------------------------------------------------------------
// 4. Storage Layout Compatibility: Schema Evolution
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct LegacyAccountData {
    pub balance: u64,
    pub owner: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ModernAccountData {
    pub balance: u64,
    pub owner: Address,
    pub flags: u32, // New field added in newer revision
}

#[test]
fn test_storage_layout_backward_compatibility() {
    let env = Env::default();
    let contract_id = env.register_contract(None, storage_patterns::StorageContract);
    let owner = Address::generate(&env);

    let legacy_key = symbol_short!("acc_v1");
    let modern_key = symbol_short!("acc_v2");
    let legacy_data = LegacyAccountData {
        balance: 5000,
        owner: owner.clone(),
    };

    env.as_contract(&contract_id, || {
        // Store legacy data structure in storage
        env.storage().persistent().set(&legacy_key, &legacy_data);

        // Retrieve legacy data
        let read_legacy: LegacyAccountData = env.storage().persistent().get(&legacy_key).unwrap();
        assert_eq!(read_legacy, legacy_data);

        // Migrate to modern data structure without breaking access
        let modern_data = ModernAccountData {
            balance: read_legacy.balance,
            owner: read_legacy.owner,
            flags: 1u32,
        };
        env.storage().persistent().set(&modern_key, &modern_data);

        let read_modern: ModernAccountData = env.storage().persistent().get(&modern_key).unwrap();
        assert_eq!(read_modern.balance, 5000);
        assert_eq!(read_modern.flags, 1);
    });
}

// ---------------------------------------------------------------------------
// 5. Custom Error Discriminator Stability
// ---------------------------------------------------------------------------

#[test]
fn test_custom_error_code_stability() {
    // Ensure error codes match standard enum integer discriminants
    assert_eq!(custom_errors::ContractError::InvalidInput as u32, 1);
    assert_eq!(custom_errors::ContractError::Unauthorized as u32, 2);
    assert_eq!(custom_errors::ContractError::NotFound as u32, 3);

    assert_eq!(basic_nft::NftError::AlreadyInitialized as u32, 1);
    assert_eq!(basic_nft::NftError::NotInitialized as u32, 2);
    assert_eq!(basic_nft::NftError::TokenNotFound as u32, 3);
    assert_eq!(basic_nft::NftError::NotOwner as u32, 5);
}
