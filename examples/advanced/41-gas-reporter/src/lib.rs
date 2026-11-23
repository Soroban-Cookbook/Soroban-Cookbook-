#![cfg_attr(target_family = "wasm", no_std)]

use soroban_sdk::Env;

pub fn measure<F>(env: &Env, f: F) -> u64
where
    F: FnOnce(&Env),
{
    let mut b = env.cost_estimate().budget();
    b.reset_default();
    f(env);
    b.cpu_instruction_cost()
}
#[cfg(test)]
mod test;
