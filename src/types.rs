use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapEvent {
    pub tx_hash: String,
    pub pool_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: i64,
    pub amount_out: i64,
    pub amount_in_usd: Option<f64>,
    pub amount_out_usd: Option<f64>,
    pub timestamp: i64,
    pub block_number: i64,
    pub log_index: i32,
    pub chain_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    pub pool_address: String,
    pub token0_address: String,
    pub token1_address: String,
    pub token0_symbol: Option<String>,
    pub token1_symbol: Option<String>,
    pub token0_decimals: Option<i32>,
    pub token1_decimals: Option<i32>,
    pub fee_tier: Option<i32>,
    pub tick_spacing: Option<i32>,
    pub liquidity: Option<i64>,
    pub sqrt_price_x96: Option<String>,
    pub tick: Option<i32>,
    pub chain_id: i64,
    pub dex_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub address: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<i32>,
    pub total_supply: Option<String>,
    pub chain_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStats {
    pub last_processed_block: i64,
    pub total_pools_indexed: i64,
    pub total_swaps_indexed: i64,
    pub chain_id: i64,
    pub dex_name: String,
    pub updated_at: i64,
}

impl SwapEvent {
    pub fn new(
        tx_hash: String,
        pool_address: String,
        token_in: String,
        token_out: String,
        amount_in: i64,
        amount_out: i64,
        timestamp: i64,
        block_number: i64,
        log_index: i32,
        chain_id: i64,
    ) -> Self {
        Self {
            tx_hash,
            pool_address,
            token_in,
            token_out,
            amount_in,
            amount_out,
            amount_in_usd: None,
            amount_out_usd: None,
            timestamp,
            block_number,
            log_index,
            chain_id,
        }
    }
}

impl PoolData {
    pub fn new(
        pool_address: String,
        token0_address: String,
        token1_address: String,
        chain_id: i64,
        dex_name: String,
    ) -> Self {
        Self {
            pool_address,
            token0_address,
            token1_address,
            token0_symbol: None,
            token1_symbol: None,
            token0_decimals: None,
            token1_decimals: None,
            fee_tier: None,
            tick_spacing: None,
            liquidity: None,
            sqrt_price_x96: None,
            tick: None,
            chain_id,
            dex_name,
        }
    }
}
