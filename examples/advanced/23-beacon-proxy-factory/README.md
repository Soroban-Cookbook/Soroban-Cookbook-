# Beacon Proxy Factory

Factory-managed beacon proxies with shared upgrades. One beacon controls many proxy instances.

## Role in Learning Path

This is the **second step** in the [upgrade patterns learning path](../README.md#upgrade-patterns--proxy-patterns). After learning basic beacons, this example shows:
- Factory pattern for deploying proxy instances
- One beacon serving many proxies
- Batch upgrades of all proxies at once
- Cost-efficient multi-proxy management
- Decoupling proxy deployment from implementation

**Prerequisites:** Start with [`02-beacon-proxy`](../02-beacon-proxy/) to understand beacon basics.

**Next steps:**
- **[`31-proxy-admin`](../31-proxy-admin/)** — Add governance and safety checks
- **[`04-upgradeable-proxy`](../04-upgradeable-proxy/)** — Alternative pattern: storage in proxy
- **[`06-beacon-management`](../06-beacon-management/)** — Versioned implementations with rollback
- **[`07-upgrade-patterns`](../07-upgrade-patterns/)** — Complete upgrade strategies

## Key Concepts

- Factory-based proxy creation
- Beacon-mediated implementation sharing
- Atomic multi-proxy upgrades
- Cost optimization for multiple instances
- Shared upgrade governance

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

```rust
// Deploy 5 proxies in one transaction (batch gas optimisation).
let proxy_addrs = factory.batch_deploy(&deployer, &5u32);
```

### Shared implementation

All proxies deployed by the factory share a single beacon.  When a proxy is called
it resolves the current implementation from the beacon on every invocation:

```
proxy.add(1, 2)
  → proxy reads beacon_addr from its storage
  → proxy calls beacon.get_implementation()
  → proxy calls impl.add(1, 2)
  → returns 3
```

### Batch upgrades (O(1) cost)

Upgrading the beacon propagates to every proxy simultaneously:

```rust
// One call atomically upgrades all N deployed proxies.
factory.upgrade_beacon(&new_implementation, &label);
```

Compare this with a per-proxy upgrade pattern which would require O(N) transactions.

### Gas optimisation

| Technique | Benefit |
|---|---|
| Factory state in `instance` storage | All factory metadata loaded in a single ledger entry read |
| `batch_deploy(count)` | Amortises per-transaction overhead across N deployments |
| Shared beacon reference | Proxies store one address; no per-proxy registry lookup |
| Deterministic salts | Proxy addresses are predictable off-chain |

## Storage layout

### Factory (`instance`)

| Key | Type | Description |
|---|---|---|
| `Admin` | `Address` | Factory administrator |
| `Beacon` | `Address` | Address of the deployed shared beacon |
| `ProxyWasmHash` | `BytesN<32>` | WASM hash used to deploy new proxies |
| `Proxies` | `Vec<Address>` | Ordered list of all deployed proxy addresses |
| `ProxyCount` | `u32` | Cached proxy count used for O(1) count and salt lookups |

### Beacon (`persistent`)

| Key | Type | Description |
|---|---|---|
| `"admin"` | `Address` | Factory contract address (beacon's admin) |
| `"impl"` | `Address` | Current implementation contract address |
| `"version"` | `u32` | Monotonically-increasing upgrade counter |
| `VersionLog(n)` | `VersionEntry` | Historical record for version `n` |

### Proxy (`persistent`)

| Key | Type | Description |
|---|---|---|
| `"beacon"` | `Address` | The beacon this proxy is bound to |
| `"admin"` | `Address` | Can re-point this proxy to a different beacon |

## Building

Because a single WASM binary can only export one set of contract entry-points,
build each contract separately using cargo features:

```bash
cargo build -p beacon-proxy-factory --target wasm32v1-none --release \
    --no-default-features --features factory

cargo build -p beacon-proxy-factory --target wasm32v1-none --release \
    --no-default-features --features beacon

cargo build -p beacon-proxy-factory --target wasm32v1-none --release \
    --no-default-features --features proxy

cargo build -p beacon-proxy-factory --target wasm32v1-none --release \
    --no-default-features --features impl-v1

cargo build -p beacon-proxy-factory --target wasm32v1-none --release \
    --no-default-features --features impl-v2
```

## Running tests

Tests run in `rlib` mode and register all contracts via `env.register()`, so
all modules are included unconditionally:

```bash
cargo test -p beacon-proxy-factory
```

## Test coverage

| # | Test | Acceptance criteria covered |
|---|---|---|
| 1 | `test_factory_beacon_init` | Shared implementation |
| 2 | `test_proxy_binds_to_beacon` | Shared implementation |
| 3 | `test_proxy_delegates_arithmetic` | Shared implementation |
| 4 | `test_deploy_multiple_proxies_shared_beacon` | Deploy multiple proxies |
| 5 | `test_upgrade_beacon_propagates_to_all_proxies` | Batch upgrades |
| 6 | `test_mul_available_after_upgrade` | Batch upgrades |
| 7 | `test_upgrade_beacon_unauthorized` | Security |
| 8 | `test_beacon_double_init_panics` | Guard rails |
| 9 | `test_proxy_double_init_panics` | Guard rails |
| 10 | `test_proxy_unique_addresses` | Deploy multiple proxies |
| 11 | `test_proxy_counter_independent_per_instance` | Shared implementation semantics |
| 12 | `test_batch_deploy_simulation` | Deploy multiple proxies, gas optimisation |
| 13 | `test_beacon_version_history` | Version auditing |
| 14 | `test_beacon_transfer_admin` | Admin management |
| 15 | `test_proxy_set_beacon` | Canary deployment pattern |
| 16 | `test_single_upgrade_updates_n_proxies` | Gas optimisation (O(1) upgrade) |
| 17 | `test_beacon_version_not_found_panics` | Guard rails |
| 18 | `test_v1_functions_work_with_multiple_proxies` | Deploy multiple proxies |
| 19 | `test_upgrade_then_rollback_via_upgrade` | Batch upgrades / rollback |
| 20 | `test_proxy_set_beacon_unauthorized` | Security |

## Related examples

- **Sequence step 4 of 6:** follows [Beacon Proxy](../02-beacon-proxy/) and
    demonstrates factory deployment of multiple proxies sharing one beacon.
- **In scope:** deploying and tracking a proxy fleet with one shared beacon.
- **Out of scope:** managing multiple independent named beacons; continue to
    [Beacon Management](../06-beacon-management/).
- [Next: Beacon Management](../06-beacon-management/) — versioned, named beacons with rollback
- [`intermediate/ajo-factory`](../../intermediate/ajo-factory/) — factory deployer pattern

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.
