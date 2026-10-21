use soroban_sdk::Env;
use std::time::Instant;

pub struct PerfMetrics {
    pub execution_time_ns: u128,
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
}

impl PerfMetrics {
    pub fn print(&self, label: &str) {
        println!("--- Performance: {} ---", label);
        println!("Execution Time: {} ns", self.execution_time_ns);
        println!("CPU Instructions: {}", self.cpu_instructions);
        println!("Memory Bytes: {}", self.memory_bytes);
        println!("--------------------------");
    }
}

pub fn measure_execution<F, R>(env: &Env, f: F) -> (R, PerfMetrics)
where
    F: FnOnce() -> R,
{
    let mut budget = env.cost_estimate().budget();
    budget.reset_unlimited();

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    let cpu = budget.cpu_instruction_cost();
    let mem = budget.memory_bytes_cost();

    let metrics = PerfMetrics {
        execution_time_ns: elapsed.as_nanos(),
        cpu_instructions: cpu,
        memory_bytes: mem,
    };

    (result, metrics)
}
