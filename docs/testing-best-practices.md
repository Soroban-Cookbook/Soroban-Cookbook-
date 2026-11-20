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

## Frontend / TypeScript Testing

Contract unit tests in Rust catch logic bugs, but users interact with the contract through a JavaScript/TypeScript frontend. Test the bindings layer — generated clients, argument encoding, wallet flows, and error surfacing — so "the contract is correct" also means "the app works".

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

The generator emits a client class per contract (e.g., `AuthorizedTokenClient`) with typed methods, `ScVal` conversion helpers, and event type definitions. Commit the generated output (or generate it in CI from the released wasm) so frontend tests always match the deployed contract interface.

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

Assert on contract errors by their **contract error name or code**, not generic messages — the generated client maps `#[contracterror]` enums to typed values, so a compliance rejection (`NotAuthorized`) is distinguishable from an insufficient-balance failure in UI tests.

> For a real example to bind against, see the [Authorized Token example](../examples/tokens/11-authorized-token/) (`examples/tokens/11-authorized-token`) and the [SEP-41 Token example](../examples/tokens/01-sep41-token/) for the base interface.

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

Fixture entries to provide: `getAccount`, `simulateTransaction`, `sendTransaction`, `getTransaction`, and `getEvents`. Keep fixtures as checked-in JSON so contract-side changes show up as diffs in review. Alternatives: mock the `rpc.Server` class in a module-level `vi.mock("@stellar/stellar-sdk")`, or run a standalone Soroban RPC mock server — keep the boundary at the RPC protocol either way, so tests still validate XDR encoding.

### What to cover

- **Happy paths** of every entry point your UI calls, decoded from real fixture responses.
- **Error surfacing**: contract error enums, `require_auth` failures, and expired transactions produce distinct UI states — test each.
- **Event handling**: feed `getEvents` fixtures into your indexer/store and assert UI reactions (e.g., an `authorize` event unfreezes the user's UI).
- **Wallet flows**: mock `@stellar/freighter-api` at its module boundary (connection granted, denied, wrong network) — see the [Wallet Integration Guide](./wallet-integration.md).
