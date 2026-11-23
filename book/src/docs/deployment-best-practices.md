# Deployment Best Practices

This page covers how to take a Soroban contract from a passing test suite to a production deployment you can monitor and upgrade safely. It adds to the step-by-step [Deployment Guide](../guides/deployment.md). That guide shows the commands; this page explains which ones to run, when, and why.

> **CLI naming:** The Soroban CLI is now called `stellar` (package `stellar-cli`). Older pages in this book still use `soroban`. The subcommands are the same, so `soroban contract deploy` is now `stellar contract deploy`.

## Contents

- [Pre-deployment Checklist](#pre-deployment-checklist)
- [Network Selection](#network-selection)
- [Reproducible Builds](#reproducible-builds)
- [Deploying](#deploying)
- [Monitoring Setup](#monitoring-setup)
- [Upgrade Strategies](#upgrade-strategies)
- [State Archival and TTL](#state-archival-and-ttl)
- [Incident Response](#incident-response)
- [Quick Reference](#quick-reference)

---

## Pre-deployment Checklist

Work through every item before deploying to mainnet. Use the same list on testnet so the process is already routine when it counts.

### Code and tests

- [ ] `cargo test` passes for the contract and for every integration test that uses it.
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- [ ] Every error path has a test, including unauthorized callers, zero and negative amounts, overflow, and calls made before initialization.
- [ ] Arithmetic uses `checked_*` operations, or the crate sets `overflow-checks = true` in its release profile.
- [ ] No `panic!`, `unwrap()`, or `expect()` is reachable from user input. Return a `#[contracterror]` instead.
- [ ] Every storage key has a deliberate durability (`instance`, `persistent`, or `temporary`) and a TTL plan. See [State Archival and TTL](#state-archival-and-ttl).

### Security

- [ ] Every state-changing function calls `require_auth()` on the right address.
- [ ] Initialization can run only once, and it is guarded against front-running. Deploy and initialize in one transaction where you can, using a constructor (`__constructor`) on SDK 22 or later.
- [ ] Admin keys are multisig accounts, not single keys. See the [Security Best Practices](./security-best-practices.md).
- [ ] An external review or audit is done for contracts that hold value. Findings are fixed or formally accepted.
- [ ] `cargo audit` shows no unresolved advisories.

### Operations

- [ ] The release WASM is built reproducibly and its hash is recorded. See [Reproducible Builds](#reproducible-builds).
- [ ] The deployer account is funded, and its key is stored in a hardware wallet or secrets manager, never in the repo.
- [ ] Initialization arguments (admin address, token addresses, fee parameters) are written down and checked by a second person.
- [ ] Monitoring and alerts are live before launch. See [Monitoring Setup](#monitoring-setup).
- [ ] The upgrade path and the emergency pause path have both been run end to end on testnet.
- [ ] A rollback plan exists: you know the WASM hash of the previous version and who can authorize an upgrade back to it.

---

## Network Selection

| Network | Use it for | Data lifetime | Funding |
|---|---|---|---|
| **Local** (`stellar container start local`) | Fast iteration, CI, and deterministic integration runs | Lost when the container stops | Built-in friendbot |
| **Futurenet** | Testing protocol features that are not yet on testnet | Reset without notice | Friendbot |
| **Testnet** | Staging, demos, wallet testing, and rehearsing a release | Reset periodically, roughly quarterly | Friendbot |
| **Mainnet** | Production | Permanent | Real XLM |

### Recommendations

- **Develop locally, stage on testnet, and release on mainnet.** Skip Futurenet unless you need a protocol feature that only exists there.
- **Do not treat testnet as permanent.** Testnet resets wipe all contracts and accounts, so keep deployment scripts that can redeploy from scratch.
- **Pin the network passphrase in config, not in code paths users can change.** A transaction signed for one network is invalid on every other network, and that is a useful safety property.
- **Use a dedicated RPC provider on mainnet.** SDF's public testnet RPC is rate-limited and meant for development. For production, use an RPC provider from the [Stellar RPC providers list](https://developers.stellar.org/docs/data/apis/rpc/providers) or run your own `stellar-rpc` node.

```bash
# Testnet
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Mainnet (replace with your provider's URL)
stellar network add mainnet \
  --rpc-url https://<your-rpc-provider>/ \
  --network-passphrase "Public Global Stellar Network ; September 2015"

# Futurenet
stellar network add futurenet \
  --rpc-url https://rpc-futurenet.stellar.org \
  --network-passphrase "Test SDF Future Network ; October 2022"
```

---

## Reproducible Builds

Anyone should be able to rebuild your WASM from source and get the same hash. Without that, nobody can check that the contract on chain matches the code you published.

1. **Pin the toolchain.** Commit `rust-toolchain.toml` with an exact version, such as `channel = "1.84.0"`, plus the `wasm32v1-none` or `wasm32-unknown-unknown` target.
2. **Commit `Cargo.lock`.**
3. **Use the release profile the cookbook uses** (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = true`).
4. **Build and optimize:**

   ```bash
   stellar contract build --meta source_repo=github:<org>/<repo>
   stellar contract optimize --wasm target/wasm32v1-none/release/my_contract.wasm
   sha256sum target/wasm32v1-none/release/my_contract.optimized.wasm
   ```

5. **Build in CI from a tagged commit.** The [stellar-expert/soroban-build-workflow](https://github.com/stellar-expert/soroban-build-workflow) GitHub Action builds the WASM, attaches it to a release, and records build metadata so explorers can show the contract as verified (SEP-55).
6. **Record the hash** in the release notes and in your deployment log.

---

## Deploying

Separate *uploading* code from *instantiating* a contract. Several contracts can share the same uploaded WASM, and upgrades reference it by hash.

```bash
# 1. Upload the WASM once; prints the WASM hash
WASM_HASH=$(stellar contract upload \
  --wasm my_contract.optimized.wasm \
  --source deployer --network testnet)

# 2. Deploy an instance. Constructor arguments come after `--`
CONTRACT_ID=$(stellar contract deploy \
  --wasm-hash "$WASM_HASH" \
  --source deployer --network testnet \
  -- --admin <ADMIN_MULTISIG_ADDRESS>)

# 3. Confirm the on-chain code matches what you built
stellar contract fetch --id "$CONTRACT_ID" --network testnet --out-file onchain.wasm
sha256sum onchain.wasm my_contract.optimized.wasm
```

Practices:

- **Use a constructor**, or deploy and initialize atomically. A separate `initialize` call can be front-run by anyone who sees the deployment first.
- **Use a deployer key that holds no privileges.** After deployment, admin rights should belong to a multisig account, not to the key that paid for the deployment.
- **Script every step.** Keep deployment scripts in the repo and run them on testnet with the same arguments you will use on mainnet.
- **Simulate first.** Commands that write to the network run a simulation before they submit. Use `--send=no` on `stellar contract invoke` to inspect resource usage and fees without submitting.
- **Log everything.** Keep a deployment log with the date, network, git tag, WASM hash, contract ID, initialization arguments, and who approved.

---

## Monitoring Setup

Set up monitoring before launch, not after the first incident.

### What to watch

| Signal | Source | Alert when |
|---|---|---|
| Contract events (transfers, admin actions, pauses, upgrades) | RPC `getEvents`, or an indexer | Any admin or upgrade event; volume spikes |
| Failed invocations | RPC `getTransactions`, or an indexer | Failure rate above baseline |
| Admin and treasury balances | RPC `getLedgerEntries` or Horizon | Balance below the threshold needed for fees or operations |
| TTL of instance and critical persistent entries | RPC `getLedgerEntries` (`liveUntilLedgerSeq`) | Fewer than N days of ledgers left |
| RPC provider health | RPC `getHealth` | Unhealthy, or the latest ledger is falling behind |
| Contract WASM hash | RPC `getLedgerEntries` on the instance | Hash changes outside a planned upgrade |

### Make the contract observable

Emit an event for every state change you might need to audit later, especially privileged actions:

```rust,ignore
env.events().publish(
    (symbol_short!("admin"), symbol_short!("upgrade")),
    (old_version, new_version, new_wasm_hash),
);
```

Use stable, documented topic names so indexers and alert rules do not break between versions. See [Events](../examples/events.md) for patterns.

### Tooling

- **Explorers** such as [Stellar Expert](https://stellar.expert) and [Stellar Lab](https://lab.stellar.org) for manual inspection.
- **Indexers** such as [Mercury](https://mercurydata.app) and [SubQuery](https://subquery.network) for event history and dashboards.
- **A small poller** that calls `getEvents` on a schedule and sends matching events to Slack, PagerDuty, or Discord. RPC keeps only a short window of event history, about 7 days by default, so poll often or use an indexer for anything longer.

---

## Upgrade Strategies

Soroban contracts are immutable unless the contract itself exposes an upgrade function. Decide which model you want before the first deployment.

### 1. Immutable

Do not include an upgrade function. To ship a new version, deploy a new contract and migrate users to it.

- **Pros:** Strongest trust guarantee, and the code cannot change underneath users.
- **Cons:** Bugs cannot be fixed in place, and state has to be migrated out.
- **Use for:** Simple tokens and contracts where users must not have to trust an admin.

### 2. In-place WASM upgrade (admin-gated)

The contract replaces its own code and keeps its address and storage:

```rust,ignore
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
    Ok(())
}
```

- Gate the function behind a **multisig** admin, and preferably a **timelock** as well, so users can see an upgrade coming and exit before it takes effect. See the [timelock example](../examples/advanced.md).
- Emit an upgrade event that includes the new hash.
- **Keep the storage layout compatible.** Add new `DataKey` variants, but never reorder or change the types of existing ones.

### 3. Upgrade + migrate

When the storage layout must change, add a versioned `migrate` function that runs exactly once per version:

```rust,ignore
pub fn migrate(env: Env) -> Result<(), Error> {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
    admin.require_auth();
    let version: u32 = env.storage().instance().get(&DataKey::Version).unwrap_or(1);
    if version >= 2 {
        return Err(Error::AlreadyMigrated);
    }
    // ...transform v1 entries into the v2 layout...
    env.storage().instance().set(&DataKey::Version, &2u32);
    Ok(())
}
```

Call `upgrade` and then `migrate` in quick succession, and pause the contract in between if users could hit a half-migrated state.

### 4. Proxy / registry

Users call a stable proxy or registry that forwards to the current implementation. This adds a cross-contract call to every invocation, but you can switch implementations without changing the user-facing address. See [Factory, Proxy, and Registry Patterns](./cross-contract-patterns.md).

### Upgrade runbook

1. Tag the release and build the WASM reproducibly. Record the hash.
2. On testnet, deploy the *old* version, fill it with realistic state, upgrade it, migrate it, and run the integration tests against it.
3. Run `stellar contract upload` on mainnet and confirm the hash matches the one you recorded.
4. Propose the upgrade through the multisig or timelock.
5. Once it executes, run `migrate` if needed. Then check `version()` and a few read functions.
6. Watch error rates and events closely for 24 hours.
7. If something goes wrong, roll back by upgrading to the previous hash. This only works if the storage layout is still compatible.

---

## State Archival and TTL

Soroban entries expire unless their TTL is extended. Expired `persistent` and `instance` entries are archived and must be restored before they can be used. Expired `temporary` entries are deleted for good.

- Extend the instance TTL inside frequently called functions, for example `env.storage().instance().extend_ttl(threshold, extend_to)`.
- Extend long-lived persistent entries when they are written or read.
- Run a scheduled job that tops up TTL for critical entries:

  ```bash
  stellar contract extend --id "$CONTRACT_ID" --durability persistent \
    --key-xdr <KEY_XDR> --ledgers-to-extend 535679 \
    --source ops --network mainnet
  ```

- Keep `stellar contract restore` in the incident runbook for entries that do get archived.

---

## Incident Response

- **Pause first, investigate second.** Ship a pause switch (see [Pause / Unpause](../examples/pause-unpause.md)) that blocks state changes but keeps reads working.
- **Decide in advance who can pause.** A smaller guardian multisig can pause, while unpausing and upgrading need the full admin set.
- **Keep the runbook next to the code**, covering who is contacted, which commands are run, and how users are told.
- **Write a post-mortem** for every incident and add a regression test for its cause.

---

## Quick Reference

| Task | Command |
|---|---|
| Build | `stellar contract build` |
| Optimize | `stellar contract optimize --wasm <file>` |
| Upload code | `stellar contract upload --wasm <file> --source <id> --network <net>` |
| Deploy instance | `stellar contract deploy --wasm-hash <hash> --source <id> --network <net> -- <ctor args>` |
| Inspect | `stellar contract info interface --id <id> --network <net>` |
| Fetch on-chain WASM | `stellar contract fetch --id <id> --network <net> --out-file <file>` |
| Dry-run an invocation | `stellar contract invoke --send=no --id <id> ... -- <fn> <args>` |
| Extend TTL | `stellar contract extend --id <id> --durability persistent --ledgers-to-extend <n>` |
| Restore archived state | `stellar contract restore --id <id> --durability persistent` |

## Related

- [Deployment Guide](../guides/deployment.md): step-by-step commands
- [Security Best Practices](./security-best-practices.md)
- [Best Practices](./best-practices.md)
- [Testing Best Practices](./testing-best-practices.md)
