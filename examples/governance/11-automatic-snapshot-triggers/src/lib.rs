#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

//! # Automatic Snapshot Triggers (Governance)
//!
//! Governance contracts need historical voting power: a vote on proposal `P`
//! must be weighed by the voter's power *when `P` was created*, otherwise
//! power can be moved around mid-vote. The naive approach writes a snapshot on
//! every governance action. With 500 votes on a hot proposal that is 500
//! persistent entries, each of which must have its TTL extended to stay alive.
//!
//! This example bounds snapshot growth with two strategies:
//!
//! - [`SnapshotMode::Interval`] (default): a value change only creates a new
//!   snapshot if at least `snapshot_interval_ledgers` have passed since the
//!   owner's previous snapshot. Changes inside the window only update the
//!   owner's *current* value (one fixed-size entry). Snapshot count per owner
//!   is at most `ledgers_elapsed / interval + 1`, regardless of activity.
//! - [`SnapshotMode::Epoch`]: snapshots are anchored to proposal creation.
//!   `create_proposal` opens a new epoch (one write). The first time an
//!   owner's value changes inside an epoch, the *pre-change* value is
//!   checkpointed for that epoch. Owners whose values do not change never
//!   write snapshots, and voting itself never writes one. Storage per owner is
//!   bounded by the number of proposals during which their value changed.
//!
//! Voting (`vote`) reads historical power and never creates snapshots.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotRecord {
    pub ledger: u32,
    pub timestamp: u64,
    pub value: i128,
}

/// Checkpoint for [`SnapshotMode::Epoch`]: `value` is the owner's value at
/// the moment epoch (proposal) `epoch` was created.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochRecord {
    pub epoch: u32,
    pub value: i128,
}

/// Snapshot policy. Defaults to `Interval`.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotMode {
    /// At most one snapshot per owner per `snapshot_interval_ledgers`.
    Interval,
    /// Checkpoint only on the first change per owner per proposal epoch.
    Epoch,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    /// Ledger at which voting power is measured (Interval mode).
    pub snapshot_ledger: u32,
    pub votes_for: i128,
    pub votes_against: i128,
    pub voter_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    /// Minimum ledgers between two snapshots of the same owner
    /// (`snapshot_interval_ledgers`).
    Frequency,
    LastSnapshot,
    TotalSnapshots,
    Enabled,
    PruneThreshold,
    Snapshots(Address),
    Mode,
    /// Number of proposals created so far == current epoch.
    Epoch,
    /// Latest value per owner (always up to date, single entry).
    Current(Address),
    EpochSnapshots(Address),
    Proposal(u32),
    Voted(u32, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SnapshotError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidFrequency = 4,
    NoSnapshotDue = 5,
    NothingToPrune = 6,
    IndexOutOfBounds = 7,
    EmptyHistory = 8,
    ProposalNotFound = 9,
    AlreadyVoted = 10,
    NoVotingPower = 11,
}

/// Upper bound on the snapshot interval (~31 days at 5s ledgers). Larger
/// values would let snapshots go stale longer than persistent TTLs live.
pub const MAX_INTERVAL_LEDGERS: u32 = 535_680;

const EVENT_NS: Symbol = symbol_short!("snapshot");

#[contract]
pub struct SnapshotTrigger;

#[contractimpl]
impl SnapshotTrigger {
    pub fn initialize(env: Env, admin: Address, frequency: u32) -> Result<(), SnapshotError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SnapshotError::AlreadyInitialized);
        }
        admin.require_auth();
        Self::validate_interval(frequency)?;
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Mode, &SnapshotMode::Interval);
        env.storage().instance().set(&DataKey::Epoch, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::Frequency, &frequency);
        env.storage().instance().set(&DataKey::LastSnapshot, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::TotalSnapshots, &0u32);
        env.storage().instance().set(&DataKey::Enabled, &true);
        env.storage()
            .instance()
            .set(&DataKey::PruneThreshold, &0u32);
        env.events()
            .publish((EVENT_NS, symbol_short!("init")), (admin, frequency));
        Ok(())
    }

    pub fn set_frequency(env: Env, admin: Address, frequency: u32) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        Self::validate_interval(frequency)?;
        let old: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Frequency)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Frequency, &frequency);
        env.events()
            .publish((EVENT_NS, symbol_short!("freq")), (old, frequency));
        Ok(())
    }

    pub fn set_enabled(env: Env, admin: Address, enabled: bool) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old: bool = env
            .storage()
            .instance()
            .get(&DataKey::Enabled)
            .unwrap_or(true);
        env.storage().instance().set(&DataKey::Enabled, &enabled);
        env.events()
            .publish((EVENT_NS, symbol_short!("enable")), (old, enabled));
        Ok(())
    }

    pub fn set_prune_threshold(
        env: Env,
        admin: Address,
        older_than: u32,
    ) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PruneThreshold)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PruneThreshold, &older_than);
        env.events()
            .publish((EVENT_NS, symbol_short!("prune_cfg")), (old, older_than));
        Ok(())
    }

    /// Switch between interval-gated and epoch-based snapshots.
    pub fn set_mode(env: Env, admin: Address, mode: SnapshotMode) -> Result<(), SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let old = Self::mode(&env);
        env.storage().instance().set(&DataKey::Mode, &mode);
        env.events()
            .publish((EVENT_NS, symbol_short!("mode")), (old, mode));
        Ok(())
    }

    pub fn get_mode(env: Env) -> SnapshotMode {
        Self::mode(&env)
    }

    /// Record a new value for `owner` (e.g. voting power after a delegation).
    ///
    /// The current value is always updated. A historical snapshot is written
    /// only when the active [`SnapshotMode`] says one is due, so repeated
    /// updates do not grow storage. Returns `true` if a snapshot was written.
    pub fn record_value(env: Env, owner: Address, value: i128) -> bool {
        owner.require_auth();
        let previous = Self::current_value(&env, &owner);
        let enabled = Self::enabled(&env);

        let snapshotted = match Self::mode(&env) {
            SnapshotMode::Interval => {
                if enabled && Self::interval_due(&env, &owner) {
                    Self::push_interval_snapshot(&env, &owner, value);
                    true
                } else {
                    false
                }
            }
            SnapshotMode::Epoch => enabled && Self::checkpoint_epoch(&env, &owner, previous),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Current(owner.clone()), &value);

        env.events().publish(
            (EVENT_NS, symbol_short!("record")),
            (owner, value, env.ledger().sequence(), snapshotted),
        );
        snapshotted
    }

    /// Time-based snapshot of the owner's current value. Fails with
    /// `NoSnapshotDue` if the owner's interval has not elapsed yet.
    pub fn auto_snapshot(env: Env, owner: Address) -> Result<(), SnapshotError> {
        if !Self::enabled(&env) {
            return Err(SnapshotError::NoSnapshotDue);
        }
        let frequency: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Frequency)
            .unwrap_or(0);
        if frequency == 0 {
            return Err(SnapshotError::InvalidFrequency);
        }
        if !Self::interval_due(&env, &owner) {
            return Err(SnapshotError::NoSnapshotDue);
        }
        let value = Self::current_value(&env, &owner);
        Self::push_interval_snapshot(&env, &owner, value);
        Ok(())
    }

    /// Next ledger at which `owner` may receive a new interval snapshot.
    pub fn next_snapshot_ledger(env: Env, owner: Address) -> u32 {
        let snapshots = Self::load_snapshots(&env, &owner);
        if snapshots.is_empty() {
            return env.ledger().sequence();
        }
        let last = snapshots.get_unchecked(snapshots.len() - 1).ledger;
        last.saturating_add(Self::interval(&env))
    }

    pub fn get_current(env: Env, owner: Address) -> i128 {
        Self::current_value(&env, &owner)
    }

    // -----------------------------------------------------------------------
    // Proposals (epoch anchors) and voting
    // -----------------------------------------------------------------------

    /// Create a proposal. This is the epoch boundary: exactly one write,
    /// regardless of how many voters exist.
    pub fn create_proposal(env: Env, proposer: Address) -> Result<u32, SnapshotError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(SnapshotError::NotInitialized);
        }
        proposer.require_auth();
        let id = Self::epoch(&env)
            .checked_add(1)
            .ok_or(SnapshotError::IndexOutOfBounds)?;
        env.storage().instance().set(&DataKey::Epoch, &id);

        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            snapshot_ledger: env.ledger().sequence(),
            votes_for: 0,
            votes_against: 0,
            voter_count: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), &proposal);
        env.events().publish(
            (EVENT_NS, symbol_short!("proposal"), id),
            (proposer, proposal.snapshot_ledger),
        );
        Ok(id)
    }

    /// Vote using power measured at proposal creation. Never writes a
    /// snapshot: only the tally and a per-voter double-vote guard.
    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        support: bool,
    ) -> Result<i128, SnapshotError> {
        voter.require_auth();
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(SnapshotError::ProposalNotFound)?;
        let voted_key = DataKey::Voted(proposal_id, voter.clone());
        if env.storage().persistent().has(&voted_key) {
            return Err(SnapshotError::AlreadyVoted);
        }
        let power = Self::power_at(&env, &voter, &proposal);
        if power <= 0 {
            return Err(SnapshotError::NoVotingPower);
        }
        if support {
            proposal.votes_for = proposal.votes_for.saturating_add(power);
        } else {
            proposal.votes_against = proposal.votes_against.saturating_add(power);
        }
        proposal.voter_count += 1;
        env.storage().persistent().set(&voted_key, &true);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.events().publish(
            (EVENT_NS, symbol_short!("vote"), proposal_id),
            (voter, support, power),
        );
        Ok(power)
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, SnapshotError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(SnapshotError::ProposalNotFound)
    }

    /// Voting power of `owner` for `proposal_id` under the active mode.
    pub fn voting_power_at(
        env: Env,
        owner: Address,
        proposal_id: u32,
    ) -> Result<i128, SnapshotError> {
        let proposal = Self::get_proposal(env.clone(), proposal_id)?;
        Ok(Self::power_at(&env, &owner, &proposal))
    }

    pub fn epoch_snapshot_count(env: Env, owner: Address) -> u32 {
        Self::load_epoch_snapshots(&env, &owner).len()
    }

    pub fn get_epoch_snapshots(env: Env, owner: Address) -> Vec<EpochRecord> {
        Self::load_epoch_snapshots(&env, &owner)
    }

    /// Prune snapshots for an owner that are older than the given ledger.
    pub fn prune(
        env: Env,
        admin: Address,
        owner: Address,
        older_than: u32,
    ) -> Result<u32, SnapshotError> {
        Self::require_admin(&env, &admin)?;
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner.clone()))
            .ok_or(SnapshotError::NothingToPrune)?;
        let original_len = snapshots.len();
        let mut kept: Vec<SnapshotRecord> = Vec::new(&env);
        for i in 0..original_len {
            let s = snapshots.get_unchecked(i);
            if s.ledger >= older_than {
                kept.push_back(s);
            }
        }
        let pruned = original_len - kept.len();
        if pruned == 0 {
            return Err(SnapshotError::NothingToPrune);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Snapshots(owner.clone()), &kept);

        env.events()
            .publish((EVENT_NS, symbol_short!("prune")), (admin, owner, pruned));
        Ok(pruned)
    }

    pub fn get_snapshot(
        env: Env,
        owner: Address,
        index: u32,
    ) -> Result<SnapshotRecord, SnapshotError> {
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .ok_or(SnapshotError::EmptyHistory)?;
        if index >= snapshots.len() {
            return Err(SnapshotError::IndexOutOfBounds);
        }
        Ok(snapshots.get_unchecked(index))
    }

    pub fn get_latest(env: Env, owner: Address) -> Result<SnapshotRecord, SnapshotError> {
        let snapshots: Vec<SnapshotRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .ok_or(SnapshotError::EmptyHistory)?;
        if snapshots.is_empty() {
            return Err(SnapshotError::EmptyHistory);
        }
        Ok(snapshots.get_unchecked(snapshots.len() - 1))
    }

    pub fn snapshot_count(env: Env, owner: Address) -> u32 {
        env.storage()
            .persistent()
            .get::<_, Vec<SnapshotRecord>>(&DataKey::Snapshots(owner))
            .map(|s| s.len())
            .unwrap_or(0)
    }

    pub fn get_all_snapshots(env: Env, owner: Address) -> Vec<SnapshotRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Snapshots(owner))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_config(env: Env) -> (u32, bool, u32, u32) {
        let frequency: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Frequency)
            .unwrap_or(0);
        let enabled: bool = env
            .storage()
            .instance()
            .get(&DataKey::Enabled)
            .unwrap_or(true);
        let last: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LastSnapshot)
            .unwrap_or(0);
        let total: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSnapshots)
            .unwrap_or(0);
        (frequency, enabled, last, total)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn validate_interval(interval: u32) -> Result<(), SnapshotError> {
        if interval == 0 || interval > MAX_INTERVAL_LEDGERS {
            return Err(SnapshotError::InvalidFrequency);
        }
        Ok(())
    }

    fn mode(env: &Env) -> SnapshotMode {
        env.storage()
            .instance()
            .get(&DataKey::Mode)
            .unwrap_or(SnapshotMode::Interval)
    }

    fn enabled(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Enabled)
            .unwrap_or(true)
    }

    fn interval(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Frequency)
            .unwrap_or(1)
    }

    fn epoch(env: &Env) -> u32 {
        env.storage().instance().get(&DataKey::Epoch).unwrap_or(0)
    }

    fn current_value(env: &Env, owner: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Current(owner.clone()))
            .unwrap_or(0)
    }

    fn load_snapshots(env: &Env, owner: &Address) -> Vec<SnapshotRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Snapshots(owner.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn load_epoch_snapshots(env: &Env, owner: &Address) -> Vec<EpochRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::EpochSnapshots(owner.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// True if `owner` has no snapshot yet or the last one is at least one
    /// interval old. Per-owner gating: one busy account cannot starve others.
    fn interval_due(env: &Env, owner: &Address) -> bool {
        let snapshots = Self::load_snapshots(env, owner);
        if snapshots.is_empty() {
            return true;
        }
        let last = snapshots.get_unchecked(snapshots.len() - 1).ledger;
        env.ledger().sequence() >= last.saturating_add(Self::interval(env))
    }

    fn push_interval_snapshot(env: &Env, owner: &Address, value: i128) {
        let now = env.ledger().sequence();
        let mut snapshots = Self::load_snapshots(env, owner);
        snapshots.push_back(SnapshotRecord {
            ledger: now,
            timestamp: env.ledger().timestamp(),
            value,
        });
        env.storage()
            .persistent()
            .set(&DataKey::Snapshots(owner.clone()), &snapshots);
        env.storage().instance().set(&DataKey::LastSnapshot, &now);
        Self::bump_total(env);
        env.events().publish(
            (EVENT_NS, symbol_short!("snap")),
            (owner.clone(), value, now),
        );
    }

    /// Epoch mode: on the first change inside the current epoch, store the
    /// pre-change value tagged with that epoch. Returns true if written.
    fn checkpoint_epoch(env: &Env, owner: &Address, previous: i128) -> bool {
        let epoch = Self::epoch(env);
        if epoch == 0 {
            // No proposal exists yet, so no historical value can be queried.
            return false;
        }
        let mut records = Self::load_epoch_snapshots(env, owner);
        if !records.is_empty() && records.get_unchecked(records.len() - 1).epoch >= epoch {
            return false;
        }
        records.push_back(EpochRecord {
            epoch,
            value: previous,
        });
        env.storage()
            .persistent()
            .set(&DataKey::EpochSnapshots(owner.clone()), &records);
        Self::bump_total(env);
        true
    }

    fn bump_total(env: &Env) {
        let total: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSnapshots)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSnapshots, &total.saturating_add(1));
    }

    fn power_at(env: &Env, owner: &Address, proposal: &Proposal) -> i128 {
        match Self::mode(env) {
            SnapshotMode::Epoch => {
                // First checkpoint with epoch >= proposal id holds the value
                // as of proposal creation; none means it never changed since.
                let records = Self::load_epoch_snapshots(env, owner);
                let (mut lo, mut hi) = (0u32, records.len());
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if records.get_unchecked(mid).epoch < proposal.id {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo < records.len() {
                    records.get_unchecked(lo).value
                } else {
                    Self::current_value(env, owner)
                }
            }
            SnapshotMode::Interval => {
                // Latest snapshot strictly before the proposal ledger, so a
                // same-ledger update cannot inflate power.
                let snapshots = Self::load_snapshots(env, owner);
                let mut power = 0;
                for i in (0..snapshots.len()).rev() {
                    let s = snapshots.get_unchecked(i);
                    if s.ledger < proposal.snapshot_ledger {
                        power = s.value;
                        break;
                    }
                }
                power
            }
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), SnapshotError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SnapshotError::NotInitialized)?;
        caller.require_auth();
        if caller != &admin {
            return Err(SnapshotError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
