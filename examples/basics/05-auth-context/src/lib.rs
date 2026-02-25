#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Vec, Symbol};

#[contract]
pub struct AuthContextContract;

#[contractimpl]
impl AuthContextContract {
    /// Returns the address of the invoker of this function.
    /// In Soroban, the standard way to retrieve and authenticate an invoker
    /// is by passing their `Address` as an argument and requiring their authorization.
    pub fn get_invoker(_env: Env, invoker: Address) -> Address {
        // This ensures the invoker has authorized this contract call
        invoker.require_auth();
        invoker
    }

    /// Returns the address of this current contract being executed.
    pub fn get_current_address(env: Env) -> Address {
        env.current_contract_address()
    }

    /// Demonstrates how to use `env.auths()` to inspect the authorization context.
    /// This is particularly useful in testing to verify that the correct
    /// authorizations were provided.
    pub fn get_auth_context(env: Env, invoker: Address) -> Vec<Address> {
        invoker.require_auth();
        // env.auths() returns a list of (Address, AuthorizedInvocation) tuples.
        // We can simply extract the addresses that have authorized the invocation.
        let mut results = Vec::new(&env);
        for auth in env.auths() {
            results.push_back(auth.0.clone());
        }
        results
    }

    /// An example of an admin-only operation using require_auth directly.
    pub fn admin_only_op(_env: Env, invoker: Address, expected_admin: Address) -> bool {
        // Enforce that the provided invoker is indeed the authorized caller
        invoker.require_auth();

        // Security check: only allow if the invoker matches the expected admin
        if invoker == expected_admin {
            // Admin-only logic would go here
            true
        } else {
            // In a real contract, this would typically panic or return an error
            false
        }
    }

    /// Explicitly checks nested authorization.
    /// This demonstrates that `require_auth` works across the entire call stack.
    pub fn check_nested_auth(_env: Env, user: Address) -> bool {
        // This will succeed if 'user' authorized this specific call,
        // even if it's coming through a proxy contract.
        user.require_auth();
        true
    }
}

/// A simple Proxy contract to demonstrate nested calls and how the auth
/// context (invoker) changes when one contract calls another.
#[contract]
pub struct ProxyContract;

#[contractimpl]
impl ProxyContract {
    /// Calls the `get_invoker` function on the `AuthContextContract`.
    /// When a user calls this proxy, and this proxy calls the AuthContextContract,
    /// the AuthContextContract will report this **Proxy's** address as the invoker,
    /// NOT the user's address.
    pub fn proxy_call(env: Env, target_contract: Address, user: Address) -> Address {
        // The proxy must first authenticate the user
        user.require_auth();

        // We create a client to call the target contract
        let client = AuthContextContractClient::new(&env, &target_contract);

        // When we call the target contract, we pass the user's address.
        // Because the target contract calls `user.require_auth()`, the SDK will
        // verify that the user authorized the entire call chain (User -> Proxy -> Target).
        client.check_nested_auth(&user);
        
        // Return the user address to confirm success
        user
    }
}

mod test;
