# Video Script: Getting Started with Soroban
## "Soroban Cookbook — Examples 01 through 03"
### Target length: 10–15 minutes

---

## Pre-recording checklist

- [ ] Terminal font size ≥ 18pt, high-contrast theme
- [ ] Browser zoom at 125% for any web pages shown
- [ ] Notifications silenced
- [ ] Repository cloned and `cargo test -p hello-world` passing before you hit record
- [ ] Microphone tested, no background noise

---

## Chapter 1 — Introduction (0:00 – 1:00)

**[Screen: repository root in VS Code or terminal]**

> "Welcome to the Soroban Cookbook. I'm going to walk you through the first
> three basic examples — Hello World, Storage Patterns, and Custom Errors.
> By the end of this video you'll understand the core building blocks of
> every Soroban smart contract: how to define a contract, how to persist
> data, and how to handle errors properly.
>
> If you haven't set up your environment yet, pause here and follow the
> Getting Started guide linked in the description — it covers installing
> Rust, the WebAssembly target, and the Stellar CLI.
>
> Let's jump in."

**[Show directory tree]**

```
examples/basics/
├── 01-hello-world/
├── 02-storage-patterns/
└── 03-custom-errors/
```

---

## Chapter 2 — 01 Hello World (1:00 – 4:30)

### 2.1 Open the file (1:00 – 1:30)

**[Open `examples/basics/01-hello-world/src/lib.rs`]**

> "The Hello World contract is intentionally the smallest possible Soroban
> contract. Every contract you ever write will start with these same four
> ingredients, so it's worth understanding each one."

### 2.2 Walk through `lib.rs` line by line (1:30 – 3:00)

**[Highlight `#![no_std]`]**

> "First line: `#![no_std]`. Soroban contracts compile to WebAssembly and
> run inside the Soroban host — a sandboxed environment that does not
> include the Rust standard library. This attribute tells the compiler not
> to link it."

**[Highlight the `use` line]**

> "We import only what we need from the SDK. `contract` and `contractimpl`
> are the two macros that wire everything together. `symbol_short` creates
> a compile-time Symbol — Soroban's gas-efficient short string type.
> `vec!` is the SDK's equivalent of the standard `vec![]` macro, but it
> allocates in host memory."

**[Highlight `#[contract] pub struct HelloContract;`]**

> "The `#[contract]` macro marks this unit struct as a Soroban contract.
> It's just a plain struct — no fields, no state. All state lives in
> storage, which we'll see in the next example."

**[Highlight `#[contractimpl]`]**

> "`#[contractimpl]` exposes the functions in this impl block as callable
> contract functions. The SDK also generates a typed client — here called
> `HelloContractClient` — that we use in tests."

**[Highlight the `hello` function]**

> "The `hello` function takes two arguments: `env`, which is the execution
> environment injected by the host — your gateway to storage, events,
> crypto, and ledger info — and `to`, the name to greet as a Symbol.
>
> It returns a `Vec<Symbol>` containing the two-word greeting. Why not a
> String? Because `soroban_sdk::String` is an immutable host object — there
> is no `format!` or string concatenation in the Wasm sandbox. A
> `Vec<Symbol>` is idiomatic, cheap, and easy for frontends to decode."

### 2.3 Open `test.rs` (3:00 – 4:00)

**[Open `examples/basics/01-hello-world/src/test.rs`]**

> "The test file shows the standard Soroban test pattern you'll use in
> every example.
>
> Step one: create a fresh `Env::default()`. Each test gets its own
> isolated environment — no shared state between tests.
>
> Step two: register the contract with `env.register_contract`. This gives
> it an on-chain address in the test environment.
>
> Step three: create the typed client. The SDK generates
> `HelloContractClient` automatically from the `#[contractimpl]` block.
>
> Step four: call the function through the client and assert the result."

### 2.4 Run the tests (4:00 – 4:30)

**[Switch to terminal]**

```bash
cargo test -p hello-world
```

**[Show passing output]**

> "All four tests pass. Notice the test names describe the behaviour, not
> just the function — that's a pattern we follow throughout the cookbook."

---

## Chapter 3 — 02 Storage Patterns (4:30 – 9:00)

### 3.1 Introduce the three storage types (4:30 – 5:30)

**[Open `examples/basics/02-storage-patterns/src/lib.rs`]**

> "Storage is where Soroban differs most from other smart contract
> platforms. Instead of one flat key-value store, you get three tiers —
> and choosing the right one matters for both cost and correctness."

**[Show the three access lines side by side]**

```rust
env.storage().persistent()   // long-lived, per-key TTL
env.storage().instance()     // contract-scoped, shared TTL
env.storage().temporary()    // ephemeral, cheapest
```

> "Persistent storage is for data that must outlive any single transaction —
> user balances, ownership records, anything other contracts depend on.
> Each key has its own TTL.
>
> Instance storage is for contract-wide configuration — the admin address,
> fee rates, feature flags. All keys share one TTL, so extending it once
> covers everything.
>
> Temporary storage is the cheapest option. Data expires after a short TTL
> and is permanently deleted — not restorable. Use it for reentrancy
> guards, short-lived caches, and intermediate computation."

### 3.2 Walk through persistent storage (5:30 – 6:30)

**[Highlight `set_persistent` and `get_persistent`]**

> "Every write to persistent storage should be followed by an `extend_ttl`
> call. The two parameters are threshold and extend-to. If the remaining
> TTL is below the threshold, it gets extended to the extend-to value.
> Setting threshold to zero means 'always extend', which wastes gas — so
> pick a meaningful value.
>
> If you forget `extend_ttl`, the entry's TTL starts decaying immediately.
> Eventually it gets archived off-ledger. Archived data can be restored,
> but restoration has a cost. Prevention is cheaper."

### 3.3 Walk through instance storage (6:30 – 7:15)

**[Highlight `set_instance` and `get_instance`]**

> "Instance storage uses the same API but without a key parameter on
> `extend_ttl` — because all instance keys share one TTL. Extending once
> covers everything stored here.
>
> Keep instance storage lean. Because all instance data is loaded together
> on every access, storing hundreds of user entries here makes every
> operation expensive. Reserve it for small, contract-wide config."

### 3.4 Walk through temporary storage (7:15 – 7:45)

**[Highlight `set_temporary` and `get_temporary`]**

> "Temporary storage needs no `extend_ttl` in most cases — the short TTL
> is the point. If you find yourself extending temporary TTLs frequently,
> that data probably belongs in persistent or instance storage instead."

### 3.5 Show the isolation test (7:45 – 8:30)

**[Open `src/test.rs`, highlight the isolation test]**

```rust
client.set_persistent(&key, &100);
client.set_temporary(&key, &200);
client.set_instance(&key, &300);

assert_eq!(client.get_persistent(&key), 100);
assert_eq!(client.get_temporary(&key), 200);
assert_eq!(client.get_instance(&key), 300);
```

> "This test shows something important: the three storage types are
> completely isolated namespaces. The same key in persistent and temporary
> storage holds independent values. This is a feature, but it can cause
> confusion if you're not expecting it."

### 3.6 Run the tests (8:30 – 9:00)

**[Terminal]**

```bash
cargo test -p storage-patterns
```

> "Six tests, all passing. The decision guide in the README is worth
> bookmarking — it walks you through exactly which storage type to reach
> for in any situation."

---

## Chapter 4 — 03 Custom Errors (9:00 – 13:00)

### 4.1 Why custom errors matter (9:00 – 9:45)

**[Open `examples/basics/03-custom-errors/src/lib.rs`]**

> "Generic panics are fine for invariant violations — things that should
> never happen. But for expected failure modes — bad input, insufficient
> balance, unauthorized access — you want typed errors. They give callers
> something to match on, give frontends numeric codes to display user
> messages, and make your contract's failure modes self-documenting."

### 4.2 Walk through the error enum (9:45 – 10:30)

**[Highlight the `ContractError` enum]**

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    InvalidInput        = 1,
    Unauthorized        = 2,
    NotFound            = 3,
    InsufficientBalance = 4,
    OperationNotAllowed = 5,
    RateLimitExceeded   = 6,
    ContractPaused      = 7,
    AlreadyExists       = 8,
}
```

> "Three things to notice.
>
> First, `#[contracterror]` — this SDK attribute registers the enum as a
> contract error type and handles the serialization so the host can
> propagate it across the boundary.
>
> Second, `#[repr(u32)]` — each variant maps to a stable numeric code.
> These codes are what frontends receive and what you'll see in transaction
> results. Start at 1, not 0 — zero is reserved.
>
> Third, the derives. `Copy` and `Clone` are required. `Debug`, `Eq`,
> `PartialEq`, `PartialOrd`, and `Ord` are needed for testing with
> `assert_eq!`."

### 4.3 Walk through a function using errors (10:30 – 11:30)

**[Highlight `validate_input`]**

```rust
pub fn validate_input(env: Env, value: i64) -> Result<(), ContractError> {
    if value <= 0 {
        env.events()
            .publish((symbol_short!("inp_err"),), ("Invalid value", value));
        Err(ContractError::InvalidInput)
    } else {
        Ok(())
    }
}
```

> "The return type is `Result<(), ContractError>`. On the happy path we
> return `Ok(())`. On failure we emit an event — so off-chain monitors can
> detect errors — and return `Err(ContractError::InvalidInput)`.
>
> Notice we don't panic. Panics are for invariant violations. This is an
> expected failure mode — the caller passed a bad value — so we return an
> error they can handle."

**[Highlight `complex_operation`]**

> "The `complex_operation` function shows how to chain multiple checks.
> Each check returns early with a specific error. The order matters: check
> the cheapest conditions first to avoid wasting gas on expensive storage
> reads when a simple input check would have caught the problem."

### 4.4 Walk through the tests (11:30 – 12:30)

**[Open `src/test.rs`, highlight `test_invalid_input_error`]**

```rust
let result = client.try_validate_input(&0);
assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
```

> "The `try_` prefix on the client method returns a `Result` instead of
> panicking. The double-wrapped `Err(Ok(...))` looks odd at first — the
> outer `Err` means the transaction failed, the inner `Ok` means it failed
> with a contract error rather than a host error. This is the standard
> pattern for testing error returns in Soroban."

**[Highlight `test_error_codes`]**

```rust
assert_eq!(ContractError::InvalidInput as u32, 1);
assert_eq!(ContractError::Unauthorized as u32, 2);
```

> "This test pins the numeric codes. If someone accidentally reorders the
> enum variants, this test catches it immediately — because frontends
> depend on these codes being stable."

### 4.5 Run the tests (12:30 – 13:00)

**[Terminal]**

```bash
cargo test -p custom-errors
```

> "Eleven tests, all passing. The event logging test at the bottom verifies
> that errors emit events — useful for off-chain monitoring."

---

## Chapter 5 — Wrap-up (13:00 – 14:00)

**[Return to repository root]**

> "Let's recap what we covered.
>
> In Hello World, you saw the four ingredients every Soroban contract needs:
> `#![no_std]`, `#[contract]`, `#[contractimpl]`, and the `Env` parameter.
>
> In Storage Patterns, you learned the three storage tiers — persistent for
> long-lived per-user data, instance for contract-wide config, and
> temporary for ephemeral data — and why TTL management matters.
>
> In Custom Errors, you saw how `#[contracterror]` gives you typed,
> numeric error codes that frontends can handle, and why `Result` is
> preferable to `panic!` for expected failure modes.
>
> These three patterns — contract structure, storage, and errors — appear
> in every contract in this cookbook. Once you're comfortable with them,
> the intermediate and advanced examples will feel familiar.
>
> Links to all three examples, the Getting Started guide, and the full
> cookbook are in the description. Thanks for watching."

---

## YouTube metadata

### Title
```
Soroban Cookbook: Getting Started — Hello World, Storage & Custom Errors (Examples 01–03)
```

### Description
```
A 10–15 minute walkthrough of the first three Soroban Cookbook examples.

Chapters:
0:00 Introduction
1:00 01 — Hello World (#[contract], #[contractimpl], Env, Symbol)
4:30 02 — Storage Patterns (persistent / instance / temporary, TTL)
9:00 03 — Custom Errors (#[contracterror], Result, error codes)
13:00 Wrap-up

Resources:
• Soroban Cookbook: https://github.com/Soroban-Cookbook/Soroban-Cookbook
• Getting Started guide: https://github.com/Soroban-Cookbook/Soroban-Cookbook/blob/main/book/src/guides/getting-started.md
• Soroban Docs: https://developers.stellar.org/docs/smart-contracts
• Stellar Discord (#soroban-dev): https://discord.gg/stellardev
```

### Tags
```
soroban, stellar, smart contracts, rust, webassembly, blockchain, tutorial, getting started, hello world, storage, error handling
```

### Thumbnail text suggestion
```
"Soroban Cookbook"  (large)
"Examples 01–03"    (smaller)
Rust crab icon + Stellar logo
```
