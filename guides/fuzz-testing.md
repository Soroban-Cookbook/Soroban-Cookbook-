# Fuzz Testing Guide

This guide covers how to set up and run fuzz tests on your Soroban smart contracts using `cargo-fuzz`.

## Overview

Fuzz testing (fuzzing) is an automated software testing technique that involves providing invalid, unexpected, or random data as inputs to a computer program. 

## Setup

First, install the `cargo-fuzz` tool:

```bash
cargo install cargo-fuzz
```

Ensure you are using the nightly toolchain for fuzzing:
```bash
rustup default nightly
```

## Running Fuzz Tests

Navigate into the `tests/fuzz` directory in the cookbook:

```bash
cd tests/fuzz
```

To run a specific fuzz target, such as the `example_fuzz` test:

```bash
cargo fuzz run example_fuzz
```

The fuzzer will run continuously, trying to find inputs that cause your contract to panic or crash.

## Writing New Fuzz Targets

To add a new fuzz target:

1. Add a new `[[bin]]` entry in `tests/fuzz/Cargo.toml`.
2. Add your contract dependency to the `dependencies` block in `tests/fuzz/Cargo.toml`.
3. Create a new `.rs` file in `tests/fuzz/fuzz_targets/`.
4. Use the `fuzz_target!` macro to define your fuzzing logic, ensuring you instantiate an `Env`, configure your contract state, and pass the fuzzed data as arguments.

## Interpreting Failures

If `cargo-fuzz` finds a crashing input, it will output the exact byte sequence that caused the crash and save it to the `artifacts/` directory (inside the `fuzz` folder).

You can reproduce the crash using:
```bash
cargo fuzz run <target_name> <path_to_artifact>
```
