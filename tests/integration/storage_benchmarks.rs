//! Storage operation benchmarks from PR #1020 / issue #791.
//! Kept as a crate-root module so it sits alongside `gas_analysis`.

use soroban_sdk::{symbol_short, vec, Bytes, Env};

#[cfg(test)]
#[test]
fn storage_benchmarks() {
    let env = Env::default();
    let id = env.register(storage_patterns::StorageContract, ());
    let client = storage_patterns::StorageContractClient::new(&env, &id);
    let key = symbol_short!("k");

    let measure = |f: &dyn Fn()| {
        env.cost_estimate().budget().reset_default();
        f();
        env.cost_estimate().budget().cpu_instruction_cost()
    };

    let persistent = measure(&|| client.set_persistent(&key, &1u64));
    let instance = measure(&|| client.set_instance(&key, &1u64));
    let temporary = measure(&|| client.set_temporary(&key, &1u64));
    println!(
        "persistent: {}, instance: {}, temporary: {}",
        persistent, instance, temporary
    );

    let mut values = vec![&env];
    for x in 0u32..10 {
        values.push_back(x);
    }
    let iter = measure(&|| {
        for x in 0..10 {
            let _ = values.get(x);
        }
    });
    println!("vec iteration cpu: {}", iter);

    let _data = Bytes::from_array(&env, &[0x41u8; 100]);
}
