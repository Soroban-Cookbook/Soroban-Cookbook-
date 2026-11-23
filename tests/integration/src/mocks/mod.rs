//! Mock contracts for testing cross-contract interactions.
//!
//! Provides reusable mock contract implementations for common testing scenarios.

pub mod auth;
pub mod token;

pub use auth::MockAuthBuilder;
pub use token::MockTokenBuilder;
