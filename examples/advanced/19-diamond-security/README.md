# Diamond Security Pattern

**A security-focused variant of the diamond pattern** that demonstrates how to harden multi-facet proxy systems against common attack vectors.

> **Base Implementation**: This example focuses on security-specific concerns. For the canonical diamond pattern with full EIP-2535 functionality, see [`06-diamond-pattern`](../06-diamond-pattern/).

This example shows how to mitigate security risks associated with shared proxies, access controls, upgrade safety, and storage collisions in diamond-based architectures.

## Security Challenges & Solutions

The Diamond pattern splits a contract's logic into multiple implementation contracts (facets) behind a single routing proxy. This structure presents several security challenges that this pattern resolves:

### 1. Access Control Per Facet (Direct Call Protection)
*   **The Risk:** If users can call facet contracts directly, they bypass the proxy's administrative state, validations, and fees, potentially causing inconsistent states or exploit opportunities.
*   **The Solution:** Facets are initialized with the Diamond Proxy's address. Each facet function requires that the caller is the Diamond Proxy:
    ```rust
    pub fn add(env: Env, caller: Address, ...) {
        caller.require_auth();
        assert_eq!(caller, expected_diamond);
    }
    ```
    This completely blocks direct, unauthorized calls.

### 2. Storage Collision Prevention
*   **The Risk:** In traditional EVM diamonds, facets share the proxy's raw storage slots. If two facets define different variables that resolve to the same slot, they overwrite each other's data (storage collision).
*   **The Solution:** Soroban is key-value based. The Proxy exposes an isolated **Namespaced Storage API** where facets read and write state using a composite key containing the facet's own contract Address:
    ```rust
    // Proxy level storage mapping
    let storage_key = DataKey::NamespacedStorage(facet_address, key);
    env.storage().persistent().set(&storage_key, &value);
    ```
    This mathematically guarantees that no two facets can collide, even if they use the same storage keys (e.g. `"count"` or `"balance"`).

### 3. Upgrade Safeguards
*   **The Risk:** Registering a facet that does not support the expected functions, or registering duplicate functions, can brick the contract or corrupt routing logic.
*   **The Solution:**
    *   **Interface Verification:** The Proxy performs a pre-flight check when adding a facet by calling its `support(functions)` method. If the facet does not confirm support, registration fails.
    *   **Duplicate Detection:** The Proxy verifies a function selector is not already mapped to another facet before updating the route.
    *   **Immutable Core Administration:** Core proxy administrative functions (`add_facet`, `remove_facet`, `upgrade_admin`) are hardcoded in the Proxy and cannot be delegation-overwritten, preventing the Diamond from being bricked.

---

## Project Structure

```text
examples/advanced/19-diamond-security/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs     # Diamond proxy, facets, and routing logic
    └── test.rs    # Verification tests (isolation, auth, upgrades)
```

---

## How to Build

```bash
# Compile the contract
cargo build -p diamond-security --target wasm32-unknown-unknown --release
```

---

## How to Test

```bash
# Run the security tests
cargo test -p diamond-security
```

The tests cover:
- Admin authentication checks.
- Pre-flight interface check validations.
- Blocked direct access to facets (`InvalidCaller`).
- Isolated storage isolation testing (verifying no collision on shared keys).
- Facet removal and routing cleanup.
- Admin upgrades.

---

## Related Examples

- **[06-diamond-pattern](../06-diamond-pattern/)** — The canonical diamond implementation with full EIP-2535 functionality including diamond-cut operations and diamond-loupe introspection
- **[05-diamond-facets](../05-diamond-facets/)** — Router orchestration patterns with inter-facet communication examples
- **[BEST_PRACTICES.md](./BEST_PRACTICES.md)** — Detailed security best practices guide for diamond deployments
