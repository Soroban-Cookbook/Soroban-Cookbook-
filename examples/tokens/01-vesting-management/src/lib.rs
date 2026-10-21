#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: Address,
    pub total_amount: i128,
    pub released_amount: i128,
    pub start_timestamp: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub revoked: bool,
    pub token: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DefaultToken,
    Schedule(Address),
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VestingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidSchedule = 4,
    NothingToClaim = 5,
    ArithmeticOverflow = 6,
    ScheduleAlreadyExists = 7,
    ScheduleNotFound = 8,
    AlreadyRevoked = 9,
    CliffNotReached = 10,
}

// ---------------------------------------------------------------------------
// Events payloads & Action definitions
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleCreatedEventData {
    pub beneficiary: Address,
    pub total_amount: i128,
    pub start_timestamp: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub token: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimEventData {
    pub beneficiary: Address,
    pub amount: i128,
    pub token: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokedEventData {
    pub beneficiary: Address,
    pub vested_amount: i128,
    pub unvested_amount: i128,
    pub token: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferredEventData {
    pub old_admin: Address,
    pub new_admin: Address,
}

const CONTRACT_NS: Symbol = symbol_short!("vest_mgmt");
const ACTION_CREATE: Symbol = symbol_short!("create");
const ACTION_CLAIM: Symbol = symbol_short!("claim");
const ACTION_REVOKE: Symbol = symbol_short!("revoke");
const ACTION_ADMIN: Symbol = symbol_short!("admin");

// ---------------------------------------------------------------------------
// Vesting Management Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Initializes the contract with an admin and an optional default token address.
    pub fn initialize(
        env: Env,
        admin: Address,
        default_token: Option<Address>,
    ) -> Result<(), VestingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VestingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        if let Some(token_addr) = default_token {
            env.storage()
                .instance()
                .set(&DataKey::DefaultToken, &token_addr);
        }

        // Emit initialization / admin action event
        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin.clone()),
            symbol_short!("init"),
        );

        Ok(())
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Returns the default token address, if set.
    pub fn get_default_token(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::DefaultToken)
    }

    /// Transfers admin rights to a new address.
    pub fn transfer_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), VestingError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VestingError::NotInitialized)?;
        if admin != stored_admin {
            return Err(VestingError::Unauthorized);
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events().publish(
            (CONTRACT_NS, ACTION_ADMIN, admin.clone()),
            AdminTransferredEventData {
                old_admin: admin,
                new_admin,
            },
        );

        Ok(())
    }

    /// Creates a new vesting schedule for a beneficiary.
    /// Admin-only.
    #[allow(clippy::too_many_arguments)]
    pub fn create_schedule(
        env: Env,
        admin: Address,
        beneficiary: Address,
        total_amount: i128,
        start_timestamp: u64,
        cliff_duration: u64,
        vesting_duration: u64,
        token: Option<Address>,
    ) -> Result<(), VestingError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VestingError::NotInitialized)?;
        if admin != stored_admin {
            return Err(VestingError::Unauthorized);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Schedule(beneficiary.clone()))
        {
            return Err(VestingError::ScheduleAlreadyExists);
        }

        if total_amount <= 0 || vesting_duration == 0 || cliff_duration > vesting_duration {
            return Err(VestingError::InvalidSchedule);
        }

        let resolved_token = match token {
            Some(t) => t,
            None => env
                .storage()
                .instance()
                .get(&DataKey::DefaultToken)
                .ok_or(VestingError::NotInitialized)?,
        };

        let schedule = VestingSchedule {
            beneficiary: beneficiary.clone(),
            total_amount,
            released_amount: 0,
            start_timestamp,
            cliff_duration,
            vesting_duration,
            revoked: false,
            token: resolved_token.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(beneficiary.clone()), &schedule);

        env.events().publish(
            (CONTRACT_NS, ACTION_CREATE, beneficiary.clone()),
            ScheduleCreatedEventData {
                beneficiary,
                total_amount,
                start_timestamp,
                cliff_duration,
                vesting_duration,
                token: resolved_token,
            },
        );

        Ok(())
    }

    /// Returns the vesting schedule for a beneficiary.
    pub fn get_schedule(env: Env, beneficiary: Address) -> Option<VestingSchedule> {
        env.storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary))
    }

    /// Returns the currently vested amount for a beneficiary.
    pub fn vested_amount(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary))
            .ok_or(VestingError::ScheduleNotFound)?;

        Self::calculate_vested_amount(&schedule, env.ledger().timestamp())
    }

    /// Returns the releasable amount for a beneficiary (vested - already released).
    pub fn releasable_amount(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary))
            .ok_or(VestingError::ScheduleNotFound)?;

        let vested = Self::calculate_vested_amount(&schedule, env.ledger().timestamp())?;
        let releasable = vested
            .checked_sub(schedule.released_amount)
            .ok_or(VestingError::ArithmeticOverflow)?;

        Ok(releasable)
    }

    /// Claims vested tokens for the beneficiary.
    /// Requires beneficiary's authorization.
    pub fn claim(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary.clone()))
            .ok_or(VestingError::ScheduleNotFound)?;

        let current_time = env.ledger().timestamp();

        if current_time < schedule.start_timestamp + schedule.cliff_duration {
            return Err(VestingError::CliffNotReached);
        }

        let vested = Self::calculate_vested_amount(&schedule, current_time)?;
        let claimable = vested
            .checked_sub(schedule.released_amount)
            .ok_or(VestingError::ArithmeticOverflow)?;

        if claimable <= 0 {
            return Err(VestingError::NothingToClaim);
        }

        schedule.released_amount = schedule
            .released_amount
            .checked_add(claimable)
            .ok_or(VestingError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(beneficiary.clone()), &schedule);

        let token_client = token::Client::new(&env, &schedule.token);
        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        env.events().publish(
            (CONTRACT_NS, ACTION_CLAIM, beneficiary.clone()),
            ClaimEventData {
                beneficiary,
                amount: claimable,
                token: schedule.token,
            },
        );

        Ok(claimable)
    }

    /// Revokes a vesting schedule.
    /// Admin-only. Transfer unvested tokens back to admin, while vested-but-unclaimed tokens remain claimable.
    pub fn revoke(env: Env, admin: Address, beneficiary: Address) -> Result<(), VestingError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VestingError::NotInitialized)?;
        if admin != stored_admin {
            return Err(VestingError::Unauthorized);
        }

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary.clone()))
            .ok_or(VestingError::ScheduleNotFound)?;

        if schedule.revoked {
            return Err(VestingError::AlreadyRevoked);
        }

        let current_time = env.ledger().timestamp();
        let vested_at_revoke = Self::calculate_vested_amount(&schedule, current_time)?;

        let unvested_remainder = schedule
            .total_amount
            .checked_sub(vested_at_revoke)
            .ok_or(VestingError::ArithmeticOverflow)?;

        schedule.total_amount = vested_at_revoke;
        schedule.revoked = true;

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(beneficiary.clone()), &schedule);

        if unvested_remainder > 0 {
            let token_client = token::Client::new(&env, &schedule.token);
            token_client.transfer(&env.current_contract_address(), &admin, &unvested_remainder);
        }

        env.events().publish(
            (CONTRACT_NS, ACTION_REVOKE, beneficiary.clone()),
            RevokedEventData {
                beneficiary,
                vested_amount: vested_at_revoke,
                unvested_amount: unvested_remainder,
                token: schedule.token,
            },
        );

        Ok(())
    }

    fn calculate_vested_amount(
        schedule: &VestingSchedule,
        current_time: u64,
    ) -> Result<i128, VestingError> {
        if schedule.revoked {
            return Ok(schedule.total_amount);
        }

        if current_time < schedule.start_timestamp + schedule.cliff_duration {
            return Ok(0);
        }

        if current_time >= schedule.start_timestamp + schedule.vesting_duration {
            return Ok(schedule.total_amount);
        }

        let elapsed_time = current_time
            .checked_sub(schedule.start_timestamp)
            .ok_or(VestingError::ArithmeticOverflow)?;

        // linear vesting: total_amount * elapsed_time / vesting_duration
        let vested = schedule
            .total_amount
            .checked_mul(elapsed_time as i128)
            .ok_or(VestingError::ArithmeticOverflow)?
            .checked_div(schedule.vesting_duration as i128)
            .ok_or(VestingError::ArithmeticOverflow)?;

        Ok(vested)
    }
}

mod test;
