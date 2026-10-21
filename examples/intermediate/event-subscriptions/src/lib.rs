#![allow(deprecated)]
//! # Event Subscription Hub
//!
//! Demonstrates a push-style subscription model for event-driven workflows.
//!
//! Instead of consumers polling for events off-chain, subscriber *contracts*
//! register interest in a topic (with an optional entity filter). When a
//! `publish` call comes in, the hub looks up every active subscription for
//! that topic, applies the filter, and pushes a cross-contract call directly
//! into each matching subscriber's `notify` entry point.
//!
//! ## Key Concepts
//!
//! - **Subscription registration** — `subscribe` records `(subscriber, topic,
//!   filter)` and returns an id the subscriber later uses to unsubscribe.
//! - **Filtering logic** — a subscription with `filter = None` matches every
//!   `publish` for its topic; `filter = Some(entity)` only matches events
//!   published for that specific entity.
//! - **Unsubscribe paths** — only the original subscriber can unsubscribe its
//!   own subscription; unsubscribing is idempotent-safe (a second attempt is
//!   rejected rather than silently ignored).
//! - **Fault isolation** — `publish` uses `try_invoke_contract` so a
//!   misbehaving or panicking subscriber cannot block delivery to the rest.

#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, vec, Address,
    Env, Error as SdkError, IntoVal, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SubscriptionError {
    SubscriptionNotFound = 1,
    NotSubscriptionOwner = 2,
    AlreadyUnsubscribed = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextId,
    Subscription(u32),
    TopicIndex(Symbol),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub topic: Symbol,
    pub filter: Option<Address>,
    pub active: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscribeEvent {
    pub subscriber: Address,
    pub topic: Symbol,
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsubscribeEvent {
    pub subscriber: Address,
    pub topic: Symbol,
    pub id: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishEvent {
    pub topic: Symbol,
    pub entity: Option<Address>,
    pub payload: i128,
    pub notified: u32,
}

const NOTIFY_FN: Symbol = symbol_short!("notify");

#[contract]
pub struct EventSubscriptionHub;

#[contractimpl]
impl EventSubscriptionHub {
    /// Register `subscriber` for `topic`, optionally narrowed to `filter`.
    ///
    /// Returns the subscription id, which `subscriber` must present to
    /// `unsubscribe` later.
    pub fn subscribe(env: Env, subscriber: Address, topic: Symbol, filter: Option<Address>) -> u32 {
        subscriber.require_auth();

        let id = next_id(&env);
        env.storage().persistent().set(
            &DataKey::Subscription(id),
            &Subscription {
                subscriber: subscriber.clone(),
                topic: topic.clone(),
                filter,
                active: true,
            },
        );

        let mut index = topic_index(&env, &topic);
        index.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::TopicIndex(topic.clone()), &index);

        SubscribeEvent {
            subscriber,
            topic,
            id,
        }
        .publish(&env);

        id
    }

    /// Remove a subscription. Only the original subscriber may unsubscribe it,
    /// and a subscription can only be unsubscribed once.
    pub fn unsubscribe(env: Env, subscriber: Address, id: u32) -> Result<(), SubscriptionError> {
        subscriber.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(SubscriptionError::SubscriptionNotFound)?;

        if sub.subscriber != subscriber {
            return Err(SubscriptionError::NotSubscriptionOwner);
        }
        if !sub.active {
            return Err(SubscriptionError::AlreadyUnsubscribed);
        }

        sub.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);

        UnsubscribeEvent {
            subscriber,
            topic: sub.topic,
            id,
        }
        .publish(&env);

        Ok(())
    }

    /// Publish an event for `topic`/`entity` and push it to every matching,
    /// active subscriber. Returns the number of subscribers successfully
    /// notified.
    ///
    /// A subscriber contract that traps or returns an error during `notify`
    /// only forfeits its own notification — it cannot block delivery to
    /// other subscribers, because the call is made through
    /// `try_invoke_contract`.
    pub fn publish(env: Env, topic: Symbol, entity: Option<Address>, payload: i128) -> u32 {
        let mut notified = 0u32;

        for id in topic_index(&env, &topic).iter() {
            let sub: Subscription = match env.storage().persistent().get(&DataKey::Subscription(id))
            {
                Some(sub) => sub,
                None => continue,
            };

            if !sub.active {
                continue;
            }
            if let Some(required) = &sub.filter {
                if entity.as_ref() != Some(required) {
                    continue;
                }
            }

            let args = vec![
                &env,
                topic.into_val(&env),
                entity.into_val(&env),
                payload.into_val(&env),
            ];
            let result = env.try_invoke_contract::<(), SdkError>(&sub.subscriber, &NOTIFY_FN, args);
            if result.is_ok() {
                notified += 1;
            }
        }

        PublishEvent {
            topic,
            entity,
            payload,
            notified,
        }
        .publish(&env);

        notified
    }

    pub fn subscription(env: Env, id: u32) -> Option<Subscription> {
        env.storage().persistent().get(&DataKey::Subscription(id))
    }

    /// Count active subscriptions for a topic (including filtered ones).
    pub fn subscriber_count(env: Env, topic: Symbol) -> u32 {
        topic_index(&env, &topic)
            .iter()
            .filter(|id| {
                env.storage()
                    .persistent()
                    .get::<_, Subscription>(&DataKey::Subscription(*id))
                    .map(|sub| sub.active)
                    .unwrap_or(false)
            })
            .count() as u32
    }
}

fn next_id(env: &Env) -> u32 {
    let id: u32 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);
    env.storage().instance().set(&DataKey::NextId, &(id + 1));
    id
}

fn topic_index(env: &Env, topic: &Symbol) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::TopicIndex(topic.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

#[cfg(test)]
mod test;
