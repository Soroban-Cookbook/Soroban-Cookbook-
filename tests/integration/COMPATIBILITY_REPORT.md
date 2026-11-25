# Cross-Version & Environment Compatibility Report

This report documents the compatibility testing methodology, version matrix, upgrade verification, and protocol simulation results for the Soroban Cookbook smart contract suite (Issue #950, Phase 6 Issue #306).

---

## 1. Executive Summary

Smart contracts deployed on Stellar must maintain deterministic execution across Soroban SDK versions, network protocol upgrades (Protocol 20, 21, 22), and contract upgrades via upgradeable proxies.

The compatibility test suite ([`tests/integration/tests/compatibility_tests.rs`](./tests/compatibility_tests.rs)) provides automated verification covering:
1. **Soroban SDK Type Representation**: Serialization and memory safety of core types (`Address`, `Val`, `Symbol`, `String`, `Vec`, `Map`, `Bytes`, `BytesN`).
2. **Network Protocol Version Simulation**: Ledger state persistence and compatibility across Protocol 20 (Mainnet launch), Protocol 21, and Protocol 22.
3. **Upgrade State Preservation**: Seamless logic migration via proxy contracts without storage corruption.
4. **Storage Layout Evolution**: Backwards-compatible schema expansion (`LegacyAccountData` → `ModernAccountData`).
5. **Error Code Numeric Stability**: Stable discriminant values preventing RPC client deserialization errors.

---

## 2. Version Compatibility Matrix

| Component | Target Version | Supported Range | Compatibility Status |
|-----------|----------------|-----------------|----------------------|
| **Soroban SDK** | `v22.0.x+` | `v20.0.0` – Current | ✅ Fully Compatible |
| **Rust Toolchain** | `1.80.0+` | `1.74.0` – Current | ✅ Fully Compatible |
| **Stellar Network Protocol** | Protocol 21 / 22 | Protocol 20+ | ✅ Verified in simulation |
| **WASM Target** | `wasm32-unknown-unknown` | Standard | ✅ Compatible |
| **Stellar XDR** | `v21.x` / `v22.x` | `v20.0` – Current | ✅ Wire-format Compatible |

---

## 3. Test Coverage & Verification Scenarios

### 1. SDK Core Types Compatibility (`test_sdk_version_type_compatibility`)
- **Symbol**: Short symbol literals (`symbol_short!`) match dynamically generated `Symbol::new` values.
- **Address**: Roundtrip conversion between `Address` and `Val` preserves cryptographic identity.
- **Collections**: `Vec` and `Map` maintain consistent indexing and lookup performance.
- **Binary Payloads**: `Bytes` (dynamic) and `BytesN` (fixed-size 32-byte) preserve byte representations.

### 2. Network Protocol Simulation (`test_network_compatibility_ledger_protocol_simulation`)
- Simulates ledger state transition from Protocol 20 (sequence 1,000) to Protocol 21 (sequence 150,000) and Protocol 22 (sequence 500,000).
- Verifies that persistent storage entries written under Protocol 20 remain accessible and writable under subsequent protocol upgrades.
- Validates TTL handling and sequence progression.

### 3. Upgrade Compatibility (`test_upgrade_compatibility_state_preservation`)
- Tests an upgradeable proxy pattern with `UpgradeableProxyImplementationV1`.
- State written to the proxy contract (e.g. `counter`) is preserved when upgrading the contract's implementation address.
- Post-upgrade calls correctly execute the new implementation logic while operating on pre-upgrade storage state.

### 4. Storage Layout Schema Evolution (`test_storage_layout_backward_compatibility`)
- Verifies that adding new fields (e.g. `flags: u32`) to existing storage structs does not break existing persistent entries.
- Validates a migration pattern from `LegacyAccountData` to `ModernAccountData`.

### 5. Error Code Discriminant Stability (`test_custom_error_code_stability`)
- Validates that error numeric representations (`#[repr(u32)]`) remain invariant across versions:
  - `custom_errors::Error::ExceedsLimit` = 1
  - `custom_errors::Error::ZeroAmount` = 2
  - `custom_errors::Error::NegativeAmount` = 3
  - `basic_nft::NftError::AlreadyInitialized` = 1
  - `basic_nft::NftError::TokenNotFound` = 3

---

## 4. Execution & Verification

### Running the Compatibility Test Suite

```bash
cargo test -p integration-tests --test compatibility_tests
```

### Continuous Integration (CI)

Integrated into `.github/workflows/test.yml` under `cargo test -p integration-tests`. Automated execution verifies that compatibility guarantees remain satisfied on every merge.
