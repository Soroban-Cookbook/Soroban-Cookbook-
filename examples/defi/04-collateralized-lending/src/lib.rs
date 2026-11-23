#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, contracttype, token::TokenClient, Address, Env, Map, Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct Position {
    pub collateral: i128,
    pub debt: i128,
}

/// Per-collateral-token risk parameters, denominated in basis points
/// (10_000 bps = 100%). Volatility profiles differ per asset, so each
/// collateral token gets its own LTV and liquidation threshold: stablecoins
/// can safely run ~9_000 bps LTV, blue-chip assets ~7_500, and long-tail
/// assets ~5_000.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CollateralConfig {
    /// Maximum debt-to-collateral ratio for new borrows.
    pub ltv_bps: u32,
    /// Debt-to-collateral ratio above which a position becomes liquidatable.
    pub liquidation_threshold_bps: u32,
}

/// Storage keys for per-token data.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CollateralConfig(Address),
}

#[contract]
pub struct LendingContract;

fn token_client<'a>(env: &'a Env, key: &str) -> TokenClient<'a> {
    let token: Address = env
        .storage()
        .instance()
        .get(&Symbol::new(env, key))
        .unwrap();
    TokenClient::new(env, &token)
}

/// The configured collateral token address.
fn collateral_token(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "collateral_token"))
        .unwrap()
}

/// Risk parameters for `token`. Falls back to the global `ltv_ratio` /
/// `liquidation_threshold` (percent, as set at initialization) when no
/// per-token config has been stored.
fn read_collateral_config(env: &Env, token: &Address) -> CollateralConfig {
    if let Some(config) = env
        .storage()
        .persistent()
        .get::<_, CollateralConfig>(&DataKey::CollateralConfig(token.clone()))
    {
        return config;
    }
    let ltv_ratio: i128 = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "ltv_ratio"))
        .unwrap();
    let liquidation_threshold: i128 = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "liquidation_threshold"))
        .unwrap();
    CollateralConfig {
        ltv_bps: (ltv_ratio * 100) as u32,
        liquidation_threshold_bps: (liquidation_threshold * 100) as u32,
    }
}

#[contractimpl]
impl LendingContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        collateral_token: Address,
        loan_token: Address,
        ltv_ratio: i128,
        liquidation_threshold: i128,
        liquidation_incentive: i128,
        partial_liquidation_ratio: i128,
    ) {
        if env
            .storage()
            .instance()
            .has(&Symbol::new(&env, "ltv_ratio"))
        {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &admin);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "collateral_token"), &collateral_token);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "loan_token"), &loan_token);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "ltv_ratio"), &ltv_ratio);
        env.storage().instance().set(
            &Symbol::new(&env, "liquidation_threshold"),
            &liquidation_threshold,
        );
        env.storage().instance().set(
            &Symbol::new(&env, "liquidation_incentive"),
            &liquidation_incentive,
        );
        env.storage().instance().set(
            &Symbol::new(&env, "partial_liquidation_ratio"),
            &partial_liquidation_ratio,
        );
        // Seed the collateral token's risk profile from the global defaults;
        // admins can override it per token via `set_collateral_config`.
        env.storage().persistent().set(
            &DataKey::CollateralConfig(collateral_token.clone()),
            &CollateralConfig {
                ltv_bps: (ltv_ratio * 100) as u32,
                liquidation_threshold_bps: (liquidation_threshold * 100) as u32,
            },
        );
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "emergency_paused"), &false);
        env.storage().instance().set(
            &Symbol::new(&env, "positions"),
            &Map::<Address, Position>::new(&env),
        );
    }

    pub fn set_emergency_pause(env: Env, admin: Address, paused: bool) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        if admin != stored_admin {
            panic!("not admin");
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "emergency_paused"), &paused);
    }

    /// Admin-only: set the risk profile for one collateral token.
    ///
    /// `ltv_bps` caps new borrowing against that collateral;
    /// `liquidation_threshold_bps` is the debt level above which a position
    /// becomes liquidatable. Both are basis points (10_000 = 100%).
    pub fn set_collateral_config(
        env: Env,
        admin: Address,
        token: Address,
        config: CollateralConfig,
    ) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        if admin != stored_admin {
            panic!("not admin");
        }
        admin.require_auth();
        if config.ltv_bps > 10_000
            || config.liquidation_threshold_bps > 10_000
            || config.ltv_bps > config.liquidation_threshold_bps
        {
            panic!("invalid collateral config");
        }
        env.storage()
            .persistent()
            .set(&DataKey::CollateralConfig(token), &config);
    }

    /// Read the stored risk profile for a collateral token.
    pub fn get_collateral_config(env: Env, token: Address) -> CollateralConfig {
        read_collateral_config(&env, &token)
    }

    pub fn deposit_collateral(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "emergency_paused"))
            .unwrap()
        {
            panic!("emergency paused");
        }
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        position.collateral += amount;
        positions.set(user.clone(), position);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);

        token_client(&env, "collateral_token").transfer(
            &user,
            env.current_contract_address(),
            &amount,
        );
    }

    pub fn withdraw_collateral(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "emergency_paused"))
            .unwrap()
        {
            panic!("emergency paused");
        }
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        if position.collateral < amount {
            panic!("insufficient collateral");
        }
        let new_collateral = position.collateral - amount;
        let config = read_collateral_config(&env, &collateral_token(&env));
        if new_collateral * config.ltv_bps as i128 / 10_000 < position.debt {
            panic!("health factor too low");
        }
        position.collateral = new_collateral;
        positions.set(user.clone(), position);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);

        token_client(&env, "collateral_token").transfer(
            &env.current_contract_address(),
            &user,
            &amount,
        );
    }

    pub fn borrow(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "emergency_paused"))
            .unwrap()
        {
            panic!("emergency paused");
        }
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        // Per-token LTV instead of one global ratio: stablecoin collateral
        // can be configured near 95%, long-tail assets down to 50%.
        let config = read_collateral_config(&env, &collateral_token(&env));
        let max_borrow = position.collateral * config.ltv_bps as i128 / 10_000;
        if position.debt + amount > max_borrow {
            panic!("exceeds maximum borrow amount");
        }
        position.debt += amount;
        positions.set(user.clone(), position);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);

        token_client(&env, "loan_token").transfer(&env.current_contract_address(), &user, &amount);
    }

    pub fn repay(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(user.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        let repay_amount = if amount > position.debt {
            position.debt
        } else {
            amount
        };
        position.debt -= repay_amount;
        positions.set(user.clone(), position);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);

        if repay_amount > 0 {
            token_client(&env, "loan_token").transfer(
                &user,
                env.current_contract_address(),
                &repay_amount,
            );
        }
    }

    pub fn liquidate(env: Env, liquidator: Address, borrower: Address, repay_amount: i128) {
        liquidator.require_auth();
        let config = read_collateral_config(&env, &collateral_token(&env));
        let liquidation_incentive: i128 = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "liquidation_incentive"))
            .unwrap();
        let partial_liquidation_ratio: i128 = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "partial_liquidation_ratio"))
            .unwrap();
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let mut position = positions.get(borrower.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        if position.debt == 0 {
            panic!("no debt to liquidate");
        }
        if position.collateral * config.liquidation_threshold_bps as i128 / 10_000
            >= position.debt
        {
            panic!("position is healthy");
        }
        let max_repay = position.debt * partial_liquidation_ratio / 100;
        let actual_repay = if repay_amount > max_repay {
            max_repay
        } else {
            repay_amount
        };
        if actual_repay <= 0 {
            panic!("repay amount too small");
        }
        let mut collateral_to_seize = actual_repay * (100 + liquidation_incentive) / 100;
        let mut final_repay = actual_repay;
        if collateral_to_seize > position.collateral {
            collateral_to_seize = position.collateral;
            final_repay = collateral_to_seize * 100 / (100 + liquidation_incentive);
        }
        position.debt -= final_repay;
        position.collateral -= collateral_to_seize;
        let mut liquidator_position = positions.get(liquidator.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        liquidator_position.collateral += collateral_to_seize;
        positions.set(liquidator.clone(), liquidator_position);
        if position.debt == 0 {
            positions.remove(borrower);
        } else {
            positions.set(borrower, position);
        }
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);

        // The liquidator pays down the debt in loan tokens; the seized
        // collateral stays in the contract, credited to their position.
        token_client(&env, "loan_token").transfer(
            &liquidator,
            env.current_contract_address(),
            &final_repay,
        );
    }

    pub fn emergency_liquidate(env: Env, admin: Address, borrower: Address) {
        admin.require_auth();
        let paused: bool = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "emergency_paused"))
            .unwrap();
        if !paused {
            panic!("not in emergency mode");
        }
        let mut positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let position = positions.get(borrower.clone()).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        if position.debt == 0 {
            panic!("no debt to liquidate");
        }
        positions.remove(borrower);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "positions"), &positions);
    }

    pub fn get_position(env: Env, user: Address) -> Position {
        let positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        positions.get(user).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        })
    }

    pub fn get_health_factor(env: Env, user: Address) -> i128 {
        let config = read_collateral_config(&env, &collateral_token(&env));
        let positions: Map<Address, Position> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "positions"))
            .unwrap();
        let position = positions.get(user).unwrap_or(Position {
            collateral: 0,
            debt: 0,
        });
        if position.debt == 0 {
            return i128::MAX;
        }
        (position.collateral * config.liquidation_threshold_bps as i128)
            / (position.debt * 10_000)
    }
}

#[cfg(test)]
mod test;
