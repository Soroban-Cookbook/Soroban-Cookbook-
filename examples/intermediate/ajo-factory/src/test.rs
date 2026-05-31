use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

struct Fixture {
    env: Env,
    factory: AjoFactoryClient<'static>,
    ajo_hash: BytesN<32>,
    savings_hash: BytesN<32>,
    escrow_hash: BytesN<32>,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
A
    let template_wasm = [0u8; 0];
    let ajo_hash = env
        .deployer()
        .upload_contract_wasm(template_wasm.as_slice());
    let savings_hash = ajo_hash.clone();
    let escrow_hash = ajo_hash.clone();

    let factory_id = env.register_contract(None, AjoFactory);
    let factory = AjoFactoryClient::new(&env, &factory_id);
    factory.initialize(&ajo_hash);

    Fixture {
        env,
        factory,
        ajo_hash,
        savings_hash,
        escrow_hash,
    }
}

#[test]
fn test_ajo_factory_workflow() {
    let f = setup();
    let creator = Address::generate(&f.env);

    let ajo_address = f.factory.create_ajo(&1_000, &10, &creator);

    let deployed_ajos = f.factory.get_deployed_ajos();
    assert_eq!(deployed_ajos.len(), 1);
    assert_eq!(deployed_ajos.get(0).unwrap(), ajo_address);

    let ajo_address2 = f.factory.create_ajo(&2_000, &10, &creator);
    assert_ne!(ajo_address, ajo_address2);
    assert_eq!(f.factory.get_deployed_ajos().len(), 2);

    let instances = f.factory.get_deployed_instances();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances.get(0).unwrap().template_id, TEMPLATE_AJO);
    assert_eq!(instances.get(0).unwrap().creator, creator);
    assert_eq!(instances.get(1).unwrap().template_id, TEMPLATE_AJO);
}

#[test]
fn test_default_ajo_template_metadata_is_registered() {
    let f = setup();

    let metadata = f.factory.get_template(&TEMPLATE_AJO);
    assert_eq!(metadata.template_id, TEMPLATE_AJO);
    assert_eq!(metadata.version, DEFAULT_VERSION);
    assert_eq!(metadata.wasm_hash, f.ajo_hash);

    let ids = f.factory.get_template_ids();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), TEMPLATE_AJO);
}

#[test]
fn test_register_templates_and_create_multiple_contract_types() {
    let f = setup();
    let creator = Address::generate(&f.env);
    let beneficiary = Address::generate(&f.env);

    f.factory
        .register_template(&TEMPLATE_SAVINGS, &f.savings_hash, &DEFAULT_VERSION);
    f.factory
        .register_template(&TEMPLATE_ESCROW, &f.escrow_hash, &DEFAULT_VERSION);

    let savings_address = f.factory.create_instance(
        &TEMPLATE_SAVINGS,
        &TemplateParams::Savings(SavingsParams {
            target_amount: 5_000,
            deadline: 1_800_000_000,
        }),
        &creator,
    );
    let escrow_address = f.factory.create_instance(
        &TEMPLATE_ESCROW,
        &TemplateParams::Escrow(EscrowParams {
            beneficiary: beneficiary.clone(),
            amount: 750,
        }),
        &creator,
    );

    assert_ne!(savings_address, escrow_address);

    let instances = f.factory.get_deployed_instances();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances.get(0).unwrap().template_id, TEMPLATE_SAVINGS);
    assert_eq!(instances.get(1).unwrap().template_id, TEMPLATE_ESCROW);
}

#[test]
fn test_factory_cannot_be_reinitialized() {
    let f = setup();

    let result = f.factory.try_initialize(&f.ajo_hash);
    assert_eq!(result, Err(Ok(FactoryError::AlreadyInitialized)));
}

#[test]
fn test_template_registration_rejects_duplicates() {
    let f = setup();

    let result = f
        .factory
        .try_register_template(&TEMPLATE_AJO, &f.ajo_hash, &DEFAULT_VERSION);
    assert_eq!(result, Err(Ok(FactoryError::TemplateAlreadyRegistered)));
}

#[test]
fn test_create_instance_rejects_unknown_template() {
    let f = setup();
    let creator = Address::generate(&f.env);

    let result = f.factory.try_create_instance(
        &symbol_short!("missing"),
        &TemplateParams::Ajo(AjoParams {
            amount: 100,
            max_members: 5,
        }),
        &creator,
    );

    assert_eq!(result, Err(Ok(FactoryError::TemplateNotFound)));
}

#[test]
fn test_create_instance_rejects_template_param_mismatch() {
    let f = setup();
    let creator = Address::generate(&f.env);

    let result = f.factory.try_create_instance(
        &TEMPLATE_AJO,
        &TemplateParams::Savings(SavingsParams {
            target_amount: 100,
            deadline: 10,
        }),
        &creator,
    );

    assert_eq!(result, Err(Ok(FactoryError::InvalidTemplateParams)));
}

#[test]
fn test_parameter_validation() {
    let f = setup();
    let creator = Address::generate(&f.env);

    assert_eq!(
        f.factory.try_create_instance(
            &TEMPLATE_AJO,
            &TemplateParams::Ajo(AjoParams {
                amount: 0,
                max_members: 5,
            }),
            &creator,
        ),
        Err(Ok(FactoryError::InvalidAmount))
    );
    assert_eq!(
        f.factory.try_create_instance(
            &TEMPLATE_AJO,
            &TemplateParams::Ajo(AjoParams {
                amount: 100,
                max_members: 1,
            }),
            &creator,
        ),
        Err(Ok(FactoryError::InvalidMaxMembers))
    );

    f.factory
        .register_template(&TEMPLATE_SAVINGS, &f.savings_hash, &DEFAULT_VERSION);

    assert_eq!(
        f.factory.try_create_instance(
            &TEMPLATE_SAVINGS,
            &TemplateParams::Savings(SavingsParams {
                target_amount: 100,
                deadline: 0,
            }),
            &creator,
        ),
        Err(Ok(FactoryError::InvalidDeadline))
    );
}

#[test]
fn test_ajo_cannot_be_reinitialized() {
    let f = setup();
    let creator = Address::generate(&f.env);

    let ajo_address = f.env.register_contract(None, Ajo);
    let ajo_client = AjoClient::new(&f.env, &ajo_address);

    ajo_client.init_ajo(&100, &10, &creator);
    let result = ajo_client.try_init_ajo(&100, &10, &creator);
    assert_eq!(result, Err(Ok(FactoryError::AlreadyInitialized)));
}
