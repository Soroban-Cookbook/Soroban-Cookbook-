#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol,
};

/// Minimum delay (in seconds) that must pass before execution
const MIN_DELAY: u64 = 60;
/// Maximum delay (in seconds) allowed when queuing
const MAX_DELAY: u64 = 86_400; // 24 hours

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Payload for an admin-action event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActionEventData {
    /// Identifier of the specific action performed.
    pub action: Symbol,
    /// Timestamp when the action was executed.
    pub timestamp: u64,
}

/// Payload for an audit-trail event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditTrailEventData {
    /// Free-form description or reference tag.
    pub details: Symbol,
    /// Ledger timestamp at emission time.
    pub timestamp: u64,
}

/// Namespace symbol used as the first topic of every event this contract emits.
const CONTRACT_NS: Symbol = symbol_short!("timelock");
/// Naming convention: `snake_case` action names in topic[1].
const ACTION_ADMIN: Symbol = symbol_short!("admin");
const ACTION_AUDIT: Symbol = symbol_short!("audit");

#[contracttype]
pub enum DataKey {
    /// Maps operation_id -> scheduled execution timestamp
    Operation(Bytes),
    /// The admin who can queue/cancel/execute
    Admin,
}

/// Possible states of an operation
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum OperationState {
    /// Not found in storage
    Unknown,
    /// Queued, waiting for delay to pass
    Pending,
    /// Delay has passed, ready to execute
    Ready,
    /// Already executed (removed from storage)
    Done,
}

#[contract]
pub struct TimelockContract;

#[contractimpl]
impl TimelockContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Audit trail for initialization
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, admin),
            AuditTrailEventData {
                details: symbol_short!("init"),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Queue an operation for delayed execution.
    ///
    /// - `operation_id`: unique identifier for this operation (caller-defined bytes)
    /// - `delay`:        seconds from now before the operation can be executed (MIN_DELAY..=MAX_DELAY)
    ///
    /// Emits a `queued` event on success.
    pub fn queue(env: Env, operation_id: Bytes, delay: u64) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        if !(MIN_DELAY..=MAX_DELAY).contains(&delay) {
            panic!("Delay out of range");
        }

        let key = DataKey::Operation(operation_id.clone());
        if env.storage().persistent().has(&key) {
            panic!("Operation already queued");
        }

        let execute_at = env.ledger().timestamp() + delay;
        env.storage().persistent().set(&key, &execute_at);
        // Keep the operation alive well beyond MAX_DELAY (7 days >> 24 h).
        // Without this, the entry could expire before execution time.
        env.storage().persistent().extend_ttl(&key, 17_280, 120_960);

        // Consistent event emission
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, operation_id),
            AdminActionEventData {
                action: symbol_short!("queue"),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Execute a queued operation after its delay has passed.
    ///
    /// Removes the operation from storage (marking it done).
    /// Emits an `executed` event on success.
    pub fn execute(env: Env, operation_id: Bytes) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let key = DataKey::Operation(operation_id.clone());
        let execute_at: u64 = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Operation not found");

        let now = env.ledger().timestamp();
        if now < execute_at {
            panic!("Too early");
        }

        // Remove so it cannot be replayed
        env.storage().persistent().remove(&key);

        // Consistent event emission
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, operation_id),
            AdminActionEventData {
                action: symbol_short!("execute"),
                timestamp: now,
            },
        );
    }

    /// Cancel a queued operation before it is executed.
    ///
    /// Emits a `cancelled` event on success.
    pub fn cancel(env: Env, operation_id: Bytes) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let key = DataKey::Operation(operation_id.clone());
        if !env.storage().persistent().has(&key) {
            panic!("Operation not found");
        }

        env.storage().persistent().remove(&key);

        // Consistent event emission
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin, operation_id),
            AdminActionEventData {
                action: symbol_short!("cancel"),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Return the scheduled execution timestamp for an operation, or 0 if not queued.
    pub fn get_execute_at(env: Env, operation_id: Bytes) -> u64 {
        let key = DataKey::Operation(operation_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Return the current state of an operation.
    pub fn get_state(env: Env, operation_id: Bytes) -> OperationState {
        let key = DataKey::Operation(operation_id);
        match env.storage().persistent().get::<DataKey, u64>(&key) {
            None => OperationState::Unknown,
            Some(execute_at) => {
                if env.ledger().timestamp() < execute_at {
                    OperationState::Pending
                } else {
                    OperationState::Ready
                }
            }
        }
    }
}

#[cfg(test)]
mod test;
