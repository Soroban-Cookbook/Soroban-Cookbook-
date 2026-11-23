#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Default threshold for auto-triggering a pause after consecutive failures.
const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
/// Default number of ledgers that a pause stays active before auto-recovery.
const DEFAULT_RECOVERY_WINDOW: u64 = 100;

/// Circuit states.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CircuitState {
    Active = 0,
    Paused = 1,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CircuitError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    CircuitPaused = 4,
    InvalidThreshold = 5,
    InvalidRecoveryWindow = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    State,
    FailureCount,
    FailureThreshold,
    RecoveryWindow,
    LastFailureTimestamp,
}

const CONTRACT_NS: Symbol = symbol_short!("circuit");
const ACTION_ADMIN: Symbol = symbol_short!("admin");
const ACTION_AUDIT: Symbol = symbol_short!("audit");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActionEventData {
    pub action: Symbol,
    pub timestamp: u64,
}

#[contract]
pub struct CircuitBreakerContract;

#[contractimpl]
impl CircuitBreakerContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), CircuitError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(CircuitError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::State, &CircuitState::Active);
        env.storage().instance().set(&DataKey::FailureCount, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::FailureThreshold, &DEFAULT_FAILURE_THRESHOLD);
        env.storage()
            .instance()
            .set(&DataKey::RecoveryWindow, &DEFAULT_RECOVERY_WINDOW);
        env.storage()
            .instance()
            .set(&DataKey::LastFailureTimestamp, &0u64);

        Ok(())
    }

    #[allow(deprecated)]
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), CircuitError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(CircuitError::NotInitialized)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin),
            AdminActionEventData {
                action: symbol_short!("set_admin"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    #[allow(deprecated)]
    pub fn set_pause(env: Env, paused: bool) -> Result<(), CircuitError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(CircuitError::NotInitialized)?;
        admin.require_auth();

        let state = if paused {
            CircuitState::Paused
        } else {
            CircuitState::Active
        };
        env.storage().instance().set(&DataKey::State, &state);
        env.events().publish(
            (CONTRACT_NS, ACTION_AUDIT, admin),
            AdminActionEventData {
                action: if paused {
                    symbol_short!("pause")
                } else {
                    symbol_short!("resume")
                },
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    pub fn configure(env: Env, threshold: u32, recovery_window: u64) -> Result<(), CircuitError> {
        if threshold == 0 {
            return Err(CircuitError::InvalidThreshold);
        }
        if recovery_window == 0 {
            return Err(CircuitError::InvalidRecoveryWindow);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(CircuitError::NotInitialized)?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::FailureThreshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::RecoveryWindow, &recovery_window);

        Ok(())
    }

    pub fn execute(env: Env, caller: Address) -> Result<(), CircuitError> {
        caller.require_auth();
        Self::ensure_active(&env)?;
        Self::record_success(&env)?;
        Ok(())
    }

    pub fn fail(env: Env, caller: Address) -> Result<(), CircuitError> {
        caller.require_auth();
        Self::ensure_active(&env)?;
        Self::record_failure(&env)?;
        Ok(())
    }

    pub fn get_state(env: Env) -> CircuitState {
        env.storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(CircuitState::Active)
    }

    pub fn get_failure_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::FailureCount)
            .unwrap_or(0)
    }

    pub fn get_failure_threshold(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::FailureThreshold)
            .unwrap_or(DEFAULT_FAILURE_THRESHOLD)
    }

    pub fn get_recovery_window(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RecoveryWindow)
            .unwrap_or(DEFAULT_RECOVERY_WINDOW)
    }

    pub fn get_last_failure_timestamp(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastFailureTimestamp)
            .unwrap_or(0)
    }

    fn ensure_active(env: &Env) -> Result<(), CircuitError> {
        let recovery_window: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RecoveryWindow)
            .unwrap_or(DEFAULT_RECOVERY_WINDOW);
        let last_failure: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LastFailureTimestamp)
            .unwrap_or(0);
        let now = env.ledger().timestamp();
        let mut state: CircuitState = env
            .storage()
            .instance()
            .get(&DataKey::State)
            .unwrap_or(CircuitState::Active);

        if state == CircuitState::Paused
            && last_failure > 0
            && now >= last_failure + recovery_window
        {
            state = CircuitState::Active;
            env.storage().instance().set(&DataKey::State, &state);
            env.storage().instance().set(&DataKey::FailureCount, &0u32);
            env.storage()
                .instance()
                .set(&DataKey::LastFailureTimestamp, &0u64);
        }

        if state != CircuitState::Active {
            return Err(CircuitError::CircuitPaused);
        }

        Ok(())
    }

    fn record_failure(env: &Env) -> Result<(), CircuitError> {
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FailureThreshold)
            .unwrap_or(DEFAULT_FAILURE_THRESHOLD);
        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FailureCount)
            .unwrap_or(0);
        count += 1;
        let now = env.ledger().timestamp();
        let last_failure_ts = if now == 0 { 1 } else { now };
        env.storage().instance().set(&DataKey::FailureCount, &count);
        env.storage()
            .instance()
            .set(&DataKey::LastFailureTimestamp, &last_failure_ts);

        if count >= threshold {
            env.storage()
                .instance()
                .set(&DataKey::State, &CircuitState::Paused);
        }

        Ok(())
    }

    fn record_success(env: &Env) -> Result<(), CircuitError> {
        env.storage().instance().set(&DataKey::FailureCount, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::LastFailureTimestamp, &0u64);
        Ok(())
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod test;
