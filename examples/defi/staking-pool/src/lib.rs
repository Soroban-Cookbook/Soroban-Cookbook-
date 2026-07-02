#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

const LOCKUP_30_DAYS: u64 = 2_592_000;
const LOCKUP_90_DAYS: u64 = 7_776_000;
const LOCKUP_180_DAYS: u64 = 15_552_000;
const PENALTY_BPS: i128 = 2_000; // 20%
const BOOST_30_DAYS_BPS: i128 = 0;
const BOOST_90_DAYS_BPS: i128 = 1_000; // 10%
const BOOST_180_DAYS_BPS: i128 = 2_500; // 25%
const BASIS_POINTS_DENOMINATOR: i128 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Stake(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInfo {
    pub amount: i128,
    pub start_ts: u64,
    pub duration: u64,
    pub boost_bps: i128,
    pub mature_ts: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockupOption {
    pub duration: u64,
    pub boost_bps: i128,
    pub label: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeEvent {
    pub account: Address,
    pub amount: i128,
    pub duration: u64,
    pub boost_bps: i128,
    pub mature_ts: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub account: Address,
    pub amount: i128,
    pub net_amount: i128,
    pub penalty: i128,
    pub matured: bool,
}

const CONTRACT_NS: Symbol = symbol_short!("staking");
const ACTION_STAKE: Symbol = symbol_short!("stake");
const ACTION_WITHDRAW: Symbol = symbol_short!("withdraw");

#[contract]
pub struct StakingPoolContract;

#[contractimpl]
impl StakingPoolContract {
    pub fn get_lockup_options(env: Env) -> Vec<LockupOption> {
        vec![
            &env,
            LockupOption {
                duration: LOCKUP_30_DAYS,
                boost_bps: BOOST_30_DAYS_BPS,
                label: symbol_short!("30d"),
            },
            LockupOption {
                duration: LOCKUP_90_DAYS,
                boost_bps: BOOST_90_DAYS_BPS,
                label: symbol_short!("90d"),
            },
            LockupOption {
                duration: LOCKUP_180_DAYS,
                boost_bps: BOOST_180_DAYS_BPS,
                label: symbol_short!("180d"),
            },
        ]
    }

    pub fn stake(env: Env, staker: Address, amount: i128, duration: u64) {
        staker.require_auth();
        if amount <= 0 {
            panic!("Stake amount must be positive");
        }

        let key = DataKey::Stake(staker.clone());
        if env.storage().persistent().has(&key) {
            panic!("Active stake exists");
        }

        let boost_bps = Self::lockup_boost(duration);
        let start_ts = env.ledger().timestamp();
        let mature_ts = start_ts + duration;

        let stake = StakeInfo {
            amount,
            start_ts,
            duration,
            boost_bps,
            mature_ts,
        };

        env.storage().persistent().set(&key, &stake);
        env.events().publish(
            (CONTRACT_NS, ACTION_STAKE, staker.clone()),
            StakeEvent {
                account: staker,
                amount,
                duration,
                boost_bps,
                mature_ts,
            },
        );
    }

    pub fn withdraw(env: Env, staker: Address) -> i128 {
        staker.require_auth();

        let key = DataKey::Stake(staker.clone());
        let stake: StakeInfo = env
            .storage()
            .persistent()
            .get(&key)
            .expect("No active stake");

        env.storage().persistent().remove(&key);
        let now = env.ledger().timestamp();

        let (net_amount, penalty, matured) = if now < stake.mature_ts {
            let penalty = stake.amount * PENALTY_BPS / BASIS_POINTS_DENOMINATOR;
            (stake.amount - penalty, penalty, false)
        } else {
            let bonus = stake.amount * stake.boost_bps / BASIS_POINTS_DENOMINATOR;
            (stake.amount + bonus, 0, true)
        };

        env.events().publish(
            (CONTRACT_NS, ACTION_WITHDRAW, staker.clone()),
            WithdrawEvent {
                account: staker,
                amount: stake.amount,
                net_amount,
                penalty,
                matured,
            },
        );

        net_amount
    }

    pub fn get_stake(env: Env, staker: Address) -> Option<StakeInfo> {
        let key = DataKey::Stake(staker);
        env.storage().persistent().get(&key)
    }

    fn lockup_boost(duration: u64) -> i128 {
        match duration {
            LOCKUP_30_DAYS => BOOST_30_DAYS_BPS,
            LOCKUP_90_DAYS => BOOST_90_DAYS_BPS,
            LOCKUP_180_DAYS => BOOST_180_DAYS_BPS,
            _ => panic!("Invalid lockup duration"),
        }
    }
}
