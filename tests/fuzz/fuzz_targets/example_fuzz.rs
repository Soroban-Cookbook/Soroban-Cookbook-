#![no_main]
#![allow(deprecated)]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{Env, Symbol};

// We will fuzz test a basic hello-world to demonstrate the fuzzing infrastructure.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if s.is_empty()
            || s.len() > 32
            || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return;
        }
        let env = Env::default();
        let to = Symbol::new(&env, s);

        let _ = hello_world::HelloContractClient::new(
            &env,
            &env.register_contract(None, hello_world::HelloContract),
        )
        .hello(&to);
    }
});
