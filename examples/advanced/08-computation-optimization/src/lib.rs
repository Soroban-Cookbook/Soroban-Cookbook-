//! # Computation Optimization
//!
//! Side-by-side `*_naive` / `*_optimized` implementations of three kinds of
//! computation optimization. Each pair returns identical results, so the only
//! difference is cost.
//!
//! | Pattern   | Naive                                   | Optimized                              |
//! |-----------|-----------------------------------------|----------------------------------------|
//! | Algorithm | `sum_to_naive`: O(n) loop               | `sum_to_optimized`: O(1) closed form   |
//! | Algorithm | `pow_mod_naive`: O(exp) multiplications | `pow_mod_optimized`: O(log exp) square-and-multiply |
//! | Loop      | `stats_naive`: three indexed passes, re-reading `len()` | `stats_optimized`: one iterator pass |
//! | Caching   | `stats_uncached`: recompute on every read | `stats_cached`: computed once at write time |
//!
//! ## Why it matters on Soroban
//!
//! Every transaction has a fixed CPU-instruction budget. A naive O(n) loop
//! over caller-supplied input can exhaust it and fail, while the optimized
//! version keeps working. Every `Vec::get` / `Vec::len` is a metered host
//! call, so the number of passes over the data matters as much as the
//! arithmetic inside them.
//!
//! ## Security notes
//!
//! * `set_values` is the only state-changing call after `init`, and it
//!   requires the admin's auth. Without that check anyone could overwrite
//!   the cached data.
//! * All arithmetic is checked or overflow-free by construction (`u128`
//!   intermediates), so no input can silently wrap.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ComputationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    EmptyInput = 3,
    InvalidModulus = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Values,
    CachedStats,
}

/// Aggregate statistics over a list of values.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats {
    pub count: u32,
    pub sum: i128,
    pub min: i64,
    pub max: i64,
}

#[contract]
pub struct ComputationOptimization;

#[contractimpl]
impl ComputationOptimization {
    pub fn init(env: Env, admin: Address) -> Result<(), ComputationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ComputationError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Algorithm optimization
    // -----------------------------------------------------------------------

    /// Sum of `1..=n` by iterating: O(n) work.
    pub fn sum_to_naive(n: u64) -> u128 {
        let mut total: u128 = 0;
        for i in 1..=n {
            total += i as u128;
        }
        total
    }

    /// Sum of `1..=n` via Gauss's formula: O(1) work.
    ///
    /// `n * (n + 1)` fits in `u128` for every `u64` n, so this cannot overflow.
    pub fn sum_to_optimized(n: u64) -> u128 {
        let n = n as u128;
        n * (n + 1) / 2
    }

    /// `base^exp mod modulus` by repeated multiplication: O(exp) work.
    pub fn pow_mod_naive(base: u64, exp: u64, modulus: u64) -> Result<u64, ComputationError> {
        if modulus == 0 {
            return Err(ComputationError::InvalidModulus);
        }
        let m = modulus as u128;
        let b = base as u128 % m;
        let mut result = 1 % m;
        for _ in 0..exp {
            result = result * b % m;
        }
        Ok(result as u64)
    }

    /// `base^exp mod modulus` by square-and-multiply: O(log exp) work.
    pub fn pow_mod_optimized(
        base: u64,
        mut exp: u64,
        modulus: u64,
    ) -> Result<u64, ComputationError> {
        if modulus == 0 {
            return Err(ComputationError::InvalidModulus);
        }
        let m = modulus as u128;
        let mut b = base as u128 % m;
        let mut result = 1 % m;
        while exp > 0 {
            if exp & 1 == 1 {
                result = result * b % m;
            }
            b = b * b % m;
            exp >>= 1;
        }
        Ok(result as u64)
    }

    // -----------------------------------------------------------------------
    // Loop optimization
    // -----------------------------------------------------------------------

    /// Three separate indexed passes, calling `len()` and `get()` (both host
    /// calls) on every iteration.
    pub fn stats_naive(values: Vec<i64>) -> Result<Stats, ComputationError> {
        if values.is_empty() {
            return Err(ComputationError::EmptyInput);
        }

        let mut sum: i128 = 0;
        let mut i = 0;
        while i < values.len() {
            sum += values.get(i).unwrap() as i128;
            i += 1;
        }

        let mut min = i64::MAX;
        let mut i = 0;
        while i < values.len() {
            min = min.min(values.get(i).unwrap());
            i += 1;
        }

        let mut max = i64::MIN;
        let mut i = 0;
        while i < values.len() {
            max = max.max(values.get(i).unwrap());
            i += 1;
        }

        Ok(Stats {
            count: values.len(),
            sum,
            min,
            max,
        })
    }

    /// One iterator pass: each element is read once, `len()` is read once.
    pub fn stats_optimized(values: Vec<i64>) -> Result<Stats, ComputationError> {
        compute_stats(&values)
    }

    // -----------------------------------------------------------------------
    // Caching
    // -----------------------------------------------------------------------

    /// Store `values` and cache their stats. Admin only.
    ///
    /// The cost of computing stats is paid once here, not on every read.
    pub fn set_values(env: Env, values: Vec<i64>) -> Result<Stats, ComputationError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ComputationError::NotInitialized)?;
        admin.require_auth();

        let stats = compute_stats(&values)?;
        env.storage().instance().set(&DataKey::Values, &values);
        env.storage().instance().set(&DataKey::CachedStats, &stats);
        Ok(stats)
    }

    /// Load the full list and recompute stats on every read.
    pub fn stats_uncached(env: Env) -> Result<Stats, ComputationError> {
        let values: Vec<i64> = env
            .storage()
            .instance()
            .get(&DataKey::Values)
            .ok_or(ComputationError::EmptyInput)?;
        compute_stats(&values)
    }

    /// Read the stats cached by `set_values`: one small storage read.
    pub fn stats_cached(env: Env) -> Result<Stats, ComputationError> {
        env.storage()
            .instance()
            .get(&DataKey::CachedStats)
            .ok_or(ComputationError::EmptyInput)
    }
}

fn compute_stats(values: &Vec<i64>) -> Result<Stats, ComputationError> {
    let count = values.len();
    if count == 0 {
        return Err(ComputationError::EmptyInput);
    }

    let mut sum: i128 = 0;
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    for v in values.iter() {
        sum += v as i128;
        min = min.min(v);
        max = max.max(v);
    }

    Ok(Stats {
        count,
        sum,
        min,
        max,
    })
#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

//! # Computation Optimization Patterns for Soroban
//!
//! This contract demonstrates key computation and gas optimization techniques:
//! 1. **Algorithm optimization**: O(1) mathematical formulas vs O(N) loops, fast exponentiation by squaring.
//! 2. **Loop optimization**: early exit and short-circuiting on neutral/zero elements.
//! 3. **Memoization / Caching**: caching expensive calculation results in instance storage to avoid redundant execution.

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol, Vec};

#[contract]
pub struct ComputationOptimizationContract;

#[contractimpl]
impl ComputationOptimizationContract {
    /// O(1) arithmetic series sum: n * (n + 1) / 2
    /// Avoids an O(N) loop consuming CPU instructions.
    pub fn sum_formula(_env: Env, n: u64) -> u64 {
        n.checked_mul(n.saturating_add(1))
            .map(|val| val / 2)
            .unwrap_or(u64::MAX)
    }

    /// O(log exp) exponentiation by squaring
    /// Greatly reduces multiplication iterations compared to naive O(exp) loop.
    pub fn pow_fast(_env: Env, mut base: u64, mut exp: u32) -> u64 {
        let mut result: u64 = 1;
        while exp > 0 {
            if exp % 2 == 1 {
                result = result.saturating_mul(base);
            }
            base = base.saturating_mul(base);
            exp /= 2;
        }
        result
    }

    /// Loop optimization with early exit on zero values.
    /// Skips further iterations and computations once zero is encountered.
    pub fn multiply_array(_env: Env, values: Vec<u64>) -> u64 {
        let mut product: u64 = 1;
        for val in values.iter() {
            if val == 0 {
                return 0; // Short-circuit: product with 0 is always 0
            }
            product = product.saturating_mul(val);
        }
        product
    }

    /// Caching/Memoization: checks instance storage for previously computed result
    /// before performing computationally heavy work.
    pub fn get_or_compute_fibonacci(env: Env, n: u32) -> u64 {
        let key: Symbol = symbol_short!("fib");
        if let Some(cached_val) = env.storage().instance().get::<(Symbol, u32), u64>(&(key, n)) {
            return cached_val;
        }

        let mut a: u64 = 0;
        let mut b: u64 = 1;
        for _ in 0..n {
            let next = a.saturating_add(b);
            a = b;
            b = next;
        }

        env.storage().instance().set(&(key, n), &a);
        a
    }
}

#[cfg(test)]
mod test;
