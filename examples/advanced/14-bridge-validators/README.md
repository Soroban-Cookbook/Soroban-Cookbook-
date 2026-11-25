# Bridge Validators

This example implements a validator registry and multi-signature threshold verification system, typically used in cross-chain bridges.
Validators can be added, removed, and slashed by an admin.
Messages require a certain threshold of voting power to be processed.

## Features

- Multi-signature validation
- Threshold signatures
- Validator registry
- Validator rotation
- Validator removal
- Slashing mechanism
- Signature verification

## Usage

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
```
