//! # Custom Struct Types in Soroban
//!
//! This example demonstrates various custom struct patterns in Soroban smart contracts:
//!
//! ## What's Covered
//!
//! ### 1. Basic Struct Definitions
//! - Using `#[contracttype]` derive macro for storage compatibility
//! - Field types and their constraints
//! - Default implementations and constructors
//!
//! ### 2. Nested Structs
//! - Structs containing other structs
//! - Hierarchical data structures
//! - Access patterns for nested fields
//!
//! ### 3. Struct Storage Patterns
//! - Direct struct storage
//! - Key-based struct storage
//! - Struct collections and maps
//!
//! ### 4. Serialization and Deserialization
//! - Automatic serialization with Soroban SDK
//! - Manual serialization techniques
//! - Data validation and conversion

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Basic Struct Definitions
// ---------------------------------------------------------------------------

/// Basic user profile struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    /// User's wallet address
    pub address: Address,
    /// User's display name
    pub name: String,
    /// User's email (optional)
    pub email: Option<String>,
    /// User's avatar hash
    pub avatar_hash: Option<String>,
    /// User's reputation score
    pub reputation: u32,
    /// Whether the user is verified
    pub verified: bool,
    /// Account creation timestamp
    pub created_at: u64,
}

/// Asset information struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetInfo {
    /// Asset contract address
    pub contract_address: Address,
    /// Asset symbol
    pub symbol: String,
    /// Asset name
    pub name: String,
    /// Number of decimal places
    pub decimals: u32,
    /// Total supply (if available)
    pub total_supply: Option<i128>,
    /// Whether asset is native
    pub native: bool,
}

/// Transaction record struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: u64,
    /// Sender address
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Asset being transferred
    pub asset: AssetInfo,
    /// Amount transferred
    pub amount: i128,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Transaction memo
    pub memo: Option<String>,
    /// Transaction status
    pub status: TransactionStatus,
}

/// Transaction status enum
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    Pending = 0,
    Completed = 1,
    Failed = 2,
    Cancelled = 3,
}

// ---------------------------------------------------------------------------
// Nested Struct Examples
// ---------------------------------------------------------------------------

/// Complex user profile with nested structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedUserProfile {
    /// Basic profile information
    pub profile: UserProfile,
    /// User preferences
    pub preferences: UserPreferences,
    /// User statistics
    pub statistics: UserStatistics,
    /// User security settings
    pub security: SecuritySettings,
}

/// User preferences struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreferences {
    /// Preferred language
    pub language: String,
    /// Theme preference
    pub theme: Theme,
    /// Notification settings
    pub notifications: NotificationSettings,
    /// Privacy settings
    pub privacy: PrivacySettings,
}

/// Theme enum
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Theme {
    Light = 0,
    Dark = 1,
    Auto = 2,
}

/// Notification settings
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationSettings {
    /// Email notifications enabled
    pub email_enabled: bool,
    /// Push notifications enabled
    pub push_enabled: bool,
    /// Transaction notifications
    pub transaction_notifications: bool,
    /// Marketing notifications
    pub marketing_notifications: bool,
}

/// Privacy settings
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivacySettings {
    /// Profile visibility
    pub profile_visibility: Visibility,
    /// Show online status
    pub show_online_status: bool,
    /// Allow direct messages
    pub allow_direct_messages: bool,
}

/// Visibility enum
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public = 0,
    Friends = 1,
    Private = 2,
}

/// User statistics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStatistics {
    /// Total transactions
    pub total_transactions: u64,
    /// Total volume traded
    pub total_volume: i128,
    /// Successful transactions
    pub successful_transactions: u64,
    /// Failed transactions
    pub failed_transactions: u64,
    /// Average transaction size
    pub avg_transaction_size: i128,
    /// Last activity timestamp
    pub last_activity: u64,
}

/// Security settings
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecuritySettings {
    /// Two-factor authentication enabled
    pub two_factor_enabled: bool,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum daily transaction limit
    pub daily_transaction_limit: i128,
    /// Require confirmation for large transactions
    pub large_transaction_threshold: i128,
    /// Trusted devices
    pub trusted_devices: Vec<Address>,
}

// ---------------------------------------------------------------------------
// Complex Nested Example: Portfolio
// ---------------------------------------------------------------------------

/// User portfolio containing multiple assets
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Portfolio {
    /// Portfolio owner
    pub owner: Address,
    /// Portfolio name
    pub name: String,
    /// Portfolio description
    pub description: Option<String>,
    /// Asset holdings
    pub holdings: Vec<AssetHolding>,
    /// Portfolio metadata
    pub metadata: PortfolioMetadata,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Individual asset holding
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetHolding {
    /// Asset information
    pub asset: AssetInfo,
    /// Quantity held
    pub quantity: i128,
    /// Average purchase price
    pub avg_purchase_price: i128,
    /// Current value (if available)
    pub current_value: Option<i128>,
    /// Purchase history
    pub purchase_history: Vec<PurchaseRecord>,
}

/// Purchase record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseRecord {
    /// Purchase timestamp
    pub timestamp: u64,
    /// Quantity purchased
    pub quantity: i128,
    /// Price per unit
    pub price: i128,
    /// Transaction fee
    pub fee: i128,
}

/// Portfolio metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioMetadata {
    /// Portfolio type
    pub portfolio_type: PortfolioType,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Investment strategy
    pub strategy: String,
    /// Target allocation percentages
    pub target_allocations: Vec<AssetAllocation>,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Portfolio type enum
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PortfolioType {
    Conservative = 0,
    Balanced = 1,
    Aggressive = 2,
    Custom = 3,
}

/// Risk level enum
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
}

/// Asset allocation target
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAllocation {
    /// Asset information
    pub asset: AssetInfo,
    /// Target percentage (0-100)
    pub target_percentage: u32,
    /// Current percentage
    pub current_percentage: u32,
}

/// Performance metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    /// Total return percentage
    pub total_return: i32,
    /// Annual return percentage
    pub annual_return: i32,
    /// Sharpe ratio
    pub sharpe_ratio: Option<i32>,
    /// Maximum drawdown
    pub max_drawdown: i32,
    /// Volatility
    pub volatility: i32,
}

// ---------------------------------------------------------------------------
// Contract Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// General errors (1000-1099)
    InvalidInput = 1000,
    Unauthorized = 1001,
    NotFound = 1002,
    AlreadyExists = 1003,
    InvalidAddress = 1004,
    InsufficientBalance = 1005,

    /// Struct validation errors (1100-1199)
    InvalidStruct = 1100,
    MissingRequiredField = 1101,
    InvalidFieldValue = 1102,
    StructTooLarge = 1103,
    SerializationError = 1104,

    /// Storage errors (1200-1299)
    StorageError = 1200,
    StorageQuotaExceeded = 1201,
    DataCorrupted = 1202,

    /// Portfolio errors (1300-1399)
    PortfolioNotFound = 1300,
    InvalidPortfolio = 1301,
    InvalidHolding = 1302,
    AllocationError = 1303,

    /// User errors (1400-1499)
    UserNotFound = 1400,
    InvalidUserProfile = 1401,
    ProfileAlreadyExists = 1402,
}

// ---------------------------------------------------------------------------
// Main Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct CustomStructsContract;

#[contractimpl]
impl CustomStructsContract {
    /// Initialize contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyExists);
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("init"), &true);
        Ok(())
    }

    /// Create a new user profile
    pub fn create_user_profile(
        env: Env,
        address: Address,
        name: String,
        email: Option<String>,
    ) -> Result<UserProfile, ContractError> {
        let profile = UserProfile {
            address: address.clone(),
            name: name.clone(),
            email,
            avatar_hash: None,
            reputation: 0,
            verified: false,
            created_at: env.ledger().timestamp(),
        };

        // Store the profile
        env.storage()
            .instance()
            .set(&(symbol_short!("profile"), address.clone()), &profile);

        Ok(profile)
    }

    /// Get user profile
    pub fn get_user_profile(env: Env, address: Address) -> Result<UserProfile, ContractError> {
        let profile: UserProfile = env
            .storage()
            .instance()
            .get(&(symbol_short!("profile"), address))
            .ok_or(ContractError::UserNotFound)?;
        Ok(profile)
    }

    /// Update user profile
    pub fn update_user_profile(
        env: Env,
        address: Address,
        name: Option<String>,
        email: Option<String>,
        avatar_hash: Option<String>,
    ) -> Result<UserProfile, ContractError> {
        let mut profile: UserProfile = env
            .storage()
            .instance()
            .get(&(symbol_short!("profile"), address.clone()))
            .ok_or(ContractError::UserNotFound)?;

        // Update fields if provided
        if let Some(new_name) = name {
            profile.name = new_name;
        }
        if let Some(new_email) = email {
            profile.email = Some(new_email);
        }
        if let Some(new_avatar) = avatar_hash {
            profile.avatar_hash = Some(new_avatar);
        }

        // Store updated profile
        env.storage()
            .instance()
            .set(&(symbol_short!("profile"), address.clone()), &profile);

        Ok(profile)
    }

    /// Create a new portfolio
    pub fn create_portfolio(
        env: Env,
        owner: Address,
        name: String,
        description: Option<String>,
        portfolio_type: PortfolioType,
    ) -> Result<Portfolio, ContractError> {
        let portfolio = Portfolio {
            owner: owner.clone(),
            name: name.clone(),
            description,
            holdings: Vec::new(&env),
            metadata: PortfolioMetadata {
                portfolio_type,
                risk_level: RiskLevel::Medium,
                strategy: String::from_str(&env, "balanced"),
                target_allocations: Vec::new(&env),
                performance: PerformanceMetrics {
                    total_return: 0,
                    annual_return: 0,
                    sharpe_ratio: None,
                    max_drawdown: 0,
                    volatility: 0,
                },
            },
            last_updated: env.ledger().timestamp(),
        };

        // Store the portfolio
        env.storage()
            .instance()
            .set(&(symbol_short!("portfolio"), owner.clone(), name.clone()), &portfolio);

        Ok(portfolio)
    }

    /// Get portfolio
    pub fn get_portfolio(
        _env: Env,
        owner: Address,
        name: String,
    ) -> Result<Portfolio, ContractError> {
        let portfolio: Portfolio = _env
            .storage()
            .instance()
            .get(&(symbol_short!("portfolio"), owner, name))
            .ok_or(ContractError::PortfolioNotFound)?;
        Ok(portfolio)
    }

    /// Add asset to portfolio
    pub fn add_asset_to_portfolio(
        env: Env,
        owner: Address,
        portfolio_name: String,
        asset: AssetInfo,
        quantity: i128,
        price: i128,
    ) -> Result<(), ContractError> {
        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&(symbol_short!("portfolio"), owner.clone(), portfolio_name.clone()))
            .ok_or(ContractError::PortfolioNotFound)?;

        // Create new holding
        let holding = AssetHolding {
            asset: asset.clone(),
            quantity,
            avg_purchase_price: price,
            current_value: None,
            purchase_history: vec![
                &env,
                PurchaseRecord {
                    timestamp: env.ledger().timestamp(),
                    quantity,
                    price,
                    fee: 0,
                },
            ],
        };

        // Add to holdings
        portfolio.holdings.push_back(holding);
        portfolio.last_updated = env.ledger().timestamp();

        // Store updated portfolio
        env.storage()
            .instance()
            .set(&(symbol_short!("portfolio"), owner, portfolio_name), &portfolio);

        Ok(())
    }

    /// Create extended user profile
    pub fn create_extended_profile(
        env: Env,
        address: Address,
        name: String,
        language: String,
    ) -> Result<ExtendedUserProfile, ContractError> {
        // First create basic profile
        let basic_profile = Self::create_user_profile(env.clone(), address.clone(), name, None)?;

        // Create extended profile
        let extended_profile = ExtendedUserProfile {
            profile: basic_profile,
            preferences: UserPreferences {
                language,
                theme: Theme::Auto,
                notifications: NotificationSettings {
                    email_enabled: true,
                    push_enabled: true,
                    transaction_notifications: true,
                    marketing_notifications: false,
                },
                privacy: PrivacySettings {
                    profile_visibility: Visibility::Public,
                    show_online_status: true,
                    allow_direct_messages: true,
                },
            },
            statistics: UserStatistics {
                total_transactions: 0,
                total_volume: 0,
                successful_transactions: 0,
                failed_transactions: 0,
                avg_transaction_size: 0,
                last_activity: env.ledger().timestamp(),
            },
            security: SecuritySettings {
                two_factor_enabled: false,
                session_timeout: 3600, // 1 hour
                daily_transaction_limit: 1000000,
                large_transaction_threshold: 10000,
                trusted_devices: Vec::new(&env),
            },
        };

        // Store extended profile
        env.storage()
            .instance()
            .set(&(symbol_short!("ext_prof"), address.clone()), &extended_profile);

        Ok(extended_profile)
    }

    /// Get extended user profile
    pub fn get_extended_profile(
        env: Env,
        address: Address,
    ) -> Result<ExtendedUserProfile, ContractError> {
        let profile: ExtendedUserProfile = env
            .storage()
            .instance()
            .get(&(symbol_short!("ext_prof"), address))
            .ok_or(ContractError::UserNotFound)?;
        Ok(profile)
    }

    /// Demonstrate struct serialization
    pub fn serialize_struct(env: Env, profile: UserProfile) -> Result<i32, ContractError> {
        // In Soroban, structs are automatically serialized when stored
        // This function demonstrates the concept by storing and retrieving
        
        // Store the struct
        let temp_key = symbol_short!("temp_ser");
        env.storage().instance().set(&temp_key, &profile);
        
        // Retrieve and convert to bytes (conceptual)
        let _retrieved: UserProfile = env
            .storage()
            .instance()
            .get(&temp_key)
            .ok_or(ContractError::SerializationError)?;
        
        // Clean up
        env.storage().instance().remove(&temp_key);
        
        // Return a simple hash representation (in real implementation, you'd use proper serialization)
        Ok(12345) // Placeholder
    }

    /// Demonstrate struct validation
    pub fn validate_struct(_env: Env, profile: UserProfile) -> Result<bool, ContractError> {
        // Validate name length
        if profile.name.len() == 0 || profile.name.len() > 100 {
            return Err(ContractError::InvalidFieldValue);
        }

        // Validate reputation range
        if profile.reputation > 1000 {
            return Err(ContractError::InvalidFieldValue);
        }

        // Validate email format if present
        if let Some(email) = &profile.email {
            if email.len() == 0 || email.len() > 255 {
                return Err(ContractError::InvalidFieldValue);
            }
            // In a real implementation, you'd validate email format
        }

        Ok(true)
    }

    /// Get all portfolios for a user
    pub fn get_user_portfolios(env: Env, _owner: Address) -> Result<Vec<String>, ContractError> {
        // This is a simplified implementation
        // In a real contract, you'd maintain an index of user portfolios
        let portfolios = Vec::new(&env);
        Ok(portfolios)
    }

    /// Calculate portfolio value
    pub fn calculate_portfolio_value(
        env: Env,
        owner: Address,
        portfolio_name: String,
    ) -> Result<i128, ContractError> {
        let portfolio: Portfolio = Self::get_portfolio(env.clone(), owner, portfolio_name)?;
        
        let mut total_value = 0i128;
        
        for holding in portfolio.holdings.iter() {
            if let Some(current_value) = holding.current_value {
                total_value += current_value;
            } else {
                // Use purchase price as fallback
                total_value += holding.quantity * holding.avg_purchase_price;
            }
        }
        
        Ok(total_value)
    }
}

// Pull in the dedicated test module.
#[cfg(test)]
mod test;
