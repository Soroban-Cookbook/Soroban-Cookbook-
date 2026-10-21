# Diamond Facets — Router Orchestration Pattern

**A facet-organization focused variant of the diamond pattern** that demonstrates router orchestration and inter-facet communication patterns.

> **Base Implementation**: This example focuses on facet coordination and cross-facet operations. For the canonical diamond pattern with full EIP-2535 functionality, see [`06-diamond-pattern`](../06-diamond-pattern/).

Demonstrates the Diamond / multi-facet architecture pattern on Soroban, extended with full **loupe-style introspection** so callers can discover which facets are registered and which selectors each facet exposes at runtime.

## Concepts

This example emphasizes **facet coordination and orchestration** — how a diamond router can atomically coordinate multiple facets in a single transaction.

| Concept | Description |
|---------|-------------|
| **Facet contracts** | Separate contracts each responsible for one concern |
| **Facet interface pattern** | Each facet exposes a typed client the router uses |
| **Storage isolation** | Distinct `DataKey` variant prefix per facet prevents key collisions |
| **Inter-facet communication** | Router orchestrates multiple facets in one transaction |

## Facets

### TokenFacet
ERC-20-like token operations:
- `mint(minter, to, amount)` — create new tokens
- `transfer(from, to, amount)` — move tokens between accounts
- `approve(owner, spender, amount)` / `transfer_from(...)` — allowance pattern
- `balance_of(owner)` / `total_supply()` — queries

### AccessFacet
Role-based permissions:
- `initialize(admin)` — bootstrap with initial admin
- `grant_role(admin, account, role)` / `revoke_role(admin, account)` — admin-only
- `get_role(account)` / `has_role(account, required_role)` — queries

Roles: `ROLE_USER = 0`, `ROLE_MINTER = 1`, `ROLE_ADMIN = 2`

### RegistryFacet
Key → value metadata store with ownership:
- `set_entry(caller, key, value)` — register or update (owner only for updates)
- `remove_entry(caller, key)` — delete (owner only)
- `get_entry(key)` / `get_owner(key)` — queries

### DiamondRouter
Orchestration layer:
- `register_facets(admin, token, access, registry)` — one-time facet registration
- `mint_and_register(...)` — atomic cross-facet operation (mint + log metadata)
- `get_facet(name)` — retrieve a facet address by name

## Storage Isolation Design

```
DataKey::Balance(addr)          ← TokenFacet only
DataKey::Allowance(addr, addr)  ← TokenFacet only
DataKey::TotalSupply            ← TokenFacet only
DataKey::Role(addr)             ← AccessFacet only
DataKey::RoleAdmin              ← AccessFacet only
DataKey::RegEntry(key)          ← RegistryFacet only
DataKey::RegOwner(key)          ← RegistryFacet only
```

## Inter-Facet Communication

The key differentiator of this example is demonstrating **atomic cross-facet operations**:

```rust
// Router atomically calls TokenFacet then RegistryFacet.
// If either fails, the whole transaction reverts.
router.mint_and_register(&admin, &recipient, &750, &key, &"metadata");
```

This pattern is essential for complex diamond architectures where operations span multiple facets but must maintain transactional atomicity.

## How to Run

```bash
# Unit tests
cargo test -p diamond-facets

# Clippy
cargo clippy -p diamond-facets --all-targets -- -D warnings

# WASM release build
cargo build -p diamond-facets --target wasm32-unknown-unknown --release
```

## Related Examples

- **[06-diamond-pattern](../06-diamond-pattern/)** — The canonical diamond implementation with full EIP-2535 functionality including dynamic diamond-cut operations and comprehensive diamond-loupe introspection
- **[05-diamond-security](../05-diamond-security/)** — Security-hardened variant with access controls, interface verification, and upgrade safeguards
