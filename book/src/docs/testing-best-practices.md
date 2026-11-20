# Testing Best Practices

Testing is a critical part of developing smart contracts on Soroban. A comprehensive testing strategy ensures that your contracts behave as expected and protects against regressions and security vulnerabilities.

## Unit Test Patterns
- **Test each function in isolation:** Ensure you test the happy path and edge cases.
- **Mock dependencies:** Use `Env::mock_all_auths()` to simplify authorization testing when not explicitly testing `require_auth()`.
- **Test error handling:** Use `#[should_panic(expected = "...")]` to verify that your contracts fail correctly.

## Integration Testing
- **Cross-contract calls:** Test workflows spanning multiple contracts using integration tests.
- **State transitions:** Ensure that sequences of actions transition the contract state correctly.
- **Integration Test Suite:** Keep integration tests in the `tests/integration/` directory, using `env.register_contract()` to load and execute contracts.

## Fuzz Testing
- **Property-based testing:** Fuzz inputs using tools like `proptest` to automatically verify contract invariants across thousands of random inputs.
- **Boundary conditions:** Fuzzers often find edge cases near the limits of integer sizes (`u32::MAX`, `i128::MAX`).

## Coverage Goals
- **Target >90% coverage:** Utilize `cargo tarpaulin` to measure test coverage.
- **Cover all branches:** Ensure that conditional branches are evaluated for both truthy and falsy states.

---

## Worked Examples

Every snippet below is copied from a test that runs in this repository.

### 1. Set up the environment and the contract

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn deposit_increases_balance() {
    let env = Env::default();
    // Accept every require_auth() in the test.
    env.mock_all_auths();

    // Register the contract natively - no Wasm build required.
    let id = env.register_contract(None, TokenContract);
    let client = TokenContractClient::new(&env, &id);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
}
```

### 2. Assert on typed errors, not panics

```rust
// A contract error is matched exactly.
assert_eq!(
    client.try_transfer(&alice, &bob, &10_000),
    Err(Ok(TokenError::InsufficientBalance))
);

// A missing authorization surfaces as a host error.
assert!(client.try_transfer(&alice, &bob, &10).is_err());
```

### 3. Travel through time

```rust
// Timelocks, vesting and rate limits are all ledger-time driven.
env.ledger().with_mut(|l| l.timestamp += 70);
env.ledger().with_mut(|l| l.sequence_number = 200);

// Or set an absolute value.
env.ledger().set_timestamp(1_000);
```

### 4. Assert on events

```rust
let events = env.events().all();
let (_, topics, data) = events.last().unwrap();

// Topics are indexed; the payload carries the details.
assert_eq!(topics.get(0).unwrap(), symbol_short!("token").into_val(&env));
```

### 5. Test the cross-contract workflow

```rust
// Register both contracts and let them talk to each other.
let amm_id = env.register_contract(None, ConstantProductAmm);
let token_id = env.register_stellar_asset_contract_v2(issuer);
let amm = ConstantProductAmmClient::new(&env, &amm_id);

amm.initialize(&token_id.address(), &token_y.address());
let lp = amm.add_liquidity(&alice, &1_000, &1_000);
assert_eq!(lp, 1_000);
```

### 6. Measure coverage

```bash
# Whole-workspace coverage report (HTML + XML + LCOV).
cargo tarpaulin --out Html

# Only the integration suite.
cargo tarpaulin -p integration-tests --out Html
```

The CI workflow `.github/workflows/test.yml` runs the coverage job and fails
the build when line coverage drops below 90%.

### 7. Fuzz an invariant

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn transfer_never_creates_money(
        balance in 0i128..1_000_000,
        amount in 0i128..1_000_000,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TokenContract);
        let client = TokenContractClient::new(&env, &id);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(&alice, &balance);
        let _ = client.try_transfer(&alice, &bob, &amount);

        prop_assert!(client.total_supply() == balance);
    }
}
```

---

## Frontend / TypeScript Testing

Contract unit tests in Rust catch logic bugs, but your users interact with the contract through a JavaScript/TypeScript frontend. Testing the bindings layer — generated clients, argument encoding, wallet flows, and error surfacing — closes the gap between "the contract is correct" and "the app works".

### Generate TypeScript clients from the contract spec

Generate a typed client directly from your built contract with `soroban-spec-typescript` (exposed through the Stellar CLI):

```bash
# Build the contract, then generate bindings from its embedded spec.
cargo build --target wasm32-unknown-unknown --release -p authorized-token

# Uses the `soroban-spec-typescript` crate under the hood.
stellar contract bindings ts \
  --wasm ./target/wasm32-unknown-unknown/release/authorized_token.wasm \
  --output-dir ./webapp/src/contracts/authorized-token \
  --network-passphrase "Test SDF Network ; September 2015"
```

The generator emits a client class per contract (e.g., `AuthorizedTokenClient`) with typed methods, `ScVal` conversion helpers, and event type definitions. Commit the generated output (or generate it in CI from the released wasm) so frontend tests always match the deployed contract interface. When the contract changes, regenerate — a stale client is the most common source of "works in Rust tests, fails in the frontend" bugs.

### Jest / Vitest contract invocation tests

Test the generated client the same way you test any other async API layer:

```ts
import { describe, it, expect, beforeAll } from "vitest";
import { Networks } from "@stellar/stellar-sdk";
import { AuthorizedTokenClient } from "./contracts/authorized-token";
import { mockRpc, TESTNET_FIXTURES } from "./test/mock-rpc";

describe("AuthorizedTokenClient", () => {
  const client = new AuthorizedTokenClient({
    contractId: "CABROAD_EXAMPLE_CONTRACT_ID",
    rpcUrl: "https://soroban-testnet.stellar.org",
    networkPassphrase: Networks.TESTNET,
  });

  beforeAll(() => mockRpc(TESTNET_FIXTURES));

  it("reports accounts as authorized by default", async () => {
    const result = await client.authorized({ address: "GALEXAMPLE…" });
    expect(result.result).toBe(true);
  });

  it("surfaces a frozen account as a NotAuthorized error", async () => {
    await expect(
      client.transfer({
        from: "GFROZEN_EXAMPLE…",
        to: "GBOB_EXAMPLE…",
        amount: 100n,
      })
    ).rejects.toThrow(/NotAuthorized/);
  });
});
```

Assert on contract errors by their **contract error name or code**, not by generic messages — the generated client maps `#[contracterror]` enums to typed values, so a compliance rejection (`NotAuthorized`) should be distinguishable from an insufficient-balance failure in your UI tests.

> For a real example to bind against, see the [Authorized Token example](../examples/tokens/11-authorized-token.md) (`examples/tokens/11-authorized-token`), whose `set_authorized` / `authorized` interface is used above, or the [SEP-41 Token example](../examples/tokens/01-sep41-token.md) for the base interface.

### Mocking the RPC layer

Frontend tests must not depend on a live network. Mock at the JSON-RPC boundary so the real SDK networking code is exercised:

```ts
// test/mock-rpc.ts — stub fetch and serve canned JSON-RPC responses
export function mockRpc(fixtures: Record<string, unknown>) {
  const realFetch = globalThis.fetch;
  beforeAll(() => {
    globalThis.fetch = async (input: RequestInfo | URL) => {
      const body = JSON.parse(String(input instanceof Request ? input.body : ""));
      const response = fixtures[body.method] ?? { jsonrpc: "2.0", id: body.id };
      return new Response(JSON.stringify(response), {
        headers: { "content-type": "application/json" },
      });
    };
  });
  afterAll(() => {
    globalThis.fetch = realFetch;
  });
}
```

Fixture entries to provide: `getAccount`, `simulateTransaction`, `sendTransaction`, `getTransaction`, and `getEvents`. Keep fixtures as checked-in JSON so a contract-side change shows up as a diff in review. Alternatives: mock the `rpc.Server` class in a module-level `vi.mock("@stellar/stellar-sdk")`, or run a standalone Soroban RPC mock server — the boundary should stay at the RPC protocol either way, so tests still validate XDR encoding.

### What to cover

- **Happy paths** of every entry point your UI calls, decoded from real fixture responses.
- **Error surfacing**: contract error enums, `require_auth` failures, and expired transactions produce distinct UI states — test each.
- **Event handling**: feed `getEvents` fixtures into your indexer/store and assert UI reactions (e.g., an `authorize` event unfreezes the user's UI).
- **Wallet flows**: mock `@stellar/freighter-api` at its module boundary (connection granted, denied, wrong network) — see the [Wallet Integration Guide](./wallet-integration.md).

---

## Related Pages

- [Testing Guide →](../guides/testing.md)
- [Code Examples →](./code-examples.md)
- [Testing Pitfalls →](./testing-pitfalls.md)
- [Wallet Integration Guide →](./wallet-integration.md)
- [Interactive Playground — Intermediate Examples →](../examples/playground-intermediate.md)
