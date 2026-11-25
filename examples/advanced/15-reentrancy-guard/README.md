# Advanced Reentrancy Protection

This example demonstrates how to implement a reentrancy guard in a Soroban smart contract. 

## Overview

In smart contract development, a reentrancy attack occurs when an external contract calls back into the calling contract before the initial execution is complete. This can lead to unexpected state changes and vulnerabilities, especially if the contract has not updated its internal state before making the external call.

Soroban inherently prevents recursive calls into the same contract context. However, when multiple contracts interact, cross-contract reentrancy and read-only reentrancy are still potential attack vectors if a malicious contract attempts to invoke the originating contract again.

## Features

1. **Reentrancy Guard Pattern**: A simple `DataKey::Entered` instance storage flag is used to lock the contract during execution.
2. **Cross-Contract Guards**: Demonstrates how external interactions (`invoke_contract`) are protected from maliciously calling back into state-modifying functions like `withdraw`.
3. **Read-Only Reentrancy**: Protects view functions (`get_balance`) to ensure that external contracts cannot read intermediate or inconsistent state while a state-modifying function is executing.

## Best Practices

- Always use the **Checks-Effects-Interactions** pattern, even with a reentrancy guard. Update state before making external calls.
- Ensure all external calls are made at the very end of your function execution.
- Use read-only reentrancy guards for functions that return critical state which might be read during an external call.

## Running Tests

To run the tests for this example:

```sh
cargo test --package reentrancy-guard
```
