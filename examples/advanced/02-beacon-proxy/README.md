# Beacon Proxy

> **Note:** This example covers a *single* beacon shared by proxies you deploy yourself. To deploy and track a fleet of proxies from a factory, see [Beacon Proxy Factory](../23-beacon-proxy-factory/). The two examples implement overlapping delegation logic; if you change one, review the other (see the Contributing guide's "update both" requirement).

A minimal beacon-and-proxy setup: the proxy resolves its implementation through
a beacon on every call, so changing the beacon's implementation updates every
proxy that uses it.

## What It Demonstrates

- A beacon that stores the current implementation and version history
- A proxy that delegates calls through its configured beacon
- Upgrading the shared implementation and re-pointing a proxy to another beacon

## Scope In the Upgradeability Sequence

This is step 3 of 6, after the [single upgradeable proxy](../04-upgradeable-proxy/)
and [proxy admin controls](../31-proxy-admin/).

- **In scope:** one beacon and proxies that share its implementation.
- **Out of scope:** deploying and tracking a proxy fleet or managing multiple
  named beacons. Continue to the [factory](../23-beacon-proxy-factory/) for
  fleet deployment; [Beacon Management](../06-beacon-management/) follows it
  for named beacons.

## Key Concepts

- Delegated call semantics (msg.sender preservation)
- Implementation contract separation
- Beacon contract pointing to current implementation
- Storage layout isolation
- Simple upgrade path

## Pattern Progression

**Basic beacon → Beacon factory → Governance → Direct upgrade → Versioning → Full patterns**

See the [advanced examples README](../README.md) for the full upgrade patterns learning path.

## Run Tests

```bash
cargo test -p beacon-proxy
```

## Pattern Comparison

| Aspect | Beacon Proxy (02) | Beacon Proxy Factory (23) |
|---|---|---|
| Deployment | Manual, one proxy at a time | Factory deploys and tracks many proxies |
| Beacon count | One beacon | One beacon, shared by all factory-made proxies |
| Proxy tracking | Caller tracks addresses | Factory maintains the fleet registry |
| Best for | Small, fixed sets of proxies | Fleet deployment at scale |

## Next

Continue with [Beacon Proxy Factory](../23-beacon-proxy-factory/), which creates
and tracks multiple proxies that share a beacon.
