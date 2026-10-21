//! Shared Test Utilities and Helpers for Soroban Integration Testing
//!
//! This library provides reusable components for Soroban smart contract testing,
//! including common test helpers, mock contracts, test data generators, and custom assertions.

#![cfg(not(target_arch = "wasm32"))]

pub mod generators;
pub mod helpers;
pub mod mocks;

pub use generators::{factories, fixtures};
pub use helpers::{assertions, common};
pub use mocks::{auth, token};
