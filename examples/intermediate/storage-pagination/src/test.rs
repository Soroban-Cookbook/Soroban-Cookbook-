#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

fn setup() -> (Env, StoragePaginationClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(StoragePagination, ());
    let client = StoragePaginationClient::new(&env, &contract_id);
    (env, client)
}

fn item(n: u32) -> Symbol {
    match n {
        0 => symbol_short!("item0"),
        1 => symbol_short!("item1"),
        2 => symbol_short!("item2"),
        3 => symbol_short!("item3"),
        4 => symbol_short!("item4"),
        5 => symbol_short!("item5"),
        6 => symbol_short!("item6"),
        7 => symbol_short!("item7"),
        8 => symbol_short!("item8"),
        9 => symbol_short!("item9"),
        10 => symbol_short!("item10"),
        11 => symbol_short!("item11"),
        12 => symbol_short!("item12"),
        13 => symbol_short!("item13"),
        14 => symbol_short!("item14"),
        15 => symbol_short!("item15"),
        16 => symbol_short!("item16"),
        17 => symbol_short!("item17"),
        18 => symbol_short!("item18"),
        19 => symbol_short!("item19"),
        20 => symbol_short!("item20"),
        21 => symbol_short!("item21"),
        22 => symbol_short!("item22"),
        23 => symbol_short!("item23"),
        24 => symbol_short!("item24"),
        _ => symbol_short!("itemX"),
    }
}

#[test]
fn test_add_items_and_count() {
    let (_env, client) = setup();

    assert_eq!(client.count(), 0);
    assert_eq!(client.add_item(&item(0)), 0);
    assert_eq!(client.add_item(&item(1)), 1);
    assert_eq!(client.add_item(&item(2)), 2);
    assert_eq!(client.count(), 3);
}

#[test]
fn test_first_page() {
    let (_env, client) = setup();
    for i in 0..10 {
        client.add_item(&item(i));
    }

    let page = client.list(&5, &None);
    assert_eq!(page.items.len(), 5);
    assert_eq!(page.items.get(0).unwrap(), item(0));
    assert_eq!(page.items.get(4).unwrap(), item(4));
    assert!(page.next_cursor.is_some());
}

#[test]
fn test_returned_cursor_and_second_page() {
    let (_env, client) = setup();
    for i in 0..10 {
        client.add_item(&item(i));
    }

    let first = client.list(&4, &None);
    assert_eq!(first.items.len(), 4);
    let cursor = first.next_cursor.clone().expect("expected next cursor");

    let second = client.list(&4, &Some(cursor));
    assert_eq!(second.items.len(), 4);
    assert_eq!(second.items.get(0).unwrap(), item(4));
    assert_eq!(second.items.get(3).unwrap(), item(7));
    assert!(second.next_cursor.is_some());
}

#[test]
fn test_partial_final_page() {
    let (_env, client) = setup();
    for i in 0..10 {
        client.add_item(&item(i));
    }

    let first = client.list(&8, &None);
    let cursor = first.next_cursor.expect("expected next cursor");
    let last = client.list(&8, &Some(cursor));

    assert_eq!(last.items.len(), 2);
    assert_eq!(last.items.get(0).unwrap(), item(8));
    assert_eq!(last.items.get(1).unwrap(), item(9));
    assert!(last.next_cursor.is_none());
}

#[test]
fn test_empty_collection() {
    let (_env, client) = setup();
    let page = client.list(&10, &None);
    assert_eq!(page.items.len(), 0);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_cursor_beyond_collection() {
    let (_env, client) = setup();
    for i in 0..5 {
        client.add_item(&item(i));
    }

    let cursor = client.cursor_from_index(&10);
    let page = client.list(&5, &Some(cursor));
    assert_eq!(page.items.len(), 0);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_cursor_exactly_at_end() {
    let (_env, client) = setup();
    for i in 0..5 {
        client.add_item(&item(i));
    }

    let cursor = client.cursor_from_index(&5);
    let page = client.list(&5, &Some(cursor));
    assert_eq!(page.items.len(), 0);
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_page_size_zero_errors() {
    let (_env, client) = setup();
    client.add_item(&item(0));
    let err = client.try_list(&0, &None).unwrap_err();
    assert_eq!(err, Ok(PaginationError::InvalidPageSize));
}

#[test]
fn test_page_size_cap() {
    let (_env, client) = setup();
    client.add_item(&item(0));
    let err = client.try_list(&(MAX_PAGE_SIZE + 1), &None).unwrap_err();
    assert_eq!(err, Ok(PaginationError::InvalidPageSize));

    let page = client.list(&MAX_PAGE_SIZE, &None);
    assert_eq!(page.items.len(), 1);
}

#[test]
fn test_malformed_cursor() {
    let (env, client) = setup();
    client.add_item(&item(0));

    let wrong_len = Bytes::from_slice(&env, b"SPG1");
    assert_eq!(
        client.try_list(&5, &Some(wrong_len)).unwrap_err(),
        Ok(PaginationError::InvalidCursor)
    );

    let wrong_magic = Bytes::from_array(&env, &[b'X', b'X', b'X', b'X', 0, 0, 0, 0]);
    assert_eq!(
        client.try_list(&5, &Some(wrong_magic)).unwrap_err(),
        Ok(PaginationError::InvalidCursor)
    );

    let truncated = Bytes::from_slice(&env, &[b'S', b'P', b'G', b'1', 0, 0, 0]);
    assert_eq!(
        client.try_list(&5, &Some(truncated)).unwrap_err(),
        Ok(PaginationError::InvalidCursor)
    );
}

#[test]
fn test_deterministic_cursor_round_trip() {
    let (env, client) = setup();

    let c0 = client.cursor_from_index(&0);
    let c7 = client.cursor_from_index(&7);
    let c0_again = client.cursor_from_index(&0);

    assert_eq!(c0, c0_again);
    assert_ne!(c0, c7);
    assert_eq!(decode_cursor(&c0).unwrap(), 0);
    assert_eq!(decode_cursor(&c7).unwrap(), 7);
    assert_eq!(c7, encode_cursor(&env, 7));
}

#[test]
fn test_full_pagination_no_duplicates_or_gaps() {
    let (_env, client) = setup();
    const N: u32 = 25;
    const PAGE: u32 = 7;

    for i in 0..N {
        client.add_item(&item(i));
    }

    let mut cursor: Option<Bytes> = None;
    let mut collected: [Option<Symbol>; N as usize] = core::array::from_fn(|_| None);
    let mut count = 0u32;
    let mut pages = 0u32;

    loop {
        let page = client.list(&PAGE, &cursor);
        pages += 1;
        assert!(page.items.len() <= PAGE);

        for i in 0..page.items.len() {
            let value = page.items.get(i).unwrap();
            let expected_index = count;
            assert_eq!(
                value,
                item(expected_index),
                "unexpected item at position {count}"
            );
            assert!(
                collected[expected_index as usize].is_none(),
                "duplicate at index {expected_index}"
            );
            collected[expected_index as usize] = Some(value);
            count += 1;
        }

        match page.next_cursor {
            Some(next) => {
                assert_eq!(page.items.len(), PAGE, "non-final pages must be full");
                // Cursor must advance strictly past items already returned.
                assert_eq!(decode_cursor(&next).unwrap(), count);
                cursor = Some(next);
            }
            None => break,
        }
    }

    assert_eq!(count, N);
    assert_eq!(pages, 4); // 7 + 7 + 7 + 4
    for i in 0..N {
        assert_eq!(collected[i as usize], Some(item(i)));
    }
}

#[test]
fn test_get_item_behavior() {
    let (_env, client) = setup();
    client.add_item(&item(0));
    client.add_item(&item(1));
    client.add_item(&item(2));

    assert_eq!(client.get_item(&0), item(0));
    assert_eq!(client.get_item(&1), item(1));
    assert_eq!(client.get_item(&2), item(2));
}

#[test]
#[should_panic(expected = "Index out of bounds")]
fn test_get_item_out_of_bounds() {
    let (_env, client) = setup();
    client.add_item(&item(0));
    let _ = client.get_item(&10);
}
