#![allow(deprecated)]
//! # Flash Loan Use Cases
//!
//! Practical examples of flash loan receiver contracts built on top of the
//! core flash loan provider (`05-flash-loans`).
//!
//! ## Use Cases
//! - **Arbitrage**: Exploit a price difference between two AMM pools in a
//!   single atomic transaction.
//! - **Refinancing**: Atomically move a debt position from one lending pool to
//!   another at a better interest rate.
//! - **Security patterns**: Demonstrate safe callback practices including
//!   validation, event emission, and rejection of untrusted callers.

#![cfg_attr(target_family = "wasm", no_std)]

pub mod arbitrage;

// Secondary example contracts share export names (`init` / `execute` / `on_flash_loan`).
// Keep them for host tests; omit from wasm so the cdylib has a single export set.
#[cfg(any(test, not(target_family = "wasm")))]
pub mod refinancing;
#[cfg(any(test, not(target_family = "wasm")))]
pub mod security;

#[cfg(test)]
mod test;
