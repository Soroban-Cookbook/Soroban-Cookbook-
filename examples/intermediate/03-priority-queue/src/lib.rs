#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol, Vec};

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
}

const CONTRACT_NS: Symbol = symbol_short!("pqueue");
const ACTION_HEAP: Symbol = symbol_short!("heap");

#[contract]
pub struct PriorityQueueContract;

#[contractimpl]
impl PriorityQueueContract {
    pub fn push(env: Env, item: Symbol, priority: i128) {
        let mut heap = Self::load_heap(&env);
        heap.push_back(HeapEntry { item, priority });
        Self::sift_up(&mut heap, heap.len() - 1);
        Self::save_heap(&env, &heap);
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

        if heap.len() > 0 {
            heap.set(0, &last);
            Self::sift_down(&mut heap, 0);
        }

        Self::save_heap(&env, &heap);
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

    fn load_heap(env: &Env) -> Vec<HeapEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::Heap)
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn save_heap(env: &Env, heap: &Vec<HeapEntry>) {
        env.storage().persistent().set(&DataKey::Heap, heap);
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
        heap.set(a, &b_val);
        heap.set(b, &a_val);
    }
}
