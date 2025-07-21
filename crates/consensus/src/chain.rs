use bits_core::{Did, PlatformError, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::config::ChainConfig;

/// A block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: Did,
    pub nonce: u64,
}

/// A transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: Did,
    pub timestamp: u64,
    pub data: TransactionData,
    pub signature: Vec<u8>,
}

/// Transaction data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    /// Token transfer
    Transfer {
        to: Did,
        amount: u64,
    },
    /// Username registration
    RegisterUsername {
        username: String,
        fee: u64,
    },
    /// Username transfer
    TransferUsername {
        username: String,
        to: Did,
        price: Option<u64>,
    },
    /// List username for sale
    ListUsername {
        username: String,
        price: u64,
    },
    /// Create content
    CreateContent {
        content_hash: String,
        content_type: String,
        price: Option<u64>,
    },
    /// Purchase content
    PurchaseContent {
        content_hash: String,
        price: u64,
    },
}

/// Simple blockchain implementation
pub struct Blockchain {
    chain: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    balances: Arc<RwLock<HashMap<Did, u64>>>,
    config: ChainConfig,
}

impl Blockchain {
    /// Create a new blockchain with default local config
    pub fn new(genesis_validator: Did) -> Self {
        Self::with_config(ChainConfig::local(genesis_validator))
    }
    
    /// Create blockchain with specific configuration
    pub fn with_config(config: ChainConfig) -> Self {
        let blockchain = Self {
            chain: Arc::new(RwLock::new(Vec::new())),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        };

        // Create genesis block
        let genesis = Block {
            index: 0,
            timestamp: config.genesis.timestamp,
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "".to_string(),
            validator: config.genesis.validator.clone(),
            nonce: 0,
        };

        let genesis_hash = Self::calculate_hash(&genesis);
        let mut genesis = genesis;
        genesis.hash = genesis_hash;

        // Initialize genesis block and balances
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut chain = blockchain.chain.write().await;
                chain.push(genesis);

                let mut balances = blockchain.balances.write().await;
                // Initialize balances from genesis allocations
                for (did, balance) in &config.genesis.allocations {
                    balances.insert(did.clone(), *balance);
                }
            })
        });

        blockchain
    }

    /// Add a new transaction to pending
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        // Verify transaction signature
        // TODO: Implement signature verification

        let mut pending = self.pending_transactions.write().await;
        pending.push(transaction);
        Ok(())
    }

    /// Mine a new block
    pub async fn mine_block(&self, validator: Did) -> Result<Block> {
        let chain = self.chain.read().await;
        let last_block = chain.last()
            .ok_or_else(|| PlatformError::Consensus("No genesis block".into()))?;

        let mut pending = self.pending_transactions.write().await;
        let transactions: Vec<Transaction> = pending.drain(..).collect();

        let mut block = Block {
            index: last_block.index + 1,
            timestamp: Self::current_timestamp(),
            transactions,
            previous_hash: last_block.hash.clone(),
            hash: "".to_string(),
            validator,
            nonce: 0,
        };

        // Simple proof of work
        while !Self::is_valid_proof(&block, self.config.consensus.difficulty) {
            block.nonce += 1;
        }

        block.hash = Self::calculate_hash(&block);

        // Process transactions and update state
        self.process_block(&block).await?;

        // Add block to chain
        let mut chain = self.chain.write().await;
        chain.push(block.clone());

        Ok(block)
    }

    /// Process a block's transactions
    async fn process_block(&self, block: &Block) -> Result<()> {
        let mut balances = self.balances.write().await;

        for tx in &block.transactions {
            match &tx.data {
                TransactionData::Transfer { to, amount } => {
                    // Deduct from sender
                    let sender_balance = balances.get(&tx.from).copied().unwrap_or(0);
                    if sender_balance < *amount {
                        return Err(PlatformError::Consensus("Insufficient balance".into()));
                    }
                    balances.insert(tx.from.clone(), sender_balance - amount);

                    // Add to receiver
                    let receiver_balance = balances.get(to).copied().unwrap_or(0);
                    balances.insert(to.clone(), receiver_balance + amount);
                }
                TransactionData::RegisterUsername { fee, .. } => {
                    // Deduct registration fee
                    let balance = balances.get(&tx.from).copied().unwrap_or(0);
                    if balance < *fee {
                        return Err(PlatformError::Consensus("Insufficient balance for fee".into()));
                    }
                    balances.insert(tx.from.clone(), balance - fee);

                    // Fee goes to validator
                    let validator_balance = balances.get(&block.validator).copied().unwrap_or(0);
                    balances.insert(block.validator.clone(), validator_balance + fee);
                }
                // TODO: Handle other transaction types
                _ => {}
            }
        }

        Ok(())
    }

    /// Get balance for a DID
    pub async fn get_balance(&self, did: &Did) -> u64 {
        let balances = self.balances.read().await;
        balances.get(did).copied().unwrap_or(0)
    }

    /// Get the latest block
    pub async fn get_latest_block(&self) -> Option<Block> {
        let chain = self.chain.read().await;
        chain.last().cloned()
    }

    /// Get block by index
    pub async fn get_block(&self, index: u64) -> Option<Block> {
        let chain = self.chain.read().await;
        chain.get(index as usize).cloned()
    }

    /// Get blockchain length
    pub async fn get_height(&self) -> u64 {
        let chain = self.chain.read().await;
        chain.len() as u64
    }

    /// Get blockchain configuration
    pub fn get_config(&self) -> &ChainConfig {
        &self.config
    }

    /// Get all blocks (for transaction history)
    pub async fn get_all_blocks(&self) -> Vec<Block> {
        let chain = self.chain.read().await;
        chain.clone()
    }

    /// Calculate hash for a block
    fn calculate_hash(block: &Block) -> String {
        let mut hasher = Hasher::new();
        hasher.update(&block.index.to_le_bytes());
        hasher.update(&block.timestamp.to_le_bytes());
        hasher.update(block.previous_hash.as_bytes());
        hasher.update(&block.nonce.to_le_bytes());

        // Include transaction data
        for tx in &block.transactions {
            hasher.update(tx.id.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Check if proof of work is valid
    fn is_valid_proof(block: &Block, difficulty: u32) -> bool {
        let hash = Self::calculate_hash(block);
        let target = "0".repeat(difficulty as usize);
        hash.starts_with(&target)
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Transaction {
    /// Create a new transaction
    pub fn new(from: Did, data: TransactionData) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = Hasher::new()
            .update(from.0.as_bytes())
            .update(&timestamp.to_le_bytes())
            .finalize()
            .to_hex()
            .to_string();

        Self {
            id,
            from,
            timestamp,
            data,
            signature: vec![], // TODO: Implement actual signing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockchain_creation() {
        let validator = Did("validator".to_string());
        let blockchain = Blockchain::new(validator.clone());

        assert_eq!(blockchain.get_height().await, 1);
        assert_eq!(blockchain.get_balance(&validator).await, 1_000_000);
    }

    #[tokio::test]
    async fn test_transaction_processing() {
        let validator = Did("validator".to_string());
        let alice = Did("alice".to_string());
        let blockchain = Blockchain::new(validator.clone());

        // Create transfer transaction
        let tx = Transaction::new(
            validator.clone(),
            TransactionData::Transfer {
                to: alice.clone(),
                amount: 1000,
            },
        );

        blockchain.add_transaction(tx).await.unwrap();
        blockchain.mine_block(validator.clone()).await.unwrap();

        assert_eq!(blockchain.get_balance(&validator).await, 999_000);
        assert_eq!(blockchain.get_balance(&alice).await, 1000);
    }
}
