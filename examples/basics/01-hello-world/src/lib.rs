#![no_std]

// Import core types and macros from the Soroban SDK
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    /// Says hello to the provided name.
    ///
    /// # Arguments
    /// * `env` - The contract environment, providing access to blockchain context
    /// * `to` - A String representing the name to greet
    ///
    /// # Returns
    /// A Vec containing two Strings: ["Hello", name]
    ///
    /// # Example
    /// ```
    /// hello(env, String::from_str(&env, "World"))
    /// // Returns: vec![env, "Hello", "World"]
    /// ```
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}

mod test;

#[cfg(test)]
mod smoke_tests {
    use super::*;
    use soroban_sdk::{vec, Env, String};

    #[test]
    fn smoke_hello_world() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let name = String::from_str(&env, "World");
        let result = client.hello(&name);

        assert_eq!(result, vec![&env, String::from_str(&env, "Hello"), name]);
    }
}
