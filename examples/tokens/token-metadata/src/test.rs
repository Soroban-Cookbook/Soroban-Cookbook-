#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

struct Fixture {
    env: Env,
    contract: TokenMetadataContractClient<'static>,
    admin: Address,
    alice: Address,
    bob: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TokenMetadataContract);
    let contract = TokenMetadataContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    contract.initialize(
        &admin,
        &String::from_str(&env, "Stellar Gold"),
        &String::from_str(&env, "SGLD"),
        &7u32,
        &String::from_str(&env, "https://example.com/sgld"),
    );

    Fixture {
        env,
        contract,
        admin,
        alice,
        bob,
    }
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

#[test]
fn initialize_stores_all_fields() {
    let f = setup();

    let meta = f.contract.metadata();
    assert_eq!(meta.name, String::from_str(&f.env, "Stellar Gold"));
    assert_eq!(meta.symbol, String::from_str(&f.env, "SGLD"));
    assert_eq!(meta.decimals, 7);
    assert_eq!(
        meta.uri,
        String::from_str(&f.env, "https://example.com/sgld")
    );
}

#[test]
fn individual_getters_match_metadata() {
    let f = setup();

    assert_eq!(f.contract.name(), String::from_str(&f.env, "Stellar Gold"));
    assert_eq!(f.contract.symbol(), String::from_str(&f.env, "SGLD"));
    assert_eq!(f.contract.decimals(), 7);
    assert_eq!(
        f.contract.uri(),
        String::from_str(&f.env, "https://example.com/sgld")
    );
}

#[test]
fn double_initialize_is_rejected() {
    let f = setup();

    assert_eq!(
        f.contract.try_initialize(
            &f.admin,
            &String::from_str(&f.env, "Other"),
            &String::from_str(&f.env, "OTH"),
            &6u32,
            &String::from_str(&f.env, ""),
        ),
        Err(Ok(MetadataError::AlreadyInitialized))
    );
}

#[test]
fn empty_name_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, TokenMetadataContract);
    let c = TokenMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    assert_eq!(
        c.try_initialize(
            &admin,
            &String::from_str(&env, ""),
            &String::from_str(&env, "SYM"),
            &6u32,
            &String::from_str(&env, ""),
        ),
        Err(Ok(MetadataError::EmptyString))
    );
}

#[test]
fn empty_symbol_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, TokenMetadataContract);
    let c = TokenMetadataContractClient::new(&env, &id);
    let admin = Address::generate(&env);

    assert_eq!(
        c.try_initialize(
            &admin,
            &String::from_str(&env, "Name"),
            &String::from_str(&env, ""),
            &6u32,
            &String::from_str(&env, ""),
        ),
        Err(Ok(MetadataError::EmptyString))
    );
}

// ---------------------------------------------------------------------------
// Metadata updates
// ---------------------------------------------------------------------------

#[test]
fn admin_can_update_mutable_fields() {
    let f = setup();

    f.contract.update_metadata(
        &String::from_str(&f.env, "Stellar Silver"),
        &String::from_str(&f.env, "SSLV"),
        &String::from_str(&f.env, "https://example.com/sslv"),
    );

    let meta = f.contract.metadata();
    assert_eq!(meta.name, String::from_str(&f.env, "Stellar Silver"));
    assert_eq!(meta.symbol, String::from_str(&f.env, "SSLV"));
    assert_eq!(
        meta.uri,
        String::from_str(&f.env, "https://example.com/sslv")
    );
    // decimals must not change
    assert_eq!(meta.decimals, 7);
}

#[test]
fn uri_can_be_cleared() {
    let f = setup();

    // Setting uri to a single space is the minimal non-empty string; an
    // empty uri is represented as an empty String which passes validation
    // only if we allow it. Here we verify the admin can set it to empty.
    // The contract allows empty uri (only name/symbol must be non-empty).
    f.contract.update_metadata(
        &String::from_str(&f.env, "Stellar Gold"),
        &String::from_str(&f.env, "SGLD"),
        &String::from_str(&f.env, ""),
    );

    assert_eq!(f.contract.uri(), String::from_str(&f.env, ""));
}

#[test]
fn update_with_empty_name_is_rejected() {
    let f = setup();

    assert_eq!(
        f.contract.try_update_metadata(
            &String::from_str(&f.env, ""),
            &String::from_str(&f.env, "SGLD"),
            &String::from_str(&f.env, ""),
        ),
        Err(Ok(MetadataError::EmptyString))
    );
}

// ---------------------------------------------------------------------------
// Mint / burn
// ---------------------------------------------------------------------------

#[test]
fn mint_increases_balance_and_supply() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000_000_000i128);

    assert_eq!(f.contract.balance(&f.alice), 1_000_000_000);
    assert_eq!(f.contract.total_supply(), 1_000_000_000);
}

#[test]
fn burn_decreases_balance_and_supply() {
    let f = setup();

    f.contract.mint(&f.alice, &500_0000000i128);
    f.contract.burn(&f.alice, &200_0000000i128);

    assert_eq!(f.contract.balance(&f.alice), 300_0000000);
    assert_eq!(f.contract.total_supply(), 300_0000000);
}

#[test]
fn burn_above_balance_is_rejected() {
    let f = setup();

    f.contract.mint(&f.alice, &100i128);

    assert_eq!(
        f.contract.try_burn(&f.alice, &101i128),
        Err(Ok(MetadataError::InsufficientBalance))
    );
}

#[test]
fn zero_mint_is_rejected() {
    let f = setup();

    assert_eq!(
        f.contract.try_mint(&f.alice, &0i128),
        Err(Ok(MetadataError::InvalidAmount))
    );
}

// ---------------------------------------------------------------------------
// Transfer
// ---------------------------------------------------------------------------

#[test]
fn transfer_moves_balance() {
    let f = setup();

    f.contract.mint(&f.alice, &1_000i128);
    f.contract.transfer(&f.alice, &f.bob, &400i128);

    assert_eq!(f.contract.balance(&f.alice), 600);
    assert_eq!(f.contract.balance(&f.bob), 400);
    assert_eq!(f.contract.total_supply(), 1_000);
}

#[test]
fn transfer_above_balance_is_rejected() {
    let f = setup();

    f.contract.mint(&f.alice, &100i128);

    assert_eq!(
        f.contract.try_transfer(&f.alice, &f.bob, &101i128),
        Err(Ok(MetadataError::InsufficientBalance))
    );
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------
// Run with: cargo test -p token-metadata -- --nocapture bench

#[cfg(test)]
mod bench {
    extern crate std;

    use super::*;
    use soroban_sdk::{Address, Env, String};

    fn setup_bench() -> (Env, TokenMetadataContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TokenMetadataContract);
        let client = TokenMetadataContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(
            &admin,
            &String::from_str(&env, "Gold"),
            &String::from_str(&env, "GLD"),
            &7u32,
            &String::from_str(&env, ""),
        );
        (env, client, admin)
    }

    #[test]
    fn bench_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TokenMetadataContract);
        let client = TokenMetadataContractClient::new(&env, &id);
        let admin = Address::generate(&env);

        env.budget().reset_default();
        client.initialize(
            &admin,
            &String::from_str(&env, "Gold"),
            &String::from_str(&env, "GLD"),
            &7u32,
            &String::from_str(&env, ""),
        );
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-metadata::initialize  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_metadata_query() {
        let (env, client, _admin) = setup_bench();
        env.budget().reset_default();
        let _ = client.metadata();
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-metadata::metadata  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_mint() {
        let (env, client, admin) = setup_bench();
        env.budget().reset_default();
        client.mint(&admin, &1_000_000_000i128);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-metadata::mint  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_transfer() {
        let (env, client, admin) = setup_bench();
        let recipient = Address::generate(&env);
        client.mint(&admin, &1_000_000_000i128);
        env.budget().reset_default();
        client.transfer(&admin, &recipient, &500_000_000i128);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-metadata::transfer  cpu={cpu}  mem={mem}");
    }

    #[test]
    fn bench_update_metadata() {
        let (env, client, _admin) = setup_bench();
        env.budget().reset_default();
        client.update_metadata(
            &String::from_str(&env, "Silver"),
            &String::from_str(&env, "SLV"),
            &String::from_str(&env, "https://example.com/slv"),
        );
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        std::println!("[bench] token-metadata::update_metadata  cpu={cpu}  mem={mem}");
    }
}
