# Local Simulation Guide

How to build, invoke, and debug Soroban contracts entirely on your machine before touching testnet.

---

## Prerequisites

- Rust with `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- Stellar CLI 22.1.0+ (`cargo install --locked stellar-cli --version 22.1.0`)

---

## How Local Simulation Works

The Stellar CLI embeds the same Soroban host used on-chain. When you run `stellar contract invoke` without `--network`, it spins up an in-process ledger, executes your WASM, and returns results — no network, no fees, no funded account required.

---

## Reproducible Local Workflow

### Step 1 — Build

```bash
cd examples/basics/01-hello-world
stellar contract build
# WASM → target/wasm32-unknown-unknown/release/hello_world.wasm
```

### Step 2 — Deploy to the local sandbox

```bash
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source-account alice \
  --network-passphrase "Standalone Network ; February 2017" \
  --rpc-url http://localhost:8000/soroban/rpc)

echo $CONTRACT_ID   # save this for subsequent calls
```

> If you don't have a local RPC node, omit `--rpc-url` and `--network-passphrase` to use the CLI's built-in sandbox mode (no node needed).

### Step 3 — Invoke

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  -- \
  hello \
  --to World
```

Expected output:

```
["Hello", "World"]
```

### Step 4 — Dry-run before every real submission

Add `--send no` to simulate without broadcasting. This is the fastest way to catch panics, auth failures, and fee surprises:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  --send no \
  -- \
  hello \
  --to World
```

The CLI prints the simulated return value, resource usage, and fee estimate — all without spending any XLM.

---

## Iterative Development Loop

```
edit src/lib.rs
      │
      ▼
stellar contract build          ← recompile WASM
      │
      ▼
stellar contract invoke --send no   ← dry-run, inspect output
      │
      ├─ panic / wrong result? → back to edit
      │
      ▼
cargo test                      ← run unit tests
      │
      ▼
stellar contract invoke         ← submit to local node (optional)
      │
      ▼
ready for testnet
```

Keep this loop tight. Dry-run catches most issues in seconds; `cargo test` catches logic errors with full Rust tooling.

---

## State Inspection

### Read any storage entry

```bash
stellar contract read \
  --id $CONTRACT_ID \
  --key '{"symbol":"admin"}'
```

### Dump all instance storage

```bash
stellar contract read \
  --id $CONTRACT_ID \
  --durability instance
```

### Dump persistent storage

```bash
stellar contract read \
  --id $CONTRACT_ID \
  --durability persistent
```

### Inspect ledger state after a call

```bash
# 1. invoke and capture the ledger sequence
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  -- set_value --val 42

# 2. read back the key you just wrote
stellar contract read \
  --id $CONTRACT_ID \
  --key '{"symbol":"val"}'
# → 42
```

---

## Debugging Tips

### 1. Enable host-side logs

The Soroban host emits diagnostic events for every `log!` call in your contract. Capture them with `--diagnostic-events`:

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  --diagnostic-events \
  -- \
  my_function \
  --arg 99
```

In your contract:

```rust
use soroban_sdk::log;

pub fn my_function(env: Env, arg: u32) -> u32 {
    log!(&env, "my_function called with arg={}", arg);
    arg * 2
}
```

### 2. Inspect emitted events

```bash
stellar events \
  --start-ledger 1 \
  --id $CONTRACT_ID
```

Events are printed as JSON. Each entry shows `type`, `topics`, and `data` — useful for verifying that your `env.events().publish(...)` calls fire with the right payloads.

### 3. Decode XDR manually

If the CLI returns raw XDR, decode it:

```bash
stellar xdr decode --type ScVal --input base64 \
  AAAAEAAAAQAAAAcAAAAFaGVsbG8AAAAA
```

### 4. Reproduce a panic

When a contract panics, the CLI prints the panic message and the host backtrace. To get the full trace, set:

```bash
RUST_BACKTRACE=1 stellar contract invoke ...
```

### 5. Isolate auth failures

If you see `HostError: Error(Auth, InvalidAction)`, the contract called `require_auth()` for an address that wasn't authorized. Reproduce with `mock_all_auths()` in a unit test to confirm which call is failing:

```rust
#[test]
fn debug_auth_failure() {
    let env = Env::default();
    env.mock_all_auths();   // bypasses auth — if this passes, auth is the problem
    let id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &id);
    client.protected_fn(&Address::generate(&env));
}
```

### 6. Advance ledger time in tests

Timelocks and TTL logic depend on `env.ledger().timestamp()`. Manipulate it directly:

```rust
env.ledger().with_mut(|li| {
    li.timestamp = 1_700_000_000;
    li.sequence_number = 100;
});
```

Call your function, then advance time and call again to test expiry paths.

---

## Practical Example: Counter Contract

A complete edit → simulate → inspect loop using a simple counter.

**Contract (`src/lib.rs`)**

```rust
use soroban_sdk::{contract, contractimpl, log, symbol_short, Env};

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn increment(env: Env) -> u32 {
        let mut count: u32 = env.storage().instance()
            .get(&symbol_short!("count"))
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&symbol_short!("count"), &count);
        log!(&env, "count is now {}", count);
        count
    }

    pub fn get(env: Env) -> u32 {
        env.storage().instance()
            .get(&symbol_short!("count"))
            .unwrap_or(0)
    }
}
```

**Build and deploy**

```bash
stellar contract build

CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/counter.wasm \
  --source-account alice)
```

**Dry-run first call**

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  --send no \
  -- increment
# → 1  (simulated, not committed)
```

**Submit and inspect state**

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  -- increment
# → 1

stellar contract read \
  --id $CONTRACT_ID \
  --key '{"symbol":"count"}'
# → 1

stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account alice \
  -- increment
# → 2

stellar contract invoke \
  --id $CONTRACT_ID \
  -- get
# → 2
```

---

## Moving to Testnet

Once your local loop is clean:

1. Add the testnet network config (see [Deployment Guide](./deployment.md#network-configuration))
2. Fund an account via Friendbot
3. Re-run the same `stellar contract deploy` and `invoke` commands with `--network testnet`

The commands are identical — only the `--network` flag changes. Everything you validated locally transfers directly.

---

## Quick Reference

| Task | Command |
|---|---|
| Build WASM | `stellar contract build` |
| Deploy locally | `stellar contract deploy --wasm <path> --source-account alice` |
| Dry-run invoke | `stellar contract invoke --id $ID --send no -- <fn>` |
| Read storage key | `stellar contract read --id $ID --key '<json>'` |
| Dump instance storage | `stellar contract read --id $ID --durability instance` |
| View events | `stellar events --start-ledger 1 --id $ID` |
| Decode XDR | `stellar xdr decode --type ScVal --input base64 <xdr>` |
| Enable diagnostics | add `--diagnostic-events` to any invoke |

---

**Next:** [Deployment Guide](./deployment.md) — deploy to testnet and mainnet.
