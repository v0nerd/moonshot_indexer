pub mod config;
pub mod moonshot;
pub mod types;

pub use config::Config;
pub use types::{IndexingStats, PoolData, SwapEvent, TokenData};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_data_creation() {
        let pool = PoolData::new(
            "0x1234567890123456789012345678901234567890".to_string(),
            "0xTokenA".to_string(),
            "0xTokenB".to_string(),
            8453,
            "moonshot".to_string(),
        );

        assert_eq!(
            pool.pool_address,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(pool.token0_address, "0xTokenA");
        assert_eq!(pool.token1_address, "0xTokenB");
        assert_eq!(pool.chain_id, 8453);
        assert_eq!(pool.dex_name, "moonshot");
    }

    #[test]
    fn test_swap_event_creation() {
        let swap = SwapEvent::new(
            "0x1234567890abcdef".to_string(),
            "0xPoolAddress".to_string(),
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
        assert_eq!(swap.pool_address, "0xPoolAddress");
        assert_eq!(swap.token_in, "token0");
        assert_eq!(swap.token_out, "token1");
        assert_eq!(swap.amount_in, 1000);
        assert_eq!(swap.amount_out, 950);
        assert_eq!(swap.chain_id, 8453);
    }

    #[test]
    fn test_json_serialization() {
        let pool = PoolData {
            pool_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0_address: "0xTokenA".to_string(),
            token1_address: "0xTokenB".to_string(),
            token0_symbol: Some("TOKENA".to_string()),
            token1_symbol: Some("TOKENB".to_string()),
            token0_decimals: Some(18),
            token1_decimals: Some(6),
            fee_tier: Some(3000),
            tick_spacing: Some(60),
            liquidity: Some(1000000),
            sqrt_price_x96: Some("123456789".to_string()),
            tick: Some(1000),
            chain_id: 8453,
            dex_name: "moonshot".to_string(),
        };

        let json = serde_json::to_string(&pool).unwrap();
        let deserialized: PoolData = serde_json::from_str(&json).unwrap();

        assert_eq!(pool.pool_address, deserialized.pool_address);
        assert_eq!(pool.token0_address, deserialized.token0_address);
        assert_eq!(pool.token1_address, deserialized.token1_address);
        assert_eq!(pool.chain_id, deserialized.chain_id);
        assert_eq!(pool.dex_name, deserialized.dex_name);
    }
}
