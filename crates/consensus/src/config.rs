use bits_core::Did;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u64,
    pub genesis: GenesisConfig,
    pub consensus: ConsensusConfig,
}

/// Genesis block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub timestamp: u64,
    pub validator: Did,
    pub allocations: HashMap<Did, u64>,
}

/// Consensus parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub block_time: u64,
    pub difficulty: u32,
    pub max_block_size: usize,
    pub min_fee: u64,
}

impl ChainConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        let genesis_validator = Did("did:key:mainnet-genesis".to_string());
        let mut allocations = HashMap::new();
        allocations.insert(genesis_validator.clone(), 1_000_000); // 1M BITS to genesis validator
        
        Self {
            name: "mainnet".to_string(),
            chain_id: 1,
            genesis: GenesisConfig {
                timestamp: 1700000000, // Fixed timestamp for mainnet
                validator: genesis_validator,
                allocations,
            },
            consensus: ConsensusConfig {
                block_time: 10,
                difficulty: 4,
                max_block_size: 1_000_000,
                min_fee: 1,
            },
        }
    }
    
    /// Create testnet configuration
    pub fn testnet() -> Self {
        let genesis_validator = Did("did:key:testnet-genesis".to_string());
        let mut allocations = HashMap::new();
        allocations.insert(genesis_validator.clone(), 10_000_000); // 10M BITS to genesis
        
        // Add some test accounts with funds
        allocations.insert(
            Did("did:key:testnet-alice".to_string()), 
            1_000_000 // 1M BITS
        );
        allocations.insert(
            Did("did:key:testnet-bob".to_string()), 
            1_000_000 // 1M BITS
        );
        allocations.insert(
            Did("did:key:testnet-faucet".to_string()), 
            100_000_000 // 100M BITS for faucet
        );
        
        Self {
            name: "testnet".to_string(),
            chain_id: 42,
            genesis: GenesisConfig {
                timestamp: 1700000000,
                validator: genesis_validator,
                allocations,
            },
            consensus: ConsensusConfig {
                block_time: 5,
                difficulty: 2, // Easier mining on testnet
                max_block_size: 2_000_000,
                min_fee: 0, // Free transactions on testnet
            },
        }
    }
    
    /// Create local development configuration
    pub fn local(genesis_validator: Did) -> Self {
        let mut allocations = HashMap::new();
        allocations.insert(genesis_validator.clone(), 1_000_000_000); // 1B BITS for dev
        
        Self {
            name: "local".to_string(),
            chain_id: 31337, // Common local chain ID
            genesis: GenesisConfig {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                validator: genesis_validator,
                allocations,
            },
            consensus: ConsensusConfig {
                block_time: 1, // Fast blocks for development
                difficulty: 1, // Minimal difficulty
                max_block_size: 10_000_000,
                min_fee: 0,
            },
        }
    }
}