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

#![no_std]

pub mod arbitrage;
pub mod refinancing;
pub mod security;

#[cfg(test)]
mod test;
