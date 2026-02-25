# Soroban Event Emission Quick Reference

## Basic Syntax

```rust
env.events().publish(topics, data);
```

## Topic Patterns

### 1-Topic (Simple)
```rust
env.events().publish(
    (symbol_short!("action"),),
    value
);
```

### 2-Topic (Tagged)
```rust
env.events().publish(
    (symbol_short!("action"), tag),
    value
);
```

### 3-Topic (Indexed)
```rust
env.events().publish(
    (namespace, action, key),
    data
);
```

### 4-Topic (Full Indexing)
```rust
env.events().publish(
    (namespace, action, primary_key, secondary_key),
    data
);
```

## Custom Event Types

```rust
#[contracttype]
pub struct MyEventData {
    pub field1: i128,
    pub field2: Symbol,
}

env.events().publish(
    (symbol_short!("myevent"),),
    MyEventData { field1: 100, field2: symbol_short!("data") }
);
```

## Naming Convention

```text
topic[0] = namespace/event_type  (always present)
topic[1] = action/category       (optional)
topic[2] = primary_index         (optional)
topic[3] = secondary_index       (optional)
data     = payload               (any SCVal)
```

## Best Practices

✅ **DO:**
- Use topic[0] for event type
- Put filterable fields in topics
- Use custom types for structured data
- Keep topic schema stable
- Test event emission

❌ **DON'T:**
- Change topic order/meaning
- Put large data in topics
- Emit unnecessary events
- Use more than 4 topics

## Testing Events

```rust
#[test]
fn test_event() {
    let env = Env::default();
    let client = MyContractClient::new(&env, &contract_id);
    
    client.my_function();
    
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    
    let (_id, topics, data) = events.get(0).unwrap();
    // Verify topics and data
}
```

## Common Patterns

### Transfer Event
```rust
env.events().publish(
    (symbol_short!("transfer"), from, to),
    amount
);
```

### State Change Event
```rust
env.events().publish(
    (symbol_short!("status"), entity_id, old_status, new_status),
    ledger_sequence
);
```

### Admin Action Event
```rust
env.events().publish(
    (symbol_short!("admin"), admin_address),
    AdminActionData { action, timestamp }
);
```

## See Full Example

`examples/basics/04-events/` - Complete implementation with 12 functions and 31 tests
