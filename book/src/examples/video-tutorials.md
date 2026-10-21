# Basic Examples Video Series

Welcome to the Soroban Cookbook Video Series! This hands-on video series is designed to take you from a complete beginner to a confident Soroban smart contract developer by walking through the core basic examples in this repository.

Each video is between **10 to 15 minutes**, focuses on one example, walks through the Rust contract code, explains the core host features, and demonstrates how to write and run unit tests.

---

## 📺 Video Directory

| # | Video Title | Target Example | Duration | Walkthrough Link |
|---|-------------|----------------|----------|------------------|
| 1 | [Soroban Basics #1: Hello World](./01-hello-world/) | [01-hello-world](file:///workspaces/Soroban-Cookbook-/examples/basics/01-hello-world) | 11:30 | [Watch Walkthrough](https://www.youtube.com/watch?v=hello-world-walkthrough) |
| 2 | [Soroban Basics #2: Storage Layers & TTL](./02-storage-patterns/) | [02-storage-patterns](file:///workspaces/Soroban-Cookbook-/examples/basics/02-storage-patterns) | 13:15 | [Watch Walkthrough](https://www.youtube.com/watch?v=storage-patterns-walkthrough) |
| 3 | [Soroban Basics #3: Authentication & authorize](./03-authentication/) | [03-authentication](file:///workspaces/Soroban-Cookbook-/examples/basics/03-authentication) | 14:45 | [Watch Walkthrough](https://www.youtube.com/watch?v=authentication-walkthrough) |
| 4 | [Soroban Basics #4: Custom Errors & enums](./03-custom-errors/) | [03-custom-errors](file:///workspaces/Soroban-Cookbook-/examples/basics/03-custom-errors) | 10:20 | [Watch Walkthrough](https://www.youtube.com/watch?v=custom-errors-walkthrough) |
| 5 | [Soroban Basics #5: Events & Indexing](./04-events/) | [04-events](file:///workspaces/Soroban-Cookbook-/examples/basics/04-events) | 12:05 | [Watch Walkthrough](https://www.youtube.com/watch?v=events-walkthrough) |
| 6 | [Soroban Basics #6: Auth Context & call chains](./05-auth-context/) | [05-auth-context](file:///workspaces/Soroban-Cookbook-/examples/basics/05-auth-context) | 12:50 | [Watch Walkthrough](https://www.youtube.com/watch?v=auth-context-walkthrough) |

---

## 📖 Video Outlines & Scripts

### Video 1: Soroban Basics #1 — Hello World (11:30)
- **0:00 - 2:00:** Introduction to Soroban SDK, cargo workspace, and the `#![no_std]` constraint.
- **2:00 - 5:30:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/01-hello-world/src/lib.rs):
  - Explaining the `#[contract]` and `#[contractimpl]` macros.
  - Explaining the `Env` and why it is the first argument to every contract function.
  - Working with Soroban `Symbol` and `Vec<Symbol>` types.
- **5:30 - 8:30:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/01-hello-world/src/test.rs):
  - Setting up the test environment via `Env::default()`.
  - Registering the contract and using the generated client.
  - Making assertions on returned collections.
- **8:30 - 11:30:** Terminal demo: Building using `cargo build --target wasm32-unknown-unknown --release` and testing using `cargo test -p hello-world`.

---

### Video 2: Soroban Basics #2 — Storage Layers & TTL (13:15)
- **0:00 - 3:00:** Overview of Soroban's state storage model: **Persistent**, **Instance**, and **Temporary** storage tiers.
- **3:00 - 7:00:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/02-storage-patterns/src/lib.rs):
  - Using `#[contracttype] enum DataKey` for structured storage keys.
  - CRUD operations: `set()`, `get()`, `has()`, and `remove()`.
  - Critical TTL extension concepts: `extend_ttl()` to prevent data archival.
- **7:00 - 10:30:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/02-storage-patterns/src/test.rs):
  - How to advance the ledger or check rent/TTL in unit tests.
  - Simulating the differences in data lifetime.
- **10:30 - 13:15:** Terminal demo: Running `cargo test -p storage-patterns` and looking at compiled WASM size.

---

### Video 3: Soroban Basics #3 — Authentication & Authorization (14:45)
- **0:00 - 3:30:** The role of cryptographically signed addresses in Soroban. Understanding the `require_auth()` primitive.
- **3:30 - 8:30:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/03-authentication/src/lib.rs):
  - Requiring signature with `address.require_auth()`.
  - Differentiating authentication vs. authorization (admin check, roles, and allowances).
  - Admin patterns, RBAC, and spending allowances.
- **8:30 - 12:00:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/03-authentication/src/test.rs):
  - Mocking auth signatures in unit tests using `env.mock_all_auths()`.
  - Asserting auth errors and unauthorized calls.
- **12:00 - 14:45:** Terminal demo: Running `cargo test -p authentication` and demonstrating auth failure scenarios.

---

### Video 4: Soroban Basics #4 — Custom Errors & Enums (10:20)
- **0:00 - 2:30:** Introduction to custom errors in Soroban. Why `#[contracterror]` is preferred over `panic!`.
- **2:30 - 6:00:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/03-custom-errors/src/lib.rs):
  - Declaring a custom `#[contracterror] #[repr(u32)]` enum.
  - Returning `Result<T, ContractError>` and mapping internal failures to descriptive error codes.
  - UX and integration benefits for frontends parsing transaction results.
- **6:00 - 8:30:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/03-custom-errors/src/test.rs):
  - Capturing and asserting errors returned by the client.
  - Testing happy and unhappy paths for error propagation.
- **8:30 - 10:20:** Terminal demo: Running `cargo test -p custom-errors` and reviewing error-code logs.

---

### Video 5: Soroban Basics #5 — Events & Indexing (12:05)
- **0:00 - 3:00:** Introduction to Soroban's event system. How event publishing works, gas costs, and topic structures.
- **3:00 - 7:30:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/04-events/src/lib.rs):
  - Publishing events using `env.events().publish(topics, data)`.
  - Structuring events: 4-topic format for query-optimized off-chain filtering.
  - Defining `#[contracttype]` structs for event payloads.
- **7:30 - 10:00:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/04-events/src/test.rs):
  - Retrieving emitted events from the environment log in unit tests.
  - Validating topic structures and serialized data payloads.
- **10:00 - 12:05:** Terminal demo: Running `cargo test -p events` and checking event logging output.

---

### Video 6: Soroban Basics #6 — Auth Context & Invocation Chains (12:50)
- **0:00 - 3:00:** Understanding execution context: `invoker` vs. `current_contract_address`. Cross-contract call security.
- **3:00 - 7:30:** Code Walkthrough of [lib.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/05-auth-context/src/lib.rs):
  - Accessing the auth context: `env.auths()`.
  - Verifying caller contracts and delegating authorizations safely in multi-contract interactions.
  - Avoiding vulnerability patterns like reentrancy and spoofed signatures.
- **7:30 - 10:30:** Testing Walkthrough of [test.rs](file:///workspaces/Soroban-Cookbook-/examples/basics/05-auth-context/src/test.rs):
  - Deploying multiple test contracts and triggering nested call chains.
  - Testing cross-contract authorizations and verifying proper failure states.
- **10:30 - 12:50:** Terminal demo: Running `cargo test -p auth-context` and validating multi-contract call sequences.
