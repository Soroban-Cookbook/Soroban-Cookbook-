# Event Patterns

Soroban events for indexing/monitoring: topics (indexed) + data (payload).

## 📊 Topic Layout

| Slot | Use | Example |
|------|-----|---------|
| 0 | Namespace | `events` |
| 1 | Action | `transfer` |
| 2 | Primary Key | `sender Addr` |
| 3 | Secondary Key | `recipient Addr` |

## 🎯 When To Emit

✅ **State changes** wallets/UI need (transfers, config)
✅ **Admin ops** (pause, upgrade)
✅ **Audit logs** (who/when)
❌ Internal math

## 💾 Code Patterns

```rust
#[contracttype]
pub struct TransferData { amount: i128, memo: u64 }

env.events().publish(
  (sym!(\"events\"), sym!(\"transfer\"), from, to),
  TransferData { amount, memo }
);
```

**Filters:**
- All events: topic0=`events`
- Sender: topic0+topic2 
- Sender→Recip: all 4 topics

## ⚡ Gas Costs
- Topics cheap, data more expensive
- Max 4 topics
- Limit emissions (no 1000-event loops)

## 🔬 Examples

**[Main: 04-events](../examples/basics/04-events/)**
- 4-topic transfers, config/admin/audit
- Custom payloads (`TransferEventData`)
- `EVENT_QUICK_REFERENCE.md` cheat sheet
- `cargo test -p events`

**[Simple Counter](../examples/basics/events/)**
- Basic increment/emit

## 🧪 Testing

```rust
let events = env.events().all();
let ev = &events.get(0).unwrap();
assert_eq!(ev.1.get(0).unwrap(), sym!(\"events\"));
```

## 📚 Related
- [Storage → Emit on writes](./storage-patterns.md)
- [Errors → Events on fail](./error-handling.md)

