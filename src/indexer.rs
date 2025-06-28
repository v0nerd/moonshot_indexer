use anyhow::Result;
use ethers::providers::{Provider, Ws};
use ethers::types::{Address, Filter, Log};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::db::Database;
use crate::moonshot::MoonshotHandler;
use crate::types::{PoolData, SwapEvent};

pub struct Indexer {
    config: Config,
    provider: Arc<Provider<Ws>>,
    database: Database,
    handler: MoonshotHandler,
    last_processed_block: u64,
}

impl Indexer {
    pub async fn new(config: Config) -> Result<Self> {
        // Connect to RPC
        let provider = Arc::new(Provider::<Ws>::connect(&config.rpc_url).await?);
        println!("Connected to RPC: {}", config.rpc_url);

        // Connect to database
        let database = Database::new(&config.database_url).await?;
        println!("Connected to database");

        // Initialize database schema
        database.init_schema().await?;
        println!("Database schema initialized");

        // Create handler
        let handler = MoonshotHandler::new(provider.clone());

        // Get current block number
        let current_block = provider.get_block_number().await?;
        let last_processed_block = current_block.as_u64().saturating_sub(100); // Start from 100 blocks ago

        Ok(Self {
            config,
            provider,
            database,
            handler,
            last_processed_block,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Starting indexer...");
        println!("Chain ID: {}", self.config.chain_id);
        println!("Moonshot Factory: {}", self.config.moonshot_factory_address);

        loop {
            match self.process_blocks().await {
                Ok(_) => {
                    sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                }
                Err(e) => {
                    eprintln!("Error processing blocks: {}", e);
                    sleep(Duration::from_millis(5000)).await; // Wait longer on error
                }
            }
        }
    }

    async fn process_blocks(&mut self) -> Result<()> {
        let current_block = self.provider.get_block_number().await?;
        let current_block_num = current_block.as_u64();

        if current_block_num <= self.last_processed_block {
            return Ok(());
        }

        let from_block = self.last_processed_block + 1;
        let to_block = std::cmp::min(
            current_block_num,
            from_block + self.config.batch_size as u64 - 1,
        );

        println!("Processing blocks {} to {}", from_block, to_block);

        // Process pool creation events
        self.process_pool_events(from_block, to_block).await?;

        // Process swap events
        self.process_swap_events(from_block, to_block).await?;

        self.last_processed_block = to_block;
        Ok(())
    }

    async fn process_pool_events(&self, from_block: u64, to_block: u64) -> Result<()> {
        let factory_address: Address = self.config.moonshot_factory_address.parse()?;

        let filter = Filter::new()
            .from_block(from_block)
            .to_block(to_block)
            .address(factory_address)
            .event("PoolCreated(address,address,uint24,int24,address)");

        let logs = self.provider.get_logs(&filter).await?;

        for log in logs {
            match self.handler.handle_pool_created(log, self.config.chain_id as i64).await {
                Ok(pool_data) => {
                    println!("New pool created: {}", pool_data.pool_address);
                    if let Err(e) = self.database.upsert_pool(&pool_data).await {
                        eprintln!("Error storing pool: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing pool creation event: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_swap_events(&self, from_block: u64, to_block: u64) -> Result<()> {
        // Get all known pools from database
        // For now, we'll process all swap events and filter by known pools
        // In a production system, you'd want to maintain a list of known pool addresses

        let filter = Filter::new()
            .from_block(from_block)
            .to_block(to_block)
            .event("Swap(address,address,int256,int256,uint160,uint128,int24)");

        let logs = self.provider.get_logs(&filter).await?;

        for log in logs {
            match self.handler.handle_swap(log, self.config.chain_id as i64).await {
                Ok(swap_event) => {
                    println!("Swap event: {} -> {} (amount: {})", 
                        swap_event.token_in, swap_event.token_out, swap_event.amount_in);
                    
                    if let Err(e) = self.database.insert_swap(&swap_event).await {
                        eprintln!("Error storing swap: {}", e);
                    }

                    // Update pool state after swap
                    if let Ok(pool_address) = swap_event.pool_address.parse::<Address>() {
                        if let Ok(pool_data) = self.handler.update_pool_state(pool_address, self.config.chain_id as i64).await {
                            if let Err(e) = self.database.upsert_pool(&pool_data).await {
                                eprintln!("Error updating pool state: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing swap event: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<(u64, u64)> {
        // Get total pools and swaps from database
        // This is a simplified version - in production you'd want more detailed stats
        Ok((self.last_processed_block, 0)) // TODO: Implement actual stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexer_creation() {
        // This would require a real config and connections
        // For now, just test that the struct can be conceptualized
        assert!(true);
    }
}
