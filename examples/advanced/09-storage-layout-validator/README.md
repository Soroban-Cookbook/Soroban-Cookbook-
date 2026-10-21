# Storage Layout Validator

This advanced example shows how a Soroban contract can compare two **declared**
storage-layout schemas and decide whether an upgrade is backwards-compatible.

## What it demonstrates

Given a `current` layout and a proposed `next` layout, the
`StorageLayoutValidator` contract produces:

1. A compatibility status
2. Collision information (duplicate keys in `next`)
3. A per-key migration plan (`Retain`, `Add`, `TypeChange`, `Remove`)

Use this pattern when designing upgradeable contracts so authors can catch
breaking storage changes before deploying a new WASM.

## Important limitation

> The validator compares layouts declared in code. It does not inspect live
> ledger entries from arbitrary deployments.

Schemas are values you construct (or hard-code) in the contract call. This
example is a teaching tool for schema compatibility rules, not a ledger
introspection utility.

## Types

| Type | Role |
| --- | --- |
| `FieldType` | `U32`, `I128`, `Address`, `Symbol`, `Bytes` |
| `FieldDef` | Declared field: `key` + `field_type` |
| `StorageLayout` | Versioned list of `FieldDef`s |
| `MigrationOp` | `Retain` / `Add` / `TypeChange` / `Remove` |
| `MigrationStep` | One key + one migration op |
| `ValidationReport` | `compatible`, `collisions`, `steps` |
| `LayoutError` | `MalformedLayout` when `version == 0` |

## Compatibility rules

| Change | Migration operation | Compatible? |
| --- | --- | --- |
| Same key + same type | `Retain` | Yes |
| New key | `Add` | Yes |
| Existing key + changed type | `TypeChange` | No |
| Existing key removed | `Remove` | No |
| Duplicate key in `next` | collision | No |

`compatible == true` only when the migration plan has no `TypeChange` or
`Remove`, and `next` has no duplicate keys.

A duplicate key in `next` short-circuits validation: the report is
incompatible, `collisions` lists each duplicated key once (first-seen order),
and no migration steps are produced. The `current` layout is assumed to already
represent a valid schema.

Layouts with `version == 0` are rejected as `LayoutError::MalformedLayout`.

## Migration guidance

The generated plan helps an upgrade author see:

- **Safe additions** — new keys marked `Add` that do not break existing data
- **Unchanged fields** — keys marked `Retain` that keep the same type
- **Breaking type changes** — keys marked `TypeChange` that need a data rewrite
  or a new key instead of an in-place type swap
- **Removals** — keys marked `Remove` that would leave orphaned or unreadable
  ledger entries unless migrated deliberately

Collisions in `next` should be fixed in the schema before interpreting any
migration steps.

## Contract API

```text
validate(current, next) -> Result<ValidationReport, LayoutError>
find_collisions(layout) -> Vec<Symbol>
```

## Testing

```bash
cargo test -p storage-layout-validator
cargo build --target wasm32-unknown-unknown --release -p storage-layout-validator
```

## Related examples

- [`01-multi-party-auth`](../01-multi-party-auth/) — multi-party authorization patterns
- [`02-timelock`](../02-timelock/) — time-delayed execution and upgrade-friendly controls
- [`08-batch-operations`](../08-batch-operations/) — typed batch operations with structured results
