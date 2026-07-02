# Data Aggregation Oracle

A production-grade oracle pattern that aggregates data from multiple sources with sophisticated validation, manipulation detection, and outlier filtering capabilities.

## What You'll Learn

- **Data Verification**: Validate data from multiple trusted sources with timestamp tracking
- **Manipulation Detection**: Identify suspicious data submissions that deviate from expected ranges
- **Multi-Source Aggregation**: Combine data from independent sources using median/mean calculations
- **Outlier Filtering**: Automatically filter extreme values based on deviation thresholds
- **Pause/Resume Controls**: Emergency controls for contract operations
- **Event-Driven Auditing**: Comprehensive event logging for all oracle operations

## Contract Overview

### Key Functions

#### Admin Functions

- `initialize(env, admin)` - Set up the oracle with an admin address
- `add_source(env, admin, source)` - Add a trusted data source (max 10)
- `remove_source(env, admin, source)` - Remove a data source
- `pause(env, admin)` - Pause all contract operations
- `resume(env, admin)` - Resume contract operations

#### Data Submission

- `submit_data(env, source, value)` - Submit data point from authorized source
  - Only registered sources can submit
  - Each submission updates the latest value and timestamp
  - Emits `DataSubmissionEventData` for audit trails

#### Data Aggregation

- `aggregate_data(env)` - Aggregate all current data with outlier detection
  - Requires minimum 3 data points
  - Returns median value, mean value, outlier count, and timestamp
  - Detects and flags suspicious deviations

#### Query Functions

- `get_sources(env)` - Retrieve list of authorized sources
- `is_paused(env)` - Check if contract is paused

### Data Structures

#### DataPoint
```rust
pub struct DataPoint {
    pub source: Address,      // Data source address
    pub value: i128,          // Submitted value
}
```

#### AggregationResult
```rust
pub struct AggregationResult {
    pub median_value: i128,       // Median from filtered data
    pub mean_value: i128,         // Mean from filtered data
    pub outliers_removed: u32,    // Count of outliers detected
    pub timestamp: u64,           // Aggregation timestamp
}
```

### Constants

- `MAX_SOURCES: u32 = 10` - Maximum trusted sources
- `MIN_DATA_POINTS: u32 = 3` - Minimum points for aggregation
- `MAX_DEVIATION_BPS: i128 = 500` - 5% maximum deviation tolerance (basis points)

## Usage

### Setup and Initialization

```rust
let env = Env::default();
let admin = Address::generate(&env);
let contract_id = env.register_contract(None, DataAggregationOracleContract);
let client = DataAggregationOracleContractClient::new(&env, &contract_id);

// Initialize the oracle
client.initialize(&admin);
```

### Adding Data Sources

```rust
let source1 = Address::generate(&env);
let source2 = Address::generate(&env);
let source3 = Address::generate(&env);

// Register trusted sources
client.add_source(&admin, &source1);
client.add_source(&admin, &source2);
client.add_source(&admin, &source3);
```

### Submitting Data

```rust
// Each source submits its data point
client.submit_data(&source1, &100_000_000); // 100 USD in stroops
client.submit_data(&source2, &101_000_000); // 101 USD
client.submit_data(&source3, &99_500_000);  // 99.5 USD
```

### Aggregating Data

```rust
// Aggregate data and get median/mean with outlier detection
let result = client.aggregate_data();

println!("Median: {}", result.median_value);
println!("Mean: {}", result.mean_value);
println!("Outliers: {}", result.outliers_removed);
```

### Emergency Controls

```rust
// Pause oracle during anomalies
client.pause(&admin);

// Resume after fixing issues
client.resume(&admin);
```

## Security Considerations

### Outlier Detection Strategy

The contract uses basis point (BPS) deviation from median to detect manipulation attempts:

- Median is calculated from all submitted data
- Each value's deviation = `|value - median| * 10000 / |median|`
- Values deviating more than 500 BPS (5%) are flagged as outliers
- Outliers are excluded from mean calculation but included in median
- Manipulation detection events are emitted for suspicious submissions

### Multi-Source Resilience

- Requires minimum 3 independent data sources for aggregation
- No single source can skew the result if 2+ sources agree
- Outlier filtering protects against price manipulation attacks
- Admin has emergency pause capability

### Storage Efficiency

- Only the latest value and timestamp stored per source
- No unbounded storage growth (fixed maximum of 10 sources)
- Instance storage for configuration, persistent storage for values

### Authorization

- Only admin can manage sources and pause/resume
- Only registered sources can submit data
- All operations are auth-enforced

## Testing

Run the full test suite:

```bash
cargo test --package data-aggregation-oracle --all-features
```

Run specific test:

```bash
cargo test --package data-aggregation-oracle test_aggregate_with_outlier_detection -- --nocapture
```

Key test scenarios:

- **Initialization**: Setup and idempotency checks
- **Source Management**: Adding, removing, and duplicate detection
- **Data Submission**: Authorization validation and event emission
- **Aggregation**: Median/mean calculation and outlier detection
- **Pause/Resume**: Emergency controls and state management

## Building & Deployment

Build the contract:

```bash
cargo build --target wasm32-unknown-unknown --release -p data-aggregation-oracle
```

Verify with clippy:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

The compiled WASM will be at:
```
target/wasm32-unknown-unknown/release/data_aggregation_oracle.wasm
```

## Event Types

### DataSubmissionEventData
Emitted on successful data submission:
- `source`: Address of submitting source
- `value`: Submitted data value
- `timestamp`: Ledger timestamp

### DataAggregationEventData
Emitted after aggregation:
- `aggregated_value`: Final median value
- `point_count`: Total data points included
- `outlier_count`: Number of outliers detected
- `timestamp`: Aggregation timestamp

### ManipulationDetectionEventData
Emitted when outlier detected (failed deviation check):
- `source`: Address of suspicious source
- `value`: Suspicious value
- `deviation_bps`: Actual deviation in basis points
- `timestamp`: Detection timestamp

## Advanced Patterns

### Future Enhancements

- **Time-Weighted Average**: Weight older data less heavily
- **Configurable Deviation**: Allow admin to adjust outlier thresholds
- **Historical Data**: Store aggregation results for trend analysis
- **Source Reputation**: Track source accuracy over time
- **Cross-Contract Integration**: Publish aggregated prices on-chain

### Production Considerations

- Coordinate with multiple independent data providers
- Implement rate limiting to prevent flash loan attacks
- Use with price feeds that have cryptographic proofs
- Monitor deviation events for anomalies
- Implement circuit breakers for extreme values

## Related Examples

- [`01-multi-party-auth`](../01-multi-party-auth/) - Authorization patterns
- [`02-timelock`](../02-timelock/) - Time-delayed execution
- [`03-oracle-pattern`](../03-oracle-pattern/) - Basic oracle pattern
