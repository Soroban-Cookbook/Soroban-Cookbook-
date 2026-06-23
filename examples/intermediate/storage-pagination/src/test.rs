#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_add_and_retrieve_items() {
    let env = Env::default();

    PaginationContract::add_item(&env, symbol_short!("item1"));
    PaginationContract::add_item(&env, symbol_short!("item2"));
    PaginationContract::add_item(&env, symbol_short!("item3"));

    assert_eq!(PaginationContract::count(&env), 3);
}

#[test]
fn test_pagination_first_page() {
    let env = Env::default();

    for i in 0..10 {
        let item = Symbol::short(format!("item{}", i).as_str());
        PaginationContract::add_item(&env, item);
    }

    let (items, next) = PaginationContract::list(&env, 0, 5);
    assert_eq!(items.len(), 5);
    assert_eq!(next, Some(5));
}

#[test]
fn test_pagination_second_page() {
    let env = Env::default();

    for i in 0..10 {
        let item = Symbol::short(format!("item{}", i).as_str());
        PaginationContract::add_item(&env, item);
    }

    let (items, next) = PaginationContract::list(&env, 5, 5);
    assert_eq!(items.len(), 5);
    assert_eq!(next, None);
}

#[test]
fn test_pagination_partial_page() {
    let env = Env::default();

    for i in 0..10 {
        let item = Symbol::short(format!("item{}", i).as_str());
        PaginationContract::add_item(&env, item);
    }

    let (items, next) = PaginationContract::list(&env, 8, 5);
    assert_eq!(items.len(), 2);
    assert_eq!(next, None);
}

#[test]
fn test_pagination_empty_collection() {
    let env = Env::default();

    let (items, next) = PaginationContract::list(&env, 0, 10);
    assert_eq!(items.len(), 0);
    assert_eq!(next, None);
}

#[test]
fn test_pagination_cursor_beyond_bounds() {
    let env = Env::default();

    for i in 0..5 {
        let item = Symbol::short(format!("item{}", i).as_str());
        PaginationContract::add_item(&env, item);
    }

    let (items, next) = PaginationContract::list(&env, 10, 5);
    assert_eq!(items.len(), 0);
    assert_eq!(next, None);
}

#[test]
fn test_get_item_by_index() {
    let env = Env::default();

    PaginationContract::add_item(&env, symbol_short!("A"));
    PaginationContract::add_item(&env, symbol_short!("B"));
    PaginationContract::add_item(&env, symbol_short!("C"));

    assert_eq!(PaginationContract::get_item(&env, 0), symbol_short!("A"));
    assert_eq!(PaginationContract::get_item(&env, 1), symbol_short!("B"));
    assert_eq!(PaginationContract::get_item(&env, 2), symbol_short!("C"));
}

#[test]
#[should_panic(expected = "Index out of bounds")]
fn test_get_item_out_of_bounds() {
    let env = Env::default();

    PaginationContract::add_item(&env, symbol_short!("A"));
    PaginationContract::get_item(&env, 10);
}

#[test]
#[should_panic(expected = "Page size must be greater than 0")]
fn test_invalid_page_size() {
    let env = Env::default();

    PaginationContract::add_item(&env, symbol_short!("A"));
    PaginationContract::list(&env, 0, 0);
}

#[test]
fn test_pagination_full_flow() {
    let env = Env::default();

    for i in 0..25 {
        let item = Symbol::short(format!("item{}", i).as_str());
        PaginationContract::add_item(&env, item);
    }

    let mut cursor = 0u32;
    let mut total_items = 0;

    loop {
        let (items, next) = PaginationContract::list(&env, cursor, 10);
        total_items += items.len();

        match next {
            Some(c) => cursor = c,
            None => break,
        }
    }

    assert_eq!(total_items, 25);
}
