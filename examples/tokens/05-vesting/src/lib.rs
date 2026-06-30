#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env,
    token,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: Address,
    pub total_allocation: i128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub claimed_amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Schedule(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VestingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidSchedule = 4,
    ClaimBeforeCliff = 5,
    NothingToClaim = 6,
    ArithmeticOverflow = 7,
    ScheduleAlreadyExists = 8,
    InsufficientTokenBalance = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleCreatedEventData {
    pub beneficiary: Address,
    pub total_allocation: i128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokensClaimedEventData {
    pub beneficiary: Address,
    pub amount: i128,
}

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Initializes the contract with an admin and the token to be vested.
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), VestingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VestingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        Ok(())
    }

    /// Creates a new vesting schedule for a beneficiary.
    /// Admin-only.
    pub fn create_schedule(
        env: Env,
        admin: Address,
        beneficiary: Address,
        total_allocation: i128,
        start_time: u64,
        cliff_duration: u64,
        vesting_duration: u64,
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

        if total_allocation <= 0 || vesting_duration == 0 {
            return Err(VestingError::InvalidSchedule);
        }

        let schedule = VestingSchedule {
            beneficiary: beneficiary.clone(),
            total_allocation,
            start_time,
            cliff_duration,
            vesting_duration,
            claimed_amount: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(beneficiary.clone()), &schedule);

        env.events().publish(
            (symbol_short!("vesting"), symbol_short!("create"), beneficiary.clone()),
            ScheduleCreatedEventData {
                beneficiary,
                total_allocation,
                start_time,
                cliff_duration,
                vesting_duration,
            },
        );

        Ok(())
    }

    /// Claims vested tokens for the caller.
    pub fn claim(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary.clone()))
            .ok_or(VestingError::InvalidSchedule)?;

        let current_time = env.ledger().timestamp();

        if current_time < schedule.start_time + schedule.cliff_duration {
            return Err(VestingError::ClaimBeforeCliff);
        }

        let vested_amount = Self::calculate_vested_amount(&schedule, current_time)?;
        let claimable_amount = vested_amount - schedule.claimed_amount;

        if claimable_amount <= 0 {
            return Err(VestingError::NothingToClaim);
        }

        schedule.claimed_amount += claimable_amount;
        env.storage()
            .persistent()
            .set(&DataKey::Schedule(beneficiary.clone()), &schedule);

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(VestingError::NotInitialized)?;
        let token_client = token::Client::new(&env, &token_address);

        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable_amount);

        env.events().publish(
            (symbol_short!("vesting"), symbol_short!("claim"), beneficiary.clone()),
            TokensClaimedEventData {
                beneficiary,
                amount: claimable_amount,
            },
        );

        Ok(claimable_amount)
    }

    /// Returns the vesting schedule for a beneficiary.
    pub fn get_schedule(env: Env, beneficiary: Address) -> Option<VestingSchedule> {
        env.storage().persistent().get(&DataKey::Schedule(beneficiary))
    }

    /// Returns the currently vested amount for a beneficiary.
    pub fn get_vested_amount(env: Env, beneficiary: Address) -> Result<i128, VestingError> {
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(beneficiary))
            .ok_or(VestingError::InvalidSchedule)?;

        Self::calculate_vested_amount(&schedule, env.ledger().timestamp())
    }

    fn calculate_vested_amount(
        schedule: &VestingSchedule,
        current_time: u64,
    ) -> Result<i128, VestingError> {
        if current_time < schedule.start_time + schedule.cliff_duration {
            return Ok(0);
        }

        if current_time >= schedule.start_time + schedule.vesting_duration {
            return Ok(schedule.total_allocation);
        }

        let elapsed_time = current_time - schedule.start_time;

        // linear vesting: total_allocation * elapsed_time / vesting_duration
        let vested = schedule
            .total_allocation
            .checked_mul(elapsed_time as i128)
            .ok_or(VestingError::ArithmeticOverflow)?
            .checked_div(schedule.vesting_duration as i128)
            .ok_or(VestingError::ArithmeticOverflow)?;

        Ok(vested)
    }
}

mod test;
