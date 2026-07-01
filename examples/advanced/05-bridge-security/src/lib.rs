#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol,
};

const BRIDGE_NS: Symbol = symbol_short!("bridge");
const ACTION_SUBMIT: Symbol = symbol_short!("submit");
const ACTION_CHALLENGE: Symbol = symbol_short!("challenge");
const ACTION_FINALIZE: Symbol = symbol_short!("finalize");
const ACTION_FRAUD: Symbol = symbol_short!("fraud");
const ACTION_PAUSE: Symbol = symbol_short!("pause");
const ACTION_UNPAUSE: Symbol = symbol_short!("unpause");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BridgeError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidConfig = 4,
    InvalidAmount = 5,
    ContractPaused = 6,
    RateLimitExceeded = 7,
    TransferNotFound = 8,
    ChallengeWindowOpen = 9,
    ChallengeWindowClosed = 10,
    TransferAlreadyResolved = 11,
    TransferChallenged = 12,
    InvalidTransferState = 13,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    RateLimitAmount,
    RateLimitWindow,
    ChallengePeriod,
    WindowStart,
    WindowUsed,
    NextTransferId,
    Transfer(u64),
    FraudProof(u64),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferStatus {
    Pending,
    Challenged,
    Finalized,
    Fraudulent,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeTransfer {
    pub operator: Address,
    pub recipient: Address,
    pub amount: i128,
    pub source_chain: u32,
    pub evidence_hash: Bytes,
    pub submitted_at: u64,
    pub status: TransferStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitState {
    pub amount_limit: i128,
    pub window_seconds: u64,
    pub window_start: u64,
    pub used_in_window: i128,
}

#[contract]
pub struct BridgeSecurityContract;

#[contractimpl]
impl BridgeSecurityContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        rate_limit_amount: i128,
        rate_limit_window: u64,
        challenge_period: u64,
    ) -> Result<(), BridgeError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BridgeError::AlreadyInitialized);
        }
        if rate_limit_amount <= 0 || rate_limit_window == 0 || challenge_period == 0 {
            return Err(BridgeError::InvalidConfig);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::RateLimitAmount, &rate_limit_amount);
        env.storage()
            .instance()
            .set(&DataKey::RateLimitWindow, &rate_limit_window);
        env.storage()
            .instance()
            .set(&DataKey::ChallengePeriod, &challenge_period);
        env.storage()
            .instance()
            .set(&DataKey::WindowStart, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::WindowUsed, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::NextTransferId, &1u64);

        Ok(())
    }

    pub fn submit_transfer(
        env: Env,
        operator: Address,
        recipient: Address,
        amount: i128,
        source_chain: u32,
        evidence_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        operator.require_auth();
        ensure_initialized(&env)?;
        require_not_paused(&env)?;
        require_positive_amount(amount)?;

        let now = env.ledger().timestamp();
        let mut rate_limit = read_rate_limit_state(&env)?;
        if now
            >= rate_limit
                .window_start
                .saturating_add(rate_limit.window_seconds)
        {
            rate_limit.window_start = now;
            rate_limit.used_in_window = 0;
        }

        let new_used = rate_limit
            .used_in_window
            .checked_add(amount)
            .ok_or(BridgeError::InvalidAmount)?;
        if new_used > rate_limit.amount_limit {
            return Err(BridgeError::RateLimitExceeded);
        }

        let transfer_id = read_next_transfer_id(&env);
        let transfer = BridgeTransfer {
            operator: operator.clone(),
            recipient: recipient.clone(),
            amount,
            source_chain,
            evidence_hash,
            submitted_at: now,
            status: TransferStatus::Pending,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Transfer(transfer_id), &transfer);
        env.storage()
            .instance()
            .set(&DataKey::WindowStart, &rate_limit.window_start);
        env.storage()
            .instance()
            .set(&DataKey::WindowUsed, &new_used);
        env.storage()
            .instance()
            .set(&DataKey::NextTransferId, &(transfer_id + 1));

        env.events().publish(
            (BRIDGE_NS, ACTION_SUBMIT, operator, recipient),
            (transfer_id, amount, source_chain),
        );

        Ok(transfer_id)
    }

    pub fn challenge_transfer(
        env: Env,
        challenger: Address,
        transfer_id: u64,
    ) -> Result<(), BridgeError> {
        challenger.require_auth();
        ensure_initialized(&env)?;
        require_not_paused(&env)?;

        let mut transfer = read_transfer(&env, transfer_id)?;
        if transfer.status != TransferStatus::Pending {
            return Err(map_non_pending_status(transfer.status));
        }

        let challenge_deadline = transfer
            .submitted_at
            .saturating_add(read_challenge_period(&env)?);
        if env.ledger().timestamp() > challenge_deadline {
            return Err(BridgeError::ChallengeWindowClosed);
        }

        transfer.status = TransferStatus::Challenged;
        env.storage()
            .persistent()
            .set(&DataKey::Transfer(transfer_id), &transfer);
        env.events()
            .publish((BRIDGE_NS, ACTION_CHALLENGE, challenger), transfer_id);

        Ok(())
    }

    pub fn submit_fraud_proof(
        env: Env,
        reviewer: Address,
        transfer_id: u64,
        proof_hash: Bytes,
    ) -> Result<(), BridgeError> {
        reviewer.require_auth();
        ensure_initialized(&env)?;
        require_not_paused(&env)?;

        let mut transfer = read_transfer(&env, transfer_id)?;
        match transfer.status {
            TransferStatus::Finalized | TransferStatus::Fraudulent => {
                return Err(BridgeError::TransferAlreadyResolved);
            }
            TransferStatus::Pending | TransferStatus::Challenged => {}
        }

        transfer.status = TransferStatus::Fraudulent;
        env.storage()
            .persistent()
            .set(&DataKey::Transfer(transfer_id), &transfer);
        env.storage()
            .persistent()
            .set(&DataKey::FraudProof(transfer_id), &proof_hash);

        env.events()
            .publish((BRIDGE_NS, ACTION_FRAUD, reviewer), transfer_id);

        Ok(())
    }

    pub fn finalize_transfer(
        env: Env,
        operator: Address,
        transfer_id: u64,
    ) -> Result<(), BridgeError> {
        operator.require_auth();
        ensure_initialized(&env)?;
        require_not_paused(&env)?;

        let mut transfer = read_transfer(&env, transfer_id)?;
        if transfer.operator != operator {
            return Err(BridgeError::Unauthorized);
        }

        match transfer.status {
            TransferStatus::Pending => {}
            TransferStatus::Challenged => return Err(BridgeError::TransferChallenged),
            TransferStatus::Finalized | TransferStatus::Fraudulent => {
                return Err(BridgeError::TransferAlreadyResolved);
            }
        }

        let challenge_deadline = transfer
            .submitted_at
            .saturating_add(read_challenge_period(&env)?);
        if env.ledger().timestamp() < challenge_deadline {
            return Err(BridgeError::ChallengeWindowOpen);
        }

        transfer.status = TransferStatus::Finalized;
        env.storage()
            .persistent()
            .set(&DataKey::Transfer(transfer_id), &transfer);
        env.events()
            .publish((BRIDGE_NS, ACTION_FINALIZE, operator), transfer_id);

        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), BridgeError> {
        set_pause_state(env, admin, true)
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), BridgeError> {
        set_pause_state(env, admin, false)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn get_transfer(env: Env, transfer_id: u64) -> Result<BridgeTransfer, BridgeError> {
        ensure_initialized(&env)?;
        read_transfer(&env, transfer_id)
    }

    pub fn get_fraud_proof(env: Env, transfer_id: u64) -> Result<Option<Bytes>, BridgeError> {
        ensure_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::FraudProof(transfer_id)))
    }

    pub fn get_rate_limit_state(env: Env) -> Result<RateLimitState, BridgeError> {
        ensure_initialized(&env)?;
        read_rate_limit_state(&env)
    }

    pub fn get_challenge_period(env: Env) -> Result<u64, BridgeError> {
        read_challenge_period(&env)
    }
}

fn set_pause_state(env: Env, admin: Address, paused: bool) -> Result<(), BridgeError> {
    admin.require_auth();
    let stored_admin = read_admin(&env)?;
    if stored_admin != admin {
        return Err(BridgeError::Unauthorized);
    }

    env.storage().instance().set(&DataKey::Paused, &paused);
    env.events().publish(
        (
            BRIDGE_NS,
            if paused { ACTION_PAUSE } else { ACTION_UNPAUSE },
            admin,
        ),
        env.ledger().timestamp(),
    );
    Ok(())
}

fn ensure_initialized(env: &Env) -> Result<(), BridgeError> {
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(BridgeError::NotInitialized)
    }
}

fn require_not_paused(env: &Env) -> Result<(), BridgeError> {
    if env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
    {
        Err(BridgeError::ContractPaused)
    } else {
        Ok(())
    }
}

fn require_positive_amount(amount: i128) -> Result<(), BridgeError> {
    if amount <= 0 {
        Err(BridgeError::InvalidAmount)
    } else {
        Ok(())
    }
}

fn read_admin(env: &Env) -> Result<Address, BridgeError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(BridgeError::NotInitialized)
}

fn read_transfer(env: &Env, transfer_id: u64) -> Result<BridgeTransfer, BridgeError> {
    env.storage()
        .persistent()
        .get(&DataKey::Transfer(transfer_id))
        .ok_or(BridgeError::TransferNotFound)
}

fn read_challenge_period(env: &Env) -> Result<u64, BridgeError> {
    env.storage()
        .instance()
        .get(&DataKey::ChallengePeriod)
        .ok_or(BridgeError::NotInitialized)
}

fn read_next_transfer_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextTransferId)
        .unwrap_or(1)
}

fn read_rate_limit_state(env: &Env) -> Result<RateLimitState, BridgeError> {
    Ok(RateLimitState {
        amount_limit: env
            .storage()
            .instance()
            .get(&DataKey::RateLimitAmount)
            .ok_or(BridgeError::NotInitialized)?,
        window_seconds: env
            .storage()
            .instance()
            .get(&DataKey::RateLimitWindow)
            .ok_or(BridgeError::NotInitialized)?,
        window_start: env
            .storage()
            .instance()
            .get(&DataKey::WindowStart)
            .unwrap_or(0),
        used_in_window: env
            .storage()
            .instance()
            .get(&DataKey::WindowUsed)
            .unwrap_or(0),
    })
}

fn map_non_pending_status(status: TransferStatus) -> BridgeError {
    match status {
        TransferStatus::Pending => BridgeError::InvalidTransferState,
        TransferStatus::Challenged => BridgeError::TransferChallenged,
        TransferStatus::Finalized | TransferStatus::Fraudulent => {
            BridgeError::TransferAlreadyResolved
        }
    }
}

#[cfg(test)]
mod test;
