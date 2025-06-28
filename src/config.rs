use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub database_url: String,
    pub log_level: String,
    pub chain_id: u64,
    pub moonshot_factory_address: String,
    pub batch_size: usize,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            rpc_url: env::var("RPC_URL")?,
            database_url: env::var("DATABASE_URL")?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            chain_id: env::var("CHAIN_ID")
                .unwrap_or_else(|_| "1".to_string())
                .parse()?,
            moonshot_factory_address: env::var("MOONSHOT_FACTORY_ADDRESS")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),
            batch_size: env::var("BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            poll_interval_ms: env::var("POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
        })
    }

    pub fn is_testnet(&self) -> bool {
        self.chain_id != 1 // Mainnet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env() {
        // Set test environment variables
        env::set_var("RPC_URL", "wss://test.example.com");
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("CHAIN_ID", "8453");

        let config = Config::from_env().unwrap();

        assert_eq!(config.rpc_url, "wss://test.example.com");
        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/test"
        );
        assert_eq!(config.chain_id, 8453);
        assert_eq!(config.log_level, "info");

        // Clean up
        env::remove_var("RPC_URL");
        env::remove_var("DATABASE_URL");
        env::remove_var("CHAIN_ID");
    }
}
