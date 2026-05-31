# Factory Templates Pattern

This example demonstrates how to build a Soroban factory that manages versioned deployment templates. A factory can register multiple template IDs, validate template-specific parameters, deploy an instance from the registered Wasm hash, and keep metadata for every created instance.

The example keeps the original `create_ajo(amount, max_members, creator)` helper, then adds a generic `create_instance(template_id, params, creator)` flow for multiple templates.

## Features

- Versioned template metadata with `TemplateMetadata`
- Template registry keyed by `Symbol` IDs such as `ajo`, `savings`, and `escrow`
- Generic deployment through `create_instance(template_id, params, creator)`
- Template-specific parameter validation before deployment
- Instance tracking with template ID, version, deployed address, and creator
- Backwards-compatible `create_ajo` helper for the original Ajo workflow

## Template IDs

| Template | Parameter Variant | Validation |
| --- | --- | --- |
| `ajo` | `TemplateParams::Ajo(AjoParams)` | `amount > 0`, `max_members >= 2` |
| `savings` | `TemplateParams::Savings(SavingsParams)` | `target_amount > 0`, `deadline > 0` |
| `escrow` | `TemplateParams::Escrow(EscrowParams)` | `amount > 0` |

## Register a Template

Upload the template contract Wasm once, then register the hash with an ID and version:

```rust
let wasm_hash = env.deployer().upload_contract_wasm(template_wasm);

factory_client.register_template(
    &symbol_short!("savings"),
    &wasm_hash,
    &symbol_short!("v1"),
);
```

The factory stores:

```rust
pub struct TemplateMetadata {
    pub template_id: Symbol,
    pub version: Symbol,
    pub wasm_hash: BytesN<32>,
}
```

## Create an Instance

```rust
let address = factory_client.create_instance(
    &symbol_short!("savings"),
    &TemplateParams::Savings(SavingsParams {
        target_amount: 5_000,
        deadline: 1_800_000_000,
    }),
    &creator,
);
```

The factory checks that:

- The template ID is registered
- The supplied parameter variant matches the template ID
- The parameters satisfy that template's validation rules

If validation succeeds, the factory deploys the registered Wasm hash and records the new instance.

## Add a New Template

1. Define a new template ID constant, for example `pub const TEMPLATE_VAULT: Symbol = symbol_short!("vault");`.
2. Add a `#[contracttype]` params struct for that template.
3. Add a new `TemplateParams` enum variant wrapping the params struct.
4. Extend `validate_template_params` with the new template ID and validation rules.
5. Register the new template hash with `register_template(template_id, wasm_hash, version)`.
6. Add tests for successful creation, metadata lookup, parameter validation, and mismatched parameter variants.

## Run Tests

The workspace defaults to the Wasm target for contract builds. Unit tests that use `soroban-sdk/testutils` must run on a native target:

```bash
cargo test -p ajo-factory --target aarch64-apple-darwin
```

For contract build validation:

```bash
cargo build -p ajo-factory
```
