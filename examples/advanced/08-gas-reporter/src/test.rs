use super::*;
use soroban_sdk::Env;

#[test]
fn test_measure() {
    let env = Env::default();
    let count = measure(&env, |_env | {
        let mut x = 0u64;
        for i in 0..100 { x = x.wrapping_add(i); }
        std::hint::black_box(x);
    });
    assert!(count > 0);
}