use anyhow::Result;
use std::env;
use tokio::signal;
use tracing::{info, error, warn};

use moonshot_indexer::config::Config;
use moonshot_indexer::indexer::Indexer;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Moonshot Indexer on Abstract Chain");
    info!("==============================================");

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            info!("Chain ID: {}", config.chain_id);
            info!("RPC URL: {}", config.rpc_url);
            info!("Factory Address: {}", config.moonshot_factory_address);
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };

    // Create and start indexer
    let mut indexer = match Indexer::new(config).await {
        Ok(indexer) => {
            info!("Indexer initialized successfully");
            indexer
        }
        Err(e) => {
            error!("Failed to initialize indexer: {}", e);
            return Err(e);
        }
    };

    info!("Starting event processing...");
    info!("Press Ctrl+C to stop the indexer");

    // Handle graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        info!("Shutdown signal received");
    };

    // Run indexer until shutdown
    tokio::select! {
        _ = indexer.start() => {
            error!("Indexer stopped unexpectedly");
        }
        _ = shutdown_signal => {
            info!("Shutting down gracefully...");
        }
    }

    info!("Indexer shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_environment_variables_loading() {
        // Test that environment variables can be loaded
        dotenv::dotenv().ok();
        
        // Test that RPC_URL exists (should fail if not set)
        let rpc_url = env::var("RPC_URL");
        assert!(rpc_url.is_ok(), "RPC_URL should be set in environment");
        
        // Test that DATABASE_URL exists (should fail if not set)
        let db_url = env::var("DATABASE_URL");
        assert!(db_url.is_ok(), "DATABASE_URL should be set in environment");
    }

    #[test]
    fn test_config_loading() {
        dotenv::dotenv().ok();
        
        // Test that config can be loaded from environment
        let config_result = Config::from_env();
        assert!(config_result.is_ok(), "Config should load from environment");
        
        if let Ok(config) = config_result {
            assert!(!config.rpc_url.is_empty(), "RPC URL should not be empty");
            assert!(!config.database_url.is_empty(), "Database URL should not be empty");
            assert!(config.chain_id > 0, "Chain ID should be positive");
        }
    }
} 