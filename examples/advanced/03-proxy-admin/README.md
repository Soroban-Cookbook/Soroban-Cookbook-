# Proxy Admin Controls

Governance and safety controls around Soroban contract upgrades. The example
combines four independent safety layers so that no single mistake can result
in an irreversible bad upgrade.

## What It Demonstrates

- Admin-only `propose_upgrade` with a configurable timelock delay
- Proposal workflow: propose → wait → execute (or cancel)
- Emergency pause switch that halts non-admin operations instantly
- Structured events for every lifecycle transition
- Auth guards and replay prevention

## Upgrade Lifecycle

```
admin calls propose_upgrade(new_hash, delay)
        │
        ▼
  ProposalState::Pending  ──── delay passes ────▶  ProposalState::Ready
        │                                                   │
  admin calls cancel_upgrade                       admin calls execute_upgrade
        │                                                   │
  proposal removed                         WASM replaced, proposal removed
```

## Contract API

| Function | Auth required | Description |
| --- | --- | --- |
| `initialize(admin)` | — | One-time setup |
| `propose_upgrade(new_wasm_hash, delay)` | admin | Queue an upgrade with a timelock |
| `cancel_upgrade()` | admin | Remove the pending proposal |
| `execute_upgrade()` | admin | Apply the upgrade after the delay |
| `pause()` | admin | Halt non-admin operations |
| `unpause()` | admin | Resume normal operations |
| `proposal_state()` | — | Returns `None`, `Pending`, or `Ready` |
| `get_proposal()` | — | Returns the full `UpgradeProposal` or `None` |
| `is_paused()` | — | Returns the current pause flag |

## Timelock Constants

| Constant | Value | Rationale |
| --- | --- | --- |
| `MIN_DELAY` | 60 s | Prevents accidental instant upgrades |
| `MAX_DELAY` | 604 800 s (7 days) | Keeps upgrades actionable within a week |

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

- [02-timelock](../02-timelock/) — Core timelock pattern this example builds on
- [01-multi-party-auth](../01-multi-party-auth/) — Threshold signatures for the admin role
- [Governance Examples](../../governance/) — DAOs that govern upgrade proposals
