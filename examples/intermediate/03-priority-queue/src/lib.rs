#![cfg_attr(target_family = "wasm", no_std)]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

const CONTRACT_NS: Symbol = symbol_short!("pqueue");
const ACTION_HEAP: Symbol = symbol_short!("heap");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum QueueError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    MismatchedLengths = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeapEntry {
    pub priority: i128,
    pub item: Symbol,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Heap,
    Admin,
}

#[contract]
pub struct PriorityQueueContract;

#[contractimpl]
impl PriorityQueueContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), QueueError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(QueueError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn push(env: Env, item: Symbol, priority: i128) {
        let mut heap = Self::load_heap(&env);
        heap.push_back(HeapEntry {
            item: item.clone(),
            priority,
        });
        let last_index = heap.len() - 1;
        Self::sift_up(&mut heap, last_index);
        Self::save_heap(&env, &heap);

        env.events().publish(
            (CONTRACT_NS, ACTION_HEAP, symbol_short!("push")),
            (item, priority),
        );
    }

    pub fn peek_max(env: Env) -> Option<Symbol> {
        Self::load_heap(&env).get(0).map(|entry| entry.item.clone())
    }

    pub fn pop_max(env: Env) -> Symbol {
        let mut heap = Self::load_heap(&env);
        let len = heap.len();
        if len == 0 {
            panic!("Empty priority queue");
        }

        let top = heap.get(0).unwrap();
        let top_item = top.item.clone();
        let last = heap.pop_back().unwrap();

        if !heap.is_empty() {
            heap.set(0, last);
            Self::sift_down(&mut heap, 0);
        }

        Self::save_heap(&env, &heap);

        env.events().publish(
            (CONTRACT_NS, ACTION_HEAP, symbol_short!("pop")),
            (top_item.clone(), top.priority),
        );

        top_item
    }

    pub fn len(env: Env) -> u32 {
        Self::load_heap(&env).len()
    }

    pub fn is_empty(env: Env) -> bool {
        Self::load_heap(&env).is_empty()
    }

    pub fn all(env: Env) -> Vec<HeapEntry> {
        Self::load_heap(&env)
    }

    pub fn bulk_push(
        env: Env,
        admin: Address,
        items: Vec<Symbol>,
        priorities: Vec<i128>,
    ) -> Result<(), QueueError> {
        Self::require_admin(&env, &admin)?;

        let items_len = items.len();
        let priorities_len = priorities.len();
        if items_len != priorities_len {
            return Err(QueueError::MismatchedLengths);
        }

        let mut heap = Self::load_heap(&env);
        for i in 0..items_len {
            heap.push_back(HeapEntry {
                item: items.get(i).unwrap(),
                priority: priorities.get(i).unwrap(),
            });
        }
        Self::heapify(&mut heap);
        Self::save_heap(&env, &heap);
        Ok(())
    }

    pub fn remove(env: Env, admin: Address, item: Symbol) -> Result<bool, QueueError> {
        Self::require_admin(&env, &admin)?;

        let mut heap = Self::load_heap(&env);
        let len = heap.len();
        if len == 0 {
            return Ok(false);
        }

        let mut found_index = None;
        for i in 0..len {
            if heap.get(i).unwrap().item == item {
                found_index = Some(i);
                break;
            }
        }

        match found_index {
            None => Ok(false),
            Some(index) => {
                let last = heap.pop_back().unwrap();
                if index < heap.len() {
                    heap.set(index, last);
                    Self::sift_up(&mut heap, index);
                    Self::sift_down(&mut heap, index);
                }
                Self::save_heap(&env, &heap);
                Ok(true)
            }
        }
    }

    pub fn merge(env: Env, admin: Address, other_queue: Address) -> Result<(), QueueError> {
        Self::require_admin(&env, &admin)?;

        let client = PriorityQueueContractClient::new(&env, &other_queue);
        let other_entries = client.all();

        let mut heap = Self::load_heap(&env);
        for i in 0..other_entries.len() {
            let entry = other_entries.get(i).unwrap();
            heap.push_back(entry);
        }
        Self::heapify(&mut heap);
        Self::save_heap(&env, &heap);
        Ok(())
    }

    fn load_heap(env: &Env) -> Vec<HeapEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::Heap)
            .unwrap_or_else(|| Vec::new(env))
    }

    fn save_heap(env: &Env, heap: &Vec<HeapEntry>) {
        env.storage().persistent().set(&DataKey::Heap, heap);
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), QueueError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(QueueError::NotInitialized)?;
        if *admin != stored_admin {
            return Err(QueueError::Unauthorized);
        }
        admin.require_auth();
        Ok(())
    }

    fn sift_up(heap: &mut Vec<HeapEntry>, mut index: u32) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if heap.get(index).unwrap().priority > heap.get(parent).unwrap().priority {
                Self::swap(heap, index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    fn sift_down(heap: &mut Vec<HeapEntry>, mut index: u32) {
        let len = heap.len();
        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut largest = index;

            if left < len && heap.get(left).unwrap().priority > heap.get(largest).unwrap().priority
            {
                largest = left;
            }
            if right < len
                && heap.get(right).unwrap().priority > heap.get(largest).unwrap().priority
            {
                largest = right;
            }
            if largest == index {
                break;
            }
            Self::swap(heap, index, largest);
            index = largest;
        }
    }

    fn swap(heap: &mut Vec<HeapEntry>, a: u32, b: u32) {
        let a_val = heap.get(a).unwrap();
        let b_val = heap.get(b).unwrap();
        heap.set(a, b_val);
        heap.set(b, a_val);
    }

    fn heapify(heap: &mut Vec<HeapEntry>) {
        let len = heap.len();
        if len <= 1 {
            return;
        }
        let start = len / 2;
        for i in (0..start).rev() {
            Self::sift_down(heap, i);
        }
    }
}

#[cfg(test)]
mod test;
