# Event Filtering

Demonstrates how to design Soroban events for efficient off-chain filtering.

## Concepts

Soroban events have up to **4 topic slots** plus a **data payload**:

```text
env.events().publish(
    (topic_0, topic_1, topic_2, topic_3),  // indexed — used for filtering
    data,                                   // not indexed — read after match
);
```

Topics are indexed by off-chain systems (Horizon, custom listeners). Data is only read once a matching event is found.

## Recommended Topic Layout

| Slot    | Purpose                        | Example              |
|---------|--------------------------------|----------------------|
| topic_0 | Contract namespace / category  | `"marketplace"`      |
| topic_1 | Action name                    | `"sale"`             |
| topic_2 | Primary entity (most filtered) | `seller: Address`    |
| topic_3 | Secondary entity               | `buyer: Address`     |
| data    | Non-indexed payload            | `{ price, token_id }`|

## Off-Chain Query Examples

```text
All events from this contract:   topic_0 == "filter"
All transfers:                   topic_0 == "filter" AND topic_1 == "transfer"
All sends by Alice:              topic_1 == "transfer" AND topic_2 == Alice
Alice → Bob transfers only:      topic_1 == "transfer" AND topic_2 == Alice AND topic_3 == Bob
```

## Examples in This Contract

| Function          | Topics | Description                                  |
|-------------------|--------|----------------------------------------------|
| `transfer_simple` | 2      | Minimal filterable event (namespace + action)|
| `transfer_from`   | 3      | Adds indexed sender address                  |
| `transfer_full`   | 4      | Full sender + recipient indexing             |
| `record_sale`     | 4      | Marketplace sale with seller + buyer indexed |
| `update_status`   | 3      | Entity state change; transition in data      |

## Best Practices

- Put the **most-queried field** in the earliest available topic slot.
- Use a **consistent namespace** in topic_0 across all events in a contract.
- Keep topics to `Symbol` / `Address` / small integers — they must fit in 4 slots.
- Reserve `data` for values that are *read* after filtering (amounts, metadata).

## Running Tests

```bash
cargo test -p event_filtering
```
