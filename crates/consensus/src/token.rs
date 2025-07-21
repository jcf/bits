use bits_core::Did;
use serde::{Deserialize, Serialize};

/// BITS token configuration
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            name: "Bits Token".to_string(),
            symbol: "BITS".to_string(),
            decimals: 18,
            total_supply: 1_000_000_000, // 1 billion BITS
        }
    }
}

/// Token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub did: Did,
    pub balance: u64,
    pub staked: u64,
    pub locked_until: Option<u64>,
}

/// Token economics parameters
pub struct TokenEconomics {
    /// Percentage of fees that go to validators
    pub validator_fee_share: u8, // 0-100
    /// Percentage of fees that get burned
    pub burn_rate: u8, // 0-100
    /// Minimum stake required to be a validator
    pub min_validator_stake: u64,
    /// Username registration base fee
    pub username_base_fee: u64,
    /// Content listing fee
    pub content_listing_fee: u64,
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            validator_fee_share: 80,  // 80% to validators
            burn_rate: 20,            // 20% burned
            min_validator_stake: 10_000, // 10k BITS minimum
            username_base_fee: 100,   // 100 BITS base
            content_listing_fee: 10,  // 10 BITS to list content
        }
    }
}

/// Staking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub validator: Did,
    pub amount: u64,
    pub delegators: Vec<Delegation>,
    pub commission_rate: u8, // 0-100 percentage
    pub active: bool,
}

/// Delegation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: Did,
    pub amount: u64,
    pub shares: u64,
}

/// Token utility functions
impl Token {
    /// Convert amount to smallest unit (like wei in Ethereum)
    pub fn to_base_units(&self, amount: f64) -> u64 {
        (amount * 10f64.powi(self.decimals as i32)) as u64
    }

    /// Convert from base units to decimal representation
    pub fn from_base_units(&self, units: u64) -> f64 {
        units as f64 / 10f64.powi(self.decimals as i32)
    }

    /// Format amount for display
    pub fn format_amount(&self, units: u64) -> String {
        let amount = self.from_base_units(units);
        format!("{:.2} {}", amount, self.symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_units() {
        let token = Token::default();
        
        // Test conversion
        assert_eq!(token.to_base_units(1.0), 1_000_000_000_000_000_000);
        assert_eq!(token.from_base_units(1_000_000_000_000_000_000), 1.0);
        
        // Test formatting
        assert_eq!(token.format_amount(1_500_000_000_000_000_000), "1.50 BITS");
    }
}