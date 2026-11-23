# Facilitator Guide: Soroban Development Workshop

This guide provides timing, talking points, common pitfalls, and Q&A preparation for each workshop module.

## Workshop Overview

| Module | Duration | Format | Key Deliverable |
|--------|----------|--------|-----------------|
| 1: Introduction & Setup | 2h | Lecture + Lab | Working dev environment |
| 2: Storage & Data Types | 3h | Lecture + Lab | Counter contract with events |
| 3: Auth & Access Control | 3h | Lecture + Lab | Vault contract with RBAC + pause |
| 4: Tokens & SEP-41 | 3h | Lecture + Lab | Pausable mint/burn token |
| 5: Advanced & Deployment | 3h | Lecture + Lab | Oracle + testnet deployment |

Total: ~14 hours (2 full days or 4 half-day sessions)

---

## Pre-Workshop Preparation

### 1. Verify Your Environment (Day Before)

```bash
# Ensure everything builds
cargo build --target wasm32-unknown-unknown --release
cargo test --workspace

# Verify stellar CLI
stellar --version
```

### 2. Prepare a Testnet Account

Generate a keypair and fund it via Friendbot before the workshop so you can demo deployment without waiting.

### 3. Check Participant Prerequisites

Send a pre-workshop email with the setup instructions from the [Participant Guide](participant-guide.md). Ask participants to install Rust and the Stellar CLI before arriving.

### 4. Have Backup Code Ready

Clone the repo with working solutions for each exercise so you can show the answer if participants get stuck.

---

## Module 1: Introduction & Environment Setup (2h)

### Timing

| Segment | Duration | Activity |
|---------|----------|----------|
| Welcome & introductions | 15 min | Icebreaker, go over agenda |
| What is Soroban (slides) | 20 min | Lecture |
| Environment setup (live demo) | 30 min | Walk through installation |
| Hello World walk-through | 20 min | Code review + run tests |
| Exercise 1 | 30 min | Hands-on lab |
| Wrap-up & Q&A | 5 min | Recap |

### Talking Points

- **Why Soroban over EVM:** Resource metering is more granular, Rust's type system catches bugs at compile time, built-in `require_auth()` is more secure than `msg.sender`.
- **WASM target:** Emphasize that `wasm32-unknown-unknown` is different from `wasm32-wasi`. Soroban uses the unknown target (no standard library).
- **`#![cfg_attr(target_family = "wasm", no_std)]`**: Explain this disables the standard library for WASM builds but keeps it for native tests. This is why tests can use `std` but contracts can't.

### Common Pitfalls

1. **Rust version too old:** Show `rustup update stable` if anyone has a version below 1.74.
2. **Old `soroban-cli` installed:** If `stellar` command conflicts, tell them to `cargo uninstall soroban-cli` first.
3. **Build failures on first try:** Usually missing the WASM target. Re-run `rustup target add wasm32-unknown-unknown`.
4. **`cargo test` hangs:** Sometimes the first build is slow downloading dependencies. Mention this is normal.

### Q&A Preparation

- "Do I need to know Rust?" — No, but it helps. The workshop covers the Soroban-specific patterns. Point to the Rust Book for language fundamentals.
- "Can I use Solidity instead?" — No, Soroban contracts are Rust-only. But check the [Ethereum to Soroban guide](../../guides/ethereum-to-soroban.md) for pattern equivalents.
- "How is this different from EVM gas?" — Soroban tracks CPU instructions, memory, and storage separately. The budget model is more transparent.

---

## Module 2: Storage & Data Types (3h)

### Timing

| Segment | Duration | Activity |
|---------|----------|----------|
| Storage types (slides) | 30 min | Lecture + diagram |
| Custom types (slides) | 20 min | Lecture |
| Events & errors (slides) | 20 min | Lecture |
| Walk through storage example | 20 min | Code review |
| Exercise 2 | 60 min | Hands-on lab |
| Wrap-up & Q&A | 10 min | Recap |

### Talking Points

- **Three storage types:** Draw a diagram on the board/whiteboard showing persistent (database), temporary (cache), instance (contract config). Emphasize TTL management for persistent and instance.
- **DataKey enum pattern:** This is the standard pattern across the entire cookbook. Show how composite keys (e.g., `Balance(Address)`) map to storage slots.
- **`#[contracttype]`:** Explain that this derives serialization so types can go into storage. Without it, the compiler will reject storage operations.
- **Events vs storage:** Events are for external observers (indexers, UIs). Storage is for the contract itself. Don't use events for data the contract needs to read later.

### Common Pitfalls

1. **Forgetting TTL extension:** Persistent storage gets archived if TTL isn't extended. Show `extend_ttl()` in the storage example.
2. **Using `String` where `Symbol` suffices:** Symbols are cheaper for short identifiers. Use `symbol_short!()` for strings ≤9 chars.
3. **Map iteration:** Soroban `Map` doesn't support iteration by value. If they need enumeration, point to the [iterable mapping pattern](../../examples/intermediate/iterable-mappings/).
4. **Type mismatch in `get()`:** `env.storage().get()` returns `Option<T>`. The type must match what was stored. Show the `.unwrap_or(default)` pattern.

### Q&A Preparation

- "When should I use temporary vs persistent?" — Temporary for rate-limit windows, caching, anything that can be recreated. Persistent for anything that must survive (balances, ownership).
- "What happens when persistent TTL expires?" — Data is archived (not deleted). It can be restored, but it costs more. Always extend TTL on writes.
- "Can I store a Vec in storage?" — Yes, but be careful about size. Large Vecs are expensive. Consider the iterable mapping pattern for large datasets.

---

## Module 3: Authentication & Access Control (3h)

### Timing

| Segment | Duration | Activity |
|---------|----------|----------|
| require_auth (slides) | 20 min | Lecture + demo |
| RBAC (slides) | 25 min | Lecture |
| Multi-sig & pausable (slides) | 25 min | Lecture |
| Walk through auth example | 15 min | Code review |
| Exercise 3 | 60 min | Hands-on lab |
| Wrap-up & Q&A | 15 min | Recap |

### Talking Points

- **`require_auth()` vs Solidity `msg.sender`:** In Solidity, the contract checks `msg.sender`. In Soroban, the host verifies signatures. This means even a malicious contract can't fake auth. Emphasize the security model difference.
- **Initialization pattern:** There's no constructor in Soroban. The `initialize()` function is a convention, and guarding against re-initialization is critical.
- **Role hierarchy:** Show the Owner > Admin > Moderator > User hierarchy. Emphasize that role checks should use `PartialOrd` comparisons or explicit matching.
- **Pausable as emergency stop:** This is the simplest and most important security pattern. Every contract handling value should consider it.

### Common Pitfalls

1. **Forgetting `require_auth()` on one function:** This is the #1 security bug. Show the audit checklist from Module 5.
2. **Checking role but not calling `require_auth()`:** Checking a role in storage is not the same as verifying the caller's signature. Both are needed.
3. **Not testing unauthorized paths:** Always write tests that verify unauthorized callers are rejected. This catches auth bugs early.
4. **Mock auth in tests:** Show the `env.mock_auths()` pattern. Without it, `require_auth()` will panic in tests.

### Q&A Preparation

- "Can a contract call another contract's `require_auth()`?" — No. `require_auth()` verifies that the address authorized the current transaction at the top level. Cross-contract calls inherit the top-level auth.
- "How does multi-sig differ from multi-party auth?" — Multi-sig requires M-of-N signatures on a single transaction. The cookbook has examples of both patterns.
- "What if I forget to guard initialize?" — An attacker can re-initialize and replace the admin. Always test for double-initialization rejection.

---

## Module 4: Tokens & SEP-41 (3h)

### Timing

| Segment | Duration | Activity |
|---------|----------|----------|
| SEP-41 overview (slides) | 20 min | Lecture |
- Token implementation walk-through | 30 min | Deep code review of SEP-41 example |
| Mint/burn & pausable (slides) | 20 min | Lecture |
| Exercise 4 | 70 min | Hands-on lab |
| Wrap-up & Q&A | 20 min | Recap + demo |

### Talking Points

- **SEP-41 vs ERC-20:** Very similar interface. Key difference: SEP-41 uses `Address` type (which can be a contract or account), while ERC-20 uses `address` (always an account).
- **`i128` for amounts:** Explain why token amounts use `i128` not `u128` — Soroban SDK uses signed integers, and `checked_add`/`checked_sub` are available on `i128`.
- **Allowance pattern:** Walk through the two-step approve → transfer_from flow. Emphasize that the spender (not the owner) calls `transfer_from` and must authenticate.
- **Event structure:** Show the `(namespace, action, from, to)` topic pattern. Explain that topics are indexed and queryable.

### Common Pitfalls

1. **Not checking `amount > 0`:** Zero or negative amounts should be rejected. Show `require_positive()`.
2. **Forgetting overflow checks:** `checked_add` and `checked_sub` return `Option<T>`. Always handle the `None` case with an error.
3. **Storing balances in instance storage:** Balances must be in persistent storage (one entry per user). Instance storage is for singletons.
4. **Transfer events on mint/burn:** The SEP-41 convention is to emit a transfer event with `from` or `to` set to the contract address. Show this in the reference.

### Q&A Preparation

- "Can I add a supply cap?" — Yes, the cookbook has a [minting strategies example](../../examples/tokens/02-minting-strategies/) with fixed-cap, unlimited, and scheduled minting.
- "How do I handle token decimals?" — Store decimals in instance storage at initialization. Standard is 7 for Stellar.
- "Can I batch transfers?" — Yes, the [SEP-41 extensions](../../examples/tokens/02-sep41-extensions/) example has batch transfer and batch approve.

---

## Module 5: Advanced Patterns & Deployment (3h)

### Timing

| Segment | Duration | Activity |
|---------|----------|----------|
| Proxy patterns (slides) | 20 min | Lecture + diagram |
| Oracle & timelock (slides) | 20 min | Lecture |
| Gas optimization (slides) | 15 min | Lecture |
| Deployment demo (slides + live) | 15 min | Live testnet deployment |
| Exercise 5 | 70 min | Hands-on lab (oracle + deployment) |
| Workshop wrap-up | 20 min | Recap + next steps + feedback |

### Talking Points

- **Proxy pattern:** Draw the User → Proxy → Implementation diagram. Explain that the proxy is a thin wrapper that stores the implementation address and delegates calls.
- **Beacon proxy:** Show how a single beacon can upgrade multiple proxies at once. Good for factory-deployed contracts.
- **Oracle staleness:** Emphasize the `max_age` check. In production, stale oracle data can lead to incorrect liquidations or trades.
- **Deployment:** Walk through the CLI commands step by step. Show the actual deployment live (have a pre-funded account ready).
- **Security checklist:** Go through the checklist from the slides. This is the most important takeaway for production readiness.

### Common Pitfalls

1. **Deployment without funding:** The account needs XLM for transaction fees. Show how to use Friendbot.
2. **Wrong network passphrase:** Testnet and mainnet have different passphrases. The CLI handles this if configured correctly.
3. **Forgetting to initialize:** A deployed but uninitialized contract will return errors. Always call `initialize()` after deploying.
4. **Contract ID confusion:** The deploy command returns a contract ID. Save it — you'll need it for every subsequent interaction.

### Q&A Preparation

- "How much does deployment cost?" — Minimal (a few XLM for rent and fees). Testnet is free via Friendbot.
- "Can I upgrade a contract after deployment?" — Only if you used a proxy pattern. Direct deployments are immutable.
- "How do I handle contract state migration?" — The cookbook has a [storage migration example](../../examples/intermediate/storage-migration/) and a [contract migrations example](../../examples/advanced/10-contract-migrations/).
- "What about mainnet?" — Same process, but use the mainnet RPC URL and passphrase. Ensure you have real XLM for fees.

---

## General Facilitation Tips

### Pacing

- Keep lecture segments to 20-30 minutes max. Break them up with questions or quick demos.
- Exercises should be the majority of time. Participants learn by doing.
- If someone is ahead, suggest they explore additional cookbook examples or add extra features.

### When Participants Get Stuck

1. Ask them to read the error message aloud — often the fix is in the message.
2. Point to the relevant cookbook example rather than giving the answer directly.
3. Pair them with another participant who has already solved that part.
4. Have the solution ready to show if they're blocked for more than 10 minutes.

### Accessibility

- Ensure all slides are readable (large font, high contrast).
- Provide printed copies of exercises for participants who prefer paper.
- Offer closed captioning if presenting online.
- Allow extra time for participants with slower machines.

### Online vs In-Person

**In-person:**
- Use a projector for live coding demos
- Walk around to help during exercises
- Encourage pair programming

**Online:**
- Screen share for demos
- Use breakout rooms for exercises
- Have a shared document for Q&A
- Record the session for later reference
