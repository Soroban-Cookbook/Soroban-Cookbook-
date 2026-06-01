# Gas Benchmarks

This page publishes repeatable benchmark baselines for Soroban Cookbook examples, including both basic and intermediate patterns.

Gas-sensitive developers can use these comparisons to choose the right example contract design and verify that new changes do not regress on-chain resource consumption.

## 📊 Baseline Comparison Table

| Example | Operation | Instructions | RAM | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `01-hello-world` | `hello()` | ~10,000 | ~1 KB | Minimal logic and no storage overhead. |
| `02-storage-patterns` | `set_persistent` | ~55,000 | ~2 KB | Persistent storage is the most expensive storage tier. |
| `02-storage-patterns` | `set_instance` | ~35,000 | ~1.5 KB | Instance storage is cheaper than persistent storage for config-like data. |
| `02-storage-patterns` | `set_temporary` | ~25,000 | ~1 KB | Temporary storage is the cheapest option for intra-transaction data. |
| `03-authentication` | `transfer()` | ~45,000 | ~2.5 KB | Authentication and storage interaction increase gas usage. |
| `05-error-handling` | `Result` return | ~12,000 | ~1.2 KB | Structured error handling is cheaper than panicking. |
| `ajo-factory` | `create_ajo()` | ~85,000 | ~4 KB | Dynamic deployment and factory bookkeeping are gas-intensive. |
| `multi-sig-patterns` | `execute()` | ~60,000 | ~3.5 KB | Threshold checks and proposal state updates add cost. |

> These baseline values are derived from Soroban example benchmarks and should be treated as approximate guidance. Variations can occur across SDK versions and host environments.

## 🔁 Repeatable Benchmark Process

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

## 🧩 Intermediate Example Benchmarks

This repository now includes dedicated benchmark coverage for intermediate examples such as:

- `examples/intermediate/multi-sig-patterns`
- `examples/intermediate/ajo-factory`

That makes it easier to compare basic and intermediate gas behavior in one place.

## 📦 CI Baselines

A new GitHub Actions job now runs the benchmark script on push and uploads the raw benchmark results as an artifact.

The job saves a `gas-benchmark-results/` artifact so repository maintainers can inspect stable baselines and compare regression trends over time.

## 💡 Benchmarking Tips

- Run the benchmark job from a clean checkout so the results reflect fresh compilation and test execution.
- Use exact example paths when comparing candidate contracts.
- Keep benchmark tests small and focused on a single key operation.
- If you add a new example, include a matching benchmark test to ensure it is captured by CI.

---

*Last updated: May 2026*
