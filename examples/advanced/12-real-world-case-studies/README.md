# Real-World Case Studies

Three case studies of mistakes that recur in production smart contracts,
each paired with the fix implemented in this crate.

## Case Study 1: Reward Claims and Checks-Effects-Interactions

**Problem.** A naive claim function reads a user's balance, sends it out
(a token transfer, a cross-contract call, anything that can hand control
to outside code), and *then* zeroes the balance:

```rust
// Vulnerable: state is updated after the external interaction.
pub fn claim_reward(env: Env, user: Address) -> Result<i128, Error> {
    user.require_auth();
    let amount = read_balance(&env, &user);
    token_client.transfer(&contract_address, &user, &amount); // external call
    write_balance(&env, &user, 0); // too late if the call above re-entered
    Ok(amount)
}
```

If the transfer can trigger contract code that calls back into
`claim_reward` before the balance is zeroed, the same reward pays out
more than once.

**Solution.** [`claim_reward`](./src/lib.rs) updates storage (and emits
its event) *before* any value leaves the contract — the
checks-effects-interactions pattern:

```rust
pub fn claim_reward(env: Env, user: Address) -> Result<i128, Error> {
    user.require_auth();
    let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    if amount <= 0 { return Err(Error::NothingToClaim); }

    env.storage().persistent().set(&key, &0i128); // effect first
    env.events().publish((EVENT_CLAIM, user), amount);
    Ok(amount) // any interaction happens after this point, in the caller
}
```

**Lessons learned.** Order state writes before external calls or before
returning control to the caller, not after. This example keeps the
payout as a returned value to stay self-contained, but the same ordering
applies directly to a production version that calls `token::Client::transfer`
as its last step.

## Case Study 2: Fee Calculation and Checked Arithmetic

**Problem.** A fee calculated with plain multiplication looks correct in
every manual test with small numbers, but overflows silently for large
ones:

```rust
// Vulnerable: overflow wraps instead of failing.
fn calculate_fee(amount: i128, fee_bps: i128) -> i128 {
    amount * fee_bps / 10_000
}
```

`amount * fee_bps` can exceed `i128::MAX` before the division ever runs.
In Soroban's release profile this panics (overflow checks are on), but
plenty of other host environments wrap silently and turn a fee into a
nonsense number.

**Solution.** [`calculate_fee`](./src/lib.rs) uses `checked_mul` /
`checked_div` and returns `Error::Overflow` instead of trusting the
result:

```rust
pub fn calculate_fee(amount: i128, fee_bps: u32) -> Result<i128, Error> {
    amount
        .checked_mul(fee_bps as i128)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .ok_or(Error::Overflow)
}
```

**Lessons learned.** Reach for `checked_*` (or `saturating_*` when
clamping is genuinely the desired behavior) anywhere a multiplication
happens before a division. Don't rely on a specific build profile's
overflow-check setting to catch this for you.

## Case Study 3: Commit-Reveal Bidding Against Front-Running

**Problem.** A bid submitted in plain text is visible to everyone —
including other bidders and anyone building the next ledger — before it
settles:

```rust
// Vulnerable: the bid amount is public the moment it's submitted.
pub fn bid(env: Env, bidder: Address, amount: i128) -> Result<(), Error> {
    bidder.require_auth();
    // Anyone watching can submit a slightly higher bid first.
    ...
}
```

Since Soroban transactions are visible before they're applied, an
observer can always react to a plain bid with a marginally higher one of
their own.

**Solution.** [`commit_bid`](./src/lib.rs) accepts only a
`sha256(amount, salt)` commitment. The real amount is disclosed later
with [`reveal_bid`](./src/lib.rs), which recomputes the hash and rejects
any reveal that doesn't match:

```rust
// Commit phase: only the hash is public.
let commitment = env.crypto().sha256(&(amount, salt).to_xdr(&env)).to_bytes();
client.commit_bid(&bidder, &commitment);

// Reveal phase: the amount only becomes public once it can no longer be front-run.
client.reveal_bid(&bidder, &amount, &salt);
```

**Lessons learned.** Whenever the order bids/values arrive in matters,
consider whether revealing the value upfront lets someone react to it.
Commit-reveal trades a two-step flow for removing that window entirely.

## Build

```bash
cargo build -p real-world-case-studies
```

## Test

```bash
cargo test -p real-world-case-studies
```
