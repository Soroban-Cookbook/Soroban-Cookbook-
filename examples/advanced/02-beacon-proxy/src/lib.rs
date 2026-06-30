//! # Beacon Proxy Pattern
//!
//! This example demonstrates the **Beacon Proxy** upgradeability pattern adapted for
//! Soroban smart contracts on the Stellar network.
//!
//! ## Architecture
//!
//! ```text
//!                ┌─────────────────────┐
//!   caller ───►  │   Proxy Contract    │
//!                │  (holds beacon addr)│
//!                └────────┬────────────┘
//!                         │  get_implementation()
//!                         ▼
//!                ┌─────────────────────┐
//!                │   Beacon Contract   │
//!                │  (current impl ptr) │
//!                └────────┬────────────┘
//!                         │  invoke_contract()
//!                         ▼
//!                ┌─────────────────────┐
//!                │  Implementation Vn  │
//!                │  (actual logic)     │
//!                └─────────────────────┘
//! ```
//!
//! ### Key Contracts
//!
//! | Contract | Role |
//! |---|---|
//! | `BeaconContract` | Maintains the single source-of-truth for the current implementation address and its version history. |
//! | `ProxyContract`  | Stores the beacon address; resolves the live implementation on every call and delegates via `invoke_contract`. |
//! | `ImplV1` / `ImplV2` | Versioned implementation contracts that contain the actual business logic. |
//!
//! ### Why Beacon over a simple Upgradeable Proxy?
//!
//! A plain upgradeable proxy stores the implementation address *inside itself*.
//! When you have **many** proxies pointing at the same logic (e.g., one per user),
//! upgrading requires touching every proxy individually — O(n) transactions.
//!
//! With the Beacon pattern, **all proxies share a single Beacon reference**.
//! Upgrading the Beacon propagates instantly to every proxy in a single transaction.
//!
//! ### Building individual WASM artefacts
//!
//! Because a single WASM binary can only export one set of contract entry-points,
//! build each contract separately using cargo features:
//!
//! ```bash
//! cargo build -p beacon-proxy --target wasm32v1-none --release --no-default-features --features beacon
//! cargo build -p beacon-proxy --target wasm32v1-none --release --no-default-features --features proxy
//! cargo build -p beacon-proxy --target wasm32v1-none --release --no-default-features --features impl-v1
//! cargo build -p beacon-proxy --target wasm32v1-none --release --no-default-features --features impl-v2
//! ```
//!
//! Tests run in `rlib` mode and register all contracts via `env.register()`, so
//! all modules are included unconditionally under `#[cfg(test)]`.
//!
//! ### Storage Layout
//!
//! #### Beacon (`persistent`)
//! | Key | Type | Description |
//! |---|---|---|
//! | `"admin"` | `Address` | Sole account authorised to upgrade |
//! | `"impl"` | `Address` | Current implementation contract address |
//! | `"version"` | `u32` | Monotonically-increasing upgrade counter |
//! | `VersionLog(n)` | `VersionEntry` | Historical record for version `n` |
//!
//! #### Proxy (`persistent`)
//! | Key | Type | Description |
//! |---|---|---|
//! | `"beacon"` | `Address` | The beacon this proxy is bound to |
//! | `"admin"` | `Address` | Proxy-level admin (can re-point to a new beacon) |

#![no_std]

// Each contract module is included in test builds (rlib) unconditionally so
// env.register() can find all four contract types.  For cdylib (WASM) builds,
// exactly one feature must be enabled so only one set of exports is emitted.
#[cfg(any(feature = "beacon", test))]
pub mod beacon;

#[cfg(any(feature = "proxy", test))]
pub mod proxy;

#[cfg(any(feature = "impl-v1", test))]
pub mod implementation_v1;

#[cfg(any(feature = "impl-v2", test))]
pub mod implementation_v2;

// Re-export types for ergonomic use in tests.
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
