#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Env, Symbol, Vec};

/// Declared storage field types supported by the layout schema.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldType {
    U32,
    I128,
    Address,
    Symbol,
    Bytes,
}

/// A single declared storage field: key plus value type.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDef {
    pub key: Symbol,
    pub field_type: FieldType,
}

/// A versioned storage-layout schema declared in code.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageLayout {
    pub version: u32,
    pub fields: Vec<FieldDef>,
}

/// Per-key migration operation derived from comparing two layouts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationOp {
    /// Key exists in both layouts with the same type.
    Retain,
    /// Key exists only in the next layout.
    Add,
    /// Key exists in both layouts but the field type changed.
    TypeChange,
    /// Key exists only in the current layout.
    Remove,
}

/// One step in a layout migration plan.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationStep {
    pub key: Symbol,
    pub op: MigrationOp,
}

/// Result of comparing a current layout to a proposed next layout.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub compatible: bool,
    pub collisions: Vec<Symbol>,
    pub steps: Vec<MigrationStep>,
}

/// Errors raised when a layout schema itself is invalid.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LayoutError {
    /// Layout version is zero (or otherwise unusable).
    MalformedLayout = 1,
}

#[contract]
pub struct StorageLayoutValidator;

#[contractimpl]
impl StorageLayoutValidator {
    /// Compare `current` → `next` and return a compatibility report.
    ///
    /// This validates **declared** schemas only. It does not inspect live
    /// ledger storage or arbitrary deployed contracts.
    ///
    /// Returns `LayoutError::MalformedLayout` when either layout has
    /// `version == 0`.
    ///
    /// Duplicate keys in `next` short-circuit validation: the report is
    /// incompatible, collisions are populated, and no migration steps are
    /// produced. The `current` layout is assumed to already be a valid schema.
    pub fn validate(
        env: Env,
        current: StorageLayout,
        next: StorageLayout,
    ) -> Result<ValidationReport, LayoutError> {
        if current.version == 0 || next.version == 0 {
            return Err(LayoutError::MalformedLayout);
        }

        let collisions = Self::find_collisions(env.clone(), next.clone());
        if !collisions.is_empty() {
            return Ok(ValidationReport {
                compatible: false,
                collisions,
                steps: Vec::new(&env),
            });
        }

        let mut steps = Vec::new(&env);
        let mut compatible = true;

        // Walk current fields: Retain / TypeChange / Remove.
        for field in current.fields.iter() {
            match find_field_type(&next.fields, &field.key) {
                Some(next_ty) if next_ty == field.field_type => {
                    steps.push_back(MigrationStep {
                        key: field.key,
                        op: MigrationOp::Retain,
                    });
                }
                Some(_) => {
                    steps.push_back(MigrationStep {
                        key: field.key,
                        op: MigrationOp::TypeChange,
                    });
                    compatible = false;
                }
                None => {
                    steps.push_back(MigrationStep {
                        key: field.key,
                        op: MigrationOp::Remove,
                    });
                    compatible = false;
                }
            }
        }

        // Walk next fields: Add anything not present in current.
        for field in next.fields.iter() {
            if find_field_type(&current.fields, &field.key).is_none() {
                steps.push_back(MigrationStep {
                    key: field.key,
                    op: MigrationOp::Add,
                });
            }
        }

        Ok(ValidationReport {
            compatible,
            collisions: Vec::new(&env),
            steps,
        })
    }

    /// Return each duplicated field key exactly once, in first-seen order.
    ///
    /// For example `[a, b, a, c, b]` yields `[a, b]`.
    pub fn find_collisions(env: Env, layout: StorageLayout) -> Vec<Symbol> {
        let mut collisions = Vec::new(&env);
        let fields = &layout.fields;
        let n = fields.len();

        for i in 0..n {
            let key = fields.get(i).unwrap().key;

            // Skip keys already considered at an earlier index.
            let mut seen_earlier = false;
            for j in 0..i {
                if fields.get(j).unwrap().key == key {
                    seen_earlier = true;
                    break;
                }
            }
            if seen_earlier {
                continue;
            }

            let mut count = 0u32;
            for j in 0..n {
                if fields.get(j).unwrap().key == key {
                    count += 1;
                }
            }
            if count > 1 {
                collisions.push_back(key);
            }
        }

        collisions
    }
}

fn find_field_type(fields: &Vec<FieldDef>, key: &Symbol) -> Option<FieldType> {
    for field in fields.iter() {
        if field.key == *key {
            return Some(field.field_type);
        }
    }
    None
}

#[cfg(test)]
mod test;
