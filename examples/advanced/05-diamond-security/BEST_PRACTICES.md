# Diamond Security Pattern: Best Practices

Multi-facet proxy patterns (commonly known as the Diamond Pattern or EIP-2535) are powerful architectural designs for smart contracts. They allow developers to bypass contract size limits, organize code into modular components (facets), and upgrade specific functionalities without altering the main entrypoint address.

However, the delegation of execution to external facets introduces unique security risk vectors. This guide outlines the core security practices for designing, implementing, and maintaining secure Diamond proxy structures on Soroban.

---

## 🔒 1. Access Control Per Facet (Direct Call Protection)

### The Risk
Because facets are deployed as standalone contracts on the blockchain, anyone can call their functions directly. If a user interacts with a facet contract directly, they bypass:
*   The Diamond Proxy's admin checks and access controls.
*   The Proxy's state constraints (e.g. paused flags).
*   Any tracking, validation, or protocol fees implemented in the Proxy router.

This bypass can leave the facet state out of sync or allow attackers to execute privileged functions if authentication is not validated properly.

### Mitigation Pattern
Facets should reject any execution that does not originate from the Diamond Proxy.
1.  **Initialize the facet** with the Diamond Proxy address during deployment.
2.  **Verify the caller's identity** in every state-modifying function:
    ```rust
    pub fn execute_logic(env: Env, caller: Address, ...) -> Result<(), Error> {
        // 1. Authenticate the caller parameter
        caller.require_auth();

        // 2. Load the expected Diamond address from instance storage
        let diamond: Address = env.storage().instance().get(&DataKey::Diamond).unwrap();

        // 3. Enforce caller identity
        if caller != diamond {
            return Err(Error::InvalidCaller);
        }

        // ... Logic ...
        Ok(())
    }
    ```

---

## 💾 2. Storage Collision Prevention

### The Risk
In traditional EVM diamonds, facets write directly into the proxy's storage slots using `delegatecall`. If Facet A defines a variable `balance` at storage slot 0, and Facet B defines a variable `paused` at storage slot 0, Facet B's writes will silently overwrite Facet A's data, causing catastrophic state corruption.

### Mitigation Pattern (Namespaced Storage)
In Soroban, storage is organized as key-value pairs (`SCVal` to `SCVal`). While facets run in their own contract instances (with distinct storage isolation) when called via `invoke_contract`, they might share the Proxy's state for common ledger variables.

If facets share the Proxy's storage, the Proxy must isolate keys using **Namespaced Keys**:
1.  **Composite Keys:** Build storage keys by combining the facet's unique contract Address or Namespace Symbol with the logical key:
    ```rust
    // In the Diamond Proxy Contract
    #[contracttype]
    pub enum DataKey {
        NamespacedStorage(Address, Symbol), // Namespace by facet address
    }
    ```
2.  **API Gating:** Expose a gated getter/setter on the Diamond Proxy that requires the facet to authenticate and automatically applies the namespace prefix:
    ```rust
    pub fn set_facet_storage(env: Env, facet: Address, key: Symbol, value: Val) {
        facet.require_auth();
        // Verify facet is registered
        assert!(is_registered_facet(&env, &facet));

        let storage_key = DataKey::NamespacedStorage(facet, key);
        env.storage().persistent().set(&storage_key, &value);
    }
    ```

---

## 🛠️ 3. Upgrade Safeguards & Interface Checking

### The Risk
Registering a facet that lacks the required method implementations or contains conflicting function names can break the routing system or leave the contract bricked.

### Mitigation Pattern
1.  **Interface Verification Check:** Facets must implement a standard trait indicating which methods they support. When registering a facet, the Proxy should invoke this checker to ensure the facet implements the registered functions:
    ```rust
    // Inside add_facet on the Proxy
    let client = FacetClient::new(&env, &facet_address);
    let supports = client.supports_interface(&registered_functions);
    if !supports {
        panic!("Facet interface mismatch");
    }
    ```
2.  **Duplicate Method Protection:** The Proxy must verify that a function name is not already mapped to an existing facet before registering a new route:
    ```rust
    for func in functions.iter() {
        let key = DataKey::FacetMap(func.clone());
        if env.storage().persistent().has(&key) {
            panic!("Duplicate function registration");
        }
    }
    ```
3.  **Immutable Core Router:** Core Proxy administration methods (like adding/removing facets, changing admin) must be hardcoded inside the Proxy contract itself and **never** delegated to external facets. This ensures that the admin always maintains the ability to recover from a faulty or malicious facet upgrade.

---

## 📋 4. Checklist for Diamond Deployments

*   [ ] **Facet Immutable Registry:** Ensure the Diamond Proxy only accepts facets that have passed a security audit.
*   [ ] **Reentrancy Protection:** Be mindful of reentrancy when calling facets. If a facet calls back into the Diamond Proxy or another facet, ensure execution paths are validated or protected by reentrancy guards.
*   [ ] **TTL Management:** Make sure both the Diamond Proxy and all facet contracts extend the Time-To-Live (TTL) of their storage keys on every write/read.
*   [ ] **Fallback Handling:** Handle unregistered function calls gracefully, returning a clear `FacetNotFound` error rather than panicking generic.
