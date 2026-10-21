# Proxy Admin

Admin-authenticated upgrade proposals with timelock and emergency pause.

## Scope In the Upgradeability Sequence

This is step 2 of 6, following the [single upgradeable proxy](../04-upgradeable-proxy/).

- **In scope:** timelocked upgrade proposals, cancellation, emergency pause,
      and lifecycle events.
- **Out of scope:** proxy call forwarding and shared implementation routing.
      Continue to the [beacon proxy](../02-beacon-proxy/) for shared upgrades.

## Role in Learning Path

This is the **third step** in the [upgrade patterns learning path](../README.md#upgrade-patterns--proxy-patterns). After learning basic and factory-managed proxies, this example adds:
- Admin authorization and access control
- Timelock delays for planned upgrades
- Emergency pause mechanisms
- Staged upgrade governance
- Risk mitigation for production systems

**Prerequisites:** 
- Start with [`02-beacon-proxy`](../02-beacon-proxy/) for beacon basics
- Then [`23-beacon-proxy-factory`](../23-beacon-proxy-factory/) for factory patterns

**Next steps:**
- **[`04-upgradeable-proxy`](../04-upgradeable-proxy/)** — Alternative pattern: storage in proxy
- **[`06-beacon-management`](../06-beacon-management/)** — Versioned implementations with rollback
- **[`07-upgrade-patterns`](../07-upgrade-patterns/)** — Complete upgrade strategies

## Key Concepts

- Role-based access control for upgrades
- Timelock-enforced delays
- Emergency pause for rollback
- Proposal lifecycle management
- Production-grade safety checks

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

## What It Demonstrates

## Security Checklist

- [ ] Admin key is a multisig or DAO address in production, not a single EOA.
- [ ] `MIN_DELAY` is long enough for stakeholders to review the new WASM hash.
- [ ] The new WASM hash is verified off-chain before calling `propose_upgrade`.
- [ ] An emergency contact procedure exists for calling `pause()` if a
      vulnerability is discovered during the timelock window.
- [ ] `execute_upgrade` is called only after the new contract has been audited
      and tested on testnet.
- [ ] Proposal storage is removed *before* the deployer call to prevent replay
      even if the deployer call reverts.

## Integrating the Pause Guard

Any entry point in a contract that embeds this pattern should call
`require_unpaused` before executing business logic:

```rust
pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), Error> {
    require_unpaused(&env)?;   // blocks when paused
    user.require_auth();
    // ... rest of logic
}
```

## Run Tests

```bash
cargo test -p proxy-admin
```

## Related Examples

- [Next: Beacon Proxy](../02-beacon-proxy/) — share one implementation across proxies
- [02-timelock](../02-timelock/) — Core timelock pattern this example builds on
- [01-multi-party-auth](../01-multi-party-auth/) — Threshold signatures for the admin role
- [Governance Examples](../../governance/) — DAOs that govern upgrade proposals

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.
