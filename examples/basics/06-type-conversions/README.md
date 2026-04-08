# Type Conversions in Soroban

Demonstrates the full range of type conversion patterns available in Soroban
smart contracts: `Val` conversions, `TryFrom`/`TryInto`, native Rust → Soroban
SDK types, and proper error handling throughout.

## What You'll Learn

- `Val` conversions — `IntoVal` / `TryFromVal` for the universal host value type
- `TryFrom`/`TryInto` — safe numeric narrowing with overflow detection
- Native → Soroban types — `String`, `Symbol`, `Bytes`, `Vec`, `Map`, `Address`
- Error handling — `ConversionError` variants, `Result` returns, panic messages
- Batch conversions — per-element error skipping without aborting the whole call

## Key Concepts

### Val Conversions

`Val` is Soroban's universal tagged value. Every type that crosses the
host–guest boundary is encoded as a `Val`.

```rust
// native → Val
let val: Val = 42u32.into_val(&env);

// Val → native (safe, returns Result)
let n: u32 = u32::try_from_val(&env, &val).unwrap_or(0);
```

### TryFrom / TryInto

Standard Rust traits for fallible narrowing conversions. Soroban contracts
use them to safely downcast numeric types without panicking on overflow.

```rust
// i128 → u32: rejects negatives and values > u32::MAX
let small: u32 = large_i128
    .try_into()
    .unwrap_or_else(|_| panic!("NumericOverflow"));
```

### Native to Soroban Types

| Rust type | Soroban type | Notes |
|-----------|-------------|-------|
| `&str`    | `String`    | `String::from_str(&env, "…")` |
| `&str`    | `Symbol`    | `Symbol::new(&env, "…")` — max 32 chars |
| `&[u8]`   | `Bytes`     | `Bytes::from_slice(&env, …)` |
| `Vec<T>`  | `Vec<T>`    | Element-by-element with `push_back` |

### Error Handling in Conversions

```rust
#[contracterror]
#[repr(u32)]
pub enum ConversionError {
    NumericOverflow      = 1,
    InvalidStringFormat  = 2,
    UnsupportedConversion = 3,
    CollectionTooLarge   = 4,
    InvalidAddress       = 5,
}
```

Use `panic!("VariantName")` for invariant violations (the host maps the
string to the matching `#[contracterror]` variant). Use `Result<T, E>` when
the caller should be able to handle the failure gracefully.

## Contract Functions

| Function | Demonstrates |
|----------|-------------|
| `convert_numbers` | `TryInto` for u32 / i64 / u128 with overflow detection |
| `convert_strings` | `String` ↔ `Symbol` boundary |
| `convert_collections` | `Vec<i32>` → `Vec<i64>` widening |
| `safe_conversions` | `TryFromVal` on a raw `Val` |
| `create_user_data` | Struct construction with validated field conversions |
| `convert_val_to_config` | `Map<Symbol, Val>` → typed struct |
| `convert_bytes_to_types` | `Bytes` → `String` / `Symbol` |
| `validate_and_convert` | Type-directed string validation |
| `batch_convert_numbers` | Best-effort batch parse, failures skipped |
| `sum_different_types` | Widening `u32` + `i64` → `i128` via `From` |
| `val_roundtrip` | Full `u32` → `Val` → `u32` roundtrip |

## Testing

```bash
cargo test -p type-conversions
```

## Building

```bash
cargo build -p type-conversions --target wasm32-unknown-unknown --release
```

## Related Examples

- [`03-custom-errors`](../03-custom-errors/) — `#[contracterror]` patterns
- [`09-primitive-types`](../09-primitive-types/) — integer overflow safety
- [`06-soroban-types`](../06-soroban-types/) — core SDK type operations
