# Performance Testing Framework

This framework allows developers to measure the gas cost (CPU and memory) and execution time of Soroban contracts within the Cookbook integration tests.

## Usage

Use the `measure_execution` helper provided in `tests::helpers::perf`.

```rust
use crate::helpers::{perf::measure_execution, setup_env};
use soroban_sdk::{Symbol, Vec};

#[test]
fn test_my_contract_performance() {
    let env = setup_env();
    let contract_id = env.register_contract(None, my_contract::MyContract);
    
    let (result, metrics) = measure_execution(&env, || {
        // Contract invocation here
        env.invoke_contract::<()>(
            &contract_id,
            &Symbol::new(&env, "my_func"),
            Vec::new(&env),
        )
    });

    // Print the collected metrics
    metrics.print("My Contract - my_func");
    
    // Assert on metrics if desired
    assert!(metrics.cpu_instructions < 1_000_000);
}
```

## Available Metrics

- `execution_time_ns`: Wall-clock execution time in nanoseconds. Note: This can vary based on host machine performance.
- `cpu_instructions`: Soroban environment CPU instruction cost.
- `memory_bytes`: Soroban environment memory allocation cost.
