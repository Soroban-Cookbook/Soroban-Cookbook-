#![allow(deprecated)]
//! # Beacon Proxy Factory
//!
//! > **Choosing between the beacon examples:** this crate is the *factory* variant.
//! > Use it when you need to deploy and track many proxies that share one beacon.
//! > For a single proxy pointing at one beacon, see
//! > `examples/advanced/02-beacon-proxy`. Both crates implement the same delegation
//! > logic, so bug fixes must be applied to both (see the contributing guide).
//!
//! This example demonstrates the **Beacon Proxy Factory** pattern — an extension of
//! the Beacon Proxy pattern where a single factory contract is responsible for
//! deploying and tracking multiple proxy contracts that all share one beacon.
//!
//! ## Architecture
//!
//! ```text
//!                ┌────────────────────────────────────────────┐
//!                │           BeaconProxyFactory               │
//!                │  - deploys Beacon on init                  │
//!                │  - deploys Proxy instances on demand       │
//!                │  - tracks all deployed proxies             │
//!                │  - single upgrade call updates all proxies │
//!                └──────────────────┬─────────────────────────┘
//!                                   │ owns
//!                                   ▼
//!                ┌────────────────────────────────┐
//!                │         Beacon Contract        │
//!                │   (single impl pointer)        │
//!                └──────────────┬─────────────────┘
//!                               │ upgrade propagates to ↓
//!                ┌──────────────┼──────────────────┐
//!                ▼              ▼                  ▼
//!          ┌──────────┐  ┌──────────┐       ┌──────────┐
//!          │ Proxy #0 │  │ Proxy #1 │  ...  │ Proxy #N │
//!          └────┬─────┘  └────┬─────┘       └────┬─────┘
//!               │             │                   │
//!               └─────────────┴───────────────────┘
//!                             │ all resolve to
//!                             ▼
//!                ┌────────────────────────┐
//!                │  Implementation Vn     │
//!                │  (actual logic)        │
//!                └────────────────────────┘
//! ```
//!
//! ## Key Contracts
//!
//! | Contract | Role |
//! |---|---|
//! | `BeaconProxyFactory` | Initialises the beacon, deploys proxy instances, tracks them, and provides batch operations. |
//! | `BeaconContract` | Single source-of-truth for the current implementation address. |
//! | `ProxyContract` | Thin delegation layer — queries the beacon on every call and forwards to the live implementation. |
//! | `ImplV1` / `ImplV2` | Versioned implementations with the actual business logic. |
//!
//! ## Comparison with the single beacon proxy
//!
//! | Aspect | `02-beacon-proxy` (single) | `23-beacon-proxy-factory` (this crate) |
//! |---|---|---|
//! | Proxies managed | One | Many, tracked by the factory |
//! | Deployment | Manual | `deploy_proxy` / `batch_deploy` |
//! | Upgrade cost | One beacon upgrade | One beacon upgrade, all proxies |
//! | Best for | Simple single-instance upgradeability | Fleets of identical instances |
//!
//! Both patterns are also summarised in the book's "Upgrade Patterns" guide.
//!
//! ## Upgrade flow
//!
//! Because all proxies reference the same beacon, a single executed beacon
//! upgrade atomically upgrades **every** deployed proxy in one transaction —
//! O(1) cost regardless of how many proxies exist.
//!
//! Upgrades are two-step: `BeaconProxyFactory::stage_beacon_upgrade` records the
//! new implementation and starts a timelock, and `execute_beacon_upgrade` applies
//! it once the delay has passed. `cancel_beacon_upgrade` aborts a staged upgrade,
//! so a wrong implementation address cannot brick every proxy in a single call.
//!
//! ## Gas optimisation
//!
//! - The factory stores the proxy list and beacon address in `instance` storage
//!   (shared, cheap reads) rather than per-key persistent storage.
//! - `batch_deploy` deploys multiple proxies in a single transaction, amortising
//!   the per-transaction overhead.
//! - The beacon address is cached in factory storage so proxies only need one
//!   cross-contract call to resolve the implementation.
//!
//! ## Building individual WASM artefacts
//!
//! ```bash
//! cargo build -p beacon-proxy-factory --target wasm32v1-none --release --no-default-features --features factory
//! cargo build -p beacon-proxy-factory --target wasm32v1-none --release --no-default-features --features beacon
//! cargo build -p beacon-proxy-factory --target wasm32v1-none --release --no-default-features --features proxy
//! cargo build -p beacon-proxy-factory --target wasm32v1-none --release --no-default-features --features impl-v1
//! cargo build -p beacon-proxy-factory --target wasm32v1-none --release --no-default-features --features impl-v2
//! ```

#![cfg_attr(target_family = "wasm", no_std)]

// Each contract module is included in test builds (rlib) unconditionally so
// env.register() can find all contract types.  For cdylib (WASM) builds,
// exactly one feature must be enabled so only one set of exports is emitted.
#[cfg(any(feature = "factory", test))]
pub mod factory;

#[cfg(any(feature = "beacon", test))]
pub mod beacon;

#[cfg(any(feature = "proxy", test))]
pub mod proxy;

#[cfg(any(feature = "impl-v1", test))]
pub mod implementation_v1;

#[cfg(any(feature = "impl-v2", test))]
pub mod implementation_v2;

// Re-export types for ergonomic use in tests.
#[cfg(any(feature = "factory", test))]
pub use factory::{BeaconProxyFactory, BeaconProxyFactoryClient};

#[cfg(any(feature = "beacon", test))]
pub use beacon::{BeaconContract, BeaconContractClient, VersionEntry};

#[cfg(any(feature = "proxy", test))]
pub use proxy::{ProxyContract, ProxyContractClient};

#[cfg(any(feature = "impl-v1", test))]
pub use implementation_v1::{ImplV1, ImplV1Client};

#[cfg(any(feature = "impl-v2", test))]
pub use implementation_v2::{ImplV2, ImplV2Client};

#[cfg(test)]
mod test;
