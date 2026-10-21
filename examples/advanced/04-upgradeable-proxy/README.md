# Upgradeable Proxy

Admin-gated implementation upgrades with proxy-owned storage preservation.

## Scope In the Upgradeability Sequence

This is step 1 of 6 and the starting point for the
[upgradeability examples](../README.md). It demonstrates a single proxy whose
implementation address can be changed; `implementation-v1` supplies the initial
business logic.

- **In scope:** direct implementation routing and proxy-owned state across upgrades.
- **Out of scope:** timelocked upgrade proposals, shared beacons, and storage
    schema migration. Continue to [Proxy Admin Controls](../31-proxy-admin/) for
    upgrade governance.

## Role in Learning Path

This is the **fourth step** in the [upgrade patterns learning path](../README.md#upgrade-patterns--proxy-patterns). An alternative to beacon-based patterns:
- Storage lives in the proxy (not separated)
- Direct implementation reference in proxy state
- Admin-controlled implementation updates
- Simpler than beacon patterns for single-proxy use cases
- Better storage efficiency for dedicated proxies

**Prerequisites:** Understand beacon-based patterns first:
- [`02-beacon-proxy`](../02-beacon-proxy/) — Basic beacon concept
- [`23-beacon-proxy-factory`](../23-beacon-proxy-factory/) — Factory patterns
- [`31-proxy-admin`](../31-proxy-admin/) — Governance patterns

**Next steps:**
- **[`06-beacon-management`](../06-beacon-management/)** — Versioned implementations with rollback
- **[`07-upgrade-patterns`](../07-upgrade-patterns/)** — Complete upgrade strategies

## Key Concepts

- Storage preservation across upgrades
- Direct implementation reference
- Admin-controlled implementation updates
- Simpler alternative to beacon pattern
- Single-proxy efficiency

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

## Beacon vs. Direct Comparison

- **Beacon pattern (02, 03):** Multiple proxies sharing one beacon; one upgrade affects all
- **Direct upgrade (04):** Dedicated proxy for each contract; storage efficiency; simpler logic

## What It Demonstrates

```bash
cargo test -p upgradeable-proxy
```

## Next

Continue with [Proxy Admin Controls](../31-proxy-admin/) to add proposal delays,
cancellation, and emergency pause controls around upgrade operations.

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.
