# Event Patterns

Soroban events consist of **topics** (indexed, filterable) and a **data payload**
(non-indexed, arbitrary). Off-chain indexers filter by topic position; the data
slot is decoded after a match.

```text
env.events().publish(
    (topic_0, topic_1, topic_2, topic_3),  // up to 4 topics — indexed
    data_payload,                           // arbitrary SCVal — not indexed
);
```

---

## 📐 Topic Ordering Convention

All events in this cookbook follow a shared `(namespace, action, [keys...])` layout:

| Slot    | Purpose              | Type              | Notes                                      |
|---------|----------------------|-------------------|--------------------------------------------|
| `[0]`   | Contract namespace   | `Symbol`          | Always present. Identifies the contract.   |
| `[1]`   | Action name          | `Symbol`          | Always present. Identifies the operation.  |
| `[2]`   | Primary index        | `Address`/`Symbol`| Optional. Most-queried entity.             |
| `[3]`   | Secondary index      | `Address`/`Symbol`| Optional. Second filter dimension.         |

**Why this order?** Off-chain indexers filter left-to-right. Putting the namespace
first means a single topic filter catches every event from a contract. The action
slot narrows to a category. Keys in slots 2–3 enable per-entity queries without
scanning all events.

---

## 📊 Event Schemas by Category

### 1. Transfer Event

Tracks token movement between two parties.

| Slot  | Value               | Type      | Indexed | Rationale                                      |
|-------|---------------------|-----------|---------|------------------------------------------------|
| `[0]` | `"events"`          | `Symbol`  | ✅      | Namespace filter — catch all contract events   |
| `[1]` | `"transfer"`        | `Symbol`  | ✅      | Action filter — catch all transfers            |
| `[2]` | `sender: Address`   | `Address` | ✅      | Filter all sends *from* a wallet               |
| `[3]` | `recipient: Address`| `Address` | ✅      | Filter all receives *to* a wallet              |
| data  | `TransferEventData` | struct    | ❌      | Amount + memo — read after topic match         |

```rust
#[contracttype]
pub struct TransferEventData {
    pub amount: i128,
    pub memo: u64,
}

env.events().publish(
    (symbol_short!("events"), symbol_short!("transfer"), sender, recipient),
    TransferEventData { amount, memo },
);
```

**Off-chain query examples:**
- All transfers: `topic[0] == "events" AND topic[1] == "transfer"`
- All sends by Alice: `+ topic[2] == Alice`
- Alice → Bob only: `+ topic[3] == Bob`

---

### 2. Config Update Event

Tracks changes to contract configuration parameters.

| Slot  | Value                   | Type     | Indexed | Rationale                                    |
|-------|-------------------------|----------|---------|----------------------------------------------|
| `[0]` | `"events"`              | `Symbol` | ✅      | Namespace filter                             |
| `[1]` | `"cfg_upd"`             | `Symbol` | ✅      | Action filter — catch all config changes     |
| `[2]` | `key: Symbol`           | `Symbol` | ✅      | Filter changes to a specific parameter only  |
| data  | `ConfigUpdateEventData` | struct   | ❌      | Old + new value — decoded after match        |

```rust
#[contracttype]
pub struct ConfigUpdateEventData {
    pub old_value: u64,
    pub new_value: u64,
}

env.events().publish(
    (symbol_short!("events"), symbol_short!("cfg_upd"), key),
    ConfigUpdateEventData { old_value, new_value },
);
```

**Why `key` in topics?** Consumers often monitor a *specific* parameter (e.g.
`"max_supply"`). Putting it in slot `[2]` avoids decoding every config event
just to check which parameter changed.

---

### 3. Admin Action Event

Tracks privileged operations for security monitoring.

| Slot  | Value                 | Type      | Indexed | Rationale                                      |
|-------|-----------------------|-----------|---------|------------------------------------------------|
| `[0]` | `"events"`            | `Symbol`  | ✅      | Namespace filter                               |
| `[1]` | `"admin"`             | `Symbol`  | ✅      | Action filter — catch all admin operations     |
| `[2]` | `admin: Address`      | `Address` | ✅      | Filter actions by a specific admin address     |
| data  | `AdminActionEventData`| struct    | ❌      | Action symbol + timestamp — decoded after match|

```rust
#[contracttype]
pub struct AdminActionEventData {
    pub action: Symbol,
    pub timestamp: u64,
}

env.events().publish(
    (symbol_short!("events"), symbol_short!("admin"), admin),
    AdminActionEventData { action, timestamp },
);
```

**Why is `action` in data, not topics?** Admin address (slot `[2]`) is the primary
filter dimension for security monitoring. The specific action (pause, upgrade, etc.)
is secondary context decoded after matching on the admin. Adding it as a 4th topic
would consume the last slot with low query value.

---

### 4. Audit Trail Event

Full accountability log: who did what, captured for compliance.

| Slot  | Value                 | Type      | Indexed | Rationale                                         |
|-------|-----------------------|-----------|---------|---------------------------------------------------|
| `[0]` | `"events"`            | `Symbol`  | ✅      | Namespace filter                                  |
| `[1]` | `"audit"`             | `Symbol`  | ✅      | Action filter — catch all audit entries           |
| `[2]` | `actor: Address`      | `Address` | ✅      | Filter all actions by a specific actor            |
| `[3]` | `action: Symbol`      | `Symbol`  | ✅      | Filter a specific action type across all actors   |
| data  | `AuditTrailEventData` | struct    | ❌      | Details + timestamp + ledger sequence             |

```rust
#[contracttype]
pub struct AuditTrailEventData {
    pub details: Symbol,
    pub timestamp: u64,
    pub sequence: u32,
}

env.events().publish(
    (symbol_short!("events"), symbol_short!("audit"), actor, action),
    AuditTrailEventData { details, timestamp, sequence },
);
```

**Why use all 4 topic slots here?** Audit queries need two dimensions: *who* acted
(actor) and *what* they did (action). Using both slots enables narrow queries like
"all `delete` actions by Alice" without decoding payloads.

---

## 🎯 Indexed Field Rationale — Summary

| Field type       | Put in topics?  | Reason                                              |
|------------------|-----------------|-----------------------------------------------------|
| Entity addresses | ✅ Yes          | Most-common filter dimension for wallets/contracts  |
| Action/category  | ✅ Yes (slot 1) | Enables per-action subscriptions                   |
| Config key name  | ✅ Yes          | Avoids decoding payload just to check key name      |
| Amounts          | ❌ No (data)    | Rarely filtered; read after topic match             |
| Timestamps       | ❌ No (data)    | Range queries not supported on-chain anyway         |
| Memo/details     | ❌ No (data)    | Descriptive only; not a filter dimension            |

**Rule of thumb:** if an off-chain consumer would filter *by* this field to decide
whether to process the event, it belongs in topics. If it's only read *after*
deciding to process the event, it belongs in data.

---

## 🎯 When To Emit

✅ State changes wallets/UI need (transfers, config updates)
✅ Admin operations (pause, upgrade, permission changes)
✅ Audit logs (who acted, when, on what)
❌ Internal calculations with no off-chain consumer
❌ High-frequency loops (avoid emitting 1000+ events per invocation)

---

## ⚡ Gas Considerations

- Topics are cheaper than data per byte
- Max 4 topic slots per event (contract address does not consume a slot)
- Prefer `Symbol` / `Address` in topics — they serialise efficiently
- Reserve the data slot for richer structs decoded after filtering

---

## 🔬 Examples

**[04-events](../examples/basics/04-events/)** — canonical structured event patterns
- 4-topic transfers, config updates, admin actions, audit trails
- Custom `#[contracttype]` payloads
- `cargo test -p soroban-events-example`

**[11-event-filtering](../examples/basics/11-event-filtering/)** — off-chain query patterns

---

## 🧪 Testing

```rust
use soroban_sdk::testutils::Events as _;

let events = env.events().all();
let (_contract_id, topics, data) = events.get(0).unwrap();

// Verify topic slot values
let ns: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
assert_eq!(ns, symbol_short!("events"));

// Verify structured data payload
let payload = TransferEventData::try_from_val(&env, &data).unwrap();
assert_eq!(payload.amount, expected_amount);
```

---

## 📚 Related

- [Storage Patterns → emit on writes](./storage-patterns.md)
- [Error Handling → emit on failures](./error-handling.md)