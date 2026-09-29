#![no_std]

use soroban_sdk::{
	contract, contractimpl, contracttype, Address, Env, Symbol, Val, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Call {
	pub target: Address,
	pub function: Symbol,
	pub args: Vec<Val>,
}

#[contract]
pub struct Multicall;

#[contractimpl]
impl Multicall {
	/// Invoke each call in order and return the corresponding results.
	pub fn aggregate(env: Env, calls: Vec<Call>) -> Vec<Val> {
		let mut results = Vec::new(&env);
		for call in calls.iter() {
			let result: Val = env.invoke_contract(&call.target, &call.function, call.args);
			results.push_back(result);
		}
		results
	}
}

#[cfg(test)]
mod test;