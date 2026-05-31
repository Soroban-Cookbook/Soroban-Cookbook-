# Hello World Contract

This is a minimal Soroban smart contract that demonstrates how to accept input and return a simple vector. When called with a target name, it returns a vector containing the greeting "Hello" and the provided name.

## Building

To build the contract into a WebAssembly (WASM) module, run the following command from this directory:

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

To run the contract's tests and ensure everything is working correctly, run:

```bash
cargo test
```

## Next Steps

For more details on writing Soroban contracts and advanced concepts, check out the [Soroban Documentation](https://soroban.stellar.org/docs).
