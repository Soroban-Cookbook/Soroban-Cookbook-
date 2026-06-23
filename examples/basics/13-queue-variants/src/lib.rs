#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol,
};

// Bounded queue -----------------------------------------------------------

#[contract]
pub struct BoundedQueueContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DropPolicy {
    DropOldest,
    DropNewest,
}

#[cfg(test)]
mod test;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BQKey {
    Meta,
    Element(i128),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BQMeta {
    pub capacity: i128,
    pub head: i128,
    pub tail: i128,
    pub len: i128,
    pub drop_policy: DropPolicy,
}

#[contractimpl]
impl BoundedQueueContract {
    pub fn initialize(env: Env, capacity: i128, drop_policy: DropPolicy) {
        if capacity <= 0 {
            panic!("capacity must be > 0");
        }
        if env.storage().instance().has(&BQKey::Meta) {
            panic!("already initialized");
        }
        let meta = BQMeta {
            capacity,
            head: 0,
            tail: 0,
            len: 0,
            drop_policy,
        };
        env.storage().instance().set(&BQKey::Meta, &meta);
    }

    fn meta(env: &Env) -> BQMeta {
        env.storage()
            .instance()
            .get(&BQKey::Meta)
            .expect("not initialized")
    }

    fn set_meta(env: &Env, m: &BQMeta) {
        env.storage().instance().set(&BQKey::Meta, m);
    }

    pub fn push(env: Env, value: Bytes) {
        let mut m = Self::meta(&env);
        if m.len >= m.capacity {
            // full
            match m.drop_policy {
                DropPolicy::DropNewest => panic!("Queue full"),
                DropPolicy::DropOldest => {
                    // drop oldest by advancing head and decrementing len
                    // remove storage at head index
                    env.storage().persistent().remove(&BQKey::Element(m.head));
                    m.head += 1;
                    m.len -= 1;
                }
            }
        }
        // insert at tail
        env.storage()
            .persistent()
            .set(&BQKey::Element(m.tail), &value);
        m.tail += 1;
        m.len += 1;
        Self::set_meta(&env, &m);
    }

    pub fn pop(env: Env) -> Bytes {
        let mut m = Self::meta(&env);
        if m.len == 0 {
            panic!("Empty");
        }
        let idx = m.head;
        let val: Bytes = env
            .storage()
            .persistent()
            .get(&BQKey::Element(idx))
            .expect("element missing");
        env.storage().persistent().remove(&BQKey::Element(idx));
        m.head += 1;
        m.len -= 1;
        Self::set_meta(&env, &m);
        val
    }

    pub fn len(env: Env) -> i128 {
        let m = Self::meta(&env);
        m.len
    }

    pub fn capacity(env: Env) -> i128 {
        let m = Self::meta(&env);
        m.capacity
    }
}

// Circular buffer ---------------------------------------------------------

#[contract]
pub struct CircularBufferContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CBKey {
    Meta,
    Element(i128),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CBMeta {
    pub capacity: i128,
    pub head: i128,
    pub tail: i128,
    pub len: i128,
}

#[contractimpl]
impl CircularBufferContract {
    pub fn initialize(env: Env, capacity: i128) {
        if capacity <= 0 {
            panic!("capacity must be > 0");
        }
        if env.storage().instance().has(&CBKey::Meta) {
            panic!("already initialized");
        }
        let meta = CBMeta {
            capacity,
            head: 0,
            tail: 0,
            len: 0,
        };
        env.storage().instance().set(&CBKey::Meta, &meta);
    }

    fn meta(env: &Env) -> CBMeta {
        env.storage()
            .instance()
            .get(&CBKey::Meta)
            .expect("not initialized")
    }

    fn set_meta(env: &Env, m: &CBMeta) {
        env.storage().instance().set(&CBKey::Meta, m);
    }

    pub fn push(env: Env, value: Bytes) {
        let mut m = Self::meta(&env);
        if m.len >= m.capacity {
            // will overwrite oldest
            // remove oldest element at head
            env.storage().persistent().remove(&CBKey::Element(m.head));
            m.head = (m.head + 1) % m.capacity;
            m.len -= 1;
        }
        env.storage()
            .persistent()
            .set(&CBKey::Element(m.tail), &value);
        m.tail = (m.tail + 1) % m.capacity;
        m.len += 1;
        Self::set_meta(&env, &m);
    }

    pub fn pop(env: Env) -> Bytes {
        let mut m = Self::meta(&env);
        if m.len == 0 {
            panic!("Empty");
        }
        let idx = m.head;
        let val: Bytes = env
            .storage()
            .persistent()
            .get(&CBKey::Element(idx))
            .expect("element missing");
        env.storage().persistent().remove(&CBKey::Element(idx));
        m.head = (m.head + 1) % m.capacity;
        m.len -= 1;
        Self::set_meta(&env, &m);
        val
    }

    pub fn len(env: Env) -> i128 {
        let m = Self::meta(&env);
        m.len
    }

    pub fn capacity(env: Env) -> i128 {
        let m = Self::meta(&env);
        m.capacity
    }
}
