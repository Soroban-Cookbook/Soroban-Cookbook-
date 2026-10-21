extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction},
    vec, IntoVal, Symbol,
};

fn setup() -> (Env, ComputationOptimizationClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let client =
        ComputationOptimizationClient::new(&env, &env.register(ComputationOptimization, ()));
    let admin = Address::generate(&env);
    client.init(&admin);
    (env, client, admin)
}

fn sample(env: &Env, n: i64) -> Vec<i64> {
    let mut values = Vec::new(env);
    for i in 0..n {
        // Mix of negative and positive values; min = -n, max = n - 2 (for n >= 2).
        values.push_back(if i % 2 == 0 { i } else { -i - 1 });
    }
    values
}

/// CPU instructions consumed by `f`, measured with a fresh budget.
fn cpu_cost(env: &Env, f: impl FnOnce()) -> u64 {
    env.cost_estimate().budget().reset_unlimited();
    f();
    env.cost_estimate().budget().cpu_instruction_cost()
}

// ---------------------------------------------------------------------------
// Algorithm optimization
// ---------------------------------------------------------------------------

#[test]
fn sum_to_naive_and_optimized_agree() {
    let (_env, client, _) = setup();
    for n in [0u64, 1, 2, 10, 100, 1_000] {
        assert_eq!(
            client.sum_to_naive(&n),
            client.sum_to_optimized(&n),
            "n = {n}"
        );
    }
    assert_eq!(client.sum_to_optimized(&100), 5050);
}

#[test]
fn sum_to_optimized_handles_u64_max_without_overflow() {
    let (_env, client, _) = setup();
    let n = u64::MAX as u128;
    assert_eq!(client.sum_to_optimized(&u64::MAX), n * (n + 1) / 2);
}

#[test]
fn pow_mod_naive_and_optimized_agree() {
    let (_env, client, _) = setup();
    for (base, exp, modulus) in [
        (2u64, 10u64, 1_000u64),
        (3, 0, 7),
        (0, 0, 7),
        (7, 13, 1),
        (123_456_789, 500, 1_000_000_007),
        (u64::MAX, 257, u64::MAX - 58),
    ] {
        assert_eq!(
            client.pow_mod_naive(&base, &exp, &modulus),
            client.pow_mod_optimized(&base, &exp, &modulus),
            "{base}^{exp} mod {modulus}"
        );
    }
    assert_eq!(client.pow_mod_optimized(&2, &10, &1_000), 24);
}

#[test]
fn pow_mod_optimized_handles_huge_exponent() {
    let (_env, client, _) = setup();
    // Fermat: a^(p-1) = 1 mod p. The naive version would need ~1e9 steps.
    let p = 1_000_000_007u64;
    assert_eq!(client.pow_mod_optimized(&5, &(p - 1), &p), 1);
}

#[test]
fn pow_mod_rejects_zero_modulus() {
    let (_env, client, _) = setup();
    assert_eq!(
        client.try_pow_mod_naive(&2, &3, &0),
        Err(Ok(ComputationError::InvalidModulus))
    );
    assert_eq!(
        client.try_pow_mod_optimized(&2, &3, &0),
        Err(Ok(ComputationError::InvalidModulus))
    );
}

// ---------------------------------------------------------------------------
// Loop optimization
// ---------------------------------------------------------------------------

#[test]
fn stats_naive_and_optimized_agree() {
    let (env, client, _) = setup();
    let values = sample(&env, 50);
    let expected = Stats {
        count: 50,
        sum: (0..50i128)
            .map(|i| if i % 2 == 0 { i } else { -i - 1 })
            .sum(),
        min: -50,
        max: 48,
    };
    assert_eq!(client.stats_naive(&values), expected);
    assert_eq!(client.stats_optimized(&values), expected);
}

#[test]
fn stats_single_element() {
    let (env, client, _) = setup();
    let values = vec![&env, -7i64];
    let expected = Stats {
        count: 1,
        sum: -7,
        min: -7,
        max: -7,
    };
    assert_eq!(client.stats_naive(&values), expected);
    assert_eq!(client.stats_optimized(&values), expected);
}

#[test]
fn stats_extreme_values_do_not_overflow() {
    let (env, client, _) = setup();
    let values = vec![&env, i64::MAX, i64::MAX, i64::MIN];
    let stats = client.stats_optimized(&values);
    assert_eq!(stats.sum, i64::MAX as i128 * 2 + i64::MIN as i128);
    assert_eq!(client.stats_naive(&values), stats);
}

#[test]
fn stats_reject_empty_input() {
    let (env, client, _) = setup();
    let empty = Vec::new(&env);
    assert_eq!(
        client.try_stats_naive(&empty),
        Err(Ok(ComputationError::EmptyInput))
    );
    assert_eq!(
        client.try_stats_optimized(&empty),
        Err(Ok(ComputationError::EmptyInput))
    );
}

#[test]
fn single_pass_loop_is_cheaper() {
    let (env, client, _) = setup();
    let values = sample(&env, 200);

    let naive = cpu_cost(&env, || {
        client.stats_naive(&values);
    });
    let optimized = cpu_cost(&env, || {
        client.stats_optimized(&values);
    });

    assert!(
        optimized < naive,
        "single pass ({optimized}) should cost less than three passes ({naive})"
    );
}

// ---------------------------------------------------------------------------
// Caching
// ---------------------------------------------------------------------------

#[test]
fn cached_and_uncached_stats_agree() {
    let (env, client, _) = setup();
    let values = sample(&env, 20);

    let written = client.set_values(&values);

    assert_eq!(written, client.stats_optimized(&values));
    assert_eq!(client.stats_cached(), written);
    assert_eq!(client.stats_uncached(), written);
}

#[test]
fn cache_is_refreshed_on_write() {
    let (env, client, _) = setup();
    client.set_values(&sample(&env, 10));
    client.set_values(&vec![&env, 1i64, 2, 3]);

    let expected = Stats {
        count: 3,
        sum: 6,
        min: 1,
        max: 3,
    };
    assert_eq!(client.stats_cached(), expected);
    assert_eq!(client.stats_uncached(), expected);
}

#[test]
fn cached_read_is_cheaper() {
    let (env, client, _) = setup();
    client.set_values(&sample(&env, 200));

    let uncached = cpu_cost(&env, || {
        client.stats_uncached();
    });
    let cached = cpu_cost(&env, || {
        client.stats_cached();
    });

    assert!(
        cached < uncached,
        "cached read ({cached}) should cost less than recomputing ({uncached})"
    );
}

#[test]
fn reads_before_any_write_return_empty_input() {
    let (_env, client, _) = setup();
    assert_eq!(
        client.try_stats_cached(),
        Err(Ok(ComputationError::EmptyInput))
    );
    assert_eq!(
        client.try_stats_uncached(),
        Err(Ok(ComputationError::EmptyInput))
    );
}

#[test]
fn set_values_rejects_empty_input() {
    let (env, client, _) = setup();
    assert_eq!(
        client.try_set_values(&Vec::new(&env)),
        Err(Ok(ComputationError::EmptyInput))
    );
}

// ---------------------------------------------------------------------------
// Initialization & authorization
// ---------------------------------------------------------------------------

#[test]
fn init_twice_fails() {
    let (env, client, _) = setup();
    assert_eq!(
        client.try_init(&Address::generate(&env)),
        Err(Ok(ComputationError::AlreadyInitialized))
    );
}

#[test]
fn set_values_before_init_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let client =
        ComputationOptimizationClient::new(&env, &env.register(ComputationOptimization, ()));
    assert_eq!(
        client.try_set_values(&vec![&env, 1i64]),
        Err(Ok(ComputationError::NotInitialized))
    );
}

#[test]
fn set_values_requires_admin_auth() {
    let (env, client, admin) = setup();
    let values = vec![&env, 1i64, 2];
    client.set_values(&values);

    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, admin);
    assert_eq!(
        auths[0].1.function,
        AuthorizedFunction::Contract((
            client.address.clone(),
            Symbol::new(&env, "set_values"),
            (values,).into_val(&env),
        ))
    );
}

#[test]
fn set_values_without_auth_is_rejected() {
    let (env, client, _) = setup();
    env.set_auths(&[]);
    assert!(client.try_set_values(&vec![&env, 1i64]).is_err());
    assert_eq!(
        client.try_stats_cached(),
        Err(Ok(ComputationError::EmptyInput))
    );
}
