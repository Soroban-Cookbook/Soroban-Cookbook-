extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, IntoVal, Symbol, Vec,
};

// Mock token contract for testing
#[contract]
struct MockTokenContract;

#[contractimpl]
impl MockTokenContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();

        let balance_key = (symbol_short!("balance"), to.clone());
        let balance: i128 = env.storage().instance().get(&balance_key).unwrap_or(0);
        env.storage().instance().set(&balance_key, &(balance + amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let from_key = (symbol_short!("balance"), from.clone());
        let to_key = (symbol_short!("balance"), to.clone());

        let from_balance: i128 = env.storage().instance().get(&from_key).unwrap_or(0);
        let to_balance: i128 = env.storage().instance().get(&to_key).unwrap_or(0);

        if from_balance < amount {
            panic!("Insufficient balance");
        }

        env.storage().instance().set(&from_key, &(from_balance - amount));
        env.storage().instance().set(&to_key, &(to_balance + amount));
    }

    pub fn balance(env: Env, address: Address) -> i128 {
        env.storage().instance().get(&(symbol_short!("balance"), address)).unwrap_or(0)
    }
}

// Helper to get token balance
fn get_token_balance(env: &Env, token_id: &Address, address: &Address) -> i128 {
    let token_client = MockTokenContractClient::new(env, token_id);
    token_client.balance(address)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator1.clone(), validator2.clone()]);

    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);

    client.initialize(&admin, &validators, &2u32);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_threshold(), 2);
    assert!(client.is_validator(&validator1));
    assert!(client.is_validator(&validator2));
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);

    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);

    client.initialize(&admin, &validators, &1u32);
    client.initialize(&admin, &validators, &1u32);
}

#[test]
fn test_lock_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    // Setup bridge
    let bridge_admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    // Mint tokens to sender
    let sender = Address::generate(&env);
    token_client.mint(&token_admin, &sender, &1000i128);

    // Lock tokens
    let recipient = Bytes::from_array(&env, &[0u8; 32]);
    let transfer = bridge_client.lock_tokens(
        &sender,
        &symbol_short!("ethereum"),
        &recipient,
        &token_id,
        &500i128,
    );

    // Verify balances
    assert_eq!(get_token_balance(&env, &token_id, &sender), 500);
    assert_eq!(get_token_balance(&env, &token_id, &bridge_id), 500);

    // Verify locked balance
    assert_eq!(bridge_client.get_locked_balance(&token_id, &sender), 500);

    // Verify nonce
    assert_eq!(bridge_client.get_nonce(), 1);

    // Verify transfer details
    assert_eq!(transfer.source_chain, symbol_short!("soroban"));
    assert_eq!(transfer.destination_chain, symbol_short!("ethereum"));
    assert_eq!(transfer.sender, sender);
    assert_eq!(transfer.recipient, recipient);
    assert_eq!(transfer.token, token_id);
    assert_eq!(transfer.amount, 500);
    assert_eq!(transfer.nonce, 0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_lock_zero_amount_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, MockTokenContract);
    let bridge_admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    let sender = Address::generate(&env);
    let recipient = Bytes::from_array(&env, &[0u8; 32]);
    bridge_client.lock_tokens(
        &sender,
        &symbol_short!("ethereum"),
        &recipient,
        &token_id,
        &0i128,
    );
}

#[test]
fn test_add_validator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let initial_validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [initial_validator.clone()]);
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &validators, &1u32);

    let new_validator = Address::generate(&env);
    client.add_validator(&admin, &new_validator);

    assert!(client.is_validator(&new_validator));
}

#[test]
#[should_panic(expected = "Validator already exists")]
fn test_add_duplicate_validator_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator.clone()]);
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &validators, &1u32);

    client.add_validator(&admin, &validator);
}

#[test]
fn test_set_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ]);
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &validators, &1u32);

    client.set_threshold(&admin, &2u32);
    assert_eq!(client.get_threshold(), 2);
}

#[test]
#[should_panic(expected = "Threshold must be at least 1")]
fn test_set_threshold_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &validators, &1u32);

    client.set_threshold(&admin, &0u32);
}

#[test]
fn test_map_token() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &validators, &1u32);

    let source_token = Address::generate(&env);
    let soroban_token = Address::generate(&env);
    client.map_token(&admin, &symbol_short!("ethereum"), &source_token, &soroban_token);
}

#[test]
fn test_burn_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    // Setup bridge
    let bridge_admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    // Mint tokens to sender
    let sender = Address::generate(&env);
    token_client.mint(&token_admin, &sender, &1000i128);

    // Burn tokens
    let recipient = Bytes::from_array(&env, &[0u8; 32]);
    let transfer = bridge_client.burn_tokens(
        &sender,
        &symbol_short!("soroban"),
        &recipient,
        &token_id,
        &300i128,
    );

    // Verify balances
    assert_eq!(get_token_balance(&env, &token_id, &sender), 700);
    assert_eq!(get_token_balance(&env, &token_id, &bridge_id), 300);

    // Verify nonce
    assert_eq!(bridge_client.get_nonce(), 1);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_burn_zero_amount_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, MockTokenContract);
    let bridge_admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    let sender = Address::generate(&env);
    let recipient = Bytes::from_array(&env, &[0u8; 32]);
    bridge_client.burn_tokens(
        &sender,
        &symbol_short!("soroban"),
        &recipient,
        &token_id,
        &0i128,
    );
}

#[test]
fn test_mint_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    // Setup bridge
    let bridge_admin = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator1.clone(), validator2.clone()]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    // Mint tokens to bridge for reserve
    token_client.mint(&token_admin, &bridge_id, &1000i128);

    // Create transfer
    let recipient_bytes = Bytes::from_array(&env, &[0u8; 32]);
    let recipient = Address::from_string_bytes(&recipient_bytes);
    let transfer = BridgeTransfer {
        source_chain: symbol_short!("ethereum"),
        destination_chain: symbol_short!("soroban"),
        sender: Address::generate(&env),
        recipient: recipient_bytes,
        token: token_id.clone(),
        amount: 200,
        nonce: 0,
    };

    // Create signatures
    let signature = ValidatorSignature {
        validator: validator1,
        signature: Bytes::from_array(&env, &[0u8; 64]),
    };
    let signatures = Vec::from_array(&env, [signature]);

    // Mint tokens
    bridge_client.mint_tokens(&transfer, &signatures);

    // Verify recipient balance
    assert_eq!(get_token_balance(&env, &token_id, &recipient), 200);
    assert_eq!(get_token_balance(&env, &token_id, &bridge_id), 800);
}

#[test]
#[should_panic(expected = "Transfer already processed")]
fn test_mint_duplicate_transfer_panics() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token and bridge
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    let bridge_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator.clone()]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    token_client.mint(&token_admin, &bridge_id, &1000i128);

    // Create transfer
    let transfer = BridgeTransfer {
        source_chain: symbol_short!("ethereum"),
        destination_chain: symbol_short!("soroban"),
        sender: Address::generate(&env),
        recipient: Bytes::from_array(&env, &[0u8; 32]),
        token: token_id.clone(),
        amount: 200,
        nonce: 0,
    };

    let signature = ValidatorSignature {
        validator,
        signature: Bytes::from_array(&env, &[0u8; 64]),
    };
    let signatures = Vec::from_array(&env, [signature]);

    // Mint twice
    bridge_client.mint_tokens(&transfer, &signatures);
    bridge_client.mint_tokens(&transfer, &signatures);
}

#[test]
fn test_release_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    // Setup bridge
    let bridge_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator.clone()]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    // First lock some tokens
    let sender = Address::generate(&env);
    token_client.mint(&token_admin, &sender, &1000i128);
    let recipient_lock = Bytes::from_array(&env, &[0u8; 32]);
    bridge_client.lock_tokens(
        &sender,
        &symbol_short!("ethereum"),
        &recipient_lock,
        &token_id,
        &500i128,
    );

    // Create release transfer
    let recipient_bytes = Bytes::from_array(&env, &[1u8; 32]);
    let recipient = Address::from_string_bytes(&recipient_bytes);
    let transfer = BridgeTransfer {
        source_chain: symbol_short!("ethereum"),
        destination_chain: symbol_short!("soroban"),
        sender: sender.clone(),
        recipient: recipient_bytes,
        token: token_id.clone(),
        amount: 300,
        nonce: 1,
    };

    // Create signature
    let signature = ValidatorSignature {
        validator,
        signature: Bytes::from_array(&env, &[0u8; 64]),
    };
    let signatures = Vec::from_array(&env, [signature]);

    // Release tokens
    bridge_client.release_tokens(&transfer, &signatures);

    // Verify balances
    assert_eq!(get_token_balance(&env, &token_id, &recipient), 300);
    assert_eq!(get_token_balance(&env, &token_id, &bridge_id), 200);
    assert_eq!(bridge_client.get_locked_balance(&token_id, &sender), 200);
}

#[test]
#[should_panic(expected = "Insufficient validator signatures")]
fn test_release_insufficient_signatures_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let token_id = env.register_contract(None, MockTokenContract);
    let bridge_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator.clone()]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &2u32);

    let transfer = BridgeTransfer {
        source_chain: symbol_short!("ethereum"),
        destination_chain: symbol_short!("soroban"),
        sender: Address::generate(&env),
        recipient: Bytes::from_array(&env, &[0u8; 32]),
        token: token_id,
        amount: 100,
        nonce: 0,
    };

    // Only one signature when threshold is 2
    let signature = ValidatorSignature {
        validator,
        signature: Bytes::from_array(&env, &[0u8; 64]),
    };
    let signatures = Vec::from_array(&env, [signature]);

    bridge_client.release_tokens(&transfer, &signatures);
}

#[test]
fn test_multiple_locks_and_releases() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    // Setup bridge
    let bridge_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let validators = Vec::from_array(&env, [validator.clone()]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    // Mint tokens
    let sender = Address::generate(&env);
    token_client.mint(&token_admin, &sender, &2000i128);

    // Lock 1
    let recipient1 = Bytes::from_array(&env, &[1u8; 32]);
    bridge_client.lock_tokens(&sender, &symbol_short!("ethereum"), &recipient1, &token_id, &500i128);
    assert_eq!(bridge_client.get_locked_balance(&token_id, &sender), 500);

    // Lock 2
    let recipient2 = Bytes::from_array(&env, &[2u8; 32]);
    bridge_client.lock_tokens(&sender, &symbol_short!("polygon"), &recipient2, &token_id, &300i128);
    assert_eq!(bridge_client.get_locked_balance(&token_id, &sender), 800);

    // Release 1
    let release_recipient = Address::from_string_bytes(&Bytes::from_array(&env, &[3u8; 32]));
    let transfer = BridgeTransfer {
        source_chain: symbol_short!("ethereum"),
        destination_chain: symbol_short!("soroban"),
        sender: sender.clone(),
        recipient: Bytes::from_array(&env, &[3u8; 32]),
        token: token_id.clone(),
        amount: 400,
        nonce: 2,
    };
    let signature = ValidatorSignature {
        validator,
        signature: Bytes::from_array(&env, &[0u8; 64]),
    };
    let signatures = Vec::from_array(&env, [signature]);
    bridge_client.release_tokens(&transfer, &signatures);

    assert_eq!(bridge_client.get_locked_balance(&token_id, &sender), 400);
    assert_eq!(get_token_balance(&env, &token_id, &release_recipient), 400);
}

#[test]
fn test_event_emission() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id = env.register_contract(None, MockTokenContract);
    let token_client = MockTokenContractClient::new(&env, &token_id);
    token_client.initialize(&token_admin);

    let bridge_admin = Address::generate(&env);
    let validators = Vec::from_array(&env, [Address::generate(&env)]);
    let bridge_id = env.register_contract(None, CrossChainBridgeContract);
    let bridge_client = CrossChainBridgeContractClient::new(&env, &bridge_id);
    bridge_client.initialize(&bridge_admin, &validators, &1u32);

    let sender = Address::generate(&env);
    token_client.mint(&token_admin, &sender, &1000i128);

    let recipient = Bytes::from_array(&env, &[0u8; 32]);
    bridge_client.lock_tokens(
        &sender,
        &symbol_short!("ethereum"),
        &recipient,
        &token_id,
        &500i128,
    );

    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1);
}
