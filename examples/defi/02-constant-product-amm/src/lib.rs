#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token::TokenClient, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TokenX,
    TokenY,
    TotalSupply,
    LpBalance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AmmError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientLiquidity = 4,
    InsufficientLpBalance = 5,
    InsufficientOutputAmount = 6,
    RatioMismatch = 7,
    InvalidTokenPair = 8,
    ArithmeticOverflow = 9,
}

const FEE_NUMERATOR: i128 = 997;
const FEE_DENOMINATOR: i128 = 1000;

#[contract]
pub struct ConstantProductAmm;

#[contractimpl]
impl ConstantProductAmm {
    pub fn initialize(env: Env, token_x: Address, token_y: Address) -> Result<(), AmmError> {
        if env.storage().instance().has(&DataKey::TokenX)
            || env.storage().instance().has(&DataKey::TokenY)
        {
            return Err(AmmError::AlreadyInitialized);
        }
        if token_x == token_y {
            return Err(AmmError::InvalidTokenPair);
        }

        env.storage().instance().set(&DataKey::TokenX, &token_x);
        env.storage().instance().set(&DataKey::TokenY, &token_y);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        Ok(())
    }

    pub fn add_liquidity(
        env: Env,
        provider: Address,
        amount_x: i128,
        amount_y: i128,
    ) -> Result<i128, AmmError> {
        require_positive(amount_x)?;
        require_positive(amount_y)?;
        provider.require_auth();

        let token_x = read_token(&env, DataKey::TokenX)?;
        let token_y = read_token(&env, DataKey::TokenY)?;
        let contract = env.current_contract_address();
        let reserve_x = TokenClient::new(&env, &token_x).balance(&contract);
        let reserve_y = TokenClient::new(&env, &token_y).balance(&contract);
        let total_supply = read_total_supply(&env);

        TokenClient::new(&env, &token_x).transfer(&provider, &contract, &amount_x);
        TokenClient::new(&env, &token_y).transfer(&provider, &contract, &amount_y);

        let minted = if total_supply == 0 {
            let liquidity = integer_sqrt(
                amount_x
                    .checked_mul(amount_y)
                    .ok_or(AmmError::ArithmeticOverflow)?,
            );
            if liquidity <= 0 {
                return Err(AmmError::InvalidAmount);
            }
            liquidity
        } else {
            if reserve_x == 0 || reserve_y == 0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            if amount_x
                .checked_mul(reserve_y)
                .ok_or(AmmError::ArithmeticOverflow)?
                != amount_y
                    .checked_mul(reserve_x)
                    .ok_or(AmmError::ArithmeticOverflow)?
            {
                return Err(AmmError::RatioMismatch);
            }
            let share_x = amount_x
                .checked_mul(total_supply)
                .ok_or(AmmError::ArithmeticOverflow)?
                .checked_div(reserve_x)
                .ok_or(AmmError::ArithmeticOverflow)?;
            let share_y = amount_y
                .checked_mul(total_supply)
                .ok_or(AmmError::ArithmeticOverflow)?
                .checked_div(reserve_y)
                .ok_or(AmmError::ArithmeticOverflow)?;
            let minted = if share_x < share_y { share_x } else { share_y };
            if minted <= 0 {
                return Err(AmmError::InvalidAmount);
            }
            minted
        };

        mint_lp(&env, &provider, minted)?;
        Ok(minted)
    }

    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        lp_amount: i128,
    ) -> Result<(i128, i128), AmmError> {
        require_positive(lp_amount)?;
        provider.require_auth();

        let token_x = read_token(&env, DataKey::TokenX)?;
        let token_y = read_token(&env, DataKey::TokenY)?;
        let contract = env.current_contract_address();
        let reserve_x = TokenClient::new(&env, &token_x).balance(&contract);
        let reserve_y = TokenClient::new(&env, &token_y).balance(&contract);
        let total_supply = read_total_supply(&env);

        if total_supply == 0 || lp_amount > total_supply {
            return Err(AmmError::InsufficientLpBalance);
        }

        let amount_x = reserve_x
            .checked_mul(lp_amount)
            .ok_or(AmmError::ArithmeticOverflow)?
            .checked_div(total_supply)
            .ok_or(AmmError::ArithmeticOverflow)?;
        let amount_y = reserve_y
            .checked_mul(lp_amount)
            .ok_or(AmmError::ArithmeticOverflow)?
            .checked_div(total_supply)
            .ok_or(AmmError::ArithmeticOverflow)?;

        if amount_x <= 0 || amount_y <= 0 {
            return Err(AmmError::InsufficientLiquidity);
        }

        burn_lp(&env, &provider, lp_amount)?;
        TokenClient::new(&env, &token_x).transfer(&contract, &provider, &amount_x);
        TokenClient::new(&env, &token_y).transfer(&contract, &provider, &amount_y);

        Ok((amount_x, amount_y))
    }

    pub fn swap(
        env: Env,
        trader: Address,
        sell_token: Address,
        sell_amount: i128,
        min_buy_amount: i128,
    ) -> Result<i128, AmmError> {
        require_positive(sell_amount)?;
        if min_buy_amount < 0 {
            return Err(AmmError::InvalidAmount);
        }
        trader.require_auth();

        let token_x = read_token(&env, DataKey::TokenX)?;
        let token_y = read_token(&env, DataKey::TokenY)?;
        let contract = env.current_contract_address();
        let reserve_x = TokenClient::new(&env, &token_x).balance(&contract);
        let reserve_y = TokenClient::new(&env, &token_y).balance(&contract);

        if reserve_x == 0 || reserve_y == 0 {
            return Err(AmmError::InsufficientLiquidity);
        }

        let (buy_token, reserve_in, reserve_out) = if sell_token == token_x {
            (token_y.clone(), reserve_x, reserve_y)
        } else if sell_token == token_y {
            (token_x.clone(), reserve_y, reserve_x)
        } else {
            return Err(AmmError::InvalidTokenPair);
        };

        let amount_in_with_fee = apply_fee(sell_amount)?;
        let amount_out = compute_amount_out(amount_in_with_fee, reserve_in, reserve_out)?;
        if amount_out < min_buy_amount {
            return Err(AmmError::InsufficientOutputAmount);
        }

        TokenClient::new(&env, &sell_token).transfer(&trader, &contract, &sell_amount);
        TokenClient::new(&env, &buy_token).transfer(&contract, &trader, &amount_out);

        Ok(amount_out)
    }

    pub fn lp_balance(env: Env, provider: Address) -> i128 {
        read_lp_balance(&env, &provider)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn reserves(env: Env) -> Result<(i128, i128), AmmError> {
        let token_x = read_token(&env, DataKey::TokenX)?;
        let token_y = read_token(&env, DataKey::TokenY)?;
        let contract = env.current_contract_address();
        let reserve_x = TokenClient::new(&env, &token_x).balance(&contract);
        let reserve_y = TokenClient::new(&env, &token_y).balance(&contract);
        Ok((reserve_x, reserve_y))
    }
}

fn read_token(env: &Env, key: DataKey) -> Result<Address, AmmError> {
    env.storage()
        .instance()
        .get(&key)
        .ok_or(AmmError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_lp_balance(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::LpBalance(provider.clone()))
        .unwrap_or(0)
}

fn set_lp_balance(env: &Env, provider: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::LpBalance(provider.clone()), &amount);
}

fn mint_lp(env: &Env, provider: &Address, amount: i128) -> Result<(), AmmError> {
    let current_balance = read_lp_balance(env, provider);
    let new_balance = current_balance
        .checked_add(amount)
        .ok_or(AmmError::ArithmeticOverflow)?;
    let total_supply = read_total_supply(env);
    let new_supply = total_supply
        .checked_add(amount)
        .ok_or(AmmError::ArithmeticOverflow)?;
    set_lp_balance(env, provider, new_balance);
    env.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
    Ok(())
}

fn burn_lp(env: &Env, provider: &Address, amount: i128) -> Result<(), AmmError> {
    let current_balance = read_lp_balance(env, provider);
    if current_balance < amount {
        return Err(AmmError::InsufficientLpBalance);
    }
    let total_supply = read_total_supply(env);
    let new_balance = current_balance - amount;
    let new_supply = total_supply - amount;
    set_lp_balance(env, provider, new_balance);
    env.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
    Ok(())
}

fn require_positive(amount: i128) -> Result<(), AmmError> {
    if amount <= 0 {
        return Err(AmmError::InvalidAmount);
    }
    Ok(())
}

fn apply_fee(amount: i128) -> Result<i128, AmmError> {
    let adjusted = amount
        .checked_mul(FEE_NUMERATOR)
        .ok_or(AmmError::ArithmeticOverflow)?
        .checked_div(FEE_DENOMINATOR)
        .ok_or(AmmError::ArithmeticOverflow)?;
    if adjusted <= 0 {
        return Err(AmmError::InvalidAmount);
    }
    Ok(adjusted)
}

fn compute_amount_out(
    amount_in: i128,
    reserve_in: i128,
    reserve_out: i128,
) -> Result<i128, AmmError> {
    if amount_in <= 0 {
        return Err(AmmError::InvalidAmount);
    }
    let numerator = amount_in
        .checked_mul(reserve_out)
        .ok_or(AmmError::ArithmeticOverflow)?;
    let denominator = reserve_in
        .checked_add(amount_in)
        .ok_or(AmmError::ArithmeticOverflow)?;
    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(AmmError::ArithmeticOverflow)?;
    if amount_out <= 0 {
        return Err(AmmError::InsufficientOutputAmount);
    }
    Ok(amount_out)
}

fn integer_sqrt(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut n = value as u128;
    let mut x = n;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + n / x) >> 1;
    }
    x as i128
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        contract, contracterror, contractimpl, contracttype, testutils::Address as _, Address, Env,
    };

    #[contracttype]
    #[derive(Clone)]
    pub enum TokenDataKey {
        Initialized,
        Balance(Address),
    }

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum TokenError {
        AlreadyInitialized = 1,
        InvalidAmount = 2,
        InsufficientBalance = 3,
    }

    #[contract]
    pub struct TestToken;

    #[contractimpl]
    impl TestToken {
        pub fn initialize(env: Env, owner: Address, balance: i128) -> Result<(), TokenError> {
            if env.storage().instance().has(&TokenDataKey::Initialized) {
                return Err(TokenError::AlreadyInitialized);
            }
            if balance < 0 {
                return Err(TokenError::InvalidAmount);
            }
            env.storage()
                .instance()
                .set(&TokenDataKey::Initialized, &true);
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(owner), &balance);
            Ok(())
        }

        pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), TokenError> {
            if amount <= 0 {
                return Err(TokenError::InvalidAmount);
            }
            let current = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(to.clone()))
                .unwrap_or(0);
            let next = current
                .checked_add(amount)
                .ok_or(TokenError::InvalidAmount)?;
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(to), &next);
            Ok(())
        }

        pub fn transfer(
            env: Env,
            from: Address,
            to: Address,
            amount: i128,
        ) -> Result<(), TokenError> {
            if amount <= 0 {
                return Err(TokenError::InvalidAmount);
            }
            from.require_auth();
            let current = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(from.clone()))
                .unwrap_or(0);
            if current < amount {
                return Err(TokenError::InsufficientBalance);
            }
            let next_from = current - amount;
            let next_to = env
                .storage()
                .persistent()
                .get(&TokenDataKey::Balance(to.clone()))
                .unwrap_or(0)
                .checked_add(amount)
                .ok_or(TokenError::InvalidAmount)?;
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(from), &next_from);
            env.storage()
                .persistent()
                .set(&TokenDataKey::Balance(to), &next_to);
            Ok(())
        }

        pub fn balance(env: Env, who: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&TokenDataKey::Balance(who))
                .unwrap_or(0)
        }
    }

    struct Fixture {
        env: Env,
        token_x: TestTokenClient<'static>,
        token_y: TestTokenClient<'static>,
        amm: ConstantProductAmmClient<'static>,
        alice: Address,
        bob: Address,
    }

    fn setup() -> Fixture {
        let env = Env::default();
        env.mock_all_auths();

        let alice = Address::random(&env);
        let bob = Address::random(&env);

        let token_x_id = env.register_contract(None, TestToken);
        let token_y_id = env.register_contract(None, TestToken);
        let token_x = TestTokenClient::new(&env, &token_x_id);
        let token_y = TestTokenClient::new(&env, &token_y_id);

        token_x.initialize(&alice, &10_000).unwrap();
        token_x.mint(&bob, &5_000).unwrap();
        token_y.initialize(&alice, &10_000).unwrap();
        token_y.mint(&bob, &5_000).unwrap();

        let amm_id = env.register_contract(None, ConstantProductAmm);
        let amm = ConstantProductAmmClient::new(&env, &amm_id);
        amm.initialize(&token_x_id, &token_y_id).unwrap();

        Fixture {
            env,
            token_x,
            token_y,
            amm,
            alice,
            bob,
        }
    }

    #[test]
    fn add_initial_liquidity_mints_lp_tokens() {
        let f = setup();

        let minted = f.amm.add_liquidity(&f.alice, &1_000, &1_000).unwrap();

        assert_eq!(minted, 1_000);
        assert_eq!(f.amm.lp_balance(&f.alice), 1_000);
        assert_eq!(f.amm.total_supply(), 1_000);
        assert_eq!(f.token_x.balance(&f.alice), 9_000);
        assert_eq!(f.token_y.balance(&f.alice), 9_000);

        let (reserve_x, reserve_y) = f.amm.reserves().unwrap();
        assert_eq!(reserve_x, 1_000);
        assert_eq!(reserve_y, 1_000);
    }

    #[test]
    fn add_liquidity_requires_exact_ratio() {
        let f = setup();

        f.amm.add_liquidity(&f.alice, &1_000, &2_000).unwrap_err();
    }

    #[test]
    fn swap_applies_fee_and_price_impact() {
        let f = setup();

        f.amm.add_liquidity(&f.alice, &10_000, &10_000).unwrap();

        let out = f
            .amm
            .swap(&f.bob, &f.token_x.contract_id(), &1_000, &900)
            .unwrap();

        assert_eq!(out, 906);
        assert_eq!(f.token_y.balance(&f.bob), 5_906);
        assert_eq!(f.token_x.balance(&f.bob), 4_000);

        let (reserve_x, reserve_y) = f.amm.reserves().unwrap();
        assert_eq!(reserve_x, 11_000);
        assert_eq!(reserve_y, 9_094);
    }

    #[test]
    fn remove_liquidity_returns_proportional_amounts() {
        let f = setup();

        f.amm.add_liquidity(&f.alice, &6_000, &6_000).unwrap();
        f.amm.add_liquidity(&f.bob, &3_000, &3_000).unwrap();

        let (returned_x, returned_y) = f.amm.remove_liquidity(&f.bob, &1_000).unwrap();

        assert_eq!(returned_x, 1_000);
        assert_eq!(returned_y, 1_000);
        assert_eq!(f.amm.lp_balance(&f.bob), 2_000);
        assert_eq!(f.amm.total_supply(), 5_000);
        assert_eq!(f.token_x.balance(&f.bob), 3_000);
        assert_eq!(f.token_y.balance(&f.bob), 3_000);
    }
}
