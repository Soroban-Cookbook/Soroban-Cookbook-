# Price Oracle

A secure and reusable price feed oracle demonstrating how to build a robust data provider on Soroban.

This example covers price submission from multiple authorized updaters, median aggregation to mitigate outlier influence, Time-Weighted Average Price (TWAP) for smoother price feeds, and stale price detection for security.

## What You'll Learn

- Implementing a multi-updater authorization model
- Median aggregation for data integrity
- TWAP (Time-Weighted Average Price) calculation
- Stale price detection and strict freshness validation
- Managing asset-specific configurations

## Overview

```
Authorized Updaters  →  Submit Prices  →  Median Aggregator  →  Price History
                                                                    ↓
                                                           Readers query Price/TWAP
```

### Key Features

1.  **Multi-Updater Authorization**: Only addresses added by the admin can submit prices.
2.  **Median Aggregation**: When multiple updaters submit prices for the same asset, the oracle calculates the median. This protects against a single compromised or faulty updater providing extreme outliers.
3.  **TWAP Calculation**: Provides a time-weighted average price over a configurable window (e.g., 1 hour), making the price feed resistant to short-term price manipulation.
4.  **Stale Price Detection**: Each asset has a `max_age` configuration. The oracle can reject price queries if the data is older than this limit.

## Contract Interface

### Initialization

```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), OracleError>;
```

### Configuration

```rust
pub fn set_asset_config(env: Env, admin: Address, asset: Symbol, config: AssetConfig) -> Result<(), OracleError>;
pub fn add_updater(env: Env, admin: Address, updater: Address) -> Result<(), OracleError>;
```

### Price Submission

```rust
pub fn submit_prices(env: Env, updater: Address, prices: Vec<(Symbol, i128)>) -> Result<(), OracleError>;
```

### Queries

```rust
pub fn get_price(env: Env, asset: Symbol) -> Result<PriceData, OracleError>;
pub fn get_price_strict(env: Env, asset: Symbol) -> Result<i128, OracleError>;
pub fn get_twap(env: Env, asset: Symbol) -> Result<i128, OracleError>;
```

## Security Considerations

- **Admin Role**: The admin has full control over authorized updaters and asset configurations. In production, this should be a multisig or a DAO.
- **Aggregation Strategy**: Median aggregation is used to ensure that a minority of malicious updaters cannot easily skew the price.
- **Stale Data**: Always use `get_price_strict` or check the `timestamp` in `get_price` to ensure you aren't using outdated information.
- **TWAP Window**: A longer TWAP window increases manipulation costs but makes the price less responsive to real market changes.

## Running Tests

```bash
cargo test -p price-oracle
```

## Building

```bash
cargo build --target wasm32-unknown-unknown --release -p price-oracle
```
