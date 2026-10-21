use super::*;
use soroban_sdk::{Bytes, Env};

#[test]
fn test_measure() {
    let env = Env::default();
    let count = measure(&env, |env| {
        env.crypto()
            .sha256(&Bytes::from_slice(env, b"metered operation"));
    });
    assert!(count > 0);
}
