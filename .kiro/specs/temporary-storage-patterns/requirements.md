# Requirements Document

## Introduction

This feature adds a dedicated, focused example demonstrating **temporary storage patterns** in Soroban smart contracts. While `examples/basics/02-storage-patterns` covers all three storage tiers at a high level, this example zooms in on temporary storage specifically — its lifecycle, practical use cases (reentrancy guards, short-lived caches, intermediate computation), TTL mechanics, gas implications, and the critical distinction that expired temporary entries are **permanently deleted** (not archivable). The example will live at `examples/basics/temporary_storage/` (the directory already exists with a skeleton) and will be accompanied by a guide entry and documentation updates consistent with existing Soroban Cookbook conventions.

## Glossary

- **Temporary_Storage**: The `env.storage().temporary()` tier in Soroban. Entries have a short TTL (minimum 16 ledgers by default), are the cheapest storage option, and are permanently deleted on expiry — they cannot be restored.
- **TTL**: Time-To-Live. The number of ledgers an entry survives before expiry. For temporary storage, expiry means permanent deletion.
- **Ledger**: A single block on the Stellar network, closing approximately every 5 seconds.
- **Reentrancy_Guard**: A flag stored in temporary storage that prevents a contract function from being called recursively within the same transaction.
- **Intermediate_Result**: A value stored in temporary storage to pass state between steps of a multi-step operation within the same or adjacent ledgers.
- **Cache**: A short-lived copy of an expensive-to-compute or off-chain value (e.g., an oracle price) stored in temporary storage to avoid redundant computation within a bounded window.
- **Contract**: A Soroban smart contract compiled to WASM and deployed on the Stellar network.
- **Env**: The Soroban environment object providing access to storage, ledger info, and host functions.
- **Gas**: The computational and storage resource cost of executing a contract operation, paid in XLM fees.
- **Persistent_Storage**: The `env.storage().persistent()` tier. Entries survive indefinitely with TTL extension and can be restored after archival.
- **Instance_Storage**: The `env.storage().instance()` tier. All keys share one TTL tied to the contract instance lifetime.

---

## Requirements

### Requirement 1: Temporary Storage Example Contract

**User Story:** As a Soroban developer, I want a focused example contract that demonstrates temporary storage patterns, so that I can understand when and how to use `env.storage().temporary()` correctly.

#### Acceptance Criteria

1. THE `TemporaryStorageContract` SHALL expose a `set_temp` function that writes a value to Temporary_Storage under a caller-supplied key.
2. THE `TemporaryStorageContract` SHALL expose a `get_temp` function that reads a value from Temporary_Storage and returns `None` when the key does not exist.
3. THE `TemporaryStorageContract` SHALL expose a `has_temp` function that returns a boolean indicating whether a key exists in Temporary_Storage.
4. THE `TemporaryStorageContract` SHALL expose a `remove_temp` function that deletes a key from Temporary_Storage.
5. THE `TemporaryStorageContract` SHALL expose a `guarded_operation` function that uses a Reentrancy_Guard stored in Temporary_Storage to prevent recursive invocation.
6. THE `TemporaryStorageContract` SHALL expose a `cache_value` function that stores a Cache entry in Temporary_Storage with an explicit TTL extension.
7. THE `TemporaryStorageContract` SHALL expose a `get_cached` function that returns the cached value or `None` if the Cache has expired.
8. THE `TemporaryStorageContract` SHALL expose a `start_pipeline` and `finish_pipeline` function pair that stores an Intermediate_Result in Temporary_Storage between two logical steps.

---

### Requirement 2: Temporary Storage Lifecycle Documentation

**User Story:** As a Soroban developer, I want clear documentation of the temporary storage lifecycle, so that I understand how TTL works and why expired entries cannot be restored.

#### Acceptance Criteria

1. THE `README.md` for the example SHALL explain that Temporary_Storage entries are permanently deleted after TTL expiry and cannot be restored, distinguishing this from Persistent_Storage archival.
2. THE `README.md` SHALL document the minimum TTL for temporary entries (16 ledgers by default on mainnet) and the approximate real-world duration (≈80 seconds at 5 seconds per Ledger).
3. THE `README.md` SHALL include a lifecycle diagram or ASCII table showing the states: `written → active → expired → deleted`.
4. WHEN a Temporary_Storage entry's TTL reaches zero, THE `TemporaryStorageContract` SHALL treat the entry as absent (i.e., `has_temp` returns `false` and `get_temp` returns `None`).
5. THE `README.md` SHALL document the `extend_ttl` API for Temporary_Storage, including the `threshold` and `extend_to` parameters, with a concrete code example.

---

### Requirement 3: Performance Considerations

**User Story:** As a Soroban developer, I want to understand the performance characteristics of temporary storage compared to other storage tiers, so that I can make informed decisions about when to use it.

#### Acceptance Criteria

1. THE `README.md` SHALL include a comparison table of read/write/TTL-extension costs across Temporary_Storage, Instance_Storage, and Persistent_Storage.
2. THE `README.md` SHALL document that Temporary_Storage has the lowest write cost of the three storage tiers.
3. THE `README.md` SHALL document that Temporary_Storage does not require rent payments because entries are automatically deleted after a short TTL.
4. THE `README.md` SHALL include a "when to use" decision guide that lists concrete scenarios where Temporary_Storage is the correct choice (e.g., Reentrancy_Guard, Cache, Intermediate_Result) and scenarios where it is the wrong choice (e.g., user balances, admin config).
5. THE `README.md` SHALL document that reading from Temporary_Storage has lower overhead than reading from Persistent_Storage because no archival-restoration path exists.

---

### Requirement 4: Gas Implications Documentation

**User Story:** As a Soroban developer, I want documented gas implications of temporary storage operations, so that I can optimize contract costs.

#### Acceptance Criteria

1. THE `README.md` SHALL document that each `set` call to Temporary_Storage incurs a write fee proportional to the entry size and a short-term ledger rent fee.
2. THE `README.md` SHALL document that calling `extend_ttl` on a Temporary_Storage entry incurs an additional fee and should only be used when the data must survive beyond the default minimum TTL.
3. THE `README.md` SHALL document that `remove` on a Temporary_Storage entry reclaims ledger space immediately and avoids the natural expiry cost.
4. THE `README.md` SHALL include a concrete gas optimization tip: prefer `remove` over waiting for natural expiry when the data is no longer needed within the same transaction.
5. WHERE a contract stores many short-lived entries, THE `README.md` SHALL recommend batching writes and using a single composite key (e.g., a struct or tuple) rather than many individual keys to reduce per-entry overhead.

---

### Requirement 5: Test Coverage

**User Story:** As a Soroban developer, I want comprehensive tests for the temporary storage example, so that I can verify the lifecycle and behavior described in the documentation.

#### Acceptance Criteria

1. THE test suite SHALL include a test that verifies a value written to Temporary_Storage is readable within the same ledger.
2. WHEN the ledger sequence advances beyond the entry's TTL, THE test suite SHALL verify that `has_temp` returns `false` and `get_temp` returns `None`.
3. THE test suite SHALL include a test that verifies the Reentrancy_Guard pattern panics with a descriptive message when the guard is already set.
4. THE test suite SHALL include a test that verifies the Cache pattern returns the cached value before TTL expiry and `None` after TTL expiry.
5. THE test suite SHALL include a test that verifies `remove` immediately makes the entry absent without waiting for TTL expiry.
6. THE test suite SHALL include a test that verifies the pipeline pattern (`start_pipeline` / `finish_pipeline`) correctly passes an Intermediate_Result between the two steps.
7. THE test suite SHALL include a round-trip test: for any value written with `set_temp`, `get_temp` with the same key SHALL return that value before TTL expiry.

---

### Requirement 6: Integration with Cookbook Structure

**User Story:** As a cookbook maintainer, I want the temporary storage example to follow existing naming, structure, and documentation conventions, so that it integrates cleanly with the rest of the cookbook.

#### Acceptance Criteria

1. THE example SHALL be placed at `examples/basics/temporary_storage/` consistent with the existing skeleton directory.
2. THE example `Cargo.toml` SHALL use `soroban-sdk = { workspace = true }` and `soroban-sdk = { workspace = true, features = ["testutils"] }` for dev-dependencies, consistent with other examples.
3. THE `book/src/examples/basics.md` SHALL be updated to reference the Temporary Storage Patterns example.
4. THE `docs/best-practices.md` SHALL be updated or confirmed to include a reference to Temporary_Storage as the correct choice for ephemeral data such as Reentrancy_Guard and Cache patterns.
5. THE example `README.md` SHALL follow the same section structure used in `examples/basics/02-storage-patterns/README.md`: introduction, concepts, code examples, testing instructions, and further reading.
6. THE example SHALL include `test_snapshots/` directories consistent with other examples in the cookbook.
