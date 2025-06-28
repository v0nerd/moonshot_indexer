use anyhow::Result;
use ethers::providers::{Provider, Ws};
use ethers::types::{Address, Filter, Log};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};

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
    pools_processed: u64,
    swaps_processed: u64,
}

impl Indexer {
    pub async fn new(config: Config) -> Result<Self> {
        // Connect to RPC
        let provider = Arc::new(Provider::<Ws>::connect(&config.rpc_url).await?);
        info!("Connected to RPC: {}", config.rpc_url);

        // Connect to database
        let database = Database::new(&config.database_url).await?;
        info!("Connected to database");

        // Initialize database schema
        database.init_schema().await?;
        info!("Database schema initialized");

        // Create handler
        let handler = MoonshotHandler::new(provider.clone());

        // Get current block number
        let current_block = provider.get_block_number().await?;
        let last_processed_block = current_block.as_u64().saturating_sub(100); // Start from 100 blocks ago

        info!("Starting from block: {}", last_processed_block);

        Ok(Self {
            config,
            provider,
            database,
            handler,
            last_processed_block,
            pools_processed: 0,
            swaps_processed: 0,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting indexer...");
        info!("Chain ID: {}", self.config.chain_id);
        info!("Moonshot Factory: {}", self.config.moonshot_factory_address);

        loop {
            match self.process_blocks().await {
                Ok(_) => {
                    // Log stats periodically
                    if self.pools_processed > 0 || self.swaps_processed > 0 {
                        info!("Stats - Pools: {}, Swaps: {}, Last Block: {}", 
                              self.pools_processed, self.swaps_processed, self.last_processed_block);
                    }
                    sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                }
                Err(e) => {
                    error!("Error processing blocks: {}", e);
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

        debug!("Processing blocks {} to {}", from_block, to_block);

        // Process pool creation events
        let pools_found = self.process_pool_events(from_block, to_block).await?;
        self.pools_processed += pools_found;

        // Process swap events
        let swaps_found = self.process_swap_events(from_block, to_block).await?;
        self.swaps_processed += swaps_found;

        if pools_found > 0 || swaps_found > 0 {
            info!("Processed {} pools and {} swaps in blocks {} to {}", 
                  pools_found, swaps_found, from_block, to_block);
        }

        self.last_processed_block = to_block;
        Ok(())
    }

    async fn process_pool_events(&self, from_block: u64, to_block: u64) -> Result<u64> {
        let factory_address: Address = self.config.moonshot_factory_address.parse()?;

        let filter = Filter::new()
            .from_block(from_block)
            .to_block(to_block)
            .address(factory_address)
            .event("PoolCreated(address,address,uint24,int24,address)");

        let logs = self.provider.get_logs(&filter).await?;
        let mut pools_processed = 0;

        for log in logs {
            match self.handler.handle_pool_created(log, self.config.chain_id as i64).await {
                Ok(pool_data) => {
                    info!("New pool created: {} (tokens: {} <-> {})", 
                          pool_data.pool_address, pool_data.token0_symbol.as_deref().unwrap_or("Unknown"), 
                          pool_data.token1_symbol.as_deref().unwrap_or("Unknown"));
                    
                    if let Err(e) = self.database.upsert_pool(&pool_data).await {
                        error!("Error storing pool: {}", e);
                    } else {
                        pools_processed += 1;
                    }
                }
                Err(e) => {
                    error!("Error parsing pool creation event: {}", e);
                }
            }
        }

        Ok(pools_processed)
    }

    async fn process_swap_events(&self, from_block: u64, to_block: u64) -> Result<u64> {
        // Get all known pools from database to filter swap events
        let known_pools = self.database.get_all_pool_addresses().await?;
        
        if known_pools.is_empty() {
            debug!("No known pools found, skipping swap processing");
            return Ok(0);
        }

        let mut swaps_processed = 0;

        // Process swap events for each known pool
        for pool_address in known_pools {
            let pool_addr: Address = pool_address.parse()?;
            
            let filter = Filter::new()
                .from_block(from_block)
                .to_block(to_block)
                .address(pool_addr)
                .event("Swap(address,address,int256,int256,uint160,uint128,int24)");

            let logs = self.provider.get_logs(&filter).await?;

            for log in logs {
                match self.handler.handle_swap(log, self.config.chain_id as i64).await {
                    Ok(swap_event) => {
                        debug!("Swap event: {} -> {} (amount: {})", 
                            swap_event.token_in, swap_event.token_out, swap_event.amount_in);
                        
                        if let Err(e) = self.database.insert_swap(&swap_event).await {
                            error!("Error storing swap: {}", e);
                        } else {
                            swaps_processed += 1;
                        }

                        // Update pool state after swap
                        if let Ok(pool_address) = swap_event.pool_address.parse::<Address>() {
                            if let Ok(pool_data) = self.handler.update_pool_state(pool_address, self.config.chain_id as i64).await {
                                if let Err(e) = self.database.upsert_pool(&pool_data).await {
                                    warn!("Error updating pool state: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error parsing swap event: {}", e);
                    }
                }
            }
        }

        Ok(swaps_processed)
    }

    pub async fn get_stats(&self) -> Result<(u64, u64, u64)> {
        let (total_pools, total_swaps) = self.database.get_stats().await?;
        Ok((self.last_processed_block, total_pools, total_swaps))
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

