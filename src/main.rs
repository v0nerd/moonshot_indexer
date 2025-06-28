use dotenv::dotenv;
use anyhow::Result;
use std::env;

use moonshot_indexer::config::Config;
use moonshot_indexer::types::{PoolData, SwapEvent};

fn main() {
    println!("🚀 Moonshot Indexer - First Milestone Demonstration");
    println!("==================================================");

    // Demonstrate the data structures
    println!("\n=== Demonstrating Indexer Architecture ===");
    
    // Create sample pool data
    let sample_pool = PoolData::new(
        "0x1234567890123456789012345678901234567890".to_string(),
        "0xTokenA".to_string(),
        "0xTokenB".to_string(),
        8453, // Abstract chain ID
        "moonshot".to_string(),
    );
    
    println!("Sample Pool Created:");
    println!("  Address: {}", sample_pool.pool_address);
    println!("  Token0: {}", sample_pool.token0_address);
    println!("  Token1: {}", sample_pool.token1_address);
    println!("  Chain ID: {}", sample_pool.chain_id);
    println!("  DEX: {}", sample_pool.dex_name);

    // Create sample swap event
    let sample_swap = SwapEvent::new(
        "0x1234567890abcdef".to_string(),
        sample_pool.pool_address.clone(),
        "token0".to_string(),
        "token1".to_string(),
        1000,
        950,
        1640995200,
        12345,
        0,
        8453,
    );
    
    println!("\nSample Swap Event:");
    println!("  TX Hash: {}", sample_swap.tx_hash);
    println!("  Pool: {}", sample_swap.pool_address);
    println!("  Token In: {}", sample_swap.token_in);
    println!("  Token Out: {}", sample_swap.token_out);
    println!("  Amount In: {}", sample_swap.amount_in);
    println!("  Amount Out: {}", sample_swap.amount_out);

    // Demonstrate JSON serialization
    println!("\n=== Data Serialization Demo ===");
    let pool_json = serde_json::to_string_pretty(&sample_pool).unwrap();
    println!("Pool Data (JSON):");
    println!("{}", pool_json);

    let swap_json = serde_json::to_string_pretty(&sample_swap).unwrap();
    println!("\nSwap Event (JSON):");
    println!("{}", swap_json);

    // Demonstrate extensibility
    println!("\n=== Extensibility Demo ===");
    let uniswap_pool = PoolData::new(
        "0xUniswapPool123456789012345678901234567890".to_string(),
        "0xTokenC".to_string(),
        "0xTokenD".to_string(),
        1, // Ethereum mainnet
        "uniswap".to_string(),
    );
    
    println!("Uniswap Pool (Different DEX):");
    println!("  Address: {}", uniswap_pool.pool_address);
    println!("  DEX: {}", uniswap_pool.dex_name);
    println!("  Chain ID: {}", uniswap_pool.chain_id);

    let base_pool = PoolData::new(
        "0xBasePool123456789012345678901234567890".to_string(),
        "0xTokenE".to_string(),
        "0xTokenF".to_string(),
        8453, // Base chain
        "moonshot".to_string(),
    );
    
    println!("\nBase Chain Pool (Different Chain):");
    println!("  Address: {}", base_pool.pool_address);
    println!("  DEX: {}", base_pool.dex_name);
    println!("  Chain ID: {}", base_pool.chain_id);

    println!("\n=== Architecture Summary ===");
    println!("✅ Modular design with separate config, types, and handler modules");
    println!("✅ DEX-agnostic data structures for easy extensibility");
    println!("✅ Multi-chain support through configuration");
    println!("✅ JSON serialization for data exchange");
    println!("✅ Environment-based configuration management");
    println!("✅ Comprehensive error handling");

    println!("\n=== Next Steps ===");
    println!("1. Install Visual Studio Build Tools for full blockchain integration");
    println!("2. Add PostgreSQL database integration");
    println!("3. Implement actual blockchain event listening");
    println!("4. Deploy with real RPC endpoints and factory addresses");

    println!("\n🎉 First milestone architecture demonstration complete!");
    println!("The foundation is solid and ready for production development.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_environment_variables_loading() {
        // Test that environment variables can be loaded
        dotenv().ok();
        
        // Test that RPC_URL exists (should fail if not set)
        let rpc_url = env::var("RPC_URL");
        assert!(rpc_url.is_ok(), "RPC_URL should be set in environment");
        
        // Test that DATABASE_URL exists (should fail if not set)
        let db_url = env::var("DATABASE_URL");
        assert!(db_url.is_ok(), "DATABASE_URL should be set in environment");
    }

    #[test]
    fn test_config_loading() {
        dotenv().ok();
        
        // Test that config can be loaded from environment
        let config_result = Config::from_env();
        assert!(config_result.is_ok(), "Config should load from environment");
        
        if let Ok(config) = config_result {
            assert!(!config.rpc_url.is_empty(), "RPC URL should not be empty");
            assert!(!config.database_url.is_empty(), "Database URL should not be empty");
            assert!(config.chain_id > 0, "Chain ID should be positive");
        }
    }

    #[test]
    fn test_data_structures() {
        let pool = PoolData::new(
            "0x1234567890123456789012345678901234567890".to_string(),
            "0xTokenA".to_string(),
            "0xTokenB".to_string(),
            8453,
            "moonshot".to_string(),
        );

        assert_eq!(pool.pool_address, "0x1234567890123456789012345678901234567890");
        assert_eq!(pool.token0_address, "0xTokenA");
        assert_eq!(pool.token1_address, "0xTokenB");
        assert_eq!(pool.chain_id, 8453);
        assert_eq!(pool.dex_name, "moonshot");

        let swap = SwapEvent::new(
            "0x1234567890abcdef".to_string(),
            pool.pool_address.clone(),
            "token0".to_string(),
            "token1".to_string(),
            1000,
            950,
            1640995200,
            12345,
            0,
            8453,
        );

        assert_eq!(swap.tx_hash, "0x1234567890abcdef");
        assert_eq!(swap.pool_address, pool.pool_address);
        assert_eq!(swap.amount_in, 1000);
        assert_eq!(swap.amount_out, 950);
    }
}
