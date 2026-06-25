use super::*;
use soroban_sdk::testutils::Address;
use soroban_sdk::vec;


#[test]
fn lock_and_locked_balance_works() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let contract_id = env.register_contract(None, TokenLockContract);

    env.ledger().set_timestamp(1_000);

    // lock 100 until 1_500
    user.require_auth();
    env.invoke_contract(
        &contract_id,
        &TokenLockContract::lock,
        (100i128, 1_500u64),
    );

    let locked: i128 = env.invoke_contract(
        &contract_id,
        &TokenLockContract::locked_balance,
        (),
    );
    assert_eq!(locked, 100);

    let schedule: Vec<LockEntry> = env.invoke_contract(
        &contract_id,
        &TokenLockContract::lock_schedule,
        (),
    );
    assert_eq!(schedule.len(), 1);
    assert_eq!(schedule.get(0).amount, 100);
    assert_eq!(schedule.get(0).unlock_time, 1_500);
}

#[test]
fn unlock_mature_entries_only() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let contract_id = env.register_contract(None, TokenLockContract);

    // Create two locks, only one matures.
    env.ledger().set_timestamp(1_000);

    env.invoke_contract(
        &contract_id,
        &TokenLockContract::lock,
        (100i128, 1_500u64),
    );
    env.invoke_contract(
        &contract_id,
        &TokenLockContract::lock,
        (50i128, 1_200u64),
    );

    // t=1_250: only the 1_200 lock should unlock.
    env.ledger().set_timestamp(1_250);
    let unlocked: i128 = env.invoke_contract(&contract_id, &TokenLockContract::unlock, ());
    assert_eq!(unlocked, 50);

    let locked: i128 = env.invoke_contract(
        &contract_id,
        &TokenLockContract::locked_balance,
        (),
    );
    assert_eq!(locked, 100);

    let schedule: Vec<LockEntry> = env.invoke_contract(
        &contract_id,
        &TokenLockContract::lock_schedule,
        (),
    );
    assert_eq!(schedule.len(), 1);
    assert_eq!(schedule.get(0).amount, 100);
    assert_eq!(schedule.get(0).unlock_time, 1_500);
}


