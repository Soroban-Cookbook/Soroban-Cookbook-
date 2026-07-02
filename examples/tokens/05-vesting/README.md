# Token Vesting Contract

A complete Token Vesting example demonstrating how to implement a secure token vesting schedule using Soroban smart contracts.

## Overview

Token vesting is a mechanism where tokens are released to a beneficiary over a period of time. This contract supports:
- **Vesting Schedule Creation**: Admin can set up schedules for beneficiaries.
- **Cliff Period**: A duration during which no tokens can be claimed.
- **Linear Vesting**: Continuous release of tokens after the cliff.
- **Secure Claims**: Beneficiaries can claim their vested tokens.

## Contract Architecture

The contract uses Soroban's persistent storage to track vesting schedules for each beneficiary and instance storage for global configuration (admin and token address).

### Storage Model

- `Admin`: The address allowed to create vesting schedules (Instance).
- `Token`: The address of the token being vested (Instance).
- `Schedule(beneficiary)`: The `VestingSchedule` for a specific beneficiary (Persistent).

### Vesting Lifecycle

1.  **Initialization**: Admin sets the admin address and the token address.
2.  **Schedule Creation**: Admin creates a schedule for a beneficiary with total allocation, start time, cliff duration, and vesting duration.
3.  **Vesting**: Tokens vest linearly from the start time until the end of the vesting duration.
4.  **Claiming**: After the cliff period, the beneficiary can claim the vested tokens that haven't been claimed yet.

## Linear Vesting Calculation

The vested amount is calculated as:
`vested = total_allocation * (current_time - start_time) / vesting_duration`

If `current_time` is before `start_time + cliff_duration`, the vested amount is 0.
If `current_time` is after `start_time + vesting_duration`, the vested amount is the `total_allocation`.

## Usage

### 1. Initialize the Contract
```rust
client.initialize(&admin, &token_address);
```

### 2. Create a Vesting Schedule
```rust
client.create_schedule(&admin, &beneficiary, &1000, &start_time, &cliff_duration, &vesting_duration);
```

### 3. Claim Vested Tokens
```rust
client.claim(&beneficiary);
```

## Running Tests
```bash
cargo test -p vesting-contract
```

## Building the Example
```bash
cargo build --target wasm32-unknown-unknown --release -p vesting-contract
```

## Security Considerations
- **Admin Authorization**: Only the admin can create vesting schedules.
- **Beneficiary Authorization**: Only the beneficiary can claim their tokens.
- **Cliff Enforcement**: Claims are blocked until the cliff period has passed.
- **Arithmetic Safety**: Uses checked arithmetic to prevent overflows.
