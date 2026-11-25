//! Test fixtures and data builders for common test scenarios.

use soroban_sdk::Address;

/// Fixture for token contract testing.
#[derive(Clone, Debug)]
pub struct TokenFixture {
    pub admin: Address,
    pub supply: u128,
    pub decimals: u32,
}

impl TokenFixture {
    /// Create a new token fixture.
    pub fn new(admin: Address, supply: u128, decimals: u32) -> Self {
        Self {
            admin,
            supply,
            decimals,
        }
    }

    /// Create a default token fixture (requires admin address).
    pub fn default_with_admin(admin: Address) -> Self {
        Self {
            admin,
            supply: 1_000_000,
            decimals: 7,
        }
    }
}

/// Builder for token fixtures.
pub struct TokenFixtureBuilder {
    admin: Address,
    supply: u128,
    decimals: u32,
}

impl TokenFixtureBuilder {
    /// Create a new token fixture builder.
    pub fn new(admin: Address) -> Self {
        Self {
            admin,
            supply: 1_000_000,
            decimals: 7,
        }
    }

    /// Set the token supply.
    pub fn with_supply(mut self, supply: u128) -> Self {
        self.supply = supply;
        self
    }

    /// Set the token decimals.
    pub fn with_decimals(mut self, decimals: u32) -> Self {
        self.decimals = decimals;
        self
    }

    /// Build the fixture.
    pub fn build(self) -> TokenFixture {
        TokenFixture {
            admin: self.admin,
            supply: self.supply,
            decimals: self.decimals,
        }
    }
}

/// Fixture for authorization contract testing.
#[derive(Clone, Debug)]
pub struct AuthFixture {
    pub admin: Address,
    pub authorized: Vec<Address>,
}

impl AuthFixture {
    /// Create a new auth fixture.
    pub fn new(admin: Address) -> Self {
        Self {
            admin,
            authorized: Vec::new(),
        }
    }

    /// Add an authorized address.
    pub fn with_authorized(mut self, address: Address) -> Self {
        self.authorized.push(address);
        self
    }
}

/// Builder for auth fixtures.
pub struct AuthFixtureBuilder {
    admin: Address,
    authorized: Vec<Address>,
}

impl AuthFixtureBuilder {
    /// Create a new auth fixture builder.
    pub fn new(admin: Address) -> Self {
        Self {
            admin,
            authorized: Vec::new(),
        }
    }

    /// Add an authorized address.
    pub fn with_authorized(mut self, address: Address) -> Self {
        self.authorized.push(address);
        self
    }

    /// Build the fixture.
    pub fn build(self) -> AuthFixture {
        AuthFixture {
            admin: self.admin,
            authorized: self.authorized,
        }
    }
}
