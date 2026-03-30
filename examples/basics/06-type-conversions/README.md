# Type Conversions in Soroban

This example demonstrates type conversion patterns in Soroban smart contracts.

## What You'll Learn

- Val conversions between Soroban types
- TryFrom/TryInto for safe conversions
- Converting native Rust types to Soroban types
- Error handling in conversions

## Key Functions

- `convert_numbers()` - Numeric type conversions with overflow checking
- `convert_strings()` - String and Symbol conversions
- `safe_conversions()` - Error-safe Val conversions
- `val_roundtrip()` - Roundtrip conversion through Val

## Usage

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
```