#![allow(deprecated)]
//! # Multi-Contract Testing Patterns
//!
//! Demonstrates how to test cross-contract interactions using Soroban's
//! mock environment, including registering multiple contracts, mocking
//! cross-contract calls, and verifying state across contract boundaries.
//!
//! This example shows a Governance contract that can authorize withdrawals
//! from a Treasury contract, demonstrating realistic multi-contract patterns.

#![cfg_attr(target_family = "wasm", no_std)]

pub mod treasury;
pub mod governance;

#[cfg(test)]
mod test;
