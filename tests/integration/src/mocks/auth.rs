//! Mock authorization contract for testing auth interactions.

use soroban_sdk::Address;

/// Configuration for a mock authorization contract.
#[derive(Clone)]
pub struct MockAuthConfig {
    pub admin: Address,
    pub authorized_addresses: Vec<Address>,
}

impl MockAuthConfig {
    pub fn new(admin: Address) -> Self {
        Self {
            admin,
            authorized_addresses: Vec::new(),
        }
    }

    pub fn with_authorized(mut self, address: Address) -> Self {
        self.authorized_addresses.push(address);
        self
    }
}

/// Builder for creating mock authorization contracts.
pub struct MockAuthBuilder {
    config: MockAuthConfig,
}

impl MockAuthBuilder {
    /// Create a new mock auth builder.
    pub fn new(admin: Address) -> Self {
        Self {
            config: MockAuthConfig::new(admin),
        }
    }

    /// Add an authorized address.
    pub fn with_authorized(mut self, address: Address) -> Self {
        self.config = self.config.with_authorized(address);
        self
    }

    /// Get the configuration.
    pub fn build(self) -> MockAuthConfig {
        self.config
    }
}
