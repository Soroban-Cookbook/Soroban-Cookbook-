#![allow(deprecated)]
use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env, Symbol, Vec};

fn setup(env: &Env) -> (PriorityQueueContractClient<'_>, Address) {
    let contract_id = env.register(PriorityQueueContract, ());
    env.mock_all_auths();
    let admin = Address::generate(env);
    let client = PriorityQueueContractClient::new(env, &contract_id);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_push_peek_pop_max_returns_highest_priority() {
    let env = Env::default();
    let (client, _) = setup(&env);

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
    let (client, _) = setup(&env);

    assert!(client.is_empty());
    client.push(&symbol_short!("first"), &3);
    client.push(&symbol_short!("second"), &2);

    assert_eq!(client.len(), 2);
    assert!(!client.is_empty());
}

/// Heap integrity: whatever order items go in, they must come out by
/// descending priority, and the root must always hold the maximum.
#[test]
fn test_heap_integrity_across_interleaved_pushes_and_pops() {
    let env = Env::default();
    let (client, _) = setup(&env);

    let items = [
        (symbol_short!("a"), 4i128),
        (symbol_short!("b"), 9),
        (symbol_short!("c"), 1),
        (symbol_short!("d"), 7),
        (symbol_short!("e"), 3),
        (symbol_short!("f"), 8),
    ];
    for (item, priority) in items.iter() {
        client.push(item, priority);
        // The root is the maximum priority pushed so far.
        let max_so_far = client.all().iter().map(|e| e.priority).max().unwrap();
        assert_eq!(client.all().get(0).unwrap().priority, max_so_far);
    }

    // Draining yields strictly descending priorities.
    let mut previous = i128::MAX;
    while !client.is_empty() {
        let expected_len = client.len() - 1;
        let top = client.all().get(0).unwrap().priority;
        assert!(
            top <= previous,
            "heap order violated: {top} after {previous}"
        );
        previous = top;
        client.pop_max();
        assert_eq!(client.len(), expected_len);
    }
}

#[test]
#[should_panic(expected = "Empty priority queue")]
fn test_pop_max_on_empty_queue_panics() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.pop_max();
}

#[test]
fn test_initialize_sets_admin() {
    let env = Env::default();
    let contract_id = env.register(PriorityQueueContract, ());
    env.mock_all_auths();
    let client = PriorityQueueContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let result = client.try_initialize(&admin);
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_initialize_cannot_be_called_twice() {
    let env = Env::default();
    let contract_id = env.register(PriorityQueueContract, ());
    env.mock_all_auths();
    let client = PriorityQueueContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(QueueError::AlreadyInitialized)));
}

#[test]
fn test_bulk_push_inserts_multiple_items() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let items = vec![
        &env,
        symbol_short!("a"),
        symbol_short!("b"),
        symbol_short!("c"),
    ];
    let priorities = vec![&env, 10i128, 5i128, 20i128];
    let result = client.try_bulk_push(&admin, &items, &priorities);
    assert_eq!(result, Ok(Ok(())));
    assert_eq!(client.len(), 3);
    assert_eq!(client.pop_max(), symbol_short!("c"));
    assert_eq!(client.pop_max(), symbol_short!("a"));
    assert_eq!(client.pop_max(), symbol_short!("b"));
}

#[test]
fn test_bulk_push_rejects_mismatched_lengths() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let items = vec![&env, symbol_short!("a"), symbol_short!("b")];
    let priorities = vec![&env, 10i128];
    let result = client.try_bulk_push(&admin, &items, &priorities);
    assert_eq!(result, Err(Ok(QueueError::MismatchedLengths)));
}

#[test]
fn test_bulk_push_rejects_unauthorized_caller() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let attacker = Address::generate(&env);

    let items: Vec<Symbol> = Vec::new(&env);
    let priorities: Vec<i128> = Vec::new(&env);
    let result = client.try_bulk_push(&attacker, &items, &priorities);
    assert_eq!(result, Err(Ok(QueueError::Unauthorized)));
}

#[test]
fn test_remove_existing_item() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.push(&symbol_short!("a"), &10);
    client.push(&symbol_short!("b"), &5);
    client.push(&symbol_short!("c"), &20);

    let result = client.try_remove(&admin, &symbol_short!("b"));
    assert_eq!(result, Ok(Ok(true)));
    assert_eq!(client.len(), 2);
    assert_eq!(client.pop_max(), symbol_short!("c"));
    assert_eq!(client.pop_max(), symbol_short!("a"));
}

#[test]
fn test_remove_non_existing_item() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.push(&symbol_short!("a"), &10);
    let result = client.try_remove(&admin, &symbol_short!("x"));
    assert_eq!(result, Ok(Ok(false)));
    assert_eq!(client.len(), 1);
}

#[test]
fn test_remove_from_empty_queue() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let result = client.try_remove(&admin, &symbol_short!("x"));
    assert_eq!(result, Ok(Ok(false)));
}

#[test]
fn test_remove_rejects_unauthorized() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let attacker = Address::generate(&env);

    client.push(&symbol_short!("a"), &10);
    let result = client.try_remove(&attacker, &symbol_short!("a"));
    assert_eq!(result, Err(Ok(QueueError::Unauthorized)));
}

#[test]
fn test_merge_combines_two_queues() {
    let env = Env::default();

    let queue1_id = env.register(PriorityQueueContract, ());
    let queue2_id = env.register(PriorityQueueContract, ());

    env.mock_all_auths();
    let admin = Address::generate(&env);

    let client1 = PriorityQueueContractClient::new(&env, &queue1_id);
    let client2 = PriorityQueueContractClient::new(&env, &queue2_id);

    client1.initialize(&admin);
    client2.initialize(&admin);

    client1.push(&symbol_short!("a"), &10);
    client1.push(&symbol_short!("b"), &5);

    client2.push(&symbol_short!("c"), &20);
    client2.push(&symbol_short!("d"), &1);

    let result = client1.try_merge(&admin, &queue2_id);
    assert_eq!(result, Ok(Ok(())));
    assert_eq!(client1.len(), 4);

    assert_eq!(client1.pop_max(), symbol_short!("c"));
    assert_eq!(client1.pop_max(), symbol_short!("a"));
    assert_eq!(client1.pop_max(), symbol_short!("b"));
    assert_eq!(client1.pop_max(), symbol_short!("d"));
}

#[test]
fn test_merge_empty_into_nonempty() {
    let env = Env::default();
    let queue1_id = env.register(PriorityQueueContract, ());
    let queue2_id = env.register(PriorityQueueContract, ());

    env.mock_all_auths();
    let admin = Address::generate(&env);

    let client1 = PriorityQueueContractClient::new(&env, &queue1_id);
    let client2 = PriorityQueueContractClient::new(&env, &queue2_id);

    client1.initialize(&admin);
    client2.initialize(&admin);

    client1.push(&symbol_short!("a"), &10);

    let result = client1.try_merge(&admin, &queue2_id);
    assert_eq!(result, Ok(Ok(())));
    assert_eq!(client1.len(), 1);
}

#[test]
fn test_merge_rejects_unauthorized() {
    let env = Env::default();
    let queue1_id = env.register(PriorityQueueContract, ());
    let queue2_id = env.register(PriorityQueueContract, ());

    env.mock_all_auths();
    let admin = Address::generate(&env);

    let client1 = PriorityQueueContractClient::new(&env, &queue1_id);
    let client2 = PriorityQueueContractClient::new(&env, &queue2_id);

    client1.initialize(&admin);
    client2.initialize(&admin);

    let attacker = Address::generate(&env);
    let result = client1.try_merge(&attacker, &queue2_id);
    assert_eq!(result, Err(Ok(QueueError::Unauthorized)));
}
