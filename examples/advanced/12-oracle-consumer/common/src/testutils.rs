//! Test doubles shared by the consumer crates. Enabled by the `testutils`
//! feature, which the consumers turn on only in `[dev-dependencies]`.

use soroban_sdk::{contract, contractimpl, Env, Symbol};

use crate::Quote;

/// A price feed whose quotes are set directly by the test.
#[contract]
pub struct MockFeed;

#[contractimpl]
impl MockFeed {
    /// Set the quote returned for `asset`.
    pub fn set_quote(env: Env, asset: Symbol, price: i128, timestamp: u64) {
        env.storage()
            .instance()
            .set(&asset, &Quote { price, timestamp });
    }

    /// An asset with no quote set reports price `0`, which every consumer
    /// rejects as out of bounds — the same way a real feed with no data yet
    /// behaves.
    pub fn quote(env: Env, asset: Symbol) -> Quote {
        env.storage().instance().get(&asset).unwrap_or(Quote {
            price: 0,
            timestamp: 0,
        })
    }
}

/// A contract sitting at a feed address that does not expose `quote` — stands
/// in for a decommissioned or misconfigured provider.
#[contract]
pub struct BrokenFeed;

#[contractimpl]
impl BrokenFeed {
    pub fn ping(env: Env) -> u32 {
        let _ = env;
        1
    }
}
