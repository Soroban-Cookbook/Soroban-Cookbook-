//! # Token Metadata
//!
//! Extends a base token with full metadata support: name, symbol, decimals,
//! and an optional URI field. The admin who initialises the contract may
//! update the mutable fields (name, symbol, uri) at any time. Decimals are
//! immutable after initialisation because changing them would silently
//! reinterpret every stored balance.
//!
//! ## Storage layout
//!
//! | Key              | Storage type | Mutable |
//! |------------------|--------------|---------|
//! | `Admin`          | instance     | no      |
//! | `Name`           | instance     | yes     |
//! | `Symbol`         | instance     | yes     |
//! | `Decimals`       | instance     | no      |
//! | `Uri`            | instance     | yes     |
//! | `Balance(addr)`  | persistent   | yes     |
//! | `TotalSupply`    | instance     | yes     |

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Name,
    Symbol,
    Decimals,
    Uri,
    Balance(Address),
    TotalSupply,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// All metadata fields returned by `metadata()`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    /// Optional URI pointing to off-chain metadata (empty string = not set).
    pub uri: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MetadataError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
    ArithmeticOverflow = 6,
    /// Decimals cannot be changed after initialisation.
    DecimalsImmutable = 7,
    EmptyString = 8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const NS: Symbol = symbol_short!("tok_meta");
const EV_INIT: Symbol = symbol_short!("init");
const EV_META: Symbol = symbol_short!("meta_upd");
const EV_MINT: Symbol = symbol_short!("mint");
const EV_BURN: Symbol = symbol_short!("burn");
const EV_XFER: Symbol = symbol_short!("transfer");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct TokenMetadataContract;

#[contractimpl]
impl TokenMetadataContract {
    /// Initialise the contract. `decimals` is permanent; all other fields can
    /// be updated later by the admin.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        uri: String,
    ) -> Result<(), MetadataError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(MetadataError::AlreadyInitialized);
        }
        require_non_empty(&name)?;
        require_non_empty(&symbol)?;

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::Uri, &uri);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);

        env.events().publish(
            (NS, EV_INIT, admin),
            TokenMetadata {
                name,
                symbol,
                decimals,
                uri,
            },
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Metadata queries
    // -----------------------------------------------------------------------

    /// Return all metadata in a single call — convenient for wallets and UIs.
    pub fn metadata(env: Env) -> Result<TokenMetadata, MetadataError> {
        Ok(TokenMetadata {
            name: read_string(&env, &DataKey::Name)?,
            symbol: read_string(&env, &DataKey::Symbol)?,
            decimals: read_decimals(&env)?,
            uri: read_string(&env, &DataKey::Uri)?,
        })
    }

    pub fn name(env: Env) -> Result<String, MetadataError> {
        read_string(&env, &DataKey::Name)
    }

    pub fn symbol(env: Env) -> Result<String, MetadataError> {
        read_string(&env, &DataKey::Symbol)
    }

    /// Decimals are immutable; this is a read-only query.
    pub fn decimals(env: Env) -> Result<u32, MetadataError> {
        read_decimals(&env)
    }

    pub fn uri(env: Env) -> Result<String, MetadataError> {
        read_string(&env, &DataKey::Uri)
    }

    // -----------------------------------------------------------------------
    // Admin: metadata updates
    // -----------------------------------------------------------------------

    /// Update name and/or symbol. Pass the current value to leave a field
    /// unchanged. Decimals cannot be updated — see `MetadataError::DecimalsImmutable`.
    pub fn update_metadata(
        env: Env,
        new_name: String,
        new_symbol: String,
        new_uri: String,
    ) -> Result<(), MetadataError> {
        let admin = read_admin(&env)?;
        admin.require_auth();
        require_non_empty(&new_name)?;
        require_non_empty(&new_symbol)?;

        env.storage().instance().set(&DataKey::Name, &new_name);
        env.storage().instance().set(&DataKey::Symbol, &new_symbol);
        env.storage().instance().set(&DataKey::Uri, &new_uri);

        env.events().publish(
            (NS, EV_META, admin),
            TokenMetadata {
                name: new_name,
                symbol: new_symbol,
                decimals: read_decimals(&env)?,
                uri: new_uri,
            },
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Token operations
    // -----------------------------------------------------------------------

    /// Mint `amount` tokens to `to`. Admin only.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), MetadataError> {
        require_positive(amount)?;
        let admin = read_admin(&env)?;
        admin.require_auth();

        let supply = read_total_supply(&env);
        let new_supply = supply
            .checked_add(amount)
            .ok_or(MetadataError::ArithmeticOverflow)?;
        let balance = read_balance(&env, &to);
        let new_balance = balance
            .checked_add(amount)
            .ok_or(MetadataError::ArithmeticOverflow)?;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &new_supply);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);

        env.events().publish((NS, EV_MINT, to), amount);
        Ok(())
    }

    /// Burn `amount` tokens from `from`. The token holder must authorise.
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), MetadataError> {
        require_positive(amount)?;
        from.require_auth();

        let balance = read_balance(&env, &from);
        if balance < amount {
            return Err(MetadataError::InsufficientBalance);
        }

        let supply = read_total_supply(&env);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(balance - amount));

        env.events().publish((NS, EV_BURN, from), amount);
        Ok(())
    }

    /// Transfer `amount` tokens from `from` to `to`.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), MetadataError> {
        require_positive(amount)?;
        from.require_auth();

        let from_balance = read_balance(&env, &from);
        if from_balance < amount {
            return Err(MetadataError::InsufficientBalance);
        }
        let to_balance = read_balance(&env, &to)
            .checked_add(amount)
            .ok_or(MetadataError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_balance);

        env.events().publish((NS, EV_XFER, from, to), amount);
        Ok(())
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        read_balance(&env, &user)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn admin(env: Env) -> Result<Address, MetadataError> {
        read_admin(&env)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_admin(env: &Env) -> Result<Address, MetadataError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(MetadataError::NotInitialized)
}

fn read_string(env: &Env, key: &DataKey) -> Result<String, MetadataError> {
    env.storage()
        .instance()
        .get(key)
        .ok_or(MetadataError::NotInitialized)
}

fn read_decimals(env: &Env) -> Result<u32, MetadataError> {
    env.storage()
        .instance()
        .get(&DataKey::Decimals)
        .ok_or(MetadataError::NotInitialized)
}

fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn read_balance(env: &Env, user: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap_or(0)
}

fn require_positive(amount: i128) -> Result<(), MetadataError> {
    if amount <= 0 {
        Err(MetadataError::InvalidAmount)
    } else {
        Ok(())
    }
}

fn require_non_empty(s: &String) -> Result<(), MetadataError> {
    if s.is_empty() {
        Err(MetadataError::EmptyString)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test;
