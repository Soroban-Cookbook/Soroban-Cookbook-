# Structured Event Patterns

This example demonstrates production-style Soroban event design with clear schemas for off-chain indexers.

## Acceptance Coverage

- Custom event types: `#[contracttype]` payload structs for transfer/config/admin/audit events.
- Multiple topics: examples using 3 and 4 topic slots.
- Indexed parameters: searchable identifiers (addresses, keys, action names) in topics.
- Event naming conventions: stable `(namespace, action, [indexes...])` layout.

## Event Model

In Soroban, each event is:

- `topics` (indexed, up to 4): filter keys used by indexers.
- `data` (not indexed): rich payload decoded after filtering.

```rust
env.events().publish((topic0, topic1, topic2, topic3), data);
```

## Naming Convention

Structured contract events in this example use:

- `topic[0]` = namespace (`"events"`)
- `topic[1]` = action (`"transfer"`, `"cfg_upd"`, `"admin"`, `"audit"`)
- `topic[2..]` = indexed entities (address/key/action)

Why this matters:

- Consistent filters across event families.
- Stable schema for indexers and analytics pipelines.
- Easier backward compatibility when adding new event types.

## Structured APIs

```rust
pub fn transfer(env: Env, sender: Address, recipient: Address, amount: i128, memo: u64)
pub fn update_config(env: Env, key: Symbol, old_value: u64, new_value: u64)
pub fn admin_action(env: Env, admin: Address, action: Symbol)
pub fn audit_trail(env: Env, actor: Address, action: Symbol, details: Symbol)
```

## Payload Types

Each structured event stores a typed payload in `data`:

- `TransferEventData { amount, memo }`
- `ConfigUpdateEventData { old_value, new_value }`
- `AdminActionEventData { action, timestamp }`
- `AuditTrailEventData { details, timestamp, sequence }`

## Topic Layout Examples

- `transfer`: `(events, transfer, sender, recipient)`
- `update_config`: `(events, cfg_upd, key)`
- `admin_action`: `(events, admin, admin_address)`
- `audit_trail`: `(events, audit, actor, action)`

## Run Tests

```bash
cargo test -p events
```

Tests validate:

- topic count and order
- indexed parameter placement
- payload decoding into custom types
- naming convention stability
