#![allow(deprecated)]
extern crate std;

use super::*;
use soroban_sdk::{symbol_short, Env, Symbol, Vec};

fn setup() -> (Env, StorageLayoutValidatorClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, StorageLayoutValidator);
    let client = StorageLayoutValidatorClient::new(&env, &contract_id);
    (env, client)
}

fn field(env: &Env, key: Symbol, field_type: FieldType) -> FieldDef {
    let _ = env;
    FieldDef { key, field_type }
}

fn layout(env: &Env, version: u32, fields: Vec<FieldDef>) -> StorageLayout {
    let _ = env;
    StorageLayout { version, fields }
}

fn step_op(steps: &Vec<MigrationStep>, key: Symbol) -> Option<MigrationOp> {
    for step in steps.iter() {
        if step.key == key {
            return Some(step.op);
        }
    }
    None
}

#[test]
fn test_compatible_addition() {
    let (env, client) = setup();

    let current = layout(
        &env,
        1,
        Vec::from_array(&env, [field(&env, symbol_short!("foo"), FieldType::U32)]),
    );
    let next = layout(
        &env,
        2,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("foo"), FieldType::U32),
                field(&env, symbol_short!("bar"), FieldType::Symbol),
            ],
        ),
    );

    let report = client.try_validate(&current, &next).unwrap().unwrap();
    assert!(report.compatible);
    assert!(report.collisions.is_empty());
    assert_eq!(
        step_op(&report.steps, symbol_short!("foo")),
        Some(MigrationOp::Retain)
    );
    assert_eq!(
        step_op(&report.steps, symbol_short!("bar")),
        Some(MigrationOp::Add)
    );
}

#[test]
fn test_type_change_incompatible() {
    let (env, client) = setup();

    let current = layout(
        &env,
        1,
        Vec::from_array(&env, [field(&env, symbol_short!("foo"), FieldType::U32)]),
    );
    let next = layout(
        &env,
        2,
        Vec::from_array(&env, [field(&env, symbol_short!("foo"), FieldType::I128)]),
    );

    let report = client.try_validate(&current, &next).unwrap().unwrap();
    assert!(!report.compatible);
    assert_eq!(
        step_op(&report.steps, symbol_short!("foo")),
        Some(MigrationOp::TypeChange)
    );
}

#[test]
fn test_find_collisions_deduplicates() {
    let (env, client) = setup();

    let layout_with_dups = layout(
        &env,
        1,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("a"), FieldType::U32),
                field(&env, symbol_short!("b"), FieldType::I128),
                field(&env, symbol_short!("a"), FieldType::U32),
                field(&env, symbol_short!("c"), FieldType::Symbol),
                field(&env, symbol_short!("b"), FieldType::I128),
            ],
        ),
    );

    let collisions = client.find_collisions(&layout_with_dups);
    assert_eq!(collisions.len(), 2);
    assert_eq!(collisions.get(0).unwrap(), symbol_short!("a"));
    assert_eq!(collisions.get(1).unwrap(), symbol_short!("b"));
}

#[test]
fn test_validate_rejects_next_collisions() {
    let (env, client) = setup();

    let current = layout(
        &env,
        1,
        Vec::from_array(&env, [field(&env, symbol_short!("foo"), FieldType::U32)]),
    );
    let next = layout(
        &env,
        2,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("foo"), FieldType::U32),
                field(&env, symbol_short!("foo"), FieldType::U32),
            ],
        ),
    );

    let report = client.try_validate(&current, &next).unwrap().unwrap();
    assert!(!report.compatible);
    assert_eq!(report.collisions.len(), 1);
    assert_eq!(report.collisions.get(0).unwrap(), symbol_short!("foo"));
    assert!(report.steps.is_empty());
}

#[test]
fn test_removal_incompatible() {
    let (env, client) = setup();

    let current = layout(
        &env,
        1,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("foo"), FieldType::U32),
                field(&env, symbol_short!("bar"), FieldType::Symbol),
            ],
        ),
    );
    let next = layout(
        &env,
        2,
        Vec::from_array(&env, [field(&env, symbol_short!("foo"), FieldType::U32)]),
    );

    let report = client.try_validate(&current, &next).unwrap().unwrap();
    assert!(!report.compatible);
    assert_eq!(
        step_op(&report.steps, symbol_short!("bar")),
        Some(MigrationOp::Remove)
    );
}

#[test]
fn test_malformed_current_layout() {
    let (env, client) = setup();

    let current = layout(&env, 0, Vec::new(&env));
    let next = layout(&env, 1, Vec::new(&env));

    assert_eq!(
        client.try_validate(&current, &next),
        Err(Ok(LayoutError::MalformedLayout))
    );
}

#[test]
fn test_malformed_next_layout() {
    let (env, client) = setup();

    let current = layout(&env, 1, Vec::new(&env));
    let next = layout(&env, 0, Vec::new(&env));

    assert_eq!(
        client.try_validate(&current, &next),
        Err(Ok(LayoutError::MalformedLayout))
    );
}

#[test]
fn test_multiple_simultaneous_changes() {
    let (env, client) = setup();

    // current: keep, change, remove
    let current = layout(
        &env,
        1,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("keep"), FieldType::U32),
                field(&env, symbol_short!("chg"), FieldType::U32),
                field(&env, symbol_short!("gone"), FieldType::Symbol),
            ],
        ),
    );
    // next: keep (retain), chg (type change), plus (add)
    let next = layout(
        &env,
        2,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("keep"), FieldType::U32),
                field(&env, symbol_short!("chg"), FieldType::I128),
                field(&env, symbol_short!("plus"), FieldType::Bytes),
            ],
        ),
    );

    let report = client.try_validate(&current, &next).unwrap().unwrap();
    assert!(!report.compatible);
    assert_eq!(
        step_op(&report.steps, symbol_short!("keep")),
        Some(MigrationOp::Retain)
    );
    assert_eq!(
        step_op(&report.steps, symbol_short!("plus")),
        Some(MigrationOp::Add)
    );
    assert_eq!(
        step_op(&report.steps, symbol_short!("chg")),
        Some(MigrationOp::TypeChange)
    );
    assert_eq!(
        step_op(&report.steps, symbol_short!("gone")),
        Some(MigrationOp::Remove)
    );
}

#[test]
fn test_empty_layouts() {
    let (env, client) = setup();

    let empty_v1 = layout(&env, 1, Vec::new(&env));
    let empty_v2 = layout(&env, 2, Vec::new(&env));
    let populated = layout(
        &env,
        2,
        Vec::from_array(
            &env,
            [field(&env, symbol_short!("foo"), FieldType::Address)],
        ),
    );

    let empty_to_empty = client.try_validate(&empty_v1, &empty_v2).unwrap().unwrap();
    assert!(empty_to_empty.compatible);
    assert!(empty_to_empty.steps.is_empty());

    let empty_to_populated = client.try_validate(&empty_v1, &populated).unwrap().unwrap();
    assert!(empty_to_populated.compatible);
    assert_eq!(
        step_op(&empty_to_populated.steps, symbol_short!("foo")),
        Some(MigrationOp::Add)
    );

    let populated_to_empty = client.try_validate(&populated, &empty_v1).unwrap().unwrap();
    assert!(!populated_to_empty.compatible);
    assert_eq!(
        step_op(&populated_to_empty.steps, symbol_short!("foo")),
        Some(MigrationOp::Remove)
    );
}

#[test]
fn test_find_collisions_clean_layout() {
    let (env, client) = setup();

    let clean = layout(
        &env,
        1,
        Vec::from_array(
            &env,
            [
                field(&env, symbol_short!("a"), FieldType::U32),
                field(&env, symbol_short!("b"), FieldType::I128),
                field(&env, symbol_short!("c"), FieldType::Symbol),
            ],
        ),
    );

    let collisions = client.find_collisions(&clean);
    assert!(collisions.is_empty());
}
