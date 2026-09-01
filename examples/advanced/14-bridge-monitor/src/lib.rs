//! # 14 Bridge Monitor
//!
//! A Soroban contract that observes a cross-chain bridge: it logs bridged
//! transactions (direction, amount, token, chains, status), tracks balance
//! drift against a configurable threshold, and maintains a resolvable alert
//! queue — the on-chain half of an off-chain monitoring stack (alert fan-out,
//! dashboards, incident response).
//!
//! The contract holds no funds and does not move tokens; it is a read-only
//! auditor + alert ledger. An off-chain relayer/indexer calls
//! `record_transaction` for every bridge event it observes and calls
//! `snapshot_balance` with the on-chain vault/token balance it sees, then a
//! watcher reads `list_alerts` and pages through `transactions`.

#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A bridged transfer observed by the monitor.
#[contracttype]
#[derive(Clone)]
pub struct BridgeTransaction {
    pub tx_id: String,
    /// "inbound" (to this chain) or "outbound" (from this chain).
    pub direction: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: i128,
    pub token: Address,
    pub status: String,
    pub observed_at: u64,
}

/// A raised alert (balance drift, failed transfer, anomaly).
#[contracttype]
#[derive(Clone)]
pub struct Alert {
    pub id: u32,
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub raised_at: u64,
    pub acknowledged: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Token,
    Threshold,
    Transactions,
    Alerts,
    AlertCount,
    TransactionCount,
    LastSnapshot,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BridgeMonitorError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    EmptyTxId = 4,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct BridgeMonitor;

#[contractimpl]
impl BridgeMonitor {
    /// Initialize with an admin and the balance-drift alert threshold.
    pub fn initialize(env: Env, admin: Address, threshold: i128) -> Result<(), BridgeMonitorError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(BridgeMonitorError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::TransactionCount, &0u32);
        env.storage().instance().set(&DataKey::AlertCount, &0u32);
        Ok(())
    }

    pub fn initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), BridgeMonitorError> {
        if !env.storage().instance().get::<_, bool>(&DataKey::Initialized).unwrap_or(false) {
            return Err(BridgeMonitorError::NotInitialized);
        }
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(BridgeMonitorError::NotInitialized)?;
        // App-level check: only the stored admin may perform admin operations,
        // regardless of the auth-mock environment (issue #765).
        if caller != &admin {
            return Err(BridgeMonitorError::Unauthorized);
        }
        caller.require_auth();
        Ok(())
    }

    /// The token monitored for balance drift.
    pub fn token(env: Env) -> Result<Address, BridgeMonitorError> {
        env.storage()
            .instance()
            .get::<_, Address>(&DataKey::Token)
            .ok_or(BridgeMonitorError::NotInitialized)
    }

    pub fn set_token(env: Env, admin: Address, token: Address) -> Result<(), BridgeMonitorError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Token, &token);
        Ok(())
    }

    /// Change the balance-drift alert threshold.
    pub fn set_threshold(env: Env, admin: Address, threshold: i128) -> Result<(), BridgeMonitorError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        Ok(())
    }

    /// Record a bridge transfer observed by the trusted indexer.
    /// Returns the transaction sequence number (1-based).
    pub fn record_transaction(
        env: Env,
        indexer: Address,
        tx_id: String,
        direction: String,
        from_chain: String,
        to_chain: String,
        amount: i128,
        token: Address,
        status: String,
    ) -> Result<u32, BridgeMonitorError> {
        Self::require_admin(&env, &indexer)?;
        if tx_id.is_empty() {
            return Err(BridgeMonitorError::EmptyTxId);
        }

        let count = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::TransactionCount)
            .unwrap_or(0);
        let next = count.saturating_add(1);
        let tx = BridgeTransaction {
            tx_id,
            direction: direction.clone(),
            from_chain,
            to_chain,
            amount,
            token: token.clone(),
            status: status.clone(),
            observed_at: env.ledger().timestamp(),
        };
        let mut txs: Vec<BridgeTransaction> = env
            .storage()
            .persistent()
            .get(&DataKey::Transactions)
            .unwrap_or_else(|| Vec::new(&env));
        txs.push_back(tx);
        env.storage().persistent().set(&DataKey::Transactions, &txs);
        env.storage().instance().set(&DataKey::TransactionCount, &next);

        env.events().publish(
            (symbol_short!("TX"), &token),
            (next, &direction, &amount, &status),
        );

        // Failed transfers are anomalies worth surfacing immediately.
        if status.eq(&String::from_str(&env, "failed")) {
            Self::raise_alert_internal(&env, "high", "failed_transfer", String::from_str(&env, &format!("bridge tx {next} failed")));
        }

        Ok(next)
    }

    /// Page through recorded transactions (1-based start, bounded count).
    pub fn transactions(env: Env, start: u32, limit: u32) -> Vec<BridgeTransaction> {
        let txs: Vec<BridgeTransaction> = env
            .storage()
            .persistent()
            .get(&DataKey::Transactions)
            .unwrap_or_else(|| Vec::new(&env));
        let mut out: Vec<BridgeTransaction> = Vec::new(&env);
        let mut skipped: u32 = 0;
        for tx in txs.iter() {
            if out.len() >= limit {
                break;
            }
            if skipped < start.saturating_sub(1) {
                skipped += 1;
                continue;
            }
            out.push_back(tx);
        }
        out
    }

    /// Total transactions recorded.
    pub fn transaction_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, u32>(&DataKey::TransactionCount)
            .unwrap_or(0)
    }

    /// Record the latest observed on-chain balance; raises a drift alert when
    /// it moves by more than the threshold vs. the last snapshot.
    pub fn snapshot_balance(env: Env, indexer: Address, observed: i128) -> Result<(), BridgeMonitorError> {
        Self::require_admin(&env, &indexer)?;
        let previous = env.storage().instance().get::<_, i128>(&DataKey::LastSnapshot);
        match previous {
            Some(prev) => {
                let threshold = env
                    .storage()
                    .instance()
                    .get::<_, i128>(&DataKey::Threshold)
                    .unwrap_or(i128::MAX);
                let drift = observed.saturating_sub(prev);
                if drift.abs() > threshold {
                    Self::raise_alert_internal(
                        &env,
                        "high",
                        "balance_drift",
                        String::from_str(&env, &format!("balance moved by {drift} (threshold {threshold})")),
                    );
                }
                env.storage().instance().set(&DataKey::LastSnapshot, &observed);
            }
            None => {
                env.storage().instance().set(&DataKey::LastSnapshot, &observed);
            }
        }
        Ok(())
    }

    /// The most recent balance snapshot (if any).
    pub fn last_snapshot(env: Env) -> Option<i128> {
        env.storage().instance().get(&DataKey::LastSnapshot)
    }

    /// All pending (unacknowledged) alerts, newest first order preserved.
    pub fn list_alerts(env: Env) -> Vec<Alert> {
        env.storage()
            .persistent()
            .get::<_, Vec<Alert>>(&DataKey::Alerts)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn alert_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, u32>(&DataKey::AlertCount)
            .unwrap_or(0)
    }

    /// Acknowledge and remove an alert by its id.
    pub fn resolve_alert(env: Env, admin: Address, id: u32) -> Result<(), BridgeMonitorError> {
        Self::require_admin(&env, &admin)?;
        let alerts: Vec<Alert> = env
            .storage()
            .persistent()
            .get(&DataKey::Alerts)
            .unwrap_or_else(|| Vec::new(&env));
        let mut remaining: Vec<Alert> = Vec::new(&env);
        for alert in alerts.iter() {
            if alert.id != id {
                remaining.push_back(alert);
            }
        }
        env.storage().persistent().set(&DataKey::Alerts, &remaining);
        env.events().publish(
            (symbol_short!("RESOLVE"),),
            id,
        );
        Ok(())
    }

    fn raise_alert_internal(env: &Env, severity: &str, kind: &str, message: String) {
        let count = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::AlertCount)
            .unwrap_or(0);
        let next_id = count.saturating_add(1);
        let alert = Alert {
            id: next_id,
            severity: String::from_str(env, severity),
            kind: String::from_str(env, kind),
            message,
            raised_at: env.ledger().timestamp(),
            acknowledged: false,
        };
        let mut alerts: Vec<Alert> = env
            .storage()
            .persistent()
            .get(&DataKey::Alerts)
            .unwrap_or_else(|| Vec::new(&env));
        alerts.push_back(alert);
        env.storage().persistent().set(&DataKey::Alerts, &alerts);
        env.storage().instance().set(&DataKey::AlertCount, &next_id);
    }
}

#[cfg(test)]
mod test;
