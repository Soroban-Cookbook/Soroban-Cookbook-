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
//! 4. Settlement, one of:
//!    - `materialize`    — cooperative close: both endpoints co-sign a *final*
//!      state; ledger channels are rebalanced immediately
//!    - `initiate_close` — unilateral close: one side submits its latest
//!      co-signed state; the channel enters `PendingClose` for the
//!      configured challenge window
//!    - `challenge_close`— during the window, anyone holding a newer
//!      co-signed state (higher `seq`) replaces the pending one
//!    - `finalize_close` — after the window, settles the highest-`seq`
//!      state that was submitted
//! 5. `close_ledger`     — cooperative close once no virtual channel routed
//!    through the ledger channel is still active
//!
//! ## Why a challenge window
//!
//! Without one, Alice could submit an old state where she held more funds
//! while Bob is offline and settle it immediately. With a window of `W`
//! ledgers, Bob (or a watchtower acting for him) has `W` ledgers to present
//! the newer state. The window is set once in `initialize` and copied into
//! every virtual channel when it is opened, so later config changes can never
//! shorten the window of an existing channel.
//!
//! ## Design notes
//!
//! - Signatures: endpoints sign `(tag, channel_id, seq, bal_a, bal_b)` with
//!   `require_auth_for_args`, the same primitive Soroban
//!   contracts use for account authorization. A real deployment would use
//!   Ed25519 off-chain signatures verified on-chain; here `require_auth`
//!   models the "holder of the secret key approves this state" property so the
//!   example stays self-contained.
//!   The `tag` separates ordinary off-chain updates (`state`) from a
//!   cooperative final state (`final`), so an old update can never be used
//!   to skip the challenge window.
//! - Dispute safety: every on-chain submission must strictly increase `seq`,
//!   and unilateral closes only settle after the challenge window.
//! - Collateral conservation: Ingrid's locked collateral can only be released
//!   by a state both endpoints signed, or after the timeout.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Symbol,
};

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
const NS_CLOSE: Symbol = symbol_short!("close");

/// Signature domain for regular off-chain balance updates.
const TAG_STATE: Symbol = symbol_short!("state");
/// Signature domain for a cooperative final state.
const TAG_FINAL: Symbol = symbol_short!("final");

/// Longest allowed challenge window (~31 days at 5s ledgers).
pub const MAX_CHALLENGE_WINDOW: u32 = 535_680;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseEventData {
    /// Virtual-channel id
    pub channel_id: u64,
    /// Sequence number of the state now pending or settled
    pub seq: u64,
    /// Last ledger at which a challenge is accepted
    pub deadline: u32,
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Admin that set the config
    Admin,
    /// Challenge window in ledgers for unilateral closes
    ChallengeWindow,
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
    /// Virtual channels routed through it that are not settled yet
    pub active_virtual: u32,
}

/// Lifecycle of a virtual channel.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelStatus {
    /// Off-chain updates in progress
    Open,
    /// Unilateral close submitted; challenges accepted until `close_deadline`
    PendingClose,
    /// Settled on-chain
    Closed,
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
    /// Current lifecycle stage
    pub status: ChannelStatus,
    /// Challenge window copied from the config at open time
    pub challenge_window: u32,
    /// Last ledger at which `challenge_close` is accepted (0 if not pending)
    pub close_deadline: u32,
}

#[contract]
pub struct VirtualChannelContract;

#[contractimpl]
impl VirtualChannelContract {
    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------

    /// One-time setup. `challenge_window` is the number of ledgers a
    /// unilateral close stays open for challenges (e.g. 17_280 ~ 24h).
    pub fn initialize(env: Env, admin: Address, challenge_window: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        if challenge_window == 0 || challenge_window > MAX_CHALLENGE_WINDOW {
            panic!("Invalid challenge window");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ChallengeWindow, &challenge_window);
    }

    pub fn get_challenge_window(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ChallengeWindow)
            .expect("Not initialized")
    }

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
            active_virtual: 0,
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
        if ch.active_virtual > 0 {
            panic!("Virtual channel still active");
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
        if alice == bob || alice == intermediary || bob == intermediary {
            panic!("Participants must be distinct");
        }
        let challenge_window = Self::get_challenge_window(env.clone());
        alice.require_auth();
        bob.require_auth();
        intermediary.require_auth();

        if ledger_a == ledger_b {
            panic!("Ledger channels must differ");
        }
        let mut la: LedgerChannel = Self::get_ledger(env.clone(), ledger_a);
        let mut lb: LedgerChannel = Self::get_ledger(env.clone(), ledger_b);
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

        la.active_virtual += 1;
        lb.active_virtual += 1;
        env.storage()
            .instance()
            .set(&DataKey::Ledger(ledger_a), &la);
        env.storage()
            .instance()
            .set(&DataKey::Ledger(ledger_b), &lb);

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
            status: ChannelStatus::Open,
            challenge_window,
            close_deadline: 0,
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

    /// Cooperative close: both endpoints co-sign a *final* state and the
    /// channel settles immediately. Allowed while open or during a pending
    /// close (a mutually signed final state overrides any dispute), as long
    /// as `seq` is higher than anything submitted before.
    pub fn materialize(env: Env, channel_id: u64, seq: u64, bal_a: i128, bal_b: i128) {
        let mut vc = Self::get_virtual(env.clone(), channel_id);
        if vc.status == ChannelStatus::Closed {
            panic!("Already materialized");
        }
        Self::check_balances(&vc, bal_a, bal_b);
        if seq <= vc.seq {
            panic!("Sequence must increase");
        }
        Self::require_cosigned(&env, &vc, TAG_FINAL, channel_id, seq, bal_a, bal_b);

        vc.seq = seq;
        vc.bal_a = bal_a;
        vc.bal_b = bal_b;
        Self::settle(&env, channel_id, vc);
    }

    /// Phase 1 of a unilateral close. Submits the latest co-signed state and
    /// starts the challenge window. `seq == 0` with the opening balances
    /// closes a channel that never had an update (no signature needed: both
    /// endpoints already approved it in `open_virtual`).
    pub fn initiate_close(env: Env, channel_id: u64, seq: u64, bal_a: i128, bal_b: i128) -> u32 {
        let mut vc = Self::get_virtual(env.clone(), channel_id);
        if vc.status != ChannelStatus::Open {
            panic!("Channel not open");
        }
        Self::check_balances(&vc, bal_a, bal_b);
        if seq == 0 {
            if bal_a != vc.amount || bal_b != 0 {
                panic!("Initial state must match opening balances");
            }
        } else {
            Self::require_cosigned(&env, &vc, TAG_STATE, channel_id, seq, bal_a, bal_b);
        }

        let deadline = env
            .ledger()
            .sequence()
            .checked_add(vc.challenge_window)
            .expect("Deadline overflow");
        vc.status = ChannelStatus::PendingClose;
        vc.close_deadline = deadline;
        vc.seq = seq;
        vc.bal_a = bal_a;
        vc.bal_b = bal_b;
        env.storage()
            .instance()
            .set(&DataKey::Virtual(channel_id), &vc);

        env.events().publish(
            (NS_CLOSE, symbol_short!("init"), channel_id),
            CloseEventData {
                channel_id,
                seq,
                deadline,
            },
        );
        deadline
    }

    /// Phase 2: replace the pending state with a newer co-signed one. Only
    /// accepted up to and including `close_deadline`. The deadline is not
    /// extended, so a close always finalizes in bounded time.
    pub fn challenge_close(env: Env, channel_id: u64, seq: u64, bal_a: i128, bal_b: i128) {
        let mut vc = Self::get_virtual(env.clone(), channel_id);
        if vc.status != ChannelStatus::PendingClose {
            panic!("No pending close");
        }
        if env.ledger().sequence() > vc.close_deadline {
            panic!("Challenge window elapsed");
        }
        if seq <= vc.seq {
            panic!("Sequence must increase");
        }
        Self::check_balances(&vc, bal_a, bal_b);
        Self::require_cosigned(&env, &vc, TAG_STATE, channel_id, seq, bal_a, bal_b);

        vc.seq = seq;
        vc.bal_a = bal_a;
        vc.bal_b = bal_b;
        env.storage()
            .instance()
            .set(&DataKey::Virtual(channel_id), &vc);

        env.events().publish(
            (NS_CLOSE, symbol_short!("chal"), channel_id),
            CloseEventData {
                channel_id,
                seq,
                deadline: vc.close_deadline,
            },
        );
    }

    /// Phase 3: once the window has passed, anyone can settle the
    /// highest-sequence state that was submitted.
    pub fn finalize_close(env: Env, channel_id: u64) {
        let vc = Self::get_virtual(env.clone(), channel_id);
        if vc.status != ChannelStatus::PendingClose {
            panic!("No pending close");
        }
        if env.ledger().sequence() <= vc.close_deadline {
            panic!("Challenge window still open");
        }
        Self::settle(&env, channel_id, vc);
    }

    pub fn get_virtual(env: Env, channel_id: u64) -> VirtualChannel {
        env.storage()
            .instance()
            .get(&DataKey::Virtual(channel_id))
            .expect("Virtual channel not found")
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn check_balances(vc: &VirtualChannel, bal_a: i128, bal_b: i128) {
        let conserved = bal_a.checked_add(bal_b) == Some(vc.amount);
        if bal_a < 0 || bal_b < 0 || !conserved {
            panic!("Balances must be non-negative and conserve amount");
        }
    }

    /// Both endpoints must have signed this exact state under `tag`.
    fn require_cosigned(
        env: &Env,
        vc: &VirtualChannel,
        tag: Symbol,
        channel_id: u64,
        seq: u64,
        bal_a: i128,
        bal_b: i128,
    ) {
        let args: soroban_sdk::Vec<soroban_sdk::Val> =
            (tag, channel_id, seq, bal_a, bal_b).into_val(env);
        vc.alice.require_auth_for_args(args.clone());
        vc.bob.require_auth_for_args(args);
    }

    /// Rebalance the backing ledger channels to the given virtual state and
    /// mark the virtual channel closed.
    fn settle(env: &Env, channel_id: u64, mut vc: VirtualChannel) {
        let mut la: LedgerChannel = Self::get_ledger(env.clone(), vc.ledger_a);
        let mut lb: LedgerChannel = Self::get_ledger(env.clone(), vc.ledger_b);

        // Endpoint deposits track the virtual balances; the intermediary
        // tops up the difference from its own collateral.
        if la.endpoint == vc.alice {
            la.endpoint_deposit = vc.bal_a;
            la.intermediary_deposit = vc.amount - vc.bal_a;
        } else {
            la.intermediary_deposit = vc.bal_a;
            la.endpoint_deposit = vc.amount - vc.bal_a;
        }
        if lb.endpoint == vc.bob {
            lb.endpoint_deposit = vc.bal_b;
            lb.intermediary_deposit = vc.amount - vc.bal_b;
        } else {
            lb.intermediary_deposit = vc.bal_b;
            lb.endpoint_deposit = vc.amount - vc.bal_b;
        }
        la.settled_seq = vc.seq;
        lb.settled_seq = vc.seq;
        la.active_virtual = la.active_virtual.saturating_sub(1);
        lb.active_virtual = lb.active_virtual.saturating_sub(1);

        vc.status = ChannelStatus::Closed;
        vc.close_deadline = 0;

        env.storage()
            .instance()
            .set(&DataKey::Ledger(vc.ledger_a), &la);
        env.storage()
            .instance()
            .set(&DataKey::Ledger(vc.ledger_b), &lb);
        env.storage()
            .instance()
            .set(&DataKey::Virtual(channel_id), &vc);

        env.events().publish(
            (NS_UPDATE, symbol_short!("mat"), channel_id),
            UpdateEventData {
                channel_id,
                seq: vc.seq,
                bal_a: vc.bal_a,
                bal_b: vc.bal_b,
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test;
