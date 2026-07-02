//! DeFi Integration Test Suite
//!
//! Covers:
//! 1. Multi-protocol workflows (Swap + Lending)
//! 2. Flash loan flows (Simple, Arbitrage, Refinancing, and Reentrancy/Safety checks)
//! 3. Liquidity Mining reward distribution, rate adjustments, and status toggles

#![cfg(not(target_arch = "wasm32"))]
#![cfg(test)]

mod helpers;
mod mocks;

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    token, Address, Env, Symbol,
};
use simple_swap::SimpleSwapContractClient;
use lending_pool::LendingPoolClient;
use soroban_flash_loan::FlashLoanContractClient;
use liquidity_mining::LiquidityMiningClient;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let addr = token_id.address();
    let client = token::Client::new(env, &addr);
    (addr, client)
}

fn mint_token(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_addr).mint(to, &amount);
}

// ---------------------------------------------------------------------------
// Custom Flash Loan Receivers
// ---------------------------------------------------------------------------

// 1. Simple Receiver for standard repayment testing
#[contract]
pub struct SimpleReceiver;

#[contractimpl]
impl SimpleReceiver {
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let token_client = token::Client::new(&env, &token);
        // Approve the flash loan contract to pull the funds back (amount + fee)
        token_client.approve(
            &env.current_contract_address(),
            &initiator,
            &(amount + fee),
            &(env.ledger().sequence() + 1),
        );
    }
}

// 2. Arbitrage Receiver simulating a multi-protocol trade path
#[contract]
pub struct ArbitrageReceiver;

#[contractimpl]
impl ArbitrageReceiver {
    pub fn init_arbitrage(
        env: Env,
        swap1: Address,
        swap2: Address,
        token_b: Address,
    ) {
        env.storage().temporary().set(&symbol_short!("swap1"), &swap1);
        env.storage().temporary().set(&symbol_short!("swap2"), &swap2);
        env.storage().temporary().set(&symbol_short!("tok_b"), &token_b);
    }

    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let token_a_client = token::Client::new(&env, &token);
        let token_b_addr: Address = env.storage().temporary().get(&symbol_short!("tok_b")).unwrap();
        let token_b_client = token::Client::new(&env, &token_b_addr);

        let swap1: Address = env.storage().temporary().get(&symbol_short!("swap1")).unwrap();
        let swap2: Address = env.storage().temporary().get(&symbol_short!("swap2")).unwrap();

        // 1. Approve swap1 to spend the flash borrowed Token A
        token_a_client.approve(&env.current_contract_address(), &swap1, &amount, &(env.ledger().sequence() + 1));

        // 2. Swap A -> B on swap1 (invoker of swap is this contract)
        let swap1_client = SimpleSwapContractClient::new(&env, &swap1);
        let quote1 = swap1_client.quote(&token, &amount);
        swap1_client.swap(&env.current_contract_address(), &token, &amount, &quote1, &env.current_contract_address());

        // 3. Approve swap2 to spend the received Token B
        let balance_b = token_b_client.balance(&env.current_contract_address());
        token_b_client.approve(&env.current_contract_address(), &swap2, &balance_b, &(env.ledger().sequence() + 1));

        // 4. Swap B -> A on swap2
        let swap2_client = SimpleSwapContractClient::new(&env, &swap2);
        let quote2 = swap2_client.quote(&token_b_addr, &balance_b);
        swap2_client.swap(&env.current_contract_address(), &token_b_addr, &balance_b, &quote2, &env.current_contract_address());

        // 5. Approve flash loan contract to pull amount + fee
        let repayment = amount + fee;
        token_a_client.approve(&env.current_contract_address(), &initiator, &repayment, &(env.ledger().sequence() + 1));
    }
}

// 3. Refinancing Receiver simulating debt movement
#[contract]
pub struct RefinancingReceiver;

#[contractimpl]
impl RefinancingReceiver {
    pub fn init_refinance(
        env: Env,
        lending_pool1: Address,
        lending_pool2: Address,
        collateral_token: Address,
        collateral_amount: i128,
    ) {
        env.storage().temporary().set(&symbol_short!("pool1"), &lending_pool1);
        env.storage().temporary().set(&symbol_short!("pool2"), &lending_pool2);
        env.storage().temporary().set(&symbol_short!("col_tok"), &collateral_token);
        env.storage().temporary().set(&symbol_short!("col_amt"), &collateral_amount);
    }

    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let pool1: Address = env.storage().temporary().get(&symbol_short!("pool1")).unwrap();
        let pool2: Address = env.storage().temporary().get(&symbol_short!("pool2")).unwrap();
        let col_tok: Address = env.storage().temporary().get(&symbol_short!("col_tok")).unwrap();
        let col_amt: i128 = env.storage().temporary().get(&symbol_short!("col_amt")).unwrap();

        let debt_client = token::Client::new(&env, &token);
        let col_client = token::Client::new(&env, &col_tok);

        let pool1_client = LendingPoolClient::new(&env, &pool1);
        let pool2_client = LendingPoolClient::new(&env, &pool2);

        // 1. Repay debt in LendingPool 1
        debt_client.approve(&env.current_contract_address(), &pool1, &amount, &(env.ledger().sequence() + 1));
        pool1_client.repay(&env.current_contract_address(), &amount);

        // 2. Withdraw collateral from LendingPool 1
        pool1_client.withdraw(&env.current_contract_address(), &col_amt);

        // 3. Deposit collateral into LendingPool 2
        col_client.approve(&env.current_contract_address(), &pool2, &col_amt, &(env.ledger().sequence() + 1));
        pool2_client.deposit(&env.current_contract_address(), &col_amt);

        // 4. Borrow from LendingPool 2 (borrow enough to cover flash loan + fee)
        let total_repay = amount + fee;
        pool2_client.borrow(&env.current_contract_address(), &total_repay);

        // 5. Approve flash loan contract to pull the repayment
        debt_client.approve(&env.current_contract_address(), &initiator, &total_repay, &(env.ledger().sequence() + 1));
    }
}

// 4. Reentrant Receiver to test flash loan locks
#[contract]
pub struct ReentrantReceiver;

#[contractimpl]
impl ReentrantReceiver {
    pub fn on_flash_loan(env: Env, initiator: Address, token: Address, amount: i128, _fee: i128) {
        let flash_loan_client = FlashLoanContractClient::new(&env, &initiator);
        // Try to re-enter flash loan
        flash_loan_client.flash_loan(&env.current_contract_address(), &token, &amount);
    }
}

// 5. Bad Receiver that doesn't approve repayment
#[contract]
pub struct BadReceiver;

#[contractimpl]
impl BadReceiver {
    pub fn on_flash_loan(_env: Env, _initiator: Address, _token: Address, _amount: i128, _fee: i128) {}
}

// ===========================================================================
// SECTION 1: Multi-Protocol Workflows (Swap + Lending)
// ===========================================================================

#[test]
fn test_swap_then_deposit_to_lending() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_a, client_a) = create_token(&env, &admin);
    let (token_b, client_b) = create_token(&env, &admin);

    // Deploy and init Swap (1 Token A = 2 Token B)
    let swap_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_id);
    swap_client.initialize(&admin, &token_a, &token_b, &2i128, &1i128);

    // Deploy and init Lending Pool
    let lending_id = env.register_contract(None, lending_pool::LendingPool);
    let lending_client = LendingPoolClient::new(&env, &lending_id);
    lending_client.initialize(&0i128, &0i128, &0i128);

    // Fund the Swap Contract with Token B reserves
    mint_token(&env, &token_b, &swap_id, 10_000);

    // Fund the User with Token A
    mint_token(&env, &token_a, &user, 1_000);

    // User swaps 500 Token A for 1000 Token B
    client_a.approve(&user, &swap_id, &500, &(env.ledger().sequence() + 1));
    swap_client.swap(&user, &token_a, &500, &1000, &user);

    assert_eq!(client_b.balance(&user), 1000);
    assert_eq!(client_a.balance(&user), 500);

    // User deposits 1000 Token B to Lending Pool
    client_b.approve(&user, &lending_id, &1000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&user, &1000);

    // Verify User position in Lending Pool
    let position = lending_client.get_user_position(&user);
    assert_eq!(position.deposit, 1000);
    assert_eq!(position.borrow, 0);
}

#[test]
fn test_borrow_from_lending_then_swap() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let depositor = Address::generate(&env);

    let (token_a, client_a) = create_token(&env, &admin);
    let (token_b, client_b) = create_token(&env, &admin);

    // Swap Contract (1 Token B = 1 Token A)
    let swap_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_id);
    swap_client.initialize(&admin, &token_b, &token_a, &1i128, &1i128);

    // Lending Pool
    let lending_id = env.register_contract(None, lending_pool::LendingPool);
    let lending_client = LendingPoolClient::new(&env, &lending_id);
    lending_client.initialize(&0i128, &0i128, &0i128);

    // Fund Lending Pool with Token B reserves by having Depositor deposit
    mint_token(&env, &token_b, &depositor, 5_000);
    client_b.approve(&depositor, &lending_id, &5_000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&depositor, &5_000);

    // Fund Swap with Token A reserves
    mint_token(&env, &token_a, &swap_id, 2_000);

    // Fund User with Token A collateral
    mint_token(&env, &token_a, &user, 1_000);
    client_a.approve(&user, &lending_id, &1_000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&user, &1_000);

    // User borrows 500 Token B (up to 80% borrow limit of 1000 collateral = 800)
    lending_client.borrow(&user, &500);
    assert_eq!(lending_client.get_user_position(&user).borrow, 500);

    // Simulate user receiving the borrowed Token B (since lending pool is only a ledger keeper)
    mint_token(&env, &token_b, &user, 500);

    // User swaps 500 Token B for 500 Token A
    client_b.approve(&user, &swap_id, &500, &(env.ledger().sequence() + 1));
    swap_client.swap(&user, &token_b, &500, &500, &user);

    assert_eq!(client_a.balance(&user), 1500);
    assert_eq!(client_b.balance(&user), 0);
}

#[test]
fn test_lending_utilization_affects_borrow_rate() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let (token_a, client_a) = create_token(&env, &admin);

    // Lending Pool with base_rate = 2%, kink_rate = 10%, kink_utilization = 80%
    let lending_id = env.register_contract(None, lending_pool::LendingPool);
    let lending_client = LendingPoolClient::new(&env, &lending_id);
    lending_client.initialize(&2i128, &10i128, &80i128);

    // User1 deposits 1000 Token A
    mint_token(&env, &token_a, &user1, 1000);
    client_a.approve(&user1, &lending_id, &1000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&user1, &1000);

    // Initial utilization is 0%, borrow rate is base_rate = 2%
    assert_eq!(lending_client.get_utilization(), 0);
    assert_eq!(lending_client.get_borrow_rate(), 2);

    // User2 deposits 1000 Token A and borrows 500
    // Total deposits = 2000, total borrows = 500 -> utilization = 25%
    mint_token(&env, &token_a, &user2, 1000);
    client_a.approve(&user2, &lending_id, &1000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&user2, &1000);
    lending_client.borrow(&user2, &500);

    assert_eq!(lending_client.get_utilization(), 25);
    // Rate = base + utilization * kink_rate / kink_utilization = 2 + 25 * 10 / 80 = 2 + 3 = 5%
    assert_eq!(lending_client.get_borrow_rate(), 5);

    // User2 deposits 1000 more (User2 total deposits = 2000, total deposits = 3000)
    // User2 borrows 1100 more (User2 total borrows = 1600)
    // User1 borrows 800 (User1 total borrows = 800)
    // Total borrows = 2400. Total deposits = 3000 -> utilization = 80%
    mint_token(&env, &token_a, &user2, 1000);
    client_a.approve(&user2, &lending_id, &1000, &(env.ledger().sequence() + 1));
    lending_client.deposit(&user2, &1000);

    lending_client.borrow(&user2, &1100);
    lending_client.borrow(&user1, &800);

    assert_eq!(lending_client.get_utilization(), 80);
    assert_eq!(lending_client.get_borrow_rate(), 12); // 2 + 10 = 12%
}

#[test]
#[should_panic(expected = "slippage exceeded")]
fn test_swap_rate_slippage_prevention() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_a, client_a) = create_token(&env, &admin);
    let (token_b, _) = create_token(&env, &admin);

    let swap_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap_client = SimpleSwapContractClient::new(&env, &swap_id);
    swap_client.initialize(&admin, &token_a, &token_b, &1i128, &1i128); // 1:1 rate

    mint_token(&env, &token_a, &user, 100);

    // Requesting a minimum buy amount of 105 when selling 100 at 1:1 rate should fail due to slippage
    client_a.approve(&user, &swap_id, &100, &(env.ledger().sequence() + 1));
    swap_client.swap(&user, &token_a, &100, &105, &user);
}

// ===========================================================================
// SECTION 2: Flash Loan Flows
// ===========================================================================

#[test]
fn test_simple_flash_loan() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);

    let (token, client) = create_token(&env, &admin);

    let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_id);
    flash_loan_client.init(&admin, &50); // 0.5% fee

    let receiver_id = env.register_contract(None, SimpleReceiver);

    // Fund the flash loan pool
    mint_token(&env, &token, &flash_loan_id, 10_000);
    // Fund receiver with the fee (0.5% of 2000 = 10)
    mint_token(&env, &token, &receiver_id, 10);

    flash_loan_client.flash_loan(&receiver_id, &token, &2000);

    // Verify repayment and fee collection
    assert_eq!(client.balance(&flash_loan_id), 10010);
    assert_eq!(client.balance(&receiver_id), 0);
}

#[test]
fn test_flash_loan_arbitrage_workflow() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);

    let (token_a, client_a) = create_token(&env, &admin);
    let (token_b, client_b) = create_token(&env, &admin);

    // 1. Setup Flash Loan Contract (0.5% fee)
    let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_id);
    flash_loan_client.init(&admin, &50);

    // 2. Setup Swap 1 (Rate: 1 Token A buys 2 Token B)
    let swap1_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap1_client = SimpleSwapContractClient::new(&env, &swap1_id);
    swap1_client.initialize(&admin, &token_a, &token_b, &2i128, &1i128);

    // 3. Setup Swap 2 (Rate: 1 Token B buys 1 Token A)
    let swap2_id = env.register_contract(None, simple_swap::SimpleSwapContract);
    let swap2_client = SimpleSwapContractClient::new(&env, &swap2_id);
    swap2_client.initialize(&admin, &token_b, &token_a, &1i128, &1i128);

    // 4. Setup Arbitrage Receiver
    let receiver_id = env.register_contract(None, ArbitrageReceiver);

    // 5. Fund Contracts
    mint_token(&env, &token_a, &flash_loan_id, 10_000); // Flash Loan pool reserves
    mint_token(&env, &token_b, &swap1_id, 20_000);       // Swap 1 B reserves
    mint_token(&env, &token_a, &swap2_id, 20_000);       // Swap 2 A reserves

    // 6. Execute Arbitrage
    let receiver_client = ArbitrageReceiverClient::new(&env, &receiver_id);
    receiver_client.init_arbitrage(&swap1_id, &swap2_id, &token_b);
    flash_loan_client.flash_loan(&receiver_id, &token_a, &1000);

    assert_eq!(client_a.balance(&receiver_id), 995);
    assert_eq!(client_a.balance(&flash_loan_id), 10005);
}

#[test]
fn test_flash_loan_refinancing() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (collateral_token, client_c) = create_token(&env, &admin);
    let (debt_token, client_d) = create_token(&env, &admin);

    // 1. Deploy Flash Loan (0.5% fee)
    let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_id);
    flash_loan_client.init(&admin, &50);

    // 2. Deploy Lending Pools (Pool 1 and Pool 2)
    let pool1_id = env.register_contract(None, lending_pool::LendingPool);
    let pool1_client = LendingPoolClient::new(&env, &pool1_id);
    pool1_client.initialize(&0i128, &0i128, &0i128);

    let pool2_id = env.register_contract(None, lending_pool::LendingPool);
    let pool2_client = LendingPoolClient::new(&env, &pool2_id);
    pool2_client.initialize(&0i128, &0i128, &0i128);

    // 3. Deploy Refinancing Receiver
    let receiver_id = env.register_contract(None, RefinancingReceiver);

    // 4. Fund Contracts
    mint_token(&env, &debt_token, &flash_loan_id, 10_000); // Flash Loan pool
    mint_token(&env, &debt_token, &pool1_id, 10_000);      // Pool 1 debt reserves
    mint_token(&env, &debt_token, &pool2_id, 10_000);      // Pool 2 debt reserves

    // 5. Setup initial User Position in Pool 1
    // Deposit 1000 Collateral Token, borrow 500 Debt Token
    mint_token(&env, &collateral_token, &receiver_id, 1000);
    // Fund receiver with the fee (2) to cover the mock pool's non-transfer of tokens
    mint_token(&env, &debt_token, &receiver_id, 2);
    
    // We mock authorization to simulate the receiver acting on its own behalf
    client_c.approve(&receiver_id, &pool1_id, &1000, &(env.ledger().sequence() + 1));
    pool1_client.deposit(&receiver_id, &1000);
    pool1_client.borrow(&receiver_id, &500);

    assert_eq!(pool1_client.get_user_position(&receiver_id).borrow, 500);
    assert_eq!(pool1_client.get_user_position(&receiver_id).deposit, 1000);

    // 6. Execute Refinance
    let receiver_client = RefinancingReceiverClient::new(&env, &receiver_id);
    receiver_client.init_refinance(&pool1_id, &pool2_id, &collateral_token, &1000);
    flash_loan_client.flash_loan(&receiver_id, &debt_token, &500);

    // Verify Pool 1 is cleared
    let pos1 = pool1_client.get_user_position(&receiver_id);
    assert_eq!(pos1.deposit, 0);
    assert_eq!(pos1.borrow, 0);

    // Verify Pool 2 has the debt and collateral now
    let pos2 = pool2_client.get_user_position(&receiver_id);
    assert_eq!(pos2.deposit, 1000);
    assert_eq!(pos2.borrow, 502); // lending pool only tracks integer division
}

#[test]
#[should_panic]
fn test_flash_loan_fails_on_under_repayment() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);

    let (token, _) = create_token(&env, &admin);

    let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_id);
    flash_loan_client.init(&admin, &50);

    let receiver_id = env.register_contract(None, BadReceiver);

    mint_token(&env, &token, &flash_loan_id, 10_000);

    // This should panic because BadReceiver does not approve the repayment transfer
    flash_loan_client.flash_loan(&receiver_id, &token, &1000);
}

#[test]
#[should_panic]
fn test_flash_loan_reentrancy_prevention() {
    let env = helpers::setup_env();
    let admin = Address::generate(&env);

    let (token, _) = create_token(&env, &admin);

    let flash_loan_id = env.register_contract(None, soroban_flash_loan::FlashLoanContract);
    let flash_loan_client = FlashLoanContractClient::new(&env, &flash_loan_id);
    flash_loan_client.init(&admin, &50);

    let receiver_id = env.register_contract(None, ReentrantReceiver);

    mint_token(&env, &token, &flash_loan_id, 10_000);

    flash_loan_client.flash_loan(&receiver_id, &token, &1000);
}

// ===========================================================================
// SECTION 3: Liquidity Mining Flows
// ===========================================================================

#[test]
fn test_liquidity_mining_single_user() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, reward_client) = create_token(&env, &admin);

    // Fund mining contract with reward tokens
    mint_token(&env, &reward_token, &mining_id, 1_000_000);

    // Add pool (rate: 100 rewards per ledger)
    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128);

    // Stake 1000 LP tokens
    mint_token(&env, &lp_token, &user, 1000);
    lp_client.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));
    mining_client.stake(&1u32, &user, &1000);

    // Advance 10 ledgers
    env.ledger().with_mut(|l| l.sequence_number += 10);

    // Verify pending rewards (10 ledgers * 100 rate = 1000)
    assert_eq!(mining_client.pending_rewards(&1u32, &user), 1000);

    // Harvest and verify
    mining_client.harvest(&1u32, &user);
    assert_eq!(reward_client.balance(&user), 1000);
}

#[test]
fn test_liquidity_mining_two_users_proportional() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, reward_client) = create_token(&env, &admin);

    mint_token(&env, &reward_token, &mining_id, 1_000_000);
    mining_client.add_pool(&1u32, &lp_token, &reward_token, &1000i128); // 1000 rewards per ledger

    // Alice stakes 250 LP, Bob stakes 750 LP (1:3 split)
    mint_token(&env, &lp_token, &alice, 250);
    mint_token(&env, &lp_token, &bob, 750);

    lp_client.approve(&alice, &mining_id, &250, &(env.ledger().sequence() + 1));
    lp_client.approve(&bob, &mining_id, &750, &(env.ledger().sequence() + 1));

    mining_client.stake(&1u32, &alice, &250);
    mining_client.stake(&1u32, &bob, &750);

    // Advance 10 ledgers
    env.ledger().with_mut(|l| l.sequence_number += 10);

    // Total reward = 10 * 1000 = 10,000. Alice gets 25%, Bob gets 75%
    assert_eq!(mining_client.pending_rewards(&1u32, &alice), 2500);
    assert_eq!(mining_client.pending_rewards(&1u32, &bob), 7500);

    mining_client.harvest(&1u32, &alice);
    mining_client.harvest(&1u32, &bob);

    assert_eq!(reward_client.balance(&alice), 2500);
    assert_eq!(reward_client.balance(&bob), 7500);
}

#[test]
fn test_liquidity_mining_pause_resume() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, reward_client) = create_token(&env, &admin);

    mint_token(&env, &reward_token, &mining_id, 1_000_000);
    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128);

    mint_token(&env, &lp_token, &user, 1000);
    lp_client.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));
    mining_client.stake(&1u32, &user, &1000);

    // 1. Advance 5 ledgers (500 rewards)
    env.ledger().with_mut(|l| l.sequence_number += 5);

    // 2. Pause the pool
    mining_client.set_pool_active(&1u32, &false);

    // 3. Advance 10 ledgers while paused (no rewards should accrue)
    env.ledger().with_mut(|l| l.sequence_number += 10);

    // 4. Resume the pool
    mining_client.set_pool_active(&1u32, &true);

    // 5. Advance 5 ledgers after resume (500 rewards)
    env.ledger().with_mut(|l| l.sequence_number += 5);

    // Total expected = 500 + 500 = 1000
    assert_eq!(mining_client.pending_rewards(&1u32, &user), 1000);

    mining_client.harvest(&1u32, &user);
    assert_eq!(reward_client.balance(&user), 1000);
}

#[test]
fn test_liquidity_mining_rate_adjustment() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, reward_client) = create_token(&env, &admin);

    mint_token(&env, &reward_token, &mining_id, 1_000_000);
    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128); // R1 = 100

    mint_token(&env, &lp_token, &user, 1000);
    lp_client.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));
    mining_client.stake(&1u32, &user, &1000);

    // Earn 5 ledgers at rate 100 -> 500 rewards
    env.ledger().with_mut(|l| l.sequence_number += 5);

    // Adjust rate to 300
    mining_client.set_reward_rate(&1u32, &300i128);

    // Earn 5 ledgers at rate 300 -> 1500 rewards
    env.ledger().with_mut(|l| l.sequence_number += 5);

    // Total expected = 500 + 1500 = 2000
    assert_eq!(mining_client.pending_rewards(&1u32, &user), 2000);

    mining_client.harvest(&1u32, &user);
    assert_eq!(reward_client.balance(&user), 2000);
}

#[test]
fn test_liquidity_mining_multiple_pools() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp1, lp_client1) = create_token(&env, &admin);
    let (rw1, rw_client1) = create_token(&env, &admin);

    let (lp2, lp_client2) = create_token(&env, &admin);
    let (rw2, rw_client2) = create_token(&env, &admin);

    mint_token(&env, &rw1, &mining_id, 1_000_000);
    mint_token(&env, &rw2, &mining_id, 1_000_000);

    // Pool 1: rate 100
    mining_client.add_pool(&1u32, &lp1, &rw1, &100i128);
    // Pool 2: rate 200
    mining_client.add_pool(&2u32, &lp2, &rw2, &200i128);

    mint_token(&env, &lp1, &user, 1000);
    mint_token(&env, &lp2, &user, 1000);

    lp_client1.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));
    lp_client2.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));

    mining_client.stake(&1u32, &user, &1000);
    mining_client.stake(&2u32, &user, &1000);

    // Advance 10 ledgers
    env.ledger().with_mut(|l| l.sequence_number += 10);

    assert_eq!(mining_client.pending_rewards(&1u32, &user), 1000);
    assert_eq!(mining_client.pending_rewards(&2u32, &user), 2000);

    mining_client.harvest(&1u32, &user);
    mining_client.harvest(&2u32, &user);

    assert_eq!(rw_client1.balance(&user), 1000);
    assert_eq!(rw_client2.balance(&user), 2000);
}

#[test]
fn test_liquidity_mining_unstake_accumulates() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, reward_client) = create_token(&env, &admin);

    mint_token(&env, &reward_token, &mining_id, 1_000_000);
    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128);

    mint_token(&env, &lp_token, &user, 1000);
    lp_client.approve(&user, &mining_id, &1000, &(env.ledger().sequence() + 1));
    mining_client.stake(&1u32, &user, &1000);

    // Earn 10 ledgers
    env.ledger().with_mut(|l| l.sequence_number += 10);

    // Unstake all LP tokens. This should collect 1000 rewards into user's pending record.
    mining_client.unstake(&1u32, &user, &1000);

    // Staked balance is 0, but user info holds pending rewards
    let user_info = mining_client.get_user_info(&1u32, &user);
    assert_eq!(user_info.staked, 0);
    assert_eq!(user_info.pending_rewards, 1000);

    // Harvest the collected rewards
    mining_client.harvest(&1u32, &user);
    assert_eq!(reward_client.balance(&user), 1000);
}

#[test]
#[should_panic(expected = "Insufficient staked balance")]
fn test_liquidity_mining_insufficient_lp() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, lp_client) = create_token(&env, &admin);
    let (reward_token, _) = create_token(&env, &admin);

    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128);

    mint_token(&env, &lp_token, &user, 500);
    lp_client.approve(&user, &mining_id, &500, &(env.ledger().sequence() + 1));
    mining_client.stake(&1u32, &user, &500);

    // Try unstaking more than staked
    mining_client.unstake(&1u32, &user, &501i128);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_liquidity_mining_zero_stake_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mining_id = env.register_contract(None, liquidity_mining::LiquidityMining);
    let mining_client = LiquidityMiningClient::new(&env, &mining_id);
    mining_client.initialize(&admin);

    let (lp_token, _) = create_token(&env, &admin);
    let (reward_token, _) = create_token(&env, &admin);

    mining_client.add_pool(&1u32, &lp_token, &reward_token, &100i128);

    mining_client.stake(&1u32, &user, &0i128);
}
