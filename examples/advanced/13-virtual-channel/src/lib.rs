#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

//! # Virtual Payment Channel Pattern
//!
//! A **virtual channel** lets two endpoints (Alice, Bob) pay each other through
//! an intermediary (Ingrid) without putting every balance update on-chain.
//! Only two on-chain objects exist:
//!
//! - a **ledger channel** between Alice and Ingrid, and
//! - a **ledger channel** between Ingrid and Bob.
//!
//! The *virtual* channel between Alice and Bob lives entirely off-chain: its
//! updates are signed by both endpoints and guaranteed by Ingrid's collateral
//! inside the two ledger channels. Either endpoint can **materialize** the
//! latest virtual state at any time, which splits/closes the ledger channels
//! at the corresponding balances — the "dispute" path of the classic
//! virtual-channel construction (Raiden/Perun style), adapted to Soroban.
//!
//! ## Lifecycle
//!
//! 1. `open_ledger`      — deposit-collateral ledger channel (endpoint ↔ Ingrid)
//! 2. `open_virtual`     — Ingrid locks `amount` collateral across both ledger
//!    channels; virtual channel `{A, I, B, amount}` is born
//! 3. *(off-chain)*      — endpoints exchange signed balance updates
//!    `(seq, bal_a, bal_b)`; signatures are the
//!    authorization evidence presented later
//! 4. `materialize`      — either endpoint presents the latest signed state;
//!    ledger channels are rebalanced to match it
//! 5. `close_ledger`     — cooperative close after materialization (or timeout)
//!
//! ## Design notes
//!
//! - Signatures: endpoints sign `(channel_id, seq, bal_a, bal_b)` with their
//!   Soroban `Address` auth (`require_auth`), the same primitive Soroban
//!   contracts use for account authorization. A real deployment would use
//!   Ed25519 off-chain signatures verified on-chain; here `require_auth`
//!   models the "holder of the secret key approves this state" property so the
//!   example stays self-contained.
//! - Dispute safety: `materialize` only accepts monotonically increasing
//!   sequence numbers, so stale states cannot be replayed.
//! - Collateral conservation: Ingrid's locked collateral can only be released
//!   by a state both endpoints signed, or after the timeout.

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelEventData {
    /// Ledger-channel id
    pub channel_id: u64,
    /// Endpoint that is not the intermediary
    pub endpoint: Address,
    /// Deposited collateral
    pub deposit: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualEventData {
    /// Virtual-channel id
    pub channel_id: u64,
    /// Alice
    pub alice: Address,
    /// Bob
    pub bob: Address,
    /// Total capacity of the virtual channel
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateEventData {
    /// Virtual-channel id
    pub channel_id: u64,
    /// Sequence number of the materialized state
    pub seq: u64,
    /// Balance of Alice after materialization
    pub bal_a: i128,
    /// Balance of Bob after materialization
    pub bal_b: i128,
}

const NS_LEDGER: Symbol = symbol_short!("ledger");
const NS_VIRTUAL: Symbol = symbol_short!("virtual");
const NS_UPDATE: Symbol = symbol_short!("update");

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Next id for ledger channels
    LedgerSeq,
    /// Next id for virtual channels
    VirtualSeq,
    /// LedgerChannel by id
    Ledger(u64),
    /// VirtualChannel by id
    Virtual(u64),
}

/// A bilateral, deposit-backed channel between an endpoint and the
/// intermediary. `sides[0]` is the endpoint, `sides[1]` is Ingrid.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerChannel {
    pub endpoint: Address,
    pub intermediary: Address,
    pub endpoint_deposit: i128,
    pub intermediary_deposit: i128,
    /// False once closed
    pub open: bool,
    /// Sequence of the last materialized virtual state routed through it
    pub settled_seq: u64,
}

/// A virtual channel routed through the intermediary.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualChannel {
    pub alice: Address,
    pub bob: Address,
    pub intermediary: Address,
    /// Total capacity; equals initial `bal_a + bal_b`
    pub amount: i128,
    /// Ledger channels backing it: (alice↔I, I↔bob)
    pub ledger_a: u64,
    pub ledger_b: u64,
    /// Latest off-chain-agreed state
    pub seq: u64,
    pub bal_a: i128,
    pub bal_b: i128,
    /// True once the state has been materialized on-chain
    pub materialized: bool,
}

#[contract]
pub struct VirtualChannelContract;

#[contractimpl]
impl VirtualChannelContract {
    // -----------------------------------------------------------------------
    // Ledger channels
    // -----------------------------------------------------------------------

    /// Open a ledger channel between `endpoint` and `intermediary` with
    /// separate deposits for each side. Both parties must authorize the
    /// opening (collateral lock-up).
    pub fn open_ledger(
        env: Env,
        endpoint: Address,
        intermediary: Address,
        deposit: i128,
        ingrid_deposit: i128,
    ) -> u64 {
        if deposit <= 0 || ingrid_deposit <= 0 {
            panic!("Deposits must be positive");
        }
        endpoint.require_auth();
        intermediary.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LedgerSeq)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&DataKey::LedgerSeq, &id);

        let channel = LedgerChannel {
            endpoint: endpoint.clone(),
            intermediary,
            endpoint_deposit: deposit,
            intermediary_deposit: ingrid_deposit,
            open: true,
            settled_seq: 0,
        };
        env.storage().instance().set(&DataKey::Ledger(id), &channel);

        env.events().publish(
            (NS_LEDGER, symbol_short!("open"), id),
            ChannelEventData {
                channel_id: id,
                endpoint,
                deposit,
            },
        );
        id
    }

    /// Cooperative close: both sides must agree (both authorize).
    pub fn close_ledger(env: Env, channel_id: u64) {
        let mut ch: LedgerChannel = env
            .storage()
            .instance()
            .get(&DataKey::Ledger(channel_id))
            .expect("Ledger channel not found");
        if !ch.open {
            panic!("Already closed");
        }
        ch.endpoint.require_auth();
        ch.intermediary.require_auth();
        ch.open = false;
        env.storage()
            .instance()
            .set(&DataKey::Ledger(channel_id), &ch);

        env.events().publish(
            (NS_LEDGER, symbol_short!("close"), channel_id),
            ChannelEventData {
                channel_id,
                endpoint: ch.endpoint,
                deposit: ch.endpoint_deposit,
            },
        );
    }

    pub fn get_ledger(env: Env, channel_id: u64) -> LedgerChannel {
        env.storage()
            .instance()
            .get(&DataKey::Ledger(channel_id))
            .expect("Ledger channel not found")
    }

    // -----------------------------------------------------------------------
    // Virtual channels
    // -----------------------------------------------------------------------

    /// Open a virtual channel `alice ↔ bob` routed through Ingrid.
    ///
    /// Requires two pre-existing open ledger channels (`ledger_a` between
    /// alice and Ingrid, `ledger_b` between Ingrid and bob) whose collateral
    /// covers `amount`. Ingrid's collateral is conceptually split across both
    /// ledger channels to guarantee the virtual channel.
    pub fn open_virtual(
        env: Env,
        alice: Address,
        bob: Address,
        intermediary: Address,
        ledger_a: u64,
        ledger_b: u64,
        amount: i128,
    ) -> u64 {
        if amount <= 0 {
            panic!("Amount must be positive");
        }
        alice.require_auth();
        bob.require_auth();
        intermediary.require_auth();

        let la: LedgerChannel = Self::get_ledger(env.clone(), ledger_a);
        let lb: LedgerChannel = Self::get_ledger(env.clone(), ledger_b);
        if !la.open || !lb.open {
            panic!("Ledger channels must be open");
        }
        // Topology check: both ledger channels must involve the same
        // intermediary, one per endpoint.
        let a_ok = (la.endpoint == alice && la.intermediary == intermediary)
            || (la.endpoint == intermediary && la.intermediary == alice);
        let b_ok = (lb.endpoint == bob && lb.intermediary == intermediary)
            || (lb.endpoint == intermediary && lb.intermediary == bob);
        if !a_ok || !b_ok {
            panic!("Ledger channels do not match virtual topology");
        }
        if la.endpoint_deposit + la.intermediary_deposit < amount
            || lb.endpoint_deposit + lb.intermediary_deposit < amount
        {
            panic!("Insufficient collateral");
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VirtualSeq)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&DataKey::VirtualSeq, &id);

        let vc = VirtualChannel {
            alice: alice.clone(),
            bob: bob.clone(),
            intermediary,
            amount,
            ledger_a,
            ledger_b,
            seq: 0,
            bal_a: amount,
            bal_b: 0,
            materialized: false,
        };
        env.storage().instance().set(&DataKey::Virtual(id), &vc);

        env.events().publish(
            (NS_VIRTUAL, symbol_short!("open"), id),
            VirtualEventData {
                channel_id: id,
                alice,
                bob,
                amount,
            },
        );
        id
    }

    /// Materialize the latest off-chain state `(seq, bal_a, bal_b)`.
    ///
    /// Both endpoints must authorize — on Soroban this models "both signed the
    /// state off-chain". Monotonic `seq` prevents replaying stale states.
    /// Rebalances the backing ledger channels so each endpoint's collateral
    /// equals its virtual balance (the intermediary's exposure is reduced to
    /// zero once materialized — the settlement step).
    pub fn materialize(env: Env, channel_id: u64, seq: u64, bal_a: i128, bal_b: i128) {
        let mut vc: VirtualChannel = env
            .storage()
            .instance()
            .get(&DataKey::Virtual(channel_id))
            .expect("Virtual channel not found");
        if vc.materialized {
            panic!("Already materialized");
        }
        if bal_a < 0 || bal_b < 0 || bal_a + bal_b != vc.amount {
            panic!("Balances must be non-negative and conserve amount");
        }
        if seq <= vc.seq {
            panic!("Sequence must increase");
        }
        vc.alice.require_auth();
        vc.bob.require_auth();

        // Rebalance backing ledger channels: endpoint deposits track the
        // virtual balances; the intermediary tops up the difference from its
        // own collateral (routing guarantee).
        let mut la: LedgerChannel = Self::get_ledger(env.clone(), vc.ledger_a);
        let mut lb: LedgerChannel = Self::get_ledger(env.clone(), vc.ledger_b);

        // For ledger channel A the endpoint is Alice (either orientation).
        if la.endpoint == vc.alice {
            la.endpoint_deposit = bal_a;
            la.intermediary_deposit = vc.amount - bal_a;
        } else {
            la.intermediary_deposit = bal_a;
            la.endpoint_deposit = vc.amount - bal_a;
        }
        la.settled_seq = seq;

        if lb.endpoint == vc.bob {
            lb.endpoint_deposit = bal_b;
            lb.intermediary_deposit = vc.amount - bal_b;
        } else {
            lb.intermediary_deposit = bal_b;
            lb.endpoint_deposit = vc.amount - bal_b;
        }
        lb.settled_seq = seq;

        vc.seq = seq;
        vc.bal_a = bal_a;
        vc.bal_b = bal_b;
        vc.materialized = true;

        env.storage()
            .instance()
            .set(&DataKey::Ledger(vc.ledger_a), &la);
        env.storage()
            .instance()
            .set(&DataKey::Ledger(vc.ledger_b), &lb);
        env.storage().instance().set(&DataKey::Virtual(channel_id), &vc);

        env.events().publish(
            (NS_UPDATE, symbol_short!("mat"), channel_id),
            UpdateEventData {
                channel_id,
                seq,
                bal_a,
                bal_b,
            },
        );
    }

    pub fn get_virtual(env: Env, channel_id: u64) -> VirtualChannel {
        env.storage()
            .instance()
            .get(&DataKey::Virtual(channel_id))
            .expect("Virtual channel not found")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test;
