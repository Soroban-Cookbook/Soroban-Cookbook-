# Events Counter

A minimal Soroban contract that maintains a counter and emits events on every state change. This contract is used by the integration test suite to verify cross-contract workflows alongside storage and authentication patterns.

## What This Example Shows

- Instance storage for a simple counter
- Event emission on `set_number`, `increment`, and `decrement`
- A reusable test helper suitable for multi-contract integration scenarios

## Project Structure

```text
examples/basics/events/
├── Cargo.toml        (package: events_example, lib: events)
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## API

| Function | Args | Returns | Description |
| --- | --- | --- | --- |
| `set_number` | `value: u32` | — | Set counter to a fixed value |
| `increment` | — | — | Increment counter by 1 |
| `decrement` | — | — | Decrement counter by 1 (panics on underflow) |
| `get_number` | — | `u32` | Read current counter value |

## Build & Test

```bash
# From repository root
cargo test -p events_example
cargo build -p events_example

# From this directory
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## See Also

- [`04-events`](../04-events/) — comprehensive structured event emission patterns
