//! Test helpers for inspecting contract events under SDK 26+.
//!
//! `ContractEvents` no longer implements collection methods directly; use
//! [`EventList`] to preserve the pre-26 test API (`len`, `get`, `last`, etc.).

use soroban_sdk::testutils::ContractEvents;
use soroban_sdk::xdr::{ContractEventBody, ScAddress};
use soroban_sdk::{Address, Env, TryFromVal, Val, Vec};

/// Wrapper around [`ContractEvents`] with legacy-friendly accessors for tests.
#[derive(Clone)]
pub struct EventList {
    env: Env,
    events: ContractEvents,
}

impl EventList {
    pub fn new(env: &Env, events: ContractEvents) -> Self {
        Self {
            env: env.clone(),
            events,
        }
    }

    pub fn len(&self) -> usize {
        self.events.events().len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.events().is_empty()
    }

    pub fn get(&self, index: u32) -> Option<(Address, Vec<Val>, Val)> {
        decode_event(&self.env, self.events.events().get(index as usize)?)
    }

    pub fn last(&self) -> Option<(Address, Vec<Val>, Val)> {
        let index = self.len().checked_sub(1)?;
        self.get(index as u32)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Address, Vec<Val>, Val)> + '_ {
        (0..self.len()).filter_map(|i| self.get(i as u32))
    }
}

fn decode_event(
    env: &Env,
    event: &soroban_sdk::xdr::ContractEvent,
) -> Option<(Address, Vec<Val>, Val)> {
    let contract_id = event.contract_id.as_ref()?;
    let addr = Address::try_from_val(env, &ScAddress::Contract(contract_id.clone())).ok()?;

    let ContractEventBody::V0(v0) = &event.body;

    let mut topics = Vec::new(env);
    for topic in v0.topics.iter() {
        topics.push_back(Val::try_from_val(env, topic).ok()?);
    }

    let data = Val::try_from_val(env, &v0.data).ok()?;
    Some((addr, topics, data))
}
