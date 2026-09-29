use super::*;
use soroban_sdk::{contract, contractimpl, vec, IntoVal, TryFromVal};

#[contract]
struct Target;

#[contractimpl]
impl Target {
	pub fn echo(_env: Env, value: u32) -> u32 {
		value
	}
}

#[test]
fn test_aggregate_returns_results_in_call_order() {
	let env = Env::default();
	let target = env.register_contract(None, Target);
	let multicall = env.register_contract(None, Multicall);
	let client = MulticallClient::new(&env, &multicall);

	let mut first_args = vec![&env];
	first_args.push_back(11_u32.into_val(&env));
	let mut second_args = vec![&env];
	second_args.push_back(29_u32.into_val(&env));
	let calls = vec![
		&env,
		Call {
			target: target.clone(),
			function: Symbol::new(&env, "echo"),
			args: first_args,
		},
		Call {
			target,
			function: Symbol::new(&env, "echo"),
			args: second_args,
		},
	];

	let results = client.aggregate(&calls);

	assert_eq!(results.len(), 2);
	assert_eq!(
		u32::try_from_val(&env, &results.get(0).unwrap()).unwrap(),
		11
	);
	assert_eq!(
		u32::try_from_val(&env, &results.get(1).unwrap()).unwrap(),
		29
	);
}

#[test]
fn test_aggregate_empty_batch() {
	let env = Env::default();
	let multicall = env.register_contract(None, Multicall);
	let client = MulticallClient::new(&env, &multicall);

	assert!(client.aggregate(&Vec::new(&env)).is_empty());
}