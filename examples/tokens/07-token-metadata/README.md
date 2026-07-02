# Token Metadata

Extends a base token with full metadata support: name, symbol, decimals, and
an optional URI. The admin who initialises the contract may update the mutable
fields at any time. Decimals are intentionally immutable after initialisation
because changing them would silently reinterpret every stored balance.

## What It Demonstrates

- Storing and querying structured metadata (`name`, `symbol`, `decimals`, `uri`)
- Immutability enforcement for `decimals` at the contract level
- Admin-gated metadata updates with `require_auth()`
- Emitting typed events on initialisation, metadata changes, mint, burn, and transfer
- Separating instance storage (config/metadata) from persistent storage (per-user balances)

## Contract API

| Function | Auth required | Description |
| --- | --- | --- |
| `initialize(admin, name, symbol, decimals, uri)` | — | One-time setup; decimals are locked here |
| `metadata()` | — | Returns all fields in a single `TokenMetadata` struct |
| `name()` | — | Returns the token name |
| `symbol()` | — | Returns the token symbol |
| `decimals()` | — | Returns the immutable decimal count |
| `uri()` | — | Returns the optional metadata URI |
| `update_metadata(name, symbol, uri)` | admin | Updates mutable fields; decimals cannot be changed |
| `mint(to, amount)` | admin | Mints tokens to an address |
| `burn(from, amount)` | `from` | Burns tokens from an address |
| `transfer(from, to, amount)` | `from` | Transfers tokens between addresses |
| `balance(user)` | — | Returns a user's token balance |
| `total_supply()` | — | Returns total tokens in circulation |

## Storage Layout

| Key | Storage type | Mutable |
| --- | --- | --- |
| `Admin` | instance | no |
| `Name` | instance | yes |
| `Symbol` | instance | yes |
| `Decimals` | instance | **no** |
| `Uri` | instance | yes |
| `Balance(addr)` | persistent | yes |
| `TotalSupply` | instance | yes |

Instance storage is used for all metadata because it is read on almost every
call and benefits from the lower cost of instance reads. Per-user balances use
persistent storage so they survive ledger TTL expiry of the instance entry.

## Why Decimals Are Immutable

```rust
// Decimals are set once and never exposed through an update path.
// Changing decimals after tokens are in circulation would mean that
// a balance of 1_000_0000000 (7 decimals = 1000.0) silently becomes
// 1_000_0000000 (6 decimals = 10000.0) — a 10× reinterpretation with
// no on-chain record of the change.
pub fn decimals(env: Env) -> Result<u32, MetadataError> {
    read_decimals(&env)
}
```

## Governance Rules

- Only the `admin` address stored at initialisation can call `update_metadata` or `mint`.
- `burn` is self-service: any token holder can burn their own balance.
- There is no admin-transfer function in this example; add one following the
  pattern in `examples/intermediate/multi-sig-patterns` if your token requires it.

## Run Tests

```bash
cargo test -p token-metadata
```
