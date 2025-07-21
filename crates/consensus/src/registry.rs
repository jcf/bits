use bits_core::{Did, PlatformError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Username registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameRegistration {
    pub username: String,
    pub owner_did: Did,
    pub price: Option<u64>, // Price in BITS tokens (None = not for sale)
    pub registered_at: u64, // Block height
    pub expires_at: Option<u64>, // Optional expiration
}

/// Username marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    pub username: String,
    pub seller_did: Did,
    pub price: u64,
    pub listed_at: u64,
}

/// Username offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameOffer {
    pub username: String,
    pub buyer_did: Did,
    pub offer_amount: u64,
    pub expires_at: u64,
}

/// Registry state
pub struct UsernameRegistry {
    /// Username -> Registration mapping
    registrations: Arc<RwLock<HashMap<String, UsernameRegistration>>>,
    /// Active marketplace listings
    listings: Arc<RwLock<HashMap<String, MarketplaceListing>>>,
    /// Active offers
    offers: Arc<RwLock<Vec<UsernameOffer>>>,
    /// Registration fee (in BITS)
    base_fee: u64,
}

impl UsernameRegistry {
    pub fn new(base_fee: u64) -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            listings: Arc::new(RwLock::new(HashMap::new())),
            offers: Arc::new(RwLock::new(Vec::new())),
            base_fee,
        }
    }

    /// Check if a username is available
    pub async fn is_available(&self, username: &str) -> bool {
        let registrations = self.registrations.read().await;
        !registrations.contains_key(username)
    }

    /// Get registration info for a username
    pub async fn get_registration(&self, username: &str) -> Option<UsernameRegistration> {
        let registrations = self.registrations.read().await;
        registrations.get(username).cloned()
    }

    /// Register a new username
    pub async fn register(
        &self,
        username: String,
        owner_did: Did,
        block_height: u64,
    ) -> Result<()> {
        // Validate username format
        if !Self::is_valid_username(&username) {
            return Err(PlatformError::Consensus("Invalid username format".into()));
        }

        let mut registrations = self.registrations.write().await;
        
        // Check if already registered
        if registrations.contains_key(&username) {
            return Err(PlatformError::Consensus("Username already registered".into()));
        }

        // Create registration
        let registration = UsernameRegistration {
            username: username.clone(),
            owner_did,
            price: None,
            registered_at: block_height,
            expires_at: None, // Permanent for now
        };

        registrations.insert(username, registration);
        Ok(())
    }

    /// Transfer username ownership
    pub async fn transfer(
        &self,
        username: &str,
        from_did: &Did,
        to_did: Did,
    ) -> Result<()> {
        let mut registrations = self.registrations.write().await;
        
        match registrations.get_mut(username) {
            Some(reg) => {
                if reg.owner_did != *from_did {
                    return Err(PlatformError::Consensus("Not the owner".into()));
                }
                reg.owner_did = to_did;
                reg.price = None; // Clear any listing
                
                // Remove from marketplace if listed
                let mut listings = self.listings.write().await;
                listings.remove(username);
                
                Ok(())
            }
            None => Err(PlatformError::Consensus("Username not found".into())),
        }
    }

    /// List username for sale
    pub async fn list_for_sale(
        &self,
        username: &str,
        seller_did: &Did,
        price: u64,
        block_height: u64,
    ) -> Result<()> {
        let mut registrations = self.registrations.write().await;
        
        match registrations.get_mut(username) {
            Some(reg) => {
                if reg.owner_did != *seller_did {
                    return Err(PlatformError::Consensus("Not the owner".into()));
                }
                reg.price = Some(price);
                
                // Add to marketplace
                let listing = MarketplaceListing {
                    username: username.to_string(),
                    seller_did: seller_did.clone(),
                    price,
                    listed_at: block_height,
                };
                
                let mut listings = self.listings.write().await;
                listings.insert(username.to_string(), listing);
                
                Ok(())
            }
            None => Err(PlatformError::Consensus("Username not found".into())),
        }
    }

    /// Remove listing
    pub async fn unlist(&self, username: &str, owner_did: &Did) -> Result<()> {
        let mut registrations = self.registrations.write().await;
        
        match registrations.get_mut(username) {
            Some(reg) => {
                if reg.owner_did != *owner_did {
                    return Err(PlatformError::Consensus("Not the owner".into()));
                }
                reg.price = None;
                
                let mut listings = self.listings.write().await;
                listings.remove(username);
                
                Ok(())
            }
            None => Err(PlatformError::Consensus("Username not found".into())),
        }
    }

    /// Get all marketplace listings
    pub async fn get_listings(&self) -> Vec<MarketplaceListing> {
        let listings = self.listings.read().await;
        listings.values().cloned().collect()
    }

    /// Search for available usernames
    pub async fn search_available(&self, pattern: &str) -> Vec<String> {
        // In a real implementation, this would be more sophisticated
        let registrations = self.registrations.read().await;
        let registered: Vec<String> = registrations.keys().cloned().collect();
        
        // For demo, just return some suggestions based on pattern
        vec![
            format!("{}-dao", pattern),
            format!("{}-labs", pattern),
            format!("{}-protocol", pattern),
            format!("the-{}", pattern),
            format!("{}-official", pattern),
        ]
        .into_iter()
        .filter(|name| !registered.contains(name))
        .collect()
    }

    /// Validate username format
    fn is_valid_username(username: &str) -> bool {
        // Rules:
        // - 3-30 characters
        // - Lowercase letters, numbers, hyphens
        // - Cannot start or end with hyphen
        // - No consecutive hyphens
        let len = username.len();
        if len < 3 || len > 30 {
            return false;
        }

        if username.starts_with('-') || username.ends_with('-') {
            return false;
        }

        if username.contains("--") {
            return false;
        }

        username.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    /// Calculate registration fee based on username length and desirability
    pub fn calculate_fee(&self, username: &str) -> u64 {
        let len = username.len();
        
        // Premium pricing for short names
        let length_multiplier = match len {
            3 => 1000,  // 3-char names are premium
            4 => 100,   // 4-char names are valuable
            5 => 10,    // 5-char names have moderate premium
            _ => 1,     // 6+ chars are base price
        };

        self.base_fee * length_multiplier
    }
}

/// Registry transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryTransaction {
    /// Register a new username
    Register {
        username: String,
        owner_did: Did,
        fee_paid: u64,
    },
    /// Transfer username to another DID
    Transfer {
        username: String,
        from_did: Did,
        to_did: Did,
        price: Option<u64>, // If sold through marketplace
    },
    /// List username for sale
    ListForSale {
        username: String,
        price: u64,
    },
    /// Remove listing
    Unlist {
        username: String,
    },
    /// Make offer on username
    MakeOffer {
        username: String,
        offer_amount: u64,
    },
    /// Accept offer
    AcceptOffer {
        username: String,
        buyer_did: Did,
        price: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_username_registration() {
        let registry = UsernameRegistry::new(100);
        let did = Did("test-did".to_string());

        // Test valid registration
        assert!(registry.is_available("alice").await);
        registry.register("alice".to_string(), did.clone(), 1).await.unwrap();
        assert!(!registry.is_available("alice").await);

        // Test duplicate registration
        let result = registry.register("alice".to_string(), did.clone(), 2).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(UsernameRegistry::is_valid_username("alice"));
        assert!(UsernameRegistry::is_valid_username("alice123"));
        assert!(UsernameRegistry::is_valid_username("alice-wonderland"));

        assert!(!UsernameRegistry::is_valid_username("al")); // Too short
        assert!(!UsernameRegistry::is_valid_username("-alice")); // Starts with hyphen
        assert!(!UsernameRegistry::is_valid_username("alice-")); // Ends with hyphen
        assert!(!UsernameRegistry::is_valid_username("alice--bob")); // Consecutive hyphens
        assert!(!UsernameRegistry::is_valid_username("Alice")); // Uppercase
        assert!(!UsernameRegistry::is_valid_username("alice@bob")); // Special char
    }

    #[test]
    fn test_fee_calculation() {
        let registry = UsernameRegistry::new(100);
        
        assert_eq!(registry.calculate_fee("xyz"), 100_000); // 3 chars = 1000x
        assert_eq!(registry.calculate_fee("bits"), 10_000); // 4 chars = 100x
        assert_eq!(registry.calculate_fee("alice"), 1_000); // 5 chars = 10x
        assert_eq!(registry.calculate_fee("alice-wonderland"), 100); // 6+ chars = 1x
    }
}