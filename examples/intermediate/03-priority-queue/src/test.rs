use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Env};

fn setup(env: &Env) -> PriorityQueueContractClient<'_> {
    let contract_id = env.register_contract(None, PriorityQueueContract);
    env.mock_all_auths();
    PriorityQueueContractClient::new(env, &contract_id)
}

#[test]
fn test_push_peek_pop_max_returns_highest_priority() {
    let env = Env::default();
    let client = setup(&env);

    client.push(&symbol_short!("low"), &1);
    client.push(&symbol_short!("medium"), &5);
    client.push(&symbol_short!("high"), &10);

    assert_eq!(client.peek_max(), Some(symbol_short!("high")));
    assert_eq!(client.pop_max(), symbol_short!("high"));
    assert_eq!(client.pop_max(), symbol_short!("medium"));
    assert_eq!(client.pop_max(), symbol_short!("low"));
    assert!(client.is_empty());
}

#[test]
fn test_len_and_is_empty_after_insertions() {
    let env = Env::default();
    let client = setup(&env);

    assert!(client.is_empty());
    client.push(&symbol_short!("first"), &3);
    client.push(&symbol_short!("second"), &2);

    assert_eq!(client.len(), 2);
    assert!(!client.is_empty());
}

#[test]
#[should_panic(expected = "Empty priority queue")]
fn test_pop_max_on_empty_queue_panics() {
    let env = Env::default();
    let client = setup(&env);
    client.pop_max();
}
