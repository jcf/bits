use bits_core::{Did, PlatformError, Result};
use std::sync::Arc;

use crate::{
    chain::{Blockchain, Transaction, TransactionData},
    registry::UsernameRegistry,
    token::{Token, TokenEconomics},
};

/// Consensus service integrating blockchain, registry, and token economics
pub struct ConsensusService {
    blockchain: Arc<Blockchain>,
    registry: Arc<UsernameRegistry>,
    token: Arc<Token>,
    economics: Arc<TokenEconomics>,
}

impl ConsensusService {
    /// Create a new consensus service
    pub fn new(genesis_validator: Did) -> Self {
        let economics = Arc::new(TokenEconomics::default());
        let registry = Arc::new(UsernameRegistry::new(economics.username_base_fee));
        let blockchain = Arc::new(Blockchain::new(genesis_validator));
        let token = Arc::new(Token::default());

        Self {
            blockchain,
            registry,
            token,
            economics,
        }
    }

    /// Register a new username
    pub async fn register_username(
        &self,
        username: String,
        owner_did: Did,
    ) -> Result<String> {
        // Check if available
        if !self.registry.is_available(&username).await {
            return Err(PlatformError::Consensus("Username already taken".into()));
        }

        // Calculate fee
        let fee = self.registry.calculate_fee(&username);

        // Check balance
        let balance = self.blockchain.get_balance(&owner_did).await;
        if balance < fee {
            return Err(PlatformError::Consensus(
                format!("Insufficient balance. Need {} BITS", self.token.format_amount(fee))
            ));
        }

        // Create transaction
        let tx = Transaction::new(
            owner_did.clone(),
            TransactionData::RegisterUsername {
                username: username.clone(),
                fee,
            },
        );

        // Add to blockchain
        self.blockchain.add_transaction(tx.clone()).await?;

        // For demo, immediately mine the block
        let validator = owner_did.clone(); // In real system, would be actual validator
        self.blockchain.mine_block(validator).await?;

        // Register in registry
        let height = self.blockchain.get_height().await;
        self.registry.register(username.clone(), owner_did, height).await?;

        Ok(tx.id)
    }

    /// Transfer username to another user
    pub async fn transfer_username(
        &self,
        username: &str,
        from_did: Did,
        to_did: Did,
        price: Option<u64>,
    ) -> Result<String> {
        // Verify ownership
        let registration = self.registry.get_registration(username).await
            .ok_or_else(|| PlatformError::Consensus("Username not found".into()))?;

        if registration.owner_did != from_did {
            return Err(PlatformError::Consensus("Not the owner".into()));
        }

        // If price is set, verify buyer has funds
        if let Some(amount) = price {
            let buyer_balance = self.blockchain.get_balance(&to_did).await;
            if buyer_balance < amount {
                return Err(PlatformError::Consensus("Buyer has insufficient funds".into()));
            }
        }

        // Create transaction
        let tx = Transaction::new(
            from_did.clone(),
            TransactionData::TransferUsername {
                username: username.to_string(),
                to: to_did.clone(),
                price,
            },
        );

        // Add to blockchain
        self.blockchain.add_transaction(tx.clone()).await?;

        // Mine block
        let validator = from_did.clone(); // In real system, would be actual validator
        self.blockchain.mine_block(validator).await?;

        // Update registry
        self.registry.transfer(username, &from_did, to_did).await?;

        Ok(tx.id)
    }

    /// List username for sale
    pub async fn list_username(
        &self,
        username: &str,
        seller_did: Did,
        price: u64,
    ) -> Result<String> {
        // Create transaction
        let tx = Transaction::new(
            seller_did.clone(),
            TransactionData::ListUsername {
                username: username.to_string(),
                price,
            },
        );

        // Add to blockchain
        self.blockchain.add_transaction(tx.clone()).await?;

        // Mine block
        let validator = seller_did.clone();
        self.blockchain.mine_block(validator).await?;

        // Update registry
        let height = self.blockchain.get_height().await;
        self.registry.list_for_sale(username, &seller_did, price, height).await?;

        Ok(tx.id)
    }

    /// Get user balance
    pub async fn get_balance(&self, did: &Did) -> u64 {
        self.blockchain.get_balance(did).await
    }

    /// Get username registration
    pub async fn get_username(&self, username: &str) -> Option<crate::registry::UsernameRegistration> {
        self.registry.get_registration(username).await
    }

    /// Get all marketplace listings
    pub async fn get_marketplace_listings(&self) -> Vec<crate::registry::MarketplaceListing> {
        self.registry.get_listings().await
    }

    /// Search available usernames
    pub async fn search_usernames(&self, pattern: &str) -> Vec<String> {
        self.registry.search_available(pattern).await
    }

    /// Get blockchain info
    pub async fn get_blockchain_info(&self) -> serde_json::Value {
        let height = self.blockchain.get_height().await;
        let latest_block = self.blockchain.get_latest_block().await;

        serde_json::json!({
            "height": height,
            "latest_block": latest_block,
            "token": {
                "name": self.token.name,
                "symbol": self.token.symbol,
                "total_supply": self.token.total_supply,
            },
            "economics": {
                "username_base_fee": self.economics.username_base_fee,
                "validator_fee_share": self.economics.validator_fee_share,
                "burn_rate": self.economics.burn_rate,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_username_registration_flow() {
        let validator = Did("validator".to_string());
        let service = ConsensusService::new(validator.clone());

        // Check initial balance
        let balance = service.get_balance(&validator).await;
        assert_eq!(balance, 1_000_000); // Initial balance

        // Register username
        let tx_id = service.register_username(
            "alice".to_string(),
            validator.clone(),
        ).await.unwrap();

        assert!(!tx_id.is_empty());

        // Check balance after registration
        let new_balance = service.get_balance(&validator).await;
        assert_eq!(new_balance, 1_000_000); // Fee goes back to validator as miner

        // Verify registration
        let reg = service.get_username("alice").await.unwrap();
        assert_eq!(reg.owner_did, validator);
    }
}