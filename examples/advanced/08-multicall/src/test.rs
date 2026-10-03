extern crate std;

use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, AuthorizedFunction, Events as _},
    vec, IntoVal, TryFromVal,
};

// ---------------------------------------------------------------------------
// Target contract used as the callee in every batch
// ---------------------------------------------------------------------------

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    /// Add `amount` to `user`'s counter. Requires `user` auth.
    pub fn add(env: Env, user: Address, amount: u32) -> u32 {
        user.require_auth();
        let next = Self::get(env.clone(), user.clone()) + amount;
        env.storage().persistent().set(&user, &next);
        next
    }

    pub fn get(env: Env, user: Address) -> u32 {
        env.storage().persistent().get(&user).unwrap_or(0)
    }

    /// Writes state and then panics, so tests can check the write is rolled back.
    pub fn fail(env: Env, user: Address) {
        env.storage().persistent().set(&user, &999u32);
        panic!("fail");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct Setup {
    env: Env,
    multicall: MulticallContractClient<'static>,
    counter: CounterClient<'static>,
    user: Address,
}

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();
    let multicall = MulticallContractClient::new(&env, &env.register(MulticallContract, ()));
    let counter = CounterClient::new(&env, &env.register(Counter, ()));
    let user = Address::generate(&env);
    Setup {
        env,
        multicall,
        counter,
        user,
    }
}

fn add_call(s: &Setup, amount: u32) -> Call {
    Call {
        contract: s.counter.address.clone(),
        function: symbol_short!("add"),
        args: vec![&s.env, s.user.into_val(&s.env), amount.into_val(&s.env)],
    }
}

fn fail_call(s: &Setup) -> Call {
    Call {
        contract: s.counter.address.clone(),
        function: symbol_short!("fail"),
        args: vec![&s.env, s.user.into_val(&s.env)],
    }
}

fn as_u32(env: &Env, v: Val) -> u32 {
    u32::try_from_val(env, &v).unwrap()
}

fn error(e: MulticallError) -> Result<MulticallError, soroban_sdk::InvokeError> {
    Ok(e)
}

// ---------------------------------------------------------------------------
// aggregate (atomic)
// ---------------------------------------------------------------------------

#[test]
fn aggregate_executes_calls_in_order() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1), add_call(&s, 2), add_call(&s, 3)];

    let results = s.multicall.aggregate(&s.user, &calls);

    assert_eq!(results.len(), 3);
    assert_eq!(as_u32(&s.env, results.get(0).unwrap()), 1);
    assert_eq!(as_u32(&s.env, results.get(1).unwrap()), 3);
    assert_eq!(as_u32(&s.env, results.get(2).unwrap()), 6);
    assert_eq!(s.counter.get(&s.user), 6);
}

#[test]
fn aggregate_requires_caller_auth() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 5)];

    s.multicall.aggregate(&s.user, &calls);

    let auths = s.env.auths();
    assert_eq!(auths.len(), 1);
    let (addr, invocation) = &auths[0];
    assert_eq!(addr, &s.user);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            s.multicall.address.clone(),
            symbol_short!("aggregate"),
            (s.user.clone(), calls.clone()).into_val(&s.env),
        ))
    );
    // The nested counter call is covered by the same signature.
    assert_eq!(invocation.sub_invocations.len(), 1);
}

#[test]
fn aggregate_without_auth_is_rejected() {
    let env = Env::default();
    let multicall = MulticallContractClient::new(&env, &env.register(MulticallContract, ()));
    let counter = env.register(Counter, ());
    let user = Address::generate(&env);
    let calls = vec![
        &env,
        Call {
            contract: counter,
            function: symbol_short!("add"),
            args: vec![&env, user.into_val(&env), 1u32.into_val(&env)],
        },
    ];

    assert!(multicall.try_aggregate(&user, &calls).is_err());
}

#[test]
fn aggregate_reverts_whole_batch_on_failure() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1), fail_call(&s), add_call(&s, 2)];

    assert!(s.multicall.try_aggregate(&s.user, &calls).is_err());
    assert_eq!(s.counter.get(&s.user), 0);
}

#[test]
fn aggregate_rejects_empty_batch() {
    let s = setup();
    let res = s.multicall.try_aggregate(&s.user, &Vec::new(&s.env));
    assert_eq!(res.unwrap_err(), error(MulticallError::EmptyBatch));
}

#[test]
fn aggregate_rejects_oversized_batch() {
    let s = setup();
    let mut calls = Vec::new(&s.env);
    for _ in 0..=MAX_CALLS {
        calls.push_back(add_call(&s, 1));
    }
    let res = s.multicall.try_aggregate(&s.user, &calls);
    assert_eq!(res.unwrap_err(), error(MulticallError::TooManyCalls));
}

#[test]
fn aggregate_accepts_max_batch() {
    let s = setup();
    let mut calls = Vec::new(&s.env);
    for _ in 0..MAX_CALLS {
        calls.push_back(add_call(&s, 1));
    }
    s.env.cost_estimate().budget().reset_unlimited();
    s.multicall.aggregate(&s.user, &calls);
    assert_eq!(s.counter.get(&s.user), MAX_CALLS);
}

#[test]
fn aggregate_rejects_self_call() {
    let s = setup();
    let calls = vec![
        &s.env,
        Call {
            contract: s.multicall.address.clone(),
            function: symbol_short!("aggregate"),
            args: Vec::new(&s.env),
        },
    ];
    let res = s.multicall.try_aggregate(&s.user, &calls);
    assert_eq!(res.unwrap_err(), error(MulticallError::SelfCall));
}

#[test]
fn aggregate_emits_batch_event() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1), add_call(&s, 1)];
    s.multicall.aggregate(&s.user, &calls);

    assert_eq!(
        s.env
            .events()
            .all()
            .filter_by_contract(&s.multicall.address),
        vec![
            &s.env,
            (
                s.multicall.address.clone(),
                (symbol_short!("multicall"), s.user.clone()).into_val(&s.env),
                2u32.into_val(&s.env),
            ),
        ]
    );
}

// ---------------------------------------------------------------------------
// aggregate_results (best effort)
// ---------------------------------------------------------------------------

#[test]
fn aggregate_results_captures_failures() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1), fail_call(&s), add_call(&s, 2)];

    let results = s.multicall.aggregate_results(&s.user, &calls, &false);

    assert_eq!(results.len(), 3);
    let (ok0, data0) = results.get(0).unwrap();
    let (ok1, data1) = results.get(1).unwrap();
    let (ok2, data2) = results.get(2).unwrap();
    assert!(ok0);
    assert_eq!(as_u32(&s.env, data0), 1);
    assert!(!ok1);
    assert!(data1.is_void());
    assert!(ok2);
    assert_eq!(as_u32(&s.env, data2), 3);

    // The failed call's write (999) was rolled back; the successful ones stay.
    assert_eq!(s.counter.get(&s.user), 3);
}

#[test]
fn aggregate_results_require_success_reverts_batch() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1), fail_call(&s)];

    let res = s.multicall.try_aggregate_results(&s.user, &calls, &true);

    assert_eq!(res.unwrap_err(), error(MulticallError::CallFailed));
    assert_eq!(s.counter.get(&s.user), 0);
}

#[test]
fn aggregate_results_require_success_passes_when_all_succeed() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 4), add_call(&s, 6)];

    let results = s.multicall.aggregate_results(&s.user, &calls, &true);

    assert!(results.iter().all(|(ok, _)| ok));
    assert_eq!(s.counter.get(&s.user), 10);
}

#[test]
fn aggregate_results_captures_missing_function() {
    let s = setup();
    let calls = vec![
        &s.env,
        Call {
            contract: s.counter.address.clone(),
            function: symbol_short!("nope"),
            args: Vec::new(&s.env),
        },
    ];

    let results = s.multicall.aggregate_results(&s.user, &calls, &false);
    assert!(!results.get(0).unwrap().0);
}

#[test]
fn aggregate_results_requires_caller_auth() {
    let s = setup();
    let calls = vec![&s.env, add_call(&s, 1)];

    s.multicall.aggregate_results(&s.user, &calls, &false);

    let auths = s.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, s.user);
    assert_eq!(
        auths[0].1.function,
        AuthorizedFunction::Contract((
            s.multicall.address.clone(),
            Symbol::new(&s.env, "aggregate_results"),
            (s.user.clone(), calls.clone(), false).into_val(&s.env),
        ))
    );
}

#[test]
fn aggregate_results_validates_batch() {
    let s = setup();
    let res = s
        .multicall
        .try_aggregate_results(&s.user, &Vec::new(&s.env), &false);
    assert_eq!(res.unwrap_err(), error(MulticallError::EmptyBatch));
}
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
