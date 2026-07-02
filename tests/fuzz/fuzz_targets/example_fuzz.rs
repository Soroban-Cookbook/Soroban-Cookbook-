#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{Env, String};

// We will fuzz test a basic hello-world to demonstrate the fuzzing infrastructure.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let env = Env::default();
        let s_symbol = String::from_str(&env, s);
        
        // As long as this doesn't panic, the fuzz test passes.
        // Hello world takes a string and returns a vector containing "Hello" and the string.
        let _ = hello_world::HelloContractClient::new(&env, &env.register_contract(None, hello_world::HelloContract)).hello(&s_symbol);
    }
});
