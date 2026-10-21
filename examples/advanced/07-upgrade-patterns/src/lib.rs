#![allow(deprecated)]
//! # Contract Upgrade Patterns
//!
//! A collection of idiomatic patterns for upgrading Soroban smart contracts
//! safely. Each module is an independent, self-contained example that can be
//! read on its own or studied as a set.
//!
//! ## Patterns covered
//!
//! | Module | Pattern | Key idea |
//! |--------|---------|----------|
//! | [`direct_upgrade`] | Direct WASM upgrade | Minimal admin-gated `update_current_contract_wasm` call |
//! | [`versioned_upgrade`] | Versioned storage + migration | `StorageVersion` key drives a migration function on first call after upgrade |
//! | [`init_guard`] | Safe initialisation | Storage flag prevents double-init; post-upgrade hook is version-aware |
//!
//! ## Why three separate modules instead of one contract?
//!
//! Each pattern addresses a distinct concern. Real production contracts often
//! combine all three, but teaching them separately makes each invariant
//! explicit. The modules cross-reference each other where relevant.
//!
//! ## Relationship to other advanced examples
//!
//! - [`31-proxy-admin`]: adds a timelock + proposal workflow on top of the
//!   direct upgrade call shown here — use that when stakeholders need a review
//!   window before a WASM swap lands.
//! - [`04-upgradeable-proxy`]: shows the *delegation proxy* pattern (swapping
//!   an implementation *address* rather than a WASM binary).
//!
//! These two patterns are complementary, not competing.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(unexpected_cfgs)]

pub mod direct_upgrade;

// Each pattern module exports overlapping names (`initialize` / `upgrade` / `admin`).
// Host tests need all three; wasm builds only the primary direct_upgrade example.
#[cfg(any(test, not(target_family = "wasm")))]
pub mod init_guard;
#[cfg(any(test, not(target_family = "wasm")))]
pub mod versioned_upgrade;

#[cfg(test)]
mod test;
