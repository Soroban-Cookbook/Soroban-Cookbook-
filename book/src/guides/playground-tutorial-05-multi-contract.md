# Playground Tutorial 5: Multi-Contract Interactions

## Overview
Learn how to build complex systems by having contracts interact with other contracts. This is essential for building DeFi applications.

**Duration**: 35-40 minutes
**Difficulty**: Advanced
**Prerequisites**: Tutorials 1-4 completion

## Learning Objectives
- Call functions in other contracts
- Pass contract addresses as parameters
- Handle cross-contract errors
- Design contract interfaces
- Implement contract factory patterns

## Step 1: Understanding Cross-Contract Communication

Soroban contracts can:
- Call functions in deployed contracts
- Pass contract addresses as parameters
- Handle errors from called contracts
- Design clear interfaces using traits

## Step 2: Create a Token Interface Contract

First, create a standard token interface:

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Address, String as SorobanString};

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Get token name
    pub fn name() -> SorobanString {
        SorobanString::from_slice("Test Token")
    }
    
    /// Get token symbol
    pub fn symbol() -> SorobanString {
        SorobanString::from_slice("TST")
    }
    
    /// Get account balance
    pub fn balance_of(account: Address) -> u128 {
        // Simplified - would read from storage
        1_000_000
    }
    
    /// Transfer tokens
    pub fn transfer(
        from: Address,
        to: Address,
        amount: u128,
    ) -> Result<bool, u32> {
        from.require_auth();
        Ok(true)
    }
}
```

## Step 3: Create a DEX Contract That Uses Tokens

Build a contract that calls the token contract:

```rust
#![no_std]
use soroban_sdk::{
    contract, contractimpl, Address, Env, Symbol, symbol_short,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    InvalidAmount = 1,
    InsufficientBalance = 2,
    PriceSlippage = 3,
    TokenTransferFailed = 4,
    InvalidPair = 5,
}

impl Into<u32> for ContractError {
    fn into(self) -> u32 {
        self as u32
    }
}

#[contract]
pub struct DexContract;

const LIQUIDITY_KEY: Symbol = symbol_short!("liq");
const PRICE_KEY: Symbol = symbol_short!("price");

#[contractimpl]
impl DexContract {
    /// Initialize liquidity pool
    pub fn init(
        env: Env,
        token_a: Address,
        token_b: Address,
        reserve_a: u128,
        reserve_b: u128,
    ) -> Result<(), ContractError> {
        // Validate amounts
        if reserve_a == 0 || reserve_b == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Store pool info
        env.storage()
            .instance()
            .set(&symbol_short!("tok_a"), &token_a);
        env.storage()
            .instance()
            .set(&symbol_short!("tok_b"), &token_b);
        env.storage()
            .instance()
            .set(&symbol_short!("res_a"), &reserve_a);
        env.storage()
            .instance()
            .set(&symbol_short!("res_b"), &reserve_b);
        
        // Calculate price ratio
        let price = (reserve_b as f64) / (reserve_a as f64);
        
        Ok(())
    }
    
    /// Execute a swap
    pub fn swap(
        env: Env,
        caller: Address,
        amount_in: u128,
        min_amount_out: u128,
        token_in: Address,
        token_out: Address,
    ) -> Result<u128, ContractError> {
        // Verify caller
        caller.require_auth();
        
        // Validate amounts
        if amount_in == 0 || min_amount_out == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Get pool reserves
        let reserve_a: u128 = env.storage()
            .instance()
            .get(&symbol_short!("res_a"))
            .ok_or(ContractError::InvalidPair)?
            .map_err(|_| ContractError::InvalidPair)?;
        
        let reserve_b: u128 = env.storage()
            .instance()
            .get(&symbol_short!("res_b"))
            .ok_or(ContractError::InvalidPair)?
            .map_err(|_| ContractError::InvalidPair)?;
        
        // Calculate output amount (simplified formula)
        // amount_out = (amount_in * reserve_b) / (reserve_a + amount_in)
        let amount_out = (amount_in as u128)
            .checked_mul(reserve_b)
            .ok_or(ContractError::InvalidAmount)?
            / (reserve_a + amount_in);
        
        // Check minimum output
        if amount_out < min_amount_out {
            return Err(ContractError::PriceSlippage);
        }
        
        // Transfer input tokens from caller to contract
        // This would call the token contract
        // token_in.transfer(caller, contract_address, amount_in)?;
        
        // Update reserves
        let new_reserve_a = reserve_a + amount_in;
        let new_reserve_b = reserve_b - amount_out;
        
        env.storage()
            .instance()
            .set(&symbol_short!("res_a"), &new_reserve_a);
        env.storage()
            .instance()
            .set(&symbol_short!("res_b"), &new_reserve_b);
        
        // Transfer output tokens to caller
        // token_out.transfer(contract_address, caller, amount_out)?;
        
        Ok(amount_out)
    }
    
    /// Get swap price
    pub fn get_price(
        env: Env,
        amount_in: u128,
    ) -> Result<u128, ContractError> {
        if amount_in == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        let reserve_a: u128 = env.storage()
            .instance()
            .get(&symbol_short!("res_a"))
            .ok_or(ContractError::InvalidPair)?
            .map_err(|_| ContractError::InvalidPair)?;
        
        let reserve_b: u128 = env.storage()
            .instance()
            .get(&symbol_short!("res_b"))
            .ok_or(ContractError::InvalidPair)?
            .map_err(|_| ContractError::InvalidPair)?;
        
        let price = (amount_in as u128)
            .checked_mul(reserve_b)
            .ok_or(ContractError::InvalidAmount)?
            / (reserve_a + amount_in);
        
        Ok(price)
    }
    
    /// Add liquidity to pool
    pub fn add_liquidity(
        env: Env,
        provider: Address,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<u128, ContractError> {
        provider.require_auth();
        
        if amount_a == 0 || amount_b == 0 {
            return Err(ContractError::InvalidAmount);
        }
        
        // Simplified: just return amount_a as LP tokens
        Ok(amount_a)
    }
}
```

## Step 4: Build and Deploy Both Contracts

1. Build both contracts:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy token contract:
   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/token_contract.wasm
   ```
   Save the contract ID as `TOKEN_ID`

3. Deploy DEX contract:
   ```bash
   soroban contract deploy --wasm target/wasm32-unknown-unknown/release/dex_contract.wasm
   ```
   Save the contract ID as `DEX_ID`

## Step 5: Initialize and Test Multi-Contract Interaction

### Initialize the Pool

```bash
soroban contract invoke --id <DEX_ID> --fn init \
  --arg <TOKEN_ID> \
  --arg <USDC_ID> \
  --arg 1000000 \
  --arg 1000000
```

### Get Price Quote

```bash
soroban contract invoke --id <DEX_ID> --fn get_price --arg 100
```

### Execute Swap

```bash
soroban contract invoke --id <DEX_ID> --fn swap \
  --arg <USER_ADDRESS> \
  --arg 100 \
  --arg 90 \
  --arg <TOKEN_ID> \
  --arg <USDC_ID> \
  --sign-with-key <USER_KEY>
```

### Add Liquidity

```bash
soroban contract invoke --id <DEX_ID> --fn add_liquidity \
  --arg <PROVIDER_ADDRESS> \
  --arg 500000 \
  --arg 500000 \
  --sign-with-key <PROVIDER_KEY>
```

## Step 6: Validation Checklist

✅ Both contracts deploy successfully
✅ DEX contract initializes with token addresses
✅ Price calculation returns correct values
✅ Swap executes with proper reserve updates
✅ Liquidity addition works correctly
✅ All error cases are handled
✅ Cross-contract calls complete without errors

## Multi-Contract Patterns

### Pattern 1: Contract Address as Parameter
```rust
pub fn swap(
    env: Env,
    token_in: Address,
    token_out: Address,
    amount: u128,
) -> Result<u128, ContractError> {
    // Can be called with any token address
}
```

### Pattern 2: Inter-Contract Call
```rust
// Call another contract's function
let client = TokenClient::new(&env, &token_address);
let balance = client.balance_of(&caller);
```

### Pattern 3: Error Propagation
```rust
match token.transfer(from, to, amount) {
    Ok(result) => Ok(result),
    Err(e) => Err(ContractError::TokenTransferFailed),
}
```

## Best Practices

1. **Define Interfaces**: Use traits for consistent contract APIs
2. **Error Handling**: Wrap external calls in error handlers
3. **Validation**: Verify all addresses before use
4. **Atomicity**: Ensure all-or-nothing execution
5. **Security**: Be careful with delegated authority

## Security Considerations

- **Reentrancy**: Soroban handles this for you
- **Price Slippage**: Always enforce minimum output
- **Front-Running**: Use mechanisms like batching
- **Contract Upgrades**: Design for evolution
- **Emergency Stops**: Consider pause mechanisms

## Advanced Topics

- Factory contracts for dynamic creation
- Proxy patterns for upgradability
- Batch operations
- Flash loans and atomic swaps
- Collateralized lending

## Troubleshooting

**Issue**: "Contract not found"
- **Solution**: Ensure contract ID is correct and deployed

**Issue**: "Invalid argument type"
- **Solution**: Check address format and type matching

**Issue**: Swap always fails
- **Solution**: Verify liquidity exists and prices are reasonable

## Next Steps

- Build a complete DEX application
- Implement a lending protocol
- Create a governance system
- Explore [Advanced Examples](../examples/advanced.md)

## Summary

You've learned to:
1. Create contract interfaces
2. Pass contract addresses between contracts
3. Handle cross-contract errors
4. Implement DEX-like functionality
5. Build complex multi-contract systems

Multi-contract interactions enable sophisticated DeFi applications!
