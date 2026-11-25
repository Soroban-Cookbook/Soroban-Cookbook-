# Benchmark Maintenance Guide

This guide describes how to keep the [benchmarks.md](./benchmarks.md) documentation current as the Soroban SDK and protocol evolve.

## Why Benchmark Maintenance Matters

Soroban's resource model and fee schedule have evolved significantly since launch:
- **Protocol 20 (Feb 2024)**: Initial mainnet launch with baseline costs
- **Protocol 21 (CAP-0054)**: ~20-30% reduction in VM instantiation costs
- **Protocol 28 (Nov 2026)**: Current fee schedule with refined resource metering

Developers rely on these numbers for cost estimation in DeFi protocols, NFT marketplaces, and other applications. Outdated numbers lead to incorrect economic models and can cause production issues.

## Review Schedule

**Quarterly reviews** (every 3 months) or whenever:
- A new Soroban SDK major/minor version is released
- A protocol upgrade changes the fee schedule
- A community member reports significant discrepancies

The [benchmark workflow](../.github/workflows/benchmarks.yml) runs automatically on the first Monday of January, April, July, and October, creating an issue if the documentation is stale.

## Update Process

### 1. Prepare Your Environment

```bash
# Clone the repository
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-

# Ensure you have the latest Rust and Soroban CLI
rustup update stable
cargo install --locked soroban-cli --features opt

# Run the full test suite to ensure everything works
cargo test --workspace
```

### 2. Run Benchmark Tests

Execute the benchmark suite and capture output:

```bash
# Storage operations
cargo test -p integration-tests storage_benchmarks -- --nocapture | tee storage_results.txt

# Token operations
cargo test -p integration-tests token_benchmarks -- --nocapture | tee token_results.txt

# DeFi operations
cargo test -p integration-tests --test defi_benchmarks -- --nocapture | tee defi_results.txt

# Cross-contract calls
cargo test -p integration-tests cross_contract_benchmark -- --nocapture | tee cross_contract_results.txt

# Collection types
cargo test -p collection-types benchmark -- --nocapture | tee collection_results.txt
```

### 3. Cross-Reference with Network Data

Compare your local benchmark results with actual network behavior:

1. **Check current resource rates** at [Stellar Lab](https://lab.stellar.org/account/resources)
2. **Use `simulateTransaction`** on testnet/mainnet for real transactions:
   ```bash
   soroban contract invoke \
     --id <contract-id> \
     --source <account> \
     --rpc-url https://soroban-testnet.stellar.org \
     -- <function-name> <args>
   ```
   Review the resource usage in the simulation output.

3. **Review recent protocol changes**:
   - [Stellar Protocol CAPs](https://github.com/stellar/stellar-protocol/tree/master/core)
   - [Soroban SDK Changelog](https://github.com/stellar/rs-soroban-sdk/blob/main/CHANGELOG.md)

### 4. Update benchmarks.md

Edit `book/src/docs/benchmarks.md`:

1. **Update the header**:
   ```markdown
   > **Document Status**: Updated for SDK X.Y.Z (Month Year)  
   > **Last Measured**: Month Year  
   > **Fee Schedule**: Mainnet/Testnet (Protocol N+)
   ```

2. **Update CPU instruction ranges** in the comparison tables:
   - Use ranges (e.g., `~80,000 - 120,000`) instead of single values to account for variance
   - Note significant changes from prior versions
   - Add explanations for any large deviations

3. **Update execution time benchmarks** if hardware or test environment has changed

4. **Update the Document Metadata section** at the bottom:
   ```markdown
   - **SDK Version**: X.Y.Z
   - **Last Updated**: Month Year
   - **Protocol Version**: N+
   - **Next Review**: Month Year (3 months from now)
   ```

5. **Add a changelog note** if there were significant changes:
   ```markdown
   ### Changelog
   
   **November 2026 (SDK 28.0.0, Protocol 28)**:
   - Updated all CPU instruction counts for current protocol
   - Reflected VM instantiation improvements from CAP-0054
   - Added real-world USD cost estimates at current XLM prices
   - Expanded methodology section
   ```

### 5. Validate Changes

Before submitting:

```bash
# Ensure the documentation builds
cd book
mdbook build

# Run the workspace tests again
cd ..
cargo test --workspace

# Check for typos and formatting
cargo install typos-cli
typos book/src/docs/benchmarks.md
```

### 6. Submit a Pull Request

```bash
git checkout -b update-benchmarks-$(date +%Y-%m)
git add book/src/docs/benchmarks.md
git commit -m "docs: update benchmarks for SDK $(grep "soroban-sdk" Cargo.toml | head -1 | cut -d'"' -f2)"
git push origin update-benchmarks-$(date +%Y-%m)
```

Create a PR with:
- **Title**: `[DX] Update benchmarks for SDK X.Y.Z`
- **Labels**: `documentation`, `benchmarks`, `P2`
- **Description**: 
  - What SDK/protocol version this is for
  - Summary of changes (e.g., "CPU costs increased by ~15% due to...")
  - Link to relevant protocol CAPs or SDK release notes
  - Mention if real mainnet transactions were tested

### 7. Post-Merge Actions

After the PR merges:
1. Close any open "quarterly benchmark review" issues
2. If there were major changes, consider announcing in:
   - GitHub Discussions
   - Stellar Discord #soroban channel
   - Monthly community call

## Automation

The `.github/workflows/benchmarks.yml` workflow:
- Runs quarterly on a schedule
- Runs on pushes to examples or test code
- Detects SDK version mismatches
- Creates issues when benchmarks are stale
- Uploads benchmark output as artifacts

You can manually trigger it via GitHub Actions > Benchmark > Run workflow.

## Tips for Accurate Measurements

1. **Run multiple times**: CPU instruction counts can vary slightly. Run 3-5 times and use the median.
2. **Avoid heavy system load**: Close other applications during benchmarking.
3. **Use release builds** for execution time measurements (not needed for CPU instruction counting, which is deterministic).
4. **Document environment**: If you're on different hardware than previous benchmarks, note it.
5. **Test edge cases**: Measure both small and large inputs to establish ranges.
6. **Compare with mainnet**: Use `simulateTransaction` to validate that local measurements align with real network behavior.

## Troubleshooting

**Q: Local benchmarks show very different numbers than the docs.**  
A: Check your SDK version (`cargo tree | grep soroban-sdk`). Ensure you're on the same version documented in `benchmarks.md`.

**Q: Benchmarks fail to run.**  
A: Some examples may not have dedicated benchmark tests. Check `tests/integration/` for available benchmark modules.

**Q: Real mainnet costs are higher than documented.**  
A: Benchmarks show CPU instructions, not fees. Fees also include I/O, storage rent, and surge pricing. Use `simulateTransaction` for accurate fee estimates.

**Q: Protocol upgraded but SDK hasn't been updated.**  
A: Wait for the SDK release that includes protocol changes, then update. Document this lag in the PR description.

## Related Documentation

- [Benchmarks.md](./benchmarks.md) - The main benchmark documentation
- [Testing Best Practices](./testing-best-practices.md) - How to write effective tests
- [Stellar Fees Documentation](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering)
- [Issue #1477](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/issues/1477) - Original request for this maintenance process

## Questions?

Open an issue with the `benchmarks` label or ask in the [Discussions](https://github.com/Soroban-Cookbook/Soroban-Cookbook-/discussions) forum.
