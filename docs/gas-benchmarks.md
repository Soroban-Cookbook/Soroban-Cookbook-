# Gas Benchmarks

This page publishes repeatable benchmark baselines for Soroban Cookbook examples, including both basic and intermediate patterns.

Gas-sensitive developers can use these comparisons to choose the right example contract design and verify that new changes do not regress on-chain resource consumption.

## Baseline Comparison Table

| Example | Operation | Instructions | RAM | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `01-hello-world` | `hello()` | ~10,000 | ~1 KB | Minimal logic and no storage overhead. |
| `02-storage-patterns` | `set_persistent` | ~55,000 | ~2 KB | Persistent storage is the most expensive storage tier. |
| `02-storage-patterns` | `set_instance` | ~35,000 | ~1.5 KB | Instance storage is cheaper than persistent storage for config-like data. |
| `02-storage-patterns` | `set_temporary` | ~25,000 | ~1 KB | Temporary storage is the cheapest option for intra-transaction data. |
| `03-authentication` | `transfer()` | ~45,000 | ~2.5 KB | Authentication and storage interaction increase gas usage. |
| `05-error-handling` | `Result` return | ~12,000 | ~1.2 KB | Structured error handling is cheaper than panicking. |
| `ajo-factory` | `create_ajo()` | ~85,000 | ~4 KB | Dynamic deployment and factory bookeeping are gas-intensive. |
| `multi-sig-patterns` | `execute()` | ~60,000 | ~3.5 KB | Threshold checks and proposal state updates add cost. |

> These baseline values are derived from Soroban example benchmarks and should be treated as approximate guidance. Variations can occur across SDK versions and host environments.

## Repeatable Benchmark Process

Benchmarks are run using the repository `scripts/benchmark.sh` helper. The script now discovers example directories with dedicated benchmark tests and can emit a stable artifact directory for CI baselines.

```bash
./scripts/benchmark.sh --output-dir gas-benchmark-results
```

If you want to benchmark a specific example only, pass the example directory:

```bash
./scripts/benchmark.sh examples/intermediate/multi-sig-patterns --output-dir gas-benchmark-results
```

### What the script does

- finds every example directory with a `Cargo.toml`
- skips directories without benchmark tests
- runs `cargo test -- --nocapture benchmark`
- writes benchmark logs to the directory passed with `--output-dir`

## Intermediate Example Benchmarks

This repository now includes dedicated benchmark coverage for intermediate examples such as:

- `examples/intermediate/multi-sig-patterns`
- `examples/intermediate/ajo-factory`

## Cross-Contract Call Benchmarks

Cross-contract calls are the primary driver of gas costs in composable contracts. The following comparisons show how common optimizations reduce overhead.

| Pattern | Example | Instructions (est.) | RAM (est.) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- |
| Separate round trips (unpacked) | `multi-sig-patterns` `execute()` | ~60,000 | ~3.5 KB | Each cross-contract call adds fixed overhead |
| Argument packing | `custom-token` `transfer()` | ~35,000 | ~2 KB | Packing calldata reduces host-function costs |
| Call batching | `ajo-factory` `create_ajo()` batched | ~75,000 | ~3.5 KB | Batching distributes base costs across calls |
| Minimized round trips | `multi-sig-patterns` `execute()` with state preload | ~45,000 | ~2.5 KB | Fewer round trips lower RAM and instruction usage |

Benchmark script: `./scripts/benchmark.sh examples/intermediate --output-dir gas-benchmark-results` captures these patterns automatically.

| Example | Operation | Instructions | RAM | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `cross-contract-calls` | `direct_call()` | ~25,000 | ~1.5 KB | Baseline direct contract invocation. |
| `cross-contract-calls` | `factory_deploy()` | ~95,000 | ~5 KB | Deployment of a child contract via factory pattern. |
| `cross-contract-calls` | `proxy_call()` | ~35,000 | ~2 KB | Delegating call through a proxy contract adds ~10K overhead. |

### Optimization Recommendations

- Prefer direct calls over proxy indirection when upgradeability is not required; proxy dispatch adds measurable overhead.
- Batch related cross-contract operations inside a single contract call to reduce repeated call overhead.
- Cache addresses and call data in instance storage when the same child contract is invoked frequently.
- Measure with `scripts/benchmark.sh` after any SDK upgrade; host-function costs can shift between versions.

## Advanced Example Benchmarks


| Example | Operation | Instructions (est.) | RAM (est.) | Key Takeaway |
| :--- | :--- | :--- | :--- | :--- |
| `01-multi-party-aut` | `multi_sig_transfer` (3 signers) | ~75,000 | ~4 KB | Auth per signer adds ~15K instructions each |
| `01-multi-party-aut` | `encode_auth_vec` (10 signers) | ~40,000 | ~2 KB | Sorting dominates; O(N log N) encoding cost |
| `02-timelock` | `queue` | ~35,000 | ~1.5 KB | State write with timestamp check |
| `02-timelock` | `execute` | ~40,000 | ~2 KB | Delay validation + state transition |
| `03-cross-chain-bridge` | `lock` | ~55,000 | ~3 KB | Mint + storage update + event |
| `03-cross-chain-bridge` | `release` | ~60,000 | ~3.5 KB | Validator set verification + burn |
| `05-bridge-security` | `rate_limited_release` | ~50.000 | ~2.5 KB | Epoch check + volume accounting |
| `05-reentrancy-guard` | `guarded_call` | ~30,000 | ~1.5 KB | Mutex flag adds ~5K  over bare call |
| `05-merkle-proofs` | `verify_proof` (depth 10) | ~45,000 | ~2 KB | Each hash adds ~4K instructions |
| `05-batch-operations` | `execute_batch` (5 ops) | ~120,000 | ~6 KB | Scales linearly; batch overhead ~20K base |
| `06-diamond-pattern` | `diamond_cut` (add facet) | ~65,000 | ~4 KB | Selector registration + storage write |
| `06-diamond-pattern` | `diamond_call` | ~35,000 | ~2 KB | Dispatch overhead ~5K  over direct call |
| `custom-token` | `transfer` | ~45,000 | ~2.5 KB | Same as base SEP-41 token |
| `custom-token` | `multi_sig_transfer` (2 signers) | ~55,000 | ~3 KB | Signer iteration + auth overhead |
| `custom-token` | `mint` | ~40,000 | ~2 KB | Auth check + supply update + event |

*Estimates based on test environment. Actual costs vary by network conditions and SDK version.*

That makes it easier to compare basic, intermediate, and advanced gas behavior in one place.

## CI Baselines

A new GitHub Actions job now runs the benchmark script on push and uploads the raw benchmark results as an artifact.

The job saves a `gas-benchmark-results/` artifact so repository maintainers can inspect stable baselines and compare regression trends over time.

## Benchmarking Tips

- Run the benchmark job from a clean checkout so the results reflect fresh compilation and test execution.
- Use exact example paths when comparing candidate contracts.
 - Keep benchmark tests small and focused on a single key operation.
 - If you add a new example, include a matching benchmark test to ensure it is captured by CI.

---

*Last updated: May 2026*
