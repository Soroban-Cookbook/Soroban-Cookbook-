#c[cfg_attr(not(test), no_std]]

use soroban_sdk::Env;

pub fn measure<F>(env: &Env, f: F) -> u64 where F: FnOnce(&Env) {
    let b = env.budget();
    b.reset();
    f(env);
    b.get_instruction_count()
}
#[cfg(test)]
mod test;