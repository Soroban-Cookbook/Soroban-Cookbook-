# Playground Tutorial 1: Your First Smart Contract

## Overview
This step-by-step tutorial will guide you through creating and deploying your first Soroban smart contract using the interactive playground.

**Duration**: 15-20 minutes
**Difficulty**: Beginner
**Prerequisites**: Basic understanding of Rust syntax

## Learning Objectives
- Set up a contract in the playground
- Write a simple "Hello World" contract
- Deploy to a local network
- Interact with the deployed contract

## Step 1: Access the Playground

1. Navigate to the Soroban Cookbook Playground
2. You should see the editor interface with a default contract template
3. Take a moment to explore the interface:
   - **Editor panel** (left): Write your Rust code
   - **Terminal** (bottom): Build and deploy commands
   - **Contract status** (right): Deployment information

## Step 2: Write Your First Contract

Replace the template code with this simple hello world contract:

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Symbol, String as SorobanString};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(name: SorobanString) -> SorobanString {
        SorobanString::from_slice(&format!("Hello, {}!", name.clone()).as_str())
    }
    
    pub fn get_message() -> SorobanString {
        SorobanString::from_slice("Welcome to Soroban!")
    }
}
```

## Step 3: Build Your Contract

1. In the terminal, run:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Watch for the build output. You should see:
   - Compilation messages
   - Success confirmation: "Finished release"

3. If there are errors, check:
   - Rust syntax correctness
   - Soroban SDK imports
   - Function signatures

## Step 4: Deploy to Local Network

1. Run the deployment command:
   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/hello_world.wasm
   ```

2. The terminal will output:
   - Network connection status
   - Contract deployment details
   - **Contract ID** (save this!)

3. Copy the contract ID for the next step

## Step 5: Interact with Your Contract

1. Once deployed, use the contract client:
   ```bash
   soroban contract invoke --id <CONTRACT_ID> --fn hello --arg "World"
   ```

2. Expected output: `"Hello, World!"`

3. Try the other function:
   ```bash
   soroban contract invoke --id <CONTRACT_ID> --fn get_message
   ```

## Step 6: Validation

Your contract successfully works if:
- ✅ Build completes without errors
- ✅ Deployment shows a contract ID
- ✅ `hello` function returns the correct greeting
- ✅ `get_message` function returns the welcome message

## Common Issues & Solutions

**Issue**: Build fails with "cannot find crate"
- **Solution**: Check that `soroban-sdk` is in your `Cargo.toml`

**Issue**: Deployment timeout
- **Solution**: Verify network connection, restart the playground

**Issue**: Contract invoke returns undefined error
- **Solution**: Ensure you're using the correct contract ID and function name

## Next Steps

- Try modifying the contract to accept multiple parameters
- Explore contract storage capabilities in Tutorial 2
- Learn about contract errors in Tutorial 3

## Tips & Hints

- Always save your contract code before building
- Use descriptive names for your functions
- Test each function after deployment
- Keep your contract ID safe for future interactions

## Resources

- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)
- [Soroban Examples](../examples-index.md)
- [Testing Guide](./testing.md)

## Summary

You've successfully:
1. Written a Soroban smart contract
2. Compiled it to WebAssembly
3. Deployed it to a local network
4. Invoked contract functions

Congratulations! You're now ready to explore more advanced features.
