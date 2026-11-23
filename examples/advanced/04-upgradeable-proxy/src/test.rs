#![cfg(test)]
#![allow(deprecated)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{ImplementationV1, ImplementationV2, ProxyContract, ProxyContractClient};

struct Contracts {
    proxy: ProxyContractClient<'static>,
    implementation_v1: Address,
    implementation_v2: Address,
    admin: Address,
}

fn setup() -> Contracts {
    let env = Env::default();
    env.mock_all_auths();
    let implementation_v1 = env.register(ImplementationV1, ());
    let implementation_v2 = env.register(ImplementationV2, ());
    let proxy = env.register(ProxyContract, ());
    let admin = Address::generate(&env);
    let client = ProxyContractClient::new(&env, &proxy);
    client.init(&admin, &implementation_v1);
    Contracts {
        proxy: client,
        implementation_v1,
        implementation_v2,
        admin,
    }
}

#[test]
fn forwards_calls_to_v1() {
    let contracts = setup();
    assert_eq!(
        contracts.proxy.get_implementation(),
        contracts.implementation_v1
    );
    assert_eq!(contracts.proxy.add(&5, &3), 8);
    assert_eq!(contracts.proxy.subtract(&10, &4), 6);
    assert_eq!(contracts.proxy.increment(&7), 7);
    assert_eq!(contracts.proxy.counter(), 7);
}

#[test]
fn upgrade_preserves_proxy_storage_and_changes_behavior() {
    let contracts = setup();
    contracts.proxy.increment(&7);
    contracts.proxy.upgrade(&contracts.implementation_v2);

    assert_eq!(
        contracts.proxy.get_implementation(),
        contracts.implementation_v2
    );
    assert_eq!(contracts.proxy.counter(), 7);
    assert_eq!(contracts.proxy.multiply(&6, &7), 42);
    assert_eq!(contracts.proxy.increment(&3), 13);
    assert_eq!(contracts.proxy.add(&2, &4), 6);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn initialization_is_one_time() {
    let contracts = setup();
    contracts
        .proxy
        .init(&contracts.admin, &contracts.implementation_v2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn upgrade_requires_admin_auth() {
    let env = Env::default();
    let implementation_v1 = env.register(ImplementationV1, ());
    let implementation_v2 = env.register(ImplementationV2, ());
    let proxy = env.register(ProxyContract, ());
    let client = ProxyContractClient::new(&env, &proxy);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin, &implementation_v1);
    env.set_auths(&[]);
    client.upgrade(&implementation_v2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn init_requires_admin_auth() {
    let env = Env::default();
    // do NOT call mock_all_auths
    let implementation_v1 = env.register(ImplementationV1, ());
    let proxy = env.register(ProxyContract, ());
    let client = ProxyContractClient::new(&env, &proxy);
    let admin = Address::generate(&env);
    client.init(&admin, &implementation_v1); // should panic — no auth mocked
}

#[test]
fn init_with_auth_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let implementation_v1 = env.register(ImplementationV1, ());
    let proxy = env.register(ProxyContract, ());
    let client = ProxyContractClient::new(&env, &proxy);
    let admin = Address::generate(&env);
    client.init(&admin, &implementation_v1);
    assert_eq!(client.get_implementation(), implementation_v1);
}
