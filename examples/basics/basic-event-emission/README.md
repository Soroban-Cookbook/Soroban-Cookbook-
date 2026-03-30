# Basic Event Emission

Demonstrates how to emit events from a Soroban smart contract using `env.events().publish()`.

## What This Example Shows

- `env.events().publish()` — the single API for emitting events
- Simple event topics — single-symbol and two-symbol topic tuples
- Event data payload — attaching a value to each event

## Event Anatomy

```rust
env.events().publish(
    (symbol_short!("action"),),   // topics tuple — up to 4 entries
    value,                        // data payload — any SCVal-compatible type
);
```

**Topics** are short, filterable identifiers placed in a tuple. Off-chain indexers use them to filter events efficiently. **Data** is the richer payload read after matching on topics.

## Contract API

| Function      | Topics                        | Data          | Description                  |
|---------------|-------------------------------|---------------|------------------------------|
| `set(value)`  | `("set",)`                    | `u32` value   | Store value, emit set event  |
| `increment()` | `("counter", "inc")`          | `u32` value   | Increment counter, emit event |
| `get()`       | —                             | —             | Read current value           |

## Build & Test

```bash
cargo test -p basic-event-emission
```
