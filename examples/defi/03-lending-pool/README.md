# Lending Pool

A basic Soroban smart contract implementing a lending pool.

## Features

- Deposit/withdraw assets
- Borrow/repay assets
- Interest rate model (with kink)
- Utilization tracking
- Borrow limit (80% of deposit)

## Usage

### Initialize

```rust
client.initialize(&base_rate, &kink_rate, &kink_utilization);
```

### Deposit

```rust
client.deposit(&user, &amount);
```

### Withdraw

```rust
client.withdraw(&user, &amount);
```

### Borrow

```rust
client.borrow(&user, &amount);
```

### Repay

```rust
client.repay(&user, &amount);
```

### Get Utilization

```rust
let utilization = client.get_utilization();
```

### Get Borrow Rate

```rust
let rate = client.get_borrow_rate();
```

### Get User Position

```rust
let position = client.get_user_position(&user);
```
