//! # Reward Distribution Strategies
//!
//! Demonstrates three common ways to release a fixed reward pool to
//! registered participants:
//!
//! - `Linear`: the pool unlocks at a constant rate over `duration` seconds.
//! - `ExponentialDecay`: the pool's remaining balance decays by `decay_bps`
//!   every `period_length` seconds, so early periods release more than later ones.
//! - `PerformanceBased`: the whole pool is available immediately, split by
//!   each participant's registered performance score.
//!
//! In all strategies a participant's individual share is proportional to
//! their registered weight relative to the sum of all registered weights.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Strategy {
    Linear,
    ExponentialDecay,
    PerformanceBased,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Strategy,
    TotalReward,
    StartTime,
    Duration,
    DecayBps,
    PeriodLength,
    TotalWeight,
    Weight(Address),
    Position(Address),
}

/// A participant's reward position, kept in a single storage entry so that
/// every operation on it costs at most one read and one write.
#[contracttype]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Position {
    /// Cumulative rewards settled so far, whether claimed or compounded.
    pub claimed: i128,
    /// Settled rewards the participant has re-staked instead of withdrawing.
    pub staked: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidConfig = 5,
    AlreadyRegistered = 6,
    NotRegistered = 7,
    NothingToClaim = 8,
    Overflow = 9,
    InsufficientClaimed = 10,
}

const EVENT_REGISTER: Symbol = symbol_short!("register");
const EVENT_CLAIM: Symbol = symbol_short!("claim");
const EVENT_STAKE: Symbol = symbol_short!("stake");
const EVENT_COMPOUND: Symbol = symbol_short!("compound");

const BPS_DENOMINATOR: i128 = 10_000;

#[contract]
pub struct RewardDistributor;

#[contractimpl]
impl RewardDistributor {
    /// Configure the reward pool and distribution strategy once.
    ///
    /// `decay_bps` and `period_length` are only meaningful for
    /// `Strategy::ExponentialDecay`: every `period_length` seconds the
    /// pool's remaining balance is multiplied by `(10_000 - decay_bps) / 10_000`.
    pub fn initialize(
        env: Env,
        admin: Address,
        strategy: Strategy,
        total_reward: i128,
        start_time: u64,
        duration: u64,
        decay_bps: u32,
        period_length: u64,
    ) -> Result<(), RewardError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RewardError::AlreadyInitialized);
        }
        if total_reward <= 0 || duration == 0 {
            return Err(RewardError::InvalidConfig);
        }
        if matches!(strategy, Strategy::ExponentialDecay)
            && (decay_bps == 0 || decay_bps as i128 >= BPS_DENOMINATOR || period_length == 0)
        {
            return Err(RewardError::InvalidConfig);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Strategy, &strategy);
        env.storage()
            .instance()
            .set(&DataKey::TotalReward, &total_reward);
        env.storage()
            .instance()
            .set(&DataKey::StartTime, &start_time);
        env.storage().instance().set(&DataKey::Duration, &duration);
        env.storage().instance().set(&DataKey::DecayBps, &decay_bps);
        env.storage()
            .instance()
            .set(&DataKey::PeriodLength, &period_length);
        env.storage().instance().set(&DataKey::TotalWeight, &0i128);
        Ok(())
    }

    /// Register a participant with a weight.
    ///
    /// For `Linear`/`ExponentialDecay` this is the participant's share of the
    /// pool; for `PerformanceBased` this is their raw performance score.
    pub fn register(
        env: Env,
        admin: Address,
        participant: Address,
        weight: i128,
    ) -> Result<(), RewardError> {
        require_admin(&env, &admin)?;
        if weight <= 0 {
            return Err(RewardError::InvalidAmount);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Weight(participant.clone()))
        {
            return Err(RewardError::AlreadyRegistered);
        }

        let total_weight: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWeight)
            .unwrap_or(0);
        let new_total = total_weight
            .checked_add(weight)
            .ok_or(RewardError::Overflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Weight(participant.clone()), &weight);
        env.storage()
            .instance()
            .set(&DataKey::TotalWeight, &new_total);
        env.events().publish((EVENT_REGISTER, participant), weight);
        Ok(())
    }

    /// Amount `participant` is currently entitled to but has not yet claimed.
    pub fn claimable(env: Env, participant: Address, now: u64) -> Result<i128, RewardError> {
        let entitled = Self::entitled(&env, &participant, now)?;
        let position = load_position(&env, &participant);
        Ok(entitled - position.claimed)
    }

    /// Claim (harvest) the currently available reward for `participant`.
    pub fn claim(env: Env, participant: Address, now: u64) -> Result<i128, RewardError> {
        participant.require_auth();

        let entitled = Self::entitled(&env, &participant, now)?;
        let mut position = load_position(&env, &participant);
        let amount = entitled - position.claimed;
        if amount <= 0 {
            return Err(RewardError::NothingToClaim);
        }

        position.claimed = entitled;
        save_position(&env, &participant, &position);
        env.events().publish((EVENT_CLAIM, participant), amount);
        Ok(amount)
    }

    /// Re-stake `amount` of previously claimed rewards.
    ///
    /// A participant can only stake rewards they have already claimed, so
    /// `staked` never exceeds `claimed`.
    pub fn stake(env: Env, participant: Address, amount: i128) -> Result<(), RewardError> {
        participant.require_auth();
        if amount <= 0 {
            return Err(RewardError::InvalidAmount);
        }

        let mut position = load_position(&env, &participant);
        let new_staked = position
            .staked
            .checked_add(amount)
            .ok_or(RewardError::Overflow)?;
        if new_staked > position.claimed {
            return Err(RewardError::InsufficientClaimed);
        }

        position.staked = new_staked;
        save_position(&env, &participant, &position);
        env.events().publish((EVENT_STAKE, participant), amount);
        Ok(())
    }

    /// Claim the currently available reward and re-stake it in one step.
    ///
    /// The result is the same as `claim` followed by `stake` with the claimed
    /// amount, but cheaper. Calling those two functions in sequence loads and
    /// saves the participant's `Position` twice. Here the position is read
    /// once, both the claimed and the staked amounts are updated in memory,
    /// and it is written back once. That halves the position storage I/O for
    /// participants who compound often.
    pub fn compound_rewards(env: Env, participant: Address, now: u64) -> Result<i128, RewardError> {
        participant.require_auth();

        let entitled = Self::entitled(&env, &participant, now)?;
        let mut position = load_position(&env, &participant);
        let amount = entitled - position.claimed;
        if amount <= 0 {
            return Err(RewardError::NothingToClaim);
        }

        position.claimed = entitled;
        position.staked = position
            .staked
            .checked_add(amount)
            .ok_or(RewardError::Overflow)?;
        save_position(&env, &participant, &position);
        env.events().publish((EVENT_COMPOUND, participant), amount);
        Ok(amount)
    }

    /// Current reward position of `participant` (zeroed if never touched).
    pub fn position(env: Env, participant: Address) -> Position {
        load_position(&env, &participant)
    }

    fn entitled(env: &Env, participant: &Address, now: u64) -> Result<i128, RewardError> {
        let weight: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Weight(participant.clone()))
            .ok_or(RewardError::NotRegistered)?;
        let total_weight: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWeight)
            .unwrap_or(0);
        let total_reward: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalReward)
            .ok_or(RewardError::NotInitialized)?;
        let strategy: Strategy = env
            .storage()
            .instance()
            .get(&DataKey::Strategy)
            .ok_or(RewardError::NotInitialized)?;

        let pool_released = match strategy {
            Strategy::PerformanceBased => total_reward,
            Strategy::Linear => {
                let start_time: u64 = env
                    .storage()
                    .instance()
                    .get(&DataKey::StartTime)
                    .unwrap_or(0);
                let duration: u64 = env
                    .storage()
                    .instance()
                    .get(&DataKey::Duration)
                    .unwrap_or(0);
                let elapsed = elapsed_capped(now, start_time, duration);
                mul_div(total_reward, elapsed as i128, duration as i128)?
            }
            Strategy::ExponentialDecay => {
                let start_time: u64 = env
                    .storage()
                    .instance()
                    .get(&DataKey::StartTime)
                    .unwrap_or(0);
                let duration: u64 = env
                    .storage()
                    .instance()
                    .get(&DataKey::Duration)
                    .unwrap_or(0);
                let decay_bps: u32 = env
                    .storage()
                    .instance()
                    .get(&DataKey::DecayBps)
                    .unwrap_or(0);
                let period_length: u64 = env
                    .storage()
                    .instance()
                    .get(&DataKey::PeriodLength)
                    .unwrap_or(1);
                let elapsed = elapsed_capped(now, start_time, duration);
                let periods = elapsed / period_length;
                let retain_bps = BPS_DENOMINATOR - decay_bps as i128;

                let remaining_factor = pow_bps(retain_bps, periods)?;
                let remaining = mul_div(total_reward, remaining_factor, BPS_DENOMINATOR)?;
                total_reward - remaining
            }
        };

        mul_div(pool_released, weight, total_weight)
    }
}

/// Seconds elapsed since `start_time`, capped at `duration` and floored at 0.
fn elapsed_capped(now: u64, start_time: u64, duration: u64) -> u64 {
    if now <= start_time {
        return 0;
    }
    let elapsed = now - start_time;
    if elapsed > duration {
        duration
    } else {
        elapsed
    }
}

/// `a * b / denom` using checked arithmetic.
fn mul_div(a: i128, b: i128, denom: i128) -> Result<i128, RewardError> {
    a.checked_mul(b)
        .and_then(|v| v.checked_div(denom))
        .ok_or(RewardError::Overflow)
}

/// `base_bps ^ exp`, where `base_bps` and the result are fixed-point values
/// scaled by `BPS_DENOMINATOR` (i.e. `10_000` represents `1.0`). Uses
/// exponentiation by squaring so the cost is `O(log exp)` instead of
/// `O(exp)`, which matters since `exp` is a user-influenced period count.
fn pow_bps(base_bps: i128, exp: u64) -> Result<i128, RewardError> {
    let mut result = BPS_DENOMINATOR;
    let mut base = base_bps;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = mul_div(result, base, BPS_DENOMINATOR)?;
        }
        base = mul_div(base, base, BPS_DENOMINATOR)?;
        e >>= 1;
    }
    Ok(result)
}

fn load_position(env: &Env, participant: &Address) -> Position {
    env.storage()
        .persistent()
        .get(&DataKey::Position(participant.clone()))
        .unwrap_or_default()
}

fn save_position(env: &Env, participant: &Address, position: &Position) {
    env.storage()
        .persistent()
        .set(&DataKey::Position(participant.clone()), position);
}

fn require_admin(env: &Env, admin: &Address) -> Result<(), RewardError> {
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(RewardError::NotInitialized)?;
    if stored_admin != *admin {
        return Err(RewardError::Unauthorized);
    }
    admin.require_auth();
    Ok(())
}

#[cfg(test)]
mod test;
