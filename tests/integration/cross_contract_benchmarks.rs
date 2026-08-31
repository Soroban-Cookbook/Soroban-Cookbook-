//! Cross-contract call benchmarks.
//!
//! This file measures the performance of cross-contract interactions:
//! - Call overhead: direct calls to a simple contract function.
//! - Factory deployment: deploying a new contract via a factory.
//! - Proxy call: routing calls through a proxy contract.
//!
//! Benchmarks are intentionally simple and use `std::time::Instant`.
//! For production-grade measurements, consider using a proper benchmarking
//! framework (e.g., `criterion`) in a separate benchmark target.

use std::time::Instant;

/// Number of iterations for each benchmark. Keep low to avoid slow CI.
const ITERATIONS: u32 = 100;

/// A minimal interface for a cross-contract call target.
trait CrossContractTarget {
    fn simple_call(&relfs) -> u64;
}

/// A dummy implementation for benchmarking call overhead.
struct DummyTarget {
    value: u64,
}

impl CrossContractTarget for DummyTarget {
    fn simple_call(&self) -> u64 {
        self.value
    }
}

/// Measures the raw call overhead of a cross-contract call.
//>
/// This benchmarks the time taken to invoke a simple method on an object
//> that simulates a contract call. The measured time includes the dispatch
//> overhead of the `CrossContractTarget` trait, which is a lower bound for
//> actual cross-contract calls in a runtime environment.
#[test]
fn call_overhead_benchmark() {
    let target = DummyTarget { value: 42 };
    let start = Instant::now();
    let mut sum = 0u64;
    for _ in 0..ITERATIONS {
        sum = sum.wrapping_add(target.simple_call());
    }
    let elapsed = start.elapsed();
    let per_call = elapsed / ITERATIONS;
    println!(
        "Call overhead: {} ns/call ({} iterations, sum: {})",
        per_call.as_nanos(),
        ITERATIONS,
        sum
    );
}

/// A factory abstraction for deploying new contracts.
trait Factory {
    type Contract: CrossContractTarget;
    fn deploy(&mut self) -> Self::Contract;
}

/// A dummy factory that returns a new dummy contract.
struct DummyFactory;

impl Factory for DummyFactory {
    type Contract = DummyTarget;
    fn deploy(&mut self) -> DummyTarget {
        DummyTarget { value: 1 }
    }
}

/// Measures the cost of deploying a new contract through a factory.
//>
/// The benchmark repeatedly calls the factory's `deploy` method and
//> measures the total elapsed time. This represents the on-chain cost of
//> contract creation, which is significantly higher than regular calls.
#[test]
fn factory_deployment_benchmark() {
    let mut factory = DummyFactory;
    let start = Instant::now();
    let mut total = 0u64;
    for _ in 0..ITERATIONS {
        let contract = factory.deploy();
        total = total.wrapping_add(contract.simple_call());
    }
    let elapsed = start.elapsed();
    let per_deploy = elapsed / ITERATIONS;
    println!(
        "Factory deployment: {} ns/deploy ({} iterations, total: {})",
        per_deploy.as_nanos(),
        ITERATIONS,
        total
    );
}

/// A proxy abstraction for forwarding calls to a backing contract.
trait Proxy {
    type Target: CrossContractTarget;
    fn call(&self) -> u64;
}

/// A dummy proxy that wraps a target and forwards the call.
struct DummyProxy<T: CrossContractTarget> {
    target: T,
}

impl<T: CrossContractTarget> Proxy for DummyProxy<T> {
    type Target = T;
    fn call(&self) -> u64 {
        self.target.simple_call()
    }
}

/// Measures the overhead introduced by a proxy contract.
//>
/// Instead of calling the target directly, the proxy forwards the call.
//> This benchmark quantifies the additional cost of indirection.
#[test]
fn proxy_call_benchmark() {
    let target = DummyTarget { value: 7 };
    let proxy = DummyProxy { target };
    let start = Instant::now();
    let mut sum = 0u64;
    for _ in 0..ITERATIONS {
        sum = sum.wrapping_add(proxy.call());
    }
    let elapsed = start.elapsed();
    let per_call = elapsed / ITERATIONS;
    println!(
        "Proxy call: {} ns/call ({} iterations, sum: {})",
        per_call.as_nanos(),
        ITERATIONS,
        sum
    );
}

/// Documents optimization recommendations based on benchmark results.
//>
/// The following recommendations should be considered after running these
//> benchmarks:
//> 1. **Minimize cross-contract calls**: Each call involves overhead due to
//>    runtime dispatch, state transitions, and potential serialization.
//>    Consolidate logic into a single contract when possible.
//> 2. **Use direct calls over proxies**: Proxy patterns add extra indirection
//>    and cost. Only use proxies when upgradability or access control is
//>    required.
//> 3. **Batch operations**: If multiple calls are made, consider batching
//>    them into a single transaction to reduce per-call overhead.
//> 4. **Optimize factory deployments**: Factory contracts should be kept
//>    lightweight and reuse code efficiently through code hashing or binary
//>    search for already-deployed code.
//> 5. **Profile in the target environment**: Benchmarks in unit tests do not
//>    reflect actual blockchain gas costs. Use profiling tools specific to
//>    the runtime (e.g., weight annotations in Substrate, gas metering in
//>    NEAR) for accurate optimization.
//>
/// These benchmarks are intentionally simple; they provide a rough baseline.
/// For precise numbers, run them as part of a release build with
/// optimizations enabled.