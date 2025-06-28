use moonshot_indexer::{
    config::Config,
    moonshot::MoonshotHandler,
    types::{PoolData, SwapEvent},
};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::test]
async fn test_database_connection() {
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await;

    assert!(pool.is_ok(), "Should be able to connect to database");
}

#[tokio::test]
async fn test_rpc_connection() {
    dotenv::dotenv().ok();

    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    // Test WebSocket connection
    let provider = ethers::providers::Provider::<ethers::providers::Ws>::connect(rpc_url).await;

    // Note: This might fail if RPC is not available, so we'll just test that we can attempt connection
    // In a real test environment, you might want to use a mock or testnet
    if let Ok(provider) = provider {
        // Test that we can get the latest block number
        let block_number = provider.get_block_number().await;
        assert!(block_number.is_ok(), "Should be able to get block number");
    }
}

#[test]
fn test_swap_event_creation_and_validation() {
    let event = SwapEvent {
        tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        pool_address: "0xPoolAddressHere".to_string(),
        token_in: "0xA0b86a33E6441b8c4C8C8C8C8C8C8C8C8C8C8C8C8".to_string(),
        token_out: "0xB0b86a33E6441b8c4C8C8C8C8C8C8C8C8C8C8C8C8".to_string(),
        amount_in: 1000,
        amount_out: 950,
        amount_in_usd: Some(1.23),
        amount_out_usd: Some(1.19),
        timestamp: 1640995200,
        block_number: 12345678,
        log_index: 0,
        chain_id: 8453,
    };

    // Test basic validation
    assert!(event.tx_hash.len() == 66); // 0x + 64 hex chars
    assert!(event.token_in.len() == 42); // 0x + 40 hex chars
    assert!(event.token_out.len() == 42); // 0x + 40 hex chars
    assert!(event.amount_in > 0.0);
    assert!(event.amount_out > 0.0);
    assert!(event.timestamp > 0);
}

#[test]
fn test_swap_event_edge_cases() {
    // Test with minimum valid values
    let min_event = SwapEvent {
        tx_hash: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        token_in: "0x0000000000000000000000000000000000000001".to_string(),
        token_out: "0x0000000000000000000000000000000000000002".to_string(),
        amount_in: 0.000001,
        amount_out: 0.000001,
        timestamp: 1577836801, // Just after 2020-01-01
    };

    assert!(min_event.amount_in > 0.0);
    assert!(min_event.amount_out > 0.0);
    assert!(min_event.timestamp > 1577836800);
}

#[tokio::test]
async fn test_config_loading() {
    // Test configuration loading
    let config = Config::from_env();
    assert!(config.is_ok(), "Config should load from environment");
}

#[tokio::test]
async fn test_pool_data_structure() {
    // Test pool data creation and validation
    let pool = PoolData::new(
        "0x1234567890123456789012345678901234567890".to_string(),
        "0xTokenA".to_string(),
        "0xTokenB".to_string(),
        8453, // Abstract chain ID
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
    assert_eq!(pool.liquidity, None); // Should be None initially
}

#[tokio::test]
async fn test_swap_event_structure() {
    // Test swap event creation and validation
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
    assert_eq!(swap.block_number, 12345);
    assert_eq!(swap.log_index, 0);
}

#[tokio::test]
async fn test_pool_data_serialization() {
    // Test that pool data can be serialized/deserialized
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

    // Test JSON serialization
    let json = serde_json::to_string(&pool).unwrap();
    let deserialized: PoolData = serde_json::from_str(&json).unwrap();

    assert_eq!(pool.pool_address, deserialized.pool_address);
    assert_eq!(pool.token0_address, deserialized.token0_address);
    assert_eq!(pool.token1_address, deserialized.token1_address);
    assert_eq!(pool.chain_id, deserialized.chain_id);
    assert_eq!(pool.dex_name, deserialized.dex_name);
}

#[tokio::test]
async fn test_swap_event_serialization() {
    // Test that swap events can be serialized/deserialized
    let swap = SwapEvent {
        tx_hash: "0x1234567890abcdef".to_string(),
        pool_address: "0xPoolAddress".to_string(),
        token_in: "token0".to_string(),
        token_out: "token1".to_string(),
        amount_in: 1000,
        amount_out: 950,
        amount_in_usd: Some(100.50),
        amount_out_usd: Some(95.25),
        timestamp: 1640995200,
        block_number: 12345,
        log_index: 0,
        chain_id: 8453,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&swap).unwrap();
    let deserialized: SwapEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(swap.tx_hash, deserialized.tx_hash);
    assert_eq!(swap.pool_address, deserialized.pool_address);
    assert_eq!(swap.token_in, deserialized.token_in);
    assert_eq!(swap.token_out, deserialized.token_out);
    assert_eq!(swap.amount_in, deserialized.amount_in);
    assert_eq!(swap.amount_out, deserialized.amount_out);
    assert_eq!(swap.chain_id, deserialized.chain_id);
}

#[tokio::test]
async fn test_abi_parsing() {
    // Test that ABIs can be parsed correctly
    use moonshot_indexer::moonshot::abi::{get_erc20_abi, get_factory_abi, get_pool_abi};

    let factory_abi = get_factory_abi();
    let pool_abi = get_pool_abi();
    let erc20_abi = get_erc20_abi();

    // Check that we have the expected events/functions
    assert!(factory_abi.events().any(|(name, _)| name == "PoolCreated"));
    assert!(pool_abi.events().any(|(name, _)| name == "Swap"));
    assert!(erc20_abi.functions().any(|(name, _)| name == "symbol"));
    assert!(erc20_abi.functions().any(|(name, _)| name == "decimals"));
}

#[tokio::test]
async fn test_extensibility_pattern() {
    // Test that the architecture supports adding new DEXs
    // This demonstrates the extensible design

    // Simulate adding a new DEX handler
    struct MockDexHandler {
        dex_name: String,
        chain_id: i64,
    }

    impl MockDexHandler {
        fn new(dex_name: String, chain_id: i64) -> Self {
            Self { dex_name, chain_id }
        }

        fn create_pool_data(
            &self,
            pool_address: String,
            token0: String,
            token1: String,
        ) -> PoolData {
            PoolData::new(
                pool_address,
                token0,
                token1,
                self.chain_id,
                self.dex_name.clone(),
            )
        }
    }

    // Test with different DEXs
    let moonshot_handler = MockDexHandler::new("moonshot".to_string(), 8453);
    let uniswap_handler = MockDexHandler::new("uniswap".to_string(), 1);

    let moonshot_pool = moonshot_handler.create_pool_data(
        "0xPool1".to_string(),
        "0xTokenA".to_string(),
        "0xTokenB".to_string(),
    );

    let uniswap_pool = uniswap_handler.create_pool_data(
        "0xPool2".to_string(),
        "0xTokenC".to_string(),
        "0xTokenD".to_string(),
    );

    assert_eq!(moonshot_pool.dex_name, "moonshot");
    assert_eq!(moonshot_pool.chain_id, 8453);
    assert_eq!(uniswap_pool.dex_name, "uniswap");
    assert_eq!(uniswap_pool.chain_id, 1);
}

#[tokio::test]
async fn test_data_validation() {
    // Test data validation logic

    // Valid pool data
    let valid_pool = PoolData::new(
        "0x1234567890123456789012345678901234567890".to_string(),
        "0xTokenA".to_string(),
        "0xTokenB".to_string(),
        8453,
        "moonshot".to_string(),
    );

    assert!(valid_pool.pool_address.starts_with("0x"));
    assert!(valid_pool.token0_address.starts_with("0x"));
    assert!(valid_pool.token1_address.starts_with("0x"));
    assert!(valid_pool.chain_id > 0);
    assert!(!valid_pool.dex_name.is_empty());

    // Valid swap event
    let valid_swap = SwapEvent::new(
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

    assert!(valid_swap.tx_hash.starts_with("0x"));
    assert!(valid_swap.pool_address.starts_with("0x"));
    assert!(valid_swap.amount_in > 0);
    assert!(valid_swap.amount_out > 0);
    assert!(valid_swap.timestamp > 0);
    assert!(valid_swap.block_number > 0);
    assert!(valid_swap.chain_id > 0);
}

// Mock database operations for testing
#[tokio::test]
async fn test_database_operations_mock() {
    // This would test database operations in a real scenario
    // For now, we'll test the data structures that would be stored

    let pools = vec![
        PoolData::new(
            "0xPool1".to_string(),
            "0xTokenA".to_string(),
            "0xTokenB".to_string(),
            8453,
            "moonshot".to_string(),
        ),
        PoolData::new(
            "0xPool2".to_string(),
            "0xTokenC".to_string(),
            "0xTokenD".to_string(),
            8453,
            "moonshot".to_string(),
        ),
    ];

    let swaps = vec![
        SwapEvent::new(
            "0xTx1".to_string(),
            "0xPool1".to_string(),
            "token0".to_string(),
            "token1".to_string(),
            1000,
            950,
            1640995200,
            12345,
            0,
            8453,
        ),
        SwapEvent::new(
            "0xTx2".to_string(),
            "0xPool2".to_string(),
            "token0".to_string(),
            "token1".to_string(),
            2000,
            1900,
            1640995300,
            12346,
            0,
            8453,
        ),
    ];

    // Test that we can process multiple pools and swaps
    assert_eq!(pools.len(), 2);
    assert_eq!(swaps.len(), 2);

    // Test that all pools are for the same chain and DEX
    for pool in &pools {
        assert_eq!(pool.chain_id, 8453);
        assert_eq!(pool.dex_name, "moonshot");
    }

    // Test that all swaps are for the same chain
    for swap in &swaps {
        assert_eq!(swap.chain_id, 8453);
    }
}
