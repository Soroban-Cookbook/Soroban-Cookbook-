//! # Generic Oracle Integration Pattern
//!
//! This example demonstrates a robust, generic **request/response oracle integration pattern** on Soroban.
//! It showcases how a smart contract can request external data and securely handle the asynchronous callback
//! from an authorized oracle updater.
//!
//! ## Workflow
//! 1. **Request**: The consumer contract invokes `OracleContract::request_data`, registering a unique callback function and query (e.g. asset pair).
//! 2. **Verification**: The Oracle contract allocates an auto-incrementing `request_id`, stores request metadata, and emits a `requested` event.
//! 3. **Off-chain Action**: An off-chain oracle operator listens to the event, fetches the required data, and submits it to `OracleContract::fulfill_request`.
//! 4. **Validation**: The Oracle contract ensures that the submitter is authorized, the request is active (not expired), the timestamp is valid (fresh and not in the future), and the value is positive.
//! 5. **Callback**: The Oracle contract dynamically invokes the callback function on the registered consumer address.
//! 6. **Consumer validation**: The consumer contract validates that the caller of its callback function is indeed the registered Oracle contract using `oracle_address.require_auth()`, processes/stores the data, and marks the request as processed.
//!
//! ## Security Considerations
//! - **Callback Authentication**: Consumers **must** call `oracle.require_auth()` in their callback function to prevent unauthorized parties from feeding rogue data.
//! - **Replay Protection**: The Oracle contract marks the request as `Fulfilled` upon execution, ensuring requests cannot be re-fulfilled.
//! - **Request Expiration**: Requests have a configured timeout, after which they are marked `Expired` and cannot be fulfilled.
//! - **Atomicity**: The callback call is synchronous and atomic. If the consumer callback panics, the entire fulfillment transaction reverts.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address, Env, Symbol, IntoVal
};

// ---------------------------------------------------------------------------
// Error Types
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not authorized.
    NotAuthorized = 3,
    /// The request ID was not found or is not in a pending state.
    RequestNotFound = 4,
    /// The request has timed out and expired.
    RequestExpired = 5,
    /// The submitted data timestamp is invalid (in the future or too stale).
    InvalidTimestamp = 6,
    /// The submitted value is invalid (e.g. negative or zero).
    InvalidValue = 7,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleRequestEvent {
    pub request_id: u32,
    pub consumer: Address,
    pub query: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleFulfillEvent {
    pub request_id: u32,
    pub value: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerUpdateEvent {
    pub request_id: u32,
    pub query: Symbol,
    pub value: i128,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Structs and State Representation
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RequestStatus {
    Pending = 0,
    Fulfilled = 1,
    Expired = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleRequest {
    /// Unique identifier for this request.
    pub id: u32,
    /// Address of the contract expecting the callback.
    pub consumer: Address,
    /// Function name of the callback on the consumer contract.
    pub callback_fn: Symbol,
    /// Specific query or asset pair requested (e.g. XLM/USD).
    pub query: Symbol,
    /// Ledger timestamp when this request was created.
    pub request_timestamp: u64,
    /// Current status of this request.
    pub status: RequestStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address authorized to rotate the updater.
    Admin,
    /// Authorized oracle operator/updater address who submits data.
    Updater,
    /// Maximum age (seconds) allowed for data timestamps.
    MaxAge,
    /// Maximum time (seconds) a request can remain pending before expiring.
    RequestTimeout,
    /// Auto-incrementing request ID counter.
    NextRequestId,
    /// Specific request details mapping: `Request(id) -> OracleRequest`
    Request(u32),
}

// ---------------------------------------------------------------------------
// Oracle Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle contract with configurations.
    pub fn initialize(
        env: Env,
        admin: Address,
        updater: Address,
        max_age: u64,
        request_timeout: u64,
    ) -> Result<(), OracleError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Updater, &updater);
        env.storage().instance().set(&DataKey::MaxAge, &max_age);
        env.storage().instance().set(&DataKey::RequestTimeout, &request_timeout);
        env.storage().instance().set(&DataKey::NextRequestId, &1u32);
        Ok(())
    }

    /// Request external data.
    ///
    /// The caller must authorize this request (`consumer.require_auth()`).
    /// Returns the assigned request ID.
    pub fn request_data(
        env: Env,
        consumer: Address,
        callback_fn: Symbol,
        query: Symbol,
    ) -> Result<u32, OracleError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::NotInitialized);
        }
        consumer.require_auth();

        let request_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::NextRequestId)
            .unwrap_or(1);
        env.storage().instance().set(&DataKey::NextRequestId, &(request_id + 1));

        let request = OracleRequest {
            id: request_id,
            consumer: consumer.clone(),
            callback_fn,
            query: query.clone(),
            request_timestamp: env.ledger().timestamp(),
            status: RequestStatus::Pending,
        };

        // Save to persistent storage since requests can bridge multiple ledgers.
        env.storage().persistent().set(&DataKey::Request(request_id), &request);

        // Emit requested event via modern event publish syntax
        OracleRequestEvent {
            request_id,
            consumer,
            query,
        }
        .publish(&env);

        Ok(request_id)
    }

    /// Fulfill a pending request. Only the authorized updater may call this.
    ///
    /// Validates the data, updates the request status, and triggers the callback.
    pub fn fulfill_request(
        env: Env,
        updater: Address,
        request_id: u32,
        value: i128,
        timestamp: u64,
    ) -> Result<(), OracleError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(OracleError::NotInitialized);
        }

        // 1. Authenticate updater
        let stored_updater: Address = env
            .storage()
            .instance()
            .get(&DataKey::Updater)
            .ok_or(OracleError::NotInitialized)?;
        if updater != stored_updater {
            return Err(OracleError::NotAuthorized);
        }
        updater.require_auth();

        // 2. Load the request
        let mut request: OracleRequest = env
            .storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .ok_or(OracleError::RequestNotFound)?;

        if request.status != RequestStatus::Pending {
            return Err(OracleError::RequestNotFound);
        }

        let now = env.ledger().timestamp();
        let timeout: u64 = env.storage().instance().get(&DataKey::RequestTimeout).unwrap();

        // 3. Expiration Check
        if now.saturating_sub(request.request_timestamp) > timeout {
            request.status = RequestStatus::Expired;
            env.storage().persistent().set(&DataKey::Request(request_id), &request);
            return Err(OracleError::RequestExpired);
        }

        // 4. Value Check (e.g., prices must be positive)
        if value <= 0 {
            return Err(OracleError::InvalidValue);
        }

        // 5. Data Freshness and Timestamp Checks
        if timestamp > now {
            return Err(OracleError::InvalidTimestamp);
        }
        let max_age: u64 = env.storage().instance().get(&DataKey::MaxAge).unwrap();
        if now.saturating_sub(timestamp) > max_age {
            return Err(OracleError::InvalidTimestamp);
        }

        // Ensure data is not older than the request creation time
        if timestamp < request.request_timestamp {
            return Err(OracleError::InvalidTimestamp);
        }

        // 6. Update request status to avoid reentrancy/replay
        request.status = RequestStatus::Fulfilled;
        env.storage().persistent().set(&DataKey::Request(request_id), &request);

        // 7. Invoke consumer callback synchronously
        env.invoke_contract::<()>(
            &request.consumer,
            &request.callback_fn,
            (request_id, value, timestamp).into_val(&env),
        );

        // Emit event via modern event publish syntax
        OracleFulfillEvent {
            request_id,
            value,
            timestamp,
        }
        .publish(&env);

        Ok(())
    }

    /// Read request details.
    pub fn get_request(env: Env, request_id: u32) -> Option<OracleRequest> {
        env.storage().persistent().get(&DataKey::Request(request_id))
    }

    /// Rotate updater address. Admin only.
    pub fn set_updater(env: Env, new_updater: Address) -> Result<(), OracleError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OracleError::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Updater, &new_updater);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Consumer Contract (Demonstrating Oracle Integration)
// ---------------------------------------------------------------------------

#[contract]
pub struct ConsumerContract;

#[contracttype]
#[derive(Clone)]
pub enum ConsumerDataKey {
    /// Registered oracle contract address
    Oracle,
    /// Store the last valid price for an asset query
    LastPrice(Symbol),
    /// Store the timestamp of the last valid price
    LastTimestamp(Symbol),
    /// Store active request IDs mapping: `PendingRequest(id) -> QuerySymbol`
    PendingRequest(u32),
}

#[contractimpl]
impl ConsumerContract {
    /// Initialize the consumer with the target Oracle contract.
    pub fn initialize(env: Env, oracle: Address) {
        if env.storage().instance().has(&ConsumerDataKey::Oracle) {
            panic!("already initialized");
        }
        env.storage().instance().set(&ConsumerDataKey::Oracle, &oracle);
    }

    /// Initiate a price request for a given asset (e.g. XLM/USD).
    pub fn request_price(env: Env, query: Symbol) -> u32 {
        let oracle: Address = env.storage().instance().get(&ConsumerDataKey::Oracle).unwrap();
        let oracle_client = OracleContractClient::new(&env, &oracle);

        let self_addr = env.current_contract_address();
        // Invoke the Oracle to request data, setting `callback` as the callback function name
        let request_id = oracle_client.request_data(&self_addr, &symbol_short!("callback"), &query);

        // Store the request metadata so we can map the incoming request ID back to the requested asset
        env.storage().persistent().set(&ConsumerDataKey::PendingRequest(request_id), &query);

        request_id
    }

    /// Callback function executed by the Oracle upon request fulfillment.
    ///
    /// # Security Checks
    /// 1. **Caller Verification**: Employs `oracle.require_auth()` to ensure only the trusted Oracle contract can invoke this.
    /// 2. **Request State**: Ensures the request ID corresponds to a registered active query, preventing arbitrary unsolicited data updates.
    pub fn callback(env: Env, request_id: u32, value: i128, timestamp: u64) {
        let oracle: Address = env.storage().instance().get(&ConsumerDataKey::Oracle).unwrap();
        // Secure Callback check: require authorization from the oracle contract address
        oracle.require_auth();

        // Load and validate that the request is expected
        let query: Symbol = env
            .storage()
            .persistent()
            .get(&ConsumerDataKey::PendingRequest(request_id))
            .expect("unrecognized or already processed request");

        // Clear the pending request to prevent replay attacks on this callback handler
        env.storage().persistent().remove(&ConsumerDataKey::PendingRequest(request_id));

        // Update consumer price feed state
        env.storage().instance().set(&ConsumerDataKey::LastPrice(query.clone()), &value);
        env.storage().instance().set(&ConsumerDataKey::LastTimestamp(query.clone()), &timestamp);

        // Emit consumer update event via modern event publish syntax
        ConsumerUpdateEvent {
            request_id,
            query,
            value,
            timestamp,
        }
        .publish(&env);
    }

    /// Read the latest price for an asset query.
    pub fn get_price(env: Env, query: Symbol) -> Option<i128> {
        env.storage().instance().get(&ConsumerDataKey::LastPrice(query))
    }

    /// Read the latest timestamp for an asset query.
    pub fn get_timestamp(env: Env, query: Symbol) -> Option<u64> {
        env.storage().instance().get(&ConsumerDataKey::LastTimestamp(query))
    }
}

// Client generation and test linkage
#[cfg(test)]
mod test;
