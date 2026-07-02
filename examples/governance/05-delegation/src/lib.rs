//! # Delegation System
//!
//! Demonstrates advanced delegation patterns for on-chain governance on Soroban.
//!
//! ## Features
//!
//! - **Full delegation**: Delegate all voting power to another address.
//! - **Partial delegation**: Delegate a specific fraction (basis points out of 10 000) of
//!   voting power to another address.
//! - **Topic-specific delegation**: Delegate voting power only for a specific governance
//!   topic (e.g., "treasury", "upgrade", "param").
//! - **Delegation registry**: On-chain lookup of all outgoing and incoming delegations.
//! - **Revocation**: Delegators can revoke any of their delegations at any time.
//! - **Chaining guard**: Prevents chains — a delegate cannot re-delegate received power.
//!
//! ## Design Patterns
//!
//! - `require_auth()` on every mutating call (03-authentication pattern).
//! - `env.events().publish(...)` with consistent `(namespace, action, key)` topics
//!   (04-events pattern).
//! - Instance storage for global config; persistent storage for per-delegation records.
//! - `extend_ttl` on all persistent keys after writes.
//! - `DelegationId` uniquely identifies a delegation as `(delegator, delegate, scope)`.
//!
//! ## Storage Layout
//!
//! | Key | Tier | Content |
//! |-----|------|---------|
//! | `Admin` | Instance | admin `Address` |
//! | `VotingPower(addr)` | Persistent | base voting power `i128` |
//! | `Delegation(DelegationId)` | Persistent | `DelegationRecord` |
//! | `DelegateIncoming(addr)` | Persistent | `Vec<DelegationId>` – delegations *to* addr |
//! | `DelegatorOutgoing(addr)` | Persistent | `Vec<DelegationId>` – delegations *from* addr |
//!
//! ## Auth Model
//!
//! - Setting voting power: admin only.
//! - Delegating: delegator calls, `delegator.require_auth()`.
//! - Revoking: delegator calls, `delegator.require_auth()`.
//! - Queries: permissionless.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Basis-points denominator (100 % = 10 000 bp).
const BP_DENOM: i128 = 10_000;

/// TTL keep-alive values (in ledgers).
const TTL_THRESHOLD: u32 = 17_280; // ~1 day
const TTL_EXTEND: u32 = 120_960; // ~7 days

/// Namespace symbol used as the first event topic.
const NS: Symbol = symbol_short!("deleg");

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DelegationError {
    /// Contract has not been initialized.
    NotInitialized = 1,
    /// Contract already initialized.
    AlreadyInitialized = 2,
    /// Caller is not the admin.
    Unauthorized = 3,
    /// Delegator and delegate must be different addresses.
    SelfDelegation = 4,
    /// Basis-points value is out of the 1–10 000 range.
    InvalidBasisPoints = 5,
    /// The delegation record does not exist.
    DelegationNotFound = 6,
    /// The caller is not the original delegator.
    NotDelegator = 7,
    /// The proposed delegate has already delegated their power to someone else
    /// (chaining is not permitted).
    DelegateeHasOutgoing = 8,
    /// The address has no voting power registered.
    NoVotingPower = 9,
    /// A delegation from this delegator to this delegate for this topic already exists.
    AlreadyDelegated = 10,
    /// The sum of all outgoing partial delegations would exceed 100 % (10 000 bp).
    ExceedsTotalPower = 11,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Scope of a delegation.
///
/// `Global` covers all proposals; `Topic(sym)` covers only proposals tagged
/// with that symbol (e.g., `symbol_short!("treasury")`).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DelegationScope {
    /// Applies to every governance topic.
    Global,
    /// Applies only to proposals tagged with this topic symbol.
    Topic(Symbol),
}

/// Unique composite identifier for a delegation.
///
/// The triple `(delegator, delegate, scope)` is the natural key; storing it
/// as a single struct lets us use it directly as a `DataKey` variant.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationId {
    pub delegator: Address,
    pub delegate: Address,
    pub scope: DelegationScope,
}

/// Full on-chain record of a single delegation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRecord {
    /// Composite key (duplicated here for convenience in query responses).
    pub id: DelegationId,
    /// Fraction of the delegator's voting power transferred, in basis points
    /// (1 bp = 0.01 %).  10 000 bp = full delegation.
    pub basis_points: i128,
    /// Ledger timestamp when the delegation was created.
    pub created_at: u64,
    /// Whether the delegation is currently active.
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Instance: admin address.
    Admin,
    /// Persistent: base voting power for an address.
    VotingPower(Address),
    /// Persistent: delegation record keyed by its composite id.
    Delegation(DelegationId),
    /// Persistent: list of delegation ids pointing *to* an address.
    DelegateIncoming(Address),
    /// Persistent: list of delegation ids pointing *from* an address.
    DelegatorOutgoing(Address),
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const EV_INIT: Symbol = symbol_short!("init");
const EV_SET_VP: Symbol = symbol_short!("set_vp");
const EV_DELEGATE: Symbol = symbol_short!("delegated");
const EV_REVOKE: Symbol = symbol_short!("revoked");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DelegationContract;

#[contractimpl]
impl DelegationContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), DelegationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(DelegationError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.events().publish((NS, EV_INIT), admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Voting power management (admin)
    // -----------------------------------------------------------------------

    /// Assign or update the base voting power for `account` (admin only).
    pub fn set_voting_power(
        env: Env,
        admin: Address,
        account: Address,
        power: i128,
    ) -> Result<(), DelegationError> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        admin.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::VotingPower(account.clone()), &power);
        env.storage().persistent().extend_ttl(
            &DataKey::VotingPower(account.clone()),
            TTL_THRESHOLD,
            TTL_EXTEND,
        );
        env.events().publish((NS, EV_SET_VP, account), power);
        Ok(())
    }

    /// Return the base (undelegated) voting power of `account`.
    pub fn get_voting_power(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::VotingPower(account))
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Delegation
    // -----------------------------------------------------------------------

    /// Delegate `basis_points` (1–10 000) of `delegator`'s voting power to
    /// `delegate`, optionally scoped to a single governance topic.
    ///
    /// - `basis_points = 10_000` → full delegation.
    /// - `basis_points < 10_000` → partial delegation.
    /// - `scope = DelegationScope::Global` → applies to all proposals.
    /// - `scope = DelegationScope::Topic(sym)` → applies only to `sym` proposals.
    ///
    /// Reverts if:
    /// - delegator == delegate (self-delegation)
    /// - basis_points is 0 or > 10 000
    /// - the delegate already has an outgoing delegation (no chaining)
    /// - the same `(delegator, delegate, scope)` triple already exists
    /// - the total outgoing basis points for this delegator would exceed 10 000
    pub fn delegate(
        env: Env,
        delegator: Address,
        delegate: Address,
        basis_points: i128,
        scope: DelegationScope,
    ) -> Result<(), DelegationError> {
        Self::require_initialized(&env)?;
        delegator.require_auth();

        // Guards
        if delegator == delegate {
            return Err(DelegationError::SelfDelegation);
        }
        if basis_points < 1 || basis_points > BP_DENOM {
            return Err(DelegationError::InvalidBasisPoints);
        }

        // Prevent delegation chains: if the delegate has any active outgoing
        // delegations, disallow this delegation.
        let delegate_outgoing: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegatorOutgoing(delegate.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        if !delegate_outgoing.is_empty() {
            return Err(DelegationError::DelegateeHasOutgoing);
        }

        // Build the composite delegation id.
        let id = DelegationId {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            scope: scope.clone(),
        };

        // Reject duplicate (same triple already active).
        if env
            .storage()
            .persistent()
            .has(&DataKey::Delegation(id.clone()))
        {
            return Err(DelegationError::AlreadyDelegated);
        }

        // Ensure total outgoing from delegator won't exceed 100 %.
        let existing_total = Self::total_outgoing_basis_points(&env, &delegator, &scope);
        if existing_total + basis_points > BP_DENOM {
            return Err(DelegationError::ExceedsTotalPower);
        }

        // Write delegation record.
        let record = DelegationRecord {
            id: id.clone(),
            basis_points,
            created_at: env.ledger().timestamp(),
            active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(id.clone()), &record);
        env.storage().persistent().extend_ttl(
            &DataKey::Delegation(id.clone()),
            TTL_THRESHOLD,
            TTL_EXTEND,
        );

        // Update outgoing index for delegator.
        let mut outgoing: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegatorOutgoing(delegator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        outgoing.push_back(id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::DelegatorOutgoing(delegator.clone()), &outgoing);
        env.storage().persistent().extend_ttl(
            &DataKey::DelegatorOutgoing(delegator.clone()),
            TTL_THRESHOLD,
            TTL_EXTEND,
        );

        // Update incoming index for delegate.
        let mut incoming: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegateIncoming(delegate.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        incoming.push_back(id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::DelegateIncoming(delegate.clone()), &incoming);
        env.storage().persistent().extend_ttl(
            &DataKey::DelegateIncoming(delegate.clone()),
            TTL_THRESHOLD,
            TTL_EXTEND,
        );

        env.events()
            .publish((NS, EV_DELEGATE, delegator, delegate), (basis_points, scope));
        Ok(())
    }

    /// Revoke an existing delegation identified by `(delegator, delegate, scope)`.
    ///
    /// Only the original delegator may revoke.  The record is marked inactive
    /// rather than deleted so it is still queryable for audit purposes.
    pub fn revoke(
        env: Env,
        delegator: Address,
        delegate: Address,
        scope: DelegationScope,
    ) -> Result<(), DelegationError> {
        Self::require_initialized(&env)?;
        delegator.require_auth();

        let id = DelegationId {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            scope: scope.clone(),
        };

        let mut record: DelegationRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Delegation(id.clone()))
            .ok_or(DelegationError::DelegationNotFound)?;

        if record.id.delegator != delegator {
            return Err(DelegationError::NotDelegator);
        }

        // Soft-delete: mark inactive for audit trail.
        record.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(id.clone()), &record);
        env.storage().persistent().extend_ttl(
            &DataKey::Delegation(id.clone()),
            TTL_THRESHOLD,
            TTL_EXTEND,
        );

        // Remove from outgoing index.
        Self::remove_from_index(&env, &DataKey::DelegatorOutgoing(delegator.clone()), &id);

        // Remove from incoming index.
        Self::remove_from_index(&env, &DataKey::DelegateIncoming(delegate.clone()), &id);

        env.events()
            .publish((NS, EV_REVOKE, delegator, delegate), scope);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries — delegation registry
    // -----------------------------------------------------------------------

    /// Fetch a single delegation record by composite key.
    pub fn get_delegation(
        env: Env,
        delegator: Address,
        delegate: Address,
        scope: DelegationScope,
    ) -> Result<DelegationRecord, DelegationError> {
        let id = DelegationId {
            delegator,
            delegate,
            scope,
        };
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(id))
            .ok_or(DelegationError::DelegationNotFound)
    }

    /// Return all active delegation ids going *out* from `delegator`.
    pub fn get_outgoing(env: Env, delegator: Address) -> Vec<DelegationId> {
        env.storage()
            .persistent()
            .get(&DataKey::DelegatorOutgoing(delegator))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return all active delegation ids coming *in* to `delegate`.
    pub fn get_incoming(env: Env, delegate: Address) -> Vec<DelegationId> {
        env.storage()
            .persistent()
            .get(&DataKey::DelegateIncoming(delegate))
            .unwrap_or_else(|| Vec::new(&env))
    }

    // -----------------------------------------------------------------------
    // Effective voting-power calculation
    // -----------------------------------------------------------------------

    /// Compute the effective voting power of `account` for a given `topic`.
    ///
    /// ```text
    /// effective = retained_own + sum(received_from_delegators)
    ///
    /// retained_own  = base_power - sum(base_power * bp / 10_000)
    ///                 for every outgoing delegation whose scope covers `topic`
    ///
    /// received      = sum(delegator_base * bp / 10_000)
    ///                 for every incoming delegation whose scope covers `topic`
    /// ```
    ///
    /// A delegation's scope *covers* a topic when it is `Global` or when it
    /// is `Topic(t)` and `t == topic`.
    pub fn effective_power(env: Env, account: Address, topic: Symbol) -> i128 {
        let base: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::VotingPower(account.clone()))
            .unwrap_or(0);

        // Deduct outgoing delegations that apply to this topic.
        let outgoing: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegatorOutgoing(account.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut delegated_out: i128 = 0;
        for id in outgoing.iter() {
            if scope_covers(&id.scope, &topic) {
                delegated_out += base * active_bp(&env, &id) / BP_DENOM;
            }
        }
        let retained = base - delegated_out;

        // Add incoming delegations that apply to this topic.
        let incoming: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegateIncoming(account.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut received: i128 = 0;
        for id in incoming.iter() {
            if scope_covers(&id.scope, &topic) {
                let delegator_base: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::VotingPower(id.delegator.clone()))
                    .unwrap_or(0);
                received += delegator_base * active_bp(&env, &id) / BP_DENOM;
            }
        }

        retained + received
    }

    /// Return the admin address (`None` before initialization).
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), DelegationError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(DelegationError::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), DelegationError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DelegationError::NotInitialized)?;
        if caller != &admin {
            return Err(DelegationError::Unauthorized);
        }
        Ok(())
    }

    /// Sum of all active outgoing basis-points from `delegator` whose scope
    /// overlaps with `new_scope`.
    ///
    /// Two scopes overlap when either is `Global`, or both are `Topic(t)` with
    /// the same `t`.
    fn total_outgoing_basis_points(
        env: &Env,
        delegator: &Address,
        new_scope: &DelegationScope,
    ) -> i128 {
        let outgoing: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegatorOutgoing(delegator.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut total: i128 = 0;
        for id in outgoing.iter() {
            let overlaps = match (new_scope, &id.scope) {
                (_, DelegationScope::Global) => true,
                (DelegationScope::Global, _) => true,
                (DelegationScope::Topic(a), DelegationScope::Topic(b)) => *a == *b,
            };
            if overlaps {
                total += active_bp(env, &id);
            }
        }
        total
    }

    /// Remove `target` from the `Vec<DelegationId>` stored under `key`.
    fn remove_from_index(env: &Env, key: &DataKey, target: &DelegationId) {
        let current: Vec<DelegationId> = env
            .storage()
            .persistent()
            .get(key)
            .unwrap_or_else(|| Vec::new(env));

        let mut updated: Vec<DelegationId> = Vec::new(env);
        for id in current.iter() {
            if id != *target {
                updated.push_back(id);
            }
        }
        env.storage().persistent().set(key, &updated);
        env.storage()
            .persistent()
            .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND);
    }
}

// ---------------------------------------------------------------------------
// Module-level helpers (outside impl so they work for both contract and tests)
// ---------------------------------------------------------------------------

/// Returns `true` when `scope` covers the given `topic`.
///
/// `Global` covers every topic; `Topic(t)` covers only `t`.
fn scope_covers(scope: &DelegationScope, topic: &Symbol) -> bool {
    match scope {
        DelegationScope::Global => true,
        DelegationScope::Topic(t) => t == topic,
    }
}

/// Look up the basis-points value for a delegation id.
/// Returns 0 if the record is missing or marked inactive.
fn active_bp(env: &Env, id: &DelegationId) -> i128 {
    let record: Option<DelegationRecord> = env
        .storage()
        .persistent()
        .get(&DataKey::Delegation(id.clone()));
    match record {
        Some(r) if r.active => r.basis_points,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test;
