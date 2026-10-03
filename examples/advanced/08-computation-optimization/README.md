# Computation Optimization

Computation optimization example with algorithm, loop, and caching optimization.
Each pattern has a `*_naive` and a `*_optimized` implementation that return
identical results, so the only difference is cost.

## Patterns

| Pattern | Naive | Optimized |
| --- | --- | --- |
| Algorithm | `sum_to_naive(n)`: O(n) loop | `sum_to_optimized(n)`: O(1) closed form |
| Algorithm | `pow_mod_naive(base, exp, m)`: O(exp) multiplications | `pow_mod_optimized(base, exp, m)`: O(log exp) square-and-multiply |
| Loop | `stats_naive(values)`: three indexed passes that call `len()`/`get()` each iteration | `stats_optimized(values)`: one iterator pass |
| Caching | `stats_uncached()`: loads the list and recomputes on every read | `stats_cached()`: reads stats computed once in `set_values` |

`set_values(values)` requires the admin's auth (set with `init(admin)`).

## Why it matters

Every Soroban transaction has a fixed CPU-instruction budget. An O(n) loop over
caller-supplied input can run out of budget and fail, while the O(1) or
O(log n) version keeps working. Each `Vec::get` / `Vec::len` is a metered host
call, so the number of passes over data matters too.

## Tests and gas measurements

The unit tests in `src/test.rs` are wired from `lib.rs` via
`#[cfg(test)] mod test;`:

```bash
cargo test -p computation-optimization
```

They check that each naive/optimized pair gives the same results, and they
cover edge cases (overflow bounds, empty input, zero modulus), admin
authorization, and the cost of each pattern. The cost tests
(`single_pass_loop_is_cheaper`, `cached_read_is_cheaper`) compare CPU
instructions from `env.cost_estimate().budget()`.

> **Note:** native unit tests meter **host calls only** (storage, `Vec`
> access, etc.), not the Rust arithmetic inside the contract. That's why
> the cost tests cover the loop and caching patterns. For the algorithm
> pairs, the tests check correctness. Measure their instruction savings on
> the compiled WASM (e.g. with `stellar contract invoke --cost`).
Demonstrates computation and gas optimization patterns in Soroban smart contracts:

- **Algorithmic Optimization**: Using closed-form formulas (e.g. arithmetic progression $O(1)$) and fast exponentiation by squaring $O(\log n)$ instead of naive loops.
- **Loop Optimization**: Short-circuiting and early termination on zero or absorbing elements to avoid wasteful iterations.
- **Memoization & Storage Caching**: Storing computed results in contract instance storage to prevent re-executing heavy computation on future calls.

## Building and Checking

Build with:
```bash
cargo build -p computation-optimization
```
