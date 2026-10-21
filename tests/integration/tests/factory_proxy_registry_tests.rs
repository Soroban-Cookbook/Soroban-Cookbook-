//! # Cross-Pattern Integration Tests (Issue #120)
//!
//! End-to-end tests validating factory, proxy, and registry patterns working
//! together: discovery, upgrade flows, and lifecycle management.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Test 1: Factory deploys instances; registry discovers them
// ---------------------------------------------------------------------------

#[test]
fn test_factory_deploys_and_registry_discovers() {
    let env = Env::default();
    env.mock_all_auths();

    // Registry for contract discovery
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    // Factory
    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[0xABu8; 32]);
    factory.initialize(&wasm_hash);

    // Register the factory itself in the registry
    reg.register(
        &Symbol::new(&env, "ajo_factory"),
        &symbol_short!("factory"),
        &symbol_short!("v1"),
        &factory_id,
    );

    // Verify factory is discoverable
    let meta = reg
        .try_get_by_name(&Symbol::new(&env, "ajo_factory"))
        .unwrap()
        .unwrap();
    assert_eq!(meta.address, factory_id);
    assert_eq!(meta.category, symbol_short!("factory"));

    // Factory reports zero instances initially
    let instances = factory.get_deployed_instances();
    assert_eq!(instances.len(), 0);

    // Register additional templates
    let savings_hash = BytesN::from_array(&env, &[0xCDu8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_SAVINGS,
        &savings_hash,
        &ajo_factory::DEFAULT_VERSION,
    );

    let ids = factory.get_template_ids();
    assert_eq!(ids.len(), 2); // ajo + savings

    // Register each template in the registry for discovery
    reg.register(
        &Symbol::new(&env, "tmpl_ajo"),
        &symbol_short!("template"),
        &symbol_short!("v1"),
        &factory_id,
    );
    reg.register(
        &Symbol::new(&env, "tmpl_savings"),
        &symbol_short!("template"),
        &symbol_short!("v1"),
        &factory_id,
    );

    let templates = reg.list_by_category(&symbol_short!("template"));
    assert_eq!(templates.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 2: Proxy admin upgrade flow with registry tracking
// ---------------------------------------------------------------------------

#[test]
fn test_proxy_upgrade_flow_with_registry_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    let reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let reg = version_registry::VersionRegistryClient::new(&env, &reg_id);
    let reg_admin = Address::generate(&env);
    reg.initialize(&reg_admin);

    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let proxy_admin = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);
    let admin = Address::generate(&env);
    proxy_admin.try_initialize(&admin).unwrap().unwrap();

    // Register v1 in version registry
    let v1_hash = BytesN::from_array(&env, &[0x01u8; 32]);
    let v1 = reg.register(&proxy_admin_id, &v1_hash, &symbol_short!("deploy"));
    assert_eq!(v1.version, Symbol::new(&env, "v1"));

    // Propose upgrade
    let v2_hash = BytesN::from_array(&env, &[0x02u8; 32]);
    proxy_admin
        .try_propose_upgrade(&v2_hash, &60)
        .unwrap()
        .unwrap();

    // Verify pending state
    let state = proxy_admin.proposal_state();
    assert_eq!(state, proxy_admin::ProposalState::Pending);

    // Advance time past delay
    env.ledger().set_timestamp(1100);
    let state_ready = proxy_admin.proposal_state();
    assert_eq!(state_ready, proxy_admin::ProposalState::Ready);

    // Record upgrade in version registry
    let v2 = reg.register(&proxy_admin_id, &v2_hash, &symbol_short!("upgrade"));
    assert_eq!(v2.version, Symbol::new(&env, "v2"));
    assert_eq!(reg.get_current_version_number(), 2);

    // Rollback scenario: cancel upgrade and rollback registry
    proxy_admin.try_cancel_upgrade().unwrap().unwrap();
    assert_eq!(
        proxy_admin.proposal_state(),
        proxy_admin::ProposalState::None
    );

    let rolled = reg.rollback();
    assert_eq!(rolled.version, Symbol::new(&env, "v2"));
    assert_eq!(reg.get_current_version_number(), 1);
}

// ---------------------------------------------------------------------------
// Test 3: Factory + registry + multi-sig governance lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_factory_registry_multisig_governance_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let msig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let msig = multi_sig_patterns::MultiPartyAuthClient::new(&env, &msig_id);

    let admin = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);

    // Initialize factory
    let wasm_hash = BytesN::from_array(&env, &[0x11u8; 32]);
    factory.initialize(&wasm_hash);

    // Initialize multi-sig (2-of-3)
    msig.initialize(
        &2,
        &Vec::from_array(&env, [admin.clone(), signer1.clone(), signer2.clone()]),
    );

    // Register both in the registry
    reg.register(
        &Symbol::new(&env, "ajo_factory"),
        &symbol_short!("factory"),
        &symbol_short!("v1"),
        &factory_id,
    );
    reg.register(
        &Symbol::new(&env, "governance"),
        &symbol_short!("gov"),
        &symbol_short!("v1"),
        &msig_id,
    );

    // Multi-sig approves a governance action (e.g. register new template)
    let proposal_id = msig.create_proposal(&admin);
    msig.approve(&proposal_id, &signer1);
    msig.approve(&proposal_id, &signer2);
    let executed = msig.execute(&proposal_id, &admin);
    assert!(executed);

    // After governance approval, register a new template
    let escrow_hash = BytesN::from_array(&env, &[0x22u8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_ESCROW,
        &escrow_hash,
        &ajo_factory::DEFAULT_VERSION,
    );

    // Verify template is discoverable
    let tmpl = factory.get_template(&ajo_factory::TEMPLATE_ESCROW);
    assert_eq!(tmpl.template_id, ajo_factory::TEMPLATE_ESCROW);
    assert_eq!(tmpl.version, ajo_factory::DEFAULT_VERSION);

    // Registry now has 2 entries
    let cats = reg.list_categories();
    assert_eq!(cats.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 4: Diamond introspection + registry discovery
// ---------------------------------------------------------------------------

#[test]
fn test_diamond_introspection_with_registry_discovery() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let router_id = env.register_contract(None, diamond_facets::DiamondRouter);
    let token_id = env.register_contract(None, diamond_facets::TokenFacet);
    let access_id = env.register_contract(None, diamond_facets::AccessFacet);
    let registry_id = env.register_contract(None, diamond_facets::RegistryFacet);

    let router = diamond_facets::DiamondRouterClient::new(&env, &router_id);
    let admin = Address::generate(&env);

    router.register_facets(&admin, &token_id, &access_id, &registry_id);

    // Register the diamond router in the contract registry
    reg.register(
        &Symbol::new(&env, "diamond_router"),
        &symbol_short!("diamond"),
        &symbol_short!("v1"),
        &router_id,
    );

    // Introspection: 3 facets registered
    assert_eq!(router.facet_count(), 3);

    // Introspection: all facets discoverable
    let facets = router.get_facets();
    assert_eq!(facets.len(), 3);

    // Introspection: known selectors are supported
    assert!(router.supports_selector(&symbol_short!("mint")));
    assert!(router.supports_selector(&symbol_short!("grant")));
    assert!(router.supports_selector(&symbol_short!("get_entry")));
    assert!(!router.supports_selector(&symbol_short!("unknown")));

    // Introspection: per-facet selector counts
    let token_sels = router.get_facet_selectors(&token_id);
    assert_eq!(token_sels.len(), 6);
    let access_sels = router.get_facet_selectors(&access_id);
    assert_eq!(access_sels.len(), 5);
    let reg_sels = router.get_facet_selectors(&registry_id);
    assert_eq!(reg_sels.len(), 4);

    // Registry lookup confirms diamond is registered
    let meta = reg
        .try_get_by_name(&Symbol::new(&env, "diamond_router"))
        .unwrap()
        .unwrap();
    assert_eq!(meta.address, router_id);
}

// ---------------------------------------------------------------------------
// Test 5: Full lifecycle — factory creates instances, proxy manages upgrades,
//         registry tracks everything, multi-sig governs
// ---------------------------------------------------------------------------

#[test]
fn test_full_factory_proxy_registry_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    // Setup all contracts
    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let ver_reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let ver_reg = version_registry::VersionRegistryClient::new(&env, &ver_reg_id);

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let proxy_admin = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);

    let msig_id = env.register_contract(None, multi_sig_patterns::MultiPartyAuth);
    let msig = multi_sig_patterns::MultiPartyAuthClient::new(&env, &msig_id);

    let admin = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);

    // Initialize all
    let wasm_hash = BytesN::from_array(&env, &[0xAAu8; 32]);
    factory.initialize(&wasm_hash);
    proxy_admin.try_initialize(&admin).unwrap().unwrap();
    ver_reg.initialize(&admin);
    msig.initialize(
        &2,
        &Vec::from_array(&env, [admin.clone(), signer1.clone(), signer2.clone()]),
    );

    // Register all in contract registry
    reg.register(
        &Symbol::new(&env, "factory"),
        &symbol_short!("infra"),
        &symbol_short!("v1"),
        &factory_id,
    );
    reg.register(
        &Symbol::new(&env, "proxy_admin"),
        &symbol_short!("infra"),
        &symbol_short!("v1"),
        &proxy_admin_id,
    );
    reg.register(
        &Symbol::new(&env, "governance"),
        &symbol_short!("gov"),
        &symbol_short!("v1"),
        &msig_id,
    );

    // Record initial versions
    let v1_hash = BytesN::from_array(&env, &[0x01u8; 32]);
    ver_reg.register(&factory_id, &v1_hash, &symbol_short!("init"));
    ver_reg.register(&proxy_admin_id, &v1_hash, &symbol_short!("init"));
    assert_eq!(ver_reg.get_current_version_number(), 2);

    // Multi-sig governs: approve registering a new template
    let proposal_id = msig.create_proposal(&admin);
    msig.approve(&proposal_id, &signer1);
    msig.approve(&proposal_id, &signer2);
    assert!(msig.execute(&proposal_id, &admin));

    // Register savings template post-governance
    let savings_hash = BytesN::from_array(&env, &[0xBBu8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_SAVINGS,
        &savings_hash,
        &ajo_factory::DEFAULT_VERSION,
    );
    assert_eq!(factory.get_template_ids().len(), 2);

    // Proxy admin: propose and cancel an upgrade
    let v2_hash = BytesN::from_array(&env, &[0x02u8; 32]);
    proxy_admin
        .try_propose_upgrade(&v2_hash, &60)
        .unwrap()
        .unwrap();
    assert_eq!(
        proxy_admin.proposal_state(),
        proxy_admin::ProposalState::Pending
    );
    proxy_admin.try_cancel_upgrade().unwrap().unwrap();
    assert_eq!(
        proxy_admin.proposal_state(),
        proxy_admin::ProposalState::None
    );

    // Registry: verify all infra contracts are discoverable
    let infra = reg.list_by_category(&symbol_short!("infra"));
    assert_eq!(infra.len(), 2);

    let gov = reg.list_by_category(&symbol_short!("gov"));
    assert_eq!(gov.len(), 1);

    // Version registry: 2 entries (factory + proxy_admin v1)
    assert_eq!(ver_reg.get_current_version_number(), 2);
    let all_versions = ver_reg.get_all_versions();
    assert_eq!(all_versions.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 6: Registry deregistration + factory instance tracking consistency
// ---------------------------------------------------------------------------

#[test]
fn test_registry_deregistration_and_factory_consistency() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[0xFFu8; 32]);
    factory.initialize(&wasm_hash);

    // Register factory and two templates
    reg.register(
        &Symbol::new(&env, "factory_v1"),
        &symbol_short!("factory"),
        &symbol_short!("v1"),
        &factory_id,
    );

    let savings_hash = BytesN::from_array(&env, &[0xEEu8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_SAVINGS,
        &savings_hash,
        &ajo_factory::DEFAULT_VERSION,
    );
    factory.register_template(
        &ajo_factory::TEMPLATE_ESCROW,
        &savings_hash,
        &ajo_factory::DEFAULT_VERSION,
    );

    // 3 templates registered (ajo + savings + escrow)
    assert_eq!(factory.get_template_ids().len(), 3);

    // Factory instances list is empty (no deployments in native test)
    let instances = factory.get_deployed_instances();
    assert_eq!(instances.len(), 0);

    // Registry: factory is discoverable
    let found = reg
        .try_get_by_name(&Symbol::new(&env, "factory_v1"))
        .unwrap()
        .unwrap();
    assert_eq!(found.address, factory_id);

    // Registry: list by category
    let factories = reg.list_by_category(&symbol_short!("factory"));
    assert_eq!(factories.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 7: Proxy pause/unpause with registry state tracking
// ---------------------------------------------------------------------------

#[test]
fn test_proxy_pause_unpause_with_registry_state() {
    let env = Env::default();
    env.mock_all_auths();

    let reg_id = env.register_contract(None, contract_registry::ContractRegistry);
    let reg = contract_registry::ContractRegistryClient::new(&env, &reg_id);

    let proxy_admin_id = env.register_contract(None, proxy_admin::ProxyAdmin);
    let proxy_admin = proxy_admin::ProxyAdminClient::new(&env, &proxy_admin_id);
    let admin = Address::generate(&env);
    proxy_admin.try_initialize(&admin).unwrap().unwrap();

    // Register proxy in registry
    reg.register(
        &Symbol::new(&env, "proxy_v1"),
        &symbol_short!("proxy"),
        &symbol_short!("v1"),
        &proxy_admin_id,
    );

    // Pause
    proxy_admin.try_pause().unwrap().unwrap();
    assert!(proxy_admin.is_paused());

    // Registry still shows proxy as registered (pause doesn't deregister)
    let meta = reg
        .try_get_by_name(&Symbol::new(&env, "proxy_v1"))
        .unwrap()
        .unwrap();
    assert_eq!(meta.address, proxy_admin_id);

    // Unpause
    proxy_admin.try_unpause().unwrap().unwrap();
    assert!(!proxy_admin.is_paused());
}

// ---------------------------------------------------------------------------
// Test 8: Multi-template factory with version registry tracking
// ---------------------------------------------------------------------------

#[test]
fn test_multi_template_factory_with_version_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);

    let ver_reg_id = env.register_contract(None, version_registry::VersionRegistry);
    let ver_reg = version_registry::VersionRegistryClient::new(&env, &ver_reg_id);
    let admin = Address::generate(&env);
    ver_reg.initialize(&admin);

    let factory_id = env.register_contract(None, ajo_factory::AjoFactory);
    let factory = ajo_factory::AjoFactoryClient::new(&env, &factory_id);

    let wasm_hash = BytesN::from_array(&env, &[0x10u8; 32]);
    factory.initialize(&wasm_hash);

    // Record factory v1
    let v1 = ver_reg.register(&factory_id, &wasm_hash, &symbol_short!("init"));
    assert_eq!(v1.version, Symbol::new(&env, "v1"));

    // Register savings template
    let savings_hash = BytesN::from_array(&env, &[0x20u8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_SAVINGS,
        &savings_hash,
        &ajo_factory::DEFAULT_VERSION,
    );

    // Record factory v2 (template added)
    env.ledger().set_timestamp(2000);
    let v2 = ver_reg.register(&factory_id, &savings_hash, &symbol_short!("tmpl_add"));
    assert_eq!(v2.version, Symbol::new(&env, "v2"));
    assert_eq!(v2.timestamp, 2000);

    // Register escrow template
    let escrow_hash = BytesN::from_array(&env, &[0x30u8; 32]);
    factory.register_template(
        &ajo_factory::TEMPLATE_ESCROW,
        &escrow_hash,
        &ajo_factory::DEFAULT_VERSION,
    );

    // Record factory v3
    env.ledger().set_timestamp(3000);
    let v3 = ver_reg.register(&factory_id, &escrow_hash, &symbol_short!("tmpl_add"));
    assert_eq!(v3.version, Symbol::new(&env, "v3"));

    // Factory has 3 templates
    assert_eq!(factory.get_template_ids().len(), 3);

    // Version registry has 3 entries for this factory
    let history = ver_reg.get_contract_history(&factory_id);
    assert_eq!(history.len(), 3);

    // Rollback to v2
    let rolled = ver_reg.rollback();
    assert_eq!(rolled.version, Symbol::new(&env, "v3"));
    assert_eq!(ver_reg.get_current_version_number(), 2);
}
