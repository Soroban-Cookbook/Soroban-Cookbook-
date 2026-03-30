# Collection Types in Soroban

This example demonstrates `Vec<T>` and `Map<K, V>` — Soroban's two built-in collection types — covering operations, iteration patterns, and performance trade-offs.

## Project Structure

```text
examples/basics/11-collection-types/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── test.rs
```

## What This Example Shows

### Vec\<T\> — Ordered Sequences

`Vec<T>` is a growable, ordered collection backed by the Soroban host.

```rust
// Create and populate a Vec
let mut items: Vec<i128> = Vec::new(&env);
items.push_back(10);
items.push_back(20);
items.push_back(30);

// Random access — O(1)
let first = items.get(0); // Some(10)
let len   = items.len();  // 3

// Remove the last element
let last = items.pop_back(); // Some(30)
```

**When to use Vec:**
- Ordered lists (transaction history, leaderboards, queues)
- Batch operations that process every element
- Small sets where the overhead of a Map is not justified

### Map\<K, V\> — Key-Value Store

`Map<K, V>` keeps entries sorted by key and provides O(log n) lookups.

```rust
// Create and populate a Map
let mut balances: Map<Symbol, i128> = Map::new(&env);
balances.set(symbol_short!("alice"), 1000);
balances.set(symbol_short!("bob"),   500);

// Key lookup — O(log n)
let alice_bal = balances.get(symbol_short!("alice")); // Some(1000)
let missing   = balances.get(symbol_short!("carol")); // None

// Remove an entry
balances.remove(symbol_short!("bob"));

// Bulk extraction
let keys   = balances.keys();   // Vec<Symbol>
let values = balances.values(); // Vec<i128>
```

**When to use Map:**
- Key-value associations (balances, roles, allowances)
- Frequent membership tests (`contains_key`)
- Situations where sorted key order matters

## Iteration Patterns

### Iterating a Vec

`Vec::iter()` returns a standard `Iterator<Item = T>`, so all Rust iterator
combinators are available.

```rust
// Sum all elements
let total: i128 = items.iter().fold(0, |acc, x| acc + x);

// Filter — collect matching elements into a new Vec
let mut positive = Vec::new(&env);
for item in items.iter() {
    if item > 0 {
        positive.push_back(item);
    }
}

// Short-circuit search
let found: bool = items.iter().any(|x| x == target);
```

### Iterating a Map

`Map::iter()` yields `(K, V)` tuples in key-sorted order.

```rust
// Sum all values
let mut total: i128 = 0;
for (_key, value) in balances.iter() {
    total += value;
}

// Build a transformed Map
let mut result = Map::new(&env);
for (key, value) in balances.iter() {
    result.set(key, value + bonus);
}
```

### Combining Vec and Map

```rust
// Build a Map from parallel key/value Vecs (zip pattern)
let mut m = Map::new(&env);
let len = keys.len().min(values.len());
for i in 0..len {
    m.set(keys.get(i).unwrap(), values.get(i).unwrap());
}
```

## Performance Considerations

| Operation        | Vec            | Map            |
|------------------|----------------|----------------|
| Indexed access   | O(1)           | —              |
| Key lookup       | O(n) scan      | O(log n)       |
| Insert / update  | O(1) push_back | O(log n)       |
| Iteration        | O(n)           | O(n)           |
| Storage overhead | Lower          | Higher         |
| Key ordering     | Insertion order| Sorted by key  |

**Tips:**
1. Prefer `Map` when you need fast key lookups across many entries.
2. Prefer `Vec` for pure sequences where you always scan all elements.
3. `Map::values()` returns values in key-sorted order — useful when order matters.
4. Avoid storing very large collections in `instance` storage; use `persistent`
   with per-item keys for unbounded datasets.

## Build

```bash
cargo build -p collection-types
```

## Test

```bash
cargo test -p collection-types
```

## Related Examples

- [10-data-types](../10-data-types/) — Overview of all Soroban types including primitives
- [08-custom-structs](../08-custom-structs/) — Storing structs inside Vec and Map
- [06-soroban-types](../06-soroban-types/) — Address, Bytes, Symbol, and String
