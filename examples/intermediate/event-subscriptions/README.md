# Event Subscription Hub

This example demonstrates a push-style subscription model for event-driven workflows: subscriber *contracts* register interest in a topic, and a publisher pushes matching notifications directly into each subscriber via a cross-contract call, rather than subscribers polling or off-chain indexers filtering event logs.

## 📖 What You'll Learn

- **Subscription registration**: recording `(subscriber, topic, filter)` and returning an id
- **Filtering logic**: wildcard subscriptions vs. subscriptions narrowed to one entity
- **Unsubscribe paths**: only the original subscriber can unsubscribe, and only once
- **Push delivery**: notifying subscribers with `try_invoke_contract` instead of only emitting logs
- **Fault isolation**: one panicking subscriber cannot block delivery to the rest

## 🔍 Contract Overview

```rust
pub fn subscribe(env: Env, subscriber: Address, topic: Symbol, filter: Option<Address>) -> u32
pub fn unsubscribe(env: Env, subscriber: Address, id: u32) -> Result<(), SubscriptionError>
pub fn publish(env: Env, topic: Symbol, entity: Option<Address>, payload: i128) -> u32
pub fn subscription(env: Env, id: u32) -> Option<Subscription>
pub fn subscriber_count(env: Env, topic: Symbol) -> u32
```

`subscribe` requires `subscriber.require_auth()` — a contract or account can only register itself, not on behalf of another address. `publish` fans out to every active subscription whose topic matches and whose `filter` is either `None` (wildcard) or equal to the published `entity`.

## Push Delivery and Fault Isolation

Each matching subscriber must implement a `notify(env, topic, entity, payload)` entry point. `publish` calls it through `Env::try_invoke_contract`:

```rust
let result = env.try_invoke_contract::<(), SdkError>(&sub.subscriber, &NOTIFY_FN, args);
if result.is_ok() {
    notified += 1;
}
```

Because the call goes through `try_invoke_contract` rather than `invoke_contract`, a subscriber that traps (panics) or returns an error only forfeits its own notification — the loop continues to the next subscriber. `publish` returns the count of subscribers successfully notified, and also emits a `PublishEvent` so off-chain indexers retain a normal event-log trail alongside the push path.

## Unsubscribe Semantics

Unsubscribing is a soft-delete: the subscription record is kept (so `subscription(id)` remains queryable) but marked `active = false`, and `publish` skips inactive subscriptions. `unsubscribe` enforces:

- the subscription id must exist (`SubscriptionNotFound`),
- only the address that created it may remove it (`NotSubscriptionOwner`),
- it can't be unsubscribed twice (`AlreadyUnsubscribed`).

## Run Tests

```bash
cargo test -p event-subscriptions
```

Tests cover subscription id assignment, wildcard vs. entity-filtered delivery, unsubscribe stopping future notifications, unauthorized/duplicate/unknown unsubscribe attempts, publishing with zero subscribers, and a panicking subscriber that doesn't block a well-behaved one.
