# Timelock Contract

A secure timelock implementation with admin controls for delayed execution of operations.

## Features

### Core Functionality
- **Queue Operations**: Schedule operations for future execution with configurable delays
- **Execute Operations**: Execute queued operations after the delay period
- **Cancel Operations**: Cancel queued operations before execution

### Admin Controls
- **Bounded Delay Updates**: Admin can update min/max delay bounds within absolute limits (30s - 7 days)
- **Emergency Pause**: Admin can pause/unpause all contract operations
- **Audit Events**: All admin actions emit audit trail events for transparency

## Usage

### Initialize
```rust
client.initialize(&admin_address);
```

### Queue an Operation
```rust
client.queue(&operation_id, &delay_seconds);
```

### Execute an Operation
```rust
client.execute(&operation_id);
```

### Admin Controls
```rust
// Update delay bounds (within 30s - 7 days)
client.update_delay_bounds(&new_min_delay, &new_max_delay);

// Emergency pause
client.set_pause(&true);  // pause
client.set_pause(&false); // unpause

// Check current settings
let (min_delay, max_delay) = client.get_delay_bounds();
let is_paused = client.is_paused();
```

## Security Features

- **Authentication**: All operations require admin authorization
- **Bounded Configuration**: Delay bounds are constrained to safe ranges
- **Emergency Controls**: Pause functionality for incident response
- **Audit Trail**: All admin actions are logged with timestamps
- **Replay Protection**: Operations are removed after execution

## Testing

Run the test suite:
```bash
cargo test
```

Build as WASM:
```bash
cargo build --target wasm32-unknown-unknown --release
```
