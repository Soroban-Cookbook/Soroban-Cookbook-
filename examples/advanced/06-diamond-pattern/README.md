# Diamond Pattern Base (EIP-2535 Soroban Adaptation)

A fully dynamic, introspectable diamond routing system for Soroban — the on-chain equivalent of EIP-2535.

## What You'll Learn

- How the **diamond storage pattern** prevents key collisions across facets using a namespaced `DataKey` enum
- How to implement a **facet registry** with add, replace, and remove operations (diamond cut)
- How to map **function selectors** (`Symbol` identifiers) to facet contract addresses at runtime
- How to implement a **fallback dispatch mechanism** using loupe introspection
- How to write **diamond-loupe** functions that expose the complete routing table on-chain

## Contract Overview

### DiamondBase

The core diamond contract. It owns the `selector → facet-address` routing table and exposes:

| Function | Description |
|----------|-------------|
| `initialize(admin)` | Bootstrap the diamond (once only) |
| `diamond_cut(admin, cuts)` | Batch add / replace / remove facet selectors |
| `facets()` | Loupe: all facets and their selectors |
| `facet_addresses()` | Loupe: all registered facet addresses |
| `facet_function_selectors(facet)` | Loupe: selectors served by a specific facet |
| `facet_address(selector)` | Loupe + fallback: which facet handles a selector |
| `selector_count()` | Total number of registered selectors |

### TokenFacet

Demonstrates the diamond storage pattern in a token context. All state lives under the `Tf*` `DataKey` variants — completely isolated from any other facet's storage.

### CounterFacet

A second independent facet with its own `Cf*` storage namespace, proving that multiple facets coexist without key collisions.

## Key Concepts

### Diamond Storage Pattern

In EIP-2535, each facet stores its data at a deterministic keccak256 slot to avoid layout collisions. Soroban achieves the same guarantee via enum variant discrimination: `TfBalance(owner)` and `CfCount(name)` use different discriminants and can never occupy the same storage entry, even when the parameter values are identical.

```rust
#[contracttype]
pub enum DataKey {
    // Diamond registry
    SelectorFacet(Symbol),      // selector → facet address
    FacetSelectorList(Address), // facet → its selectors

    // TokenFacet namespace
    TfBalance(Address),         // isolated from CounterFacet
    TfTotalSupply,

    // CounterFacet namespace
    CfCount(Symbol),            // isolated from TokenFacet
}
```

### Facet Registry (Diamond Cut)

Facets are registered dynamically at runtime:

```rust
let mut cuts = Vec::new(&env);
cuts.push_back(FacetCut {
    facet_address: token_contract_id,
    action: FacetCutAction::Add,
    selectors: vec![&env, symbol_short!("tf_mint"), symbol_short!("tf_trsf")],
});
diamond.diamond_cut(&admin, &cuts);
```

Three actions mirror EIP-2535:
- **Add** — register new selectors pointing to a new facet
- **Replace** — re-point existing selectors to a different facet
- **Remove** — deregister selectors entirely (prunes the facet from the list when empty)

### Function Selector Mapping

`Symbol` values serve as function selectors. The registry maps each selector to the facet address that implements it:

```rust
// Register
diamond.diamond_cut(&admin, &[FacetCut {
    facet_address: token_id,
    action: FacetCutAction::Add,
    selectors: vec![&env, symbol_short!("tf_mint")],
}]);

// Lookup
let facet = diamond.facet_address(&symbol_short!("tf_mint")); // Some(token_id)
```

### Fallback Dispatch Mechanism

Callers resolve the correct facet contract via `facet_address`, then dispatch to it directly — the Soroban equivalent of EIP-2535's fallback function routing:

```rust
let facet_addr = diamond
    .facet_address(&symbol_short!("tf_mint"))
    .expect("selector not registered");

// Dispatch to the resolved facet
let token_client = TokenFacetClient::new(&env, &facet_addr);
token_client.tf_mint(&minter, &recipient, &1_000i128);
```

## Security Considerations

- **Admin-only cuts**: Only the address set during `initialize` may call `diamond_cut`. Adding multi-sig or timelock control on top is strongly recommended for production use.
- **No duplicate selectors**: `Add` panics on duplicate registrations, preventing accidental shadowing.
- **Replace enforces existence**: `Replace` panics if the selector is not already registered, ensuring routing tables stay consistent.
- **Namespace isolation**: The `DataKey` enum guarantees that facet A's storage is never readable or writable by facet B, eliminating cross-facet storage corruption.
- **Facet pruning**: `Remove` automatically drops facets with no remaining selectors from the global list, keeping the loupe data accurate.

## Testing

```bash
cargo test -p diamond-pattern
```

The test suite covers 22 scenarios:
- Diamond initialisation and double-init guard
- Diamond cut: add, replace, remove, duplicate-add guard
- Diamond loupe: facets, addresses, per-facet selectors, unknown selectors
- Fallback dispatch via loupe resolution
- TokenFacet: mint, transfer, insufficient-balance guard, approve + transfer-from, allowance-exceeded guard
- CounterFacet: increment, reset, independent named counters
- Cross-facet storage isolation

## Building

```bash
# Native (for tests)
cargo build -p diamond-pattern

# WASM (for on-chain deployment)
cargo build --target wasm32-unknown-unknown --release -p diamond-pattern
```
