# Farming Pool Management

This example demonstrates a **Farming Pool** (Yield Farming) contract with comprehensive **Admin Controls**. It shows how to manage multiple staking pools, adjust reward parameters, and handle emergency situations.

## 📋 Features

- **Multi-Pool Support**: Manage multiple staking pools with different tokens.
- **Admin Controls**:
    - **Add/Remove Pools**: Dynamically manage available farming opportunities.
    - **Reward Rate Adjustment**: Change rewards per ledger for any pool.
    - **Pool Weighting**: Set relative weights for pools (can be used for reward distribution logic).
    - **Emergency Withdraw**: Admin can recover tokens from the contract in emergency cases.
- **Yield Farming Logic**:
    - **Precision Scaling**: Uses 1e12 scaling for reward calculations to maintain precision.
    - **Pending Rewards**: Automatic calculation and distribution of rewards on deposit/withdraw.
    - **Safe Transfers**: Ensures the contract doesn't fail if its reward balance is low.

## 🧠 Key Concepts

### 1. Reward Calculation Pattern

The contract uses the standard "Accumulated Reward Per Share" pattern:
`acc_reward_per_share += reward_amount * precision / total_staked`

When a user interacts with the pool:
`pending_reward = (user_stake * acc_reward_per_share) - reward_debt`

### 2. Admin Authorization

All management functions use `require_auth()` and verify the caller against the stored `Admin` address:

```rust
pub fn add_pool(env: Env, admin: Address, ...) {
    admin.require_auth();
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    if admin != stored_admin { panic!("Unauthorized"); }
    // ...
}
```

## 🛠️ Usage

### For Admin

1.  **Initialize**: Set the admin address.
2.  **Add Pool**: Specify staking token, reward token, rate, and weight.
3.  **Adjust**: Use `set_reward_rate` or `set_pool_weight` to tune the pool.
4.  **Emergency**: Use `emergency_withdraw_admin` if funds need to be recovered.

### For Users

1.  **Deposit**: Transfer staking tokens to the contract and start earning.
2.  **Withdraw**: Pull out staking tokens along with earned rewards.
3.  **Claim**: (Handled automatically during deposit/withdraw, or can be implemented as a standalone `claim` function).

## 🧪 Testing

The example includes tests for the full lifecycle:
- Successful initialization and admin auth.
- Pool creation and management.
- Accurate reward accumulation over multiple ledgers.
- Emergency recovery of funds.

```bash
cargo test -p farming-pool
```
