# Beacon Proxy

A minimal beacon-and-proxy setup: the proxy resolves its implementation through
a beacon on every call, so changing the beacon's implementation updates every
proxy that uses it.

## What It Demonstrates

- A beacon that stores the current implementation and version history
- A proxy that delegates calls through its configured beacon
- Upgrading the shared implementation and re-pointing a proxy to another beacon

## Scope In the Upgradeability Sequence

This is step 3 of 6, after the [single upgradeable proxy](../04-upgradeable-proxy/)
and [proxy admin controls](../03-proxy-admin/).

- **In scope:** one beacon and proxies that share its implementation.
- **Out of scope:** deploying and tracking a proxy fleet or managing multiple
  named beacons. Continue to the [factory](../03-beacon-proxy-factory/) for
  fleet deployment; [Beacon Management](../06-beacon-management/) follows it
  for named beacons.

## Run Tests

```bash
cargo test -p beacon-proxy
```

## Next

Continue with [Beacon Proxy Factory](../03-beacon-proxy-factory/), which creates
and tracks multiple proxies that share a beacon.
