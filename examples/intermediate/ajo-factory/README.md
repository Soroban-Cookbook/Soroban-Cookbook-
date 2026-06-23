# Factory Templates Pattern

This example demonstrates how to build a Soroban factory that manages versioned deployment templates. A factory can register multiple template IDs, validate template-specific parameters, deploy an instance from the registered Wasm hash, and keep metadata for every created instance.

## Features

- Versioned template metadata with `TemplateMetadata`
- Template registry keyed by `Symbol` IDs such as `ajo`, `savings`, and `escrow`
- Generic deployment through `deploy_template(creator, template_id, version, params)`
- Template-specific parameter validation before deployment
- Instance tracking with template ID, version, deployed address, and creator

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
    &admin,
    &symbol_short!("savings"),
    &symbol_short!("v1"),
    &wasm_hash,
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
let address = factory_client.deploy_template(
    &creator,
    &symbol_short!("savings"),
    &symbol_short!("v1"),
    &TemplateParams::Savings(SavingsParams {
        target_amount: 5_000,
        deadline: 1_800_000_000,
    }),
);
```

The factory checks that:

- The template ID is registered
- The supplied parameter variant matches the template ID
- The parameters satisfy that template's validation rules

If validation succeeds, the factory deploys the registered Wasm hash and records the new instance.

## Run Tests

```bash
cargo test -p ajo-factory
```

For contract build validation:

```bash
cargo build -p ajo-factory
```
