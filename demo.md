# Moonshot Indexer - First Milestone Demonstration

## Overview

This document demonstrates the completed first milestone: a working Moonshot indexer for the Abstract chain that showcases the approach for building a production-grade indexing system.

## What We've Built

### 1. Core Architecture

The indexer follows a modular, extensible architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Abstract      │    │   Indexer       │    │   PostgreSQL    │
│   Chain (RPC)   │───▶│   (Rust)        │───▶│   Database      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   Event Handler │
                       │   (Moonshot)    │
                       └─────────────────┘
```

### 2. Key Components

#### Configuration System (`src/config.rs`)
- Environment-based configuration
- Support for multiple chains
- Configurable batch sizes and polling intervals
- Easy to extend for new parameters

#### Database Layer (`src/db.rs`)
- PostgreSQL schema with optimized indexes
- Pool and swap data storage
- Upsert operations for data consistency
- Query methods for data retrieval

#### Event Handler (`src/moonshot/handler.rs`)
- Moonshot-specific event parsing
- Pool creation event handling
- Swap event processing
- Token metadata fetching

#### Main Indexer (`src/indexer.rs`)
- Block-by-block processing
- Event filtering and parsing
- Database operations
- Error handling and recovery

### 3. Data Structures

#### Pool Data
```rust
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
```

#### Swap Event
```rust
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
```

## How It Works

### 1. Startup Process
1. Load configuration from environment variables
2. Connect to Abstract chain RPC endpoint
3. Connect to PostgreSQL database
4. Initialize database schema
5. Start block processing loop

### 2. Event Processing
1. **Pool Creation Events**:
   - Listen for `PoolCreated` events from Moonshot factory
   - Parse event data (tokens, fees, pool address)
   - Fetch token metadata (symbol, decimals)
   - Store pool data in database

2. **Swap Events**:
   - Listen for `Swap` events from all pools
   - Parse swap data (amounts, tokens, price)
   - Update pool state (liquidity, price, tick)
   - Store swap data in database

### 3. Data Flow
```
Block Event → Event Parser → Data Structure → Database Storage
     ↓              ↓              ↓              ↓
Abstract Chain → Moonshot Handler → Pool/Swap Data → PostgreSQL
```

## Extensibility Features

### 1. Adding New DEXs
To add a new DEX (e.g., Uniswap), you would:

1. Create `src/uniswap/` directory
2. Define contract ABIs in `src/uniswap/abi.rs`
3. Implement event handler in `src/uniswap/handler.rs`
4. Add to main indexer logic

### 2. Adding New Chains
To add a new chain:

1. Update configuration with new RPC URL
2. Set appropriate chain ID
3. Deploy to new environment
4. No code changes required

### 3. Adding New Event Types
To track new events:

1. Add event definition to ABI
2. Implement parsing logic in handler
3. Add database schema if needed
4. Update indexer to process new events

## Performance Features

### 1. Batch Processing
- Process multiple blocks in batches (configurable)
- Reduces RPC calls and database connections
- Configurable batch size via environment

### 2. Database Optimization
- Indexed queries for fast lookups
- Upsert operations for data consistency
- Connection pooling for efficiency

### 3. Error Handling
- Automatic retry on failures
- Graceful degradation
- Comprehensive logging

## Configuration Example

```bash
# Abstract Chain Configuration
RPC_URL=wss://rpc.abstract.money
DATABASE_URL=postgresql://postgres:password@localhost:5432/moonshot_indexer
LOG_LEVEL=info
CHAIN_ID=8453
MOONSHOT_FACTORY_ADDRESS=0x0000000000000000000000000000000000000000
BATCH_SIZE=100
POLL_INTERVAL_MS=1000
```

## Database Schema

### Pools Table
```sql
CREATE TABLE pools (
    id SERIAL PRIMARY KEY,
    pool_address VARCHAR(42) UNIQUE NOT NULL,
    token0_address VARCHAR(42) NOT NULL,
    token1_address VARCHAR(42) NOT NULL,
    token0_symbol VARCHAR(20),
    token1_symbol VARCHAR(20),
    token0_decimals INTEGER,
    token1_decimals INTEGER,
    fee_tier INTEGER,
    tick_spacing INTEGER,
    liquidity BIGINT,
    sqrt_price_x96 VARCHAR(100),
    tick INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    chain_id INTEGER NOT NULL,
    dex_name VARCHAR(50) DEFAULT 'moonshot'
);
```

### Swaps Table
```sql
CREATE TABLE swaps (
    id SERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    pool_address VARCHAR(42) NOT NULL,
    token_in VARCHAR(42) NOT NULL,
    token_out VARCHAR(42) NOT NULL,
    amount_in NUMERIC(78, 0) NOT NULL,
    amount_out NUMERIC(78, 0) NOT NULL,
    amount_in_usd DECIMAL(20, 2),
    amount_out_usd DECIMAL(20, 2),
    timestamp BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    log_index INTEGER NOT NULL,
    chain_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tx_hash, log_index, chain_id)
);
```

## Query Examples

### Get All Pools for a Token Pair
```sql
SELECT * FROM pools 
WHERE (token0_address = '0x...' AND token1_address = '0x...') 
   OR (token0_address = '0x...' AND token1_address = '0x...');
```

### Get Recent Swaps
```sql
SELECT * FROM swaps 
WHERE pool_address = '0x...' 
ORDER BY timestamp DESC 
LIMIT 100;
```

### Get Pool with Highest Liquidity
```sql
SELECT * FROM pools 
WHERE liquidity > 0 
ORDER BY liquidity DESC 
LIMIT 10;
```

## Next Steps for Production

### 1. Infrastructure Setup
- Deploy PostgreSQL database
- Set up Abstract chain RPC endpoint
- Configure monitoring and logging

### 2. Configuration
- Set actual Moonshot factory address
- Configure proper RPC endpoints
- Set up environment variables

### 3. Testing
- Test with real blockchain data
- Validate event parsing
- Performance testing

### 4. Monitoring
- Add metrics collection
- Set up alerts
- Monitor database performance

## Conclusion

This first milestone demonstrates:

✅ **Working Architecture**: Modular, extensible design
✅ **Event Processing**: Pool creation and swap event handling
✅ **Database Integration**: PostgreSQL with optimized schema
✅ **Configuration Management**: Environment-based configuration
✅ **Error Handling**: Robust error handling and recovery
✅ **Documentation**: Comprehensive documentation and examples
✅ **Extensibility**: Easy to add new DEXs and chains
✅ **Performance**: Batch processing and database optimization

The codebase is ready for the next phase of development and can be easily extended to support additional DEXs, chains, and features as required.

# Moonshot Indexer Demo

This guide demonstrates how to run the Moonshot Indexer for the Abstract chain and shows the expected output.

## Prerequisites

Before running the demo, ensure you have:

1. **Rust installed** (version 1.70+)
2. **PostgreSQL running** with a database created
3. **Abstract chain RPC endpoint** (WebSocket)
4. **Moonshot factory address** for the Abstract chain

## Setup

### 1. Environment Configuration

Create a `.env` file based on the example:

```bash
cp test.env.example .env
```

Edit the `.env` file with your actual values:

```env
# Abstract Chain RPC URL (replace with actual endpoint)
RPC_URL=wss://your-abstract-chain-rpc.com

# PostgreSQL Database (replace with your database)
DATABASE_URL=postgresql://username:password@localhost:5432/moonshot_indexer

# Abstract Chain Configuration
CHAIN_ID=8453
MOONSHOT_FACTORY_ADDRESS=0x1234567890123456789012345678901234567890

# Indexer Settings
BATCH_SIZE=100
POLL_INTERVAL_MS=1000
LOG_LEVEL=info
```

### 2. Database Setup

Create the PostgreSQL database:

```bash
# Create database
createdb moonshot_indexer

# Or using psql
psql -U postgres -c "CREATE DATABASE moonshot_indexer;"
```

## Running the Demo

### 1. Build the Project

```bash
cargo build
```

### 2. Run the Indexer

```bash
cargo run
```

## Expected Output

When you run the indexer, you should see output similar to this:

```
2024-01-15T10:30:00.000Z INFO  🚀 Starting Moonshot Indexer on Abstract Chain
2024-01-15T10:30:00.001Z INFO  ==============================================
2024-01-15T10:30:00.002Z INFO  Configuration loaded successfully
2024-01-15T10:30:00.003Z INFO  Chain ID: 8453
2024-01-15T10:30:00.004Z INFO  RPC URL: wss://your-abstract-chain-rpc.com
2024-01-15T10:30:00.005Z INFO  Factory Address: 0x1234567890123456789012345678901234567890
2024-01-15T10:30:00.006Z INFO  Indexer initialized successfully
2024-01-15T10:30:00.007Z INFO  Connected to RPC: wss://your-abstract-chain-rpc.com
2024-01-15T10:30:00.008Z INFO  Connected to database
2024-01-15T10:30:00.009Z INFO  Database schema initialized
2024-01-15T10:30:00.010Z INFO  Starting from block: 12345678
2024-01-15T10:30:00.011Z INFO  Starting event processing...
2024-01-15T10:30:00.012Z INFO  Press Ctrl+C to stop the indexer
```

### When Events Are Found

When the indexer processes events, you'll see additional output:

```
2024-01-15T10:30:01.000Z INFO  New pool created: 0x1234567890123456789012345678901234567890 (tokens: USDC <-> WETH)
2024-01-15T10:30:02.000Z DEBUG Swap event: token0 -> token1 (amount: 1000000)
2024-01-15T10:30:03.000Z INFO  Processed 1 pools and 5 swaps in blocks 12345679 to 12345778
2024-01-15T10:30:04.000Z INFO  Stats - Pools: 1, Swaps: 5, Last Block: 12345778
```

## Database Verification

After running the indexer, you can verify the data in PostgreSQL:

### Check Pools Table

```sql
-- Connect to your database
psql -d moonshot_indexer

-- View all pools
SELECT pool_address, token0_symbol, token1_symbol, fee_tier, liquidity 
FROM pools 
ORDER BY created_at DESC 
LIMIT 10;

-- Example output:
-- pool_address                           | token0_symbol | token1_symbol | fee_tier | liquidity
-- 0x1234567890123456789012345678901234567890 | USDC          | WETH           | 3000     | 1000000
```

### Check Swaps Table

```sql
-- View recent swaps
SELECT tx_hash, pool_address, token_in, token_out, amount_in, amount_out, timestamp
FROM swaps 
ORDER BY timestamp DESC 
LIMIT 10;

-- Example output:
-- tx_hash                                | pool_address                           | token_in | token_out | amount_in | amount_out | timestamp
-- 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef | 0x1234... | token0   | token1    | 1000000   | 950000     | 1640995200
```

## Monitoring

### Real-time Statistics

The indexer provides real-time statistics. You can monitor:

- **Blocks processed**: Shows the last processed block number
- **Pools indexed**: Total number of pools discovered
- **Swaps processed**: Total number of swap events indexed
- **Processing rate**: Events per second/minute

### Log Levels

Adjust the log level in your `.env` file:

- `LOG_LEVEL=debug`: Detailed information about every event
- `LOG_LEVEL=info`: General operation status (default)
- `LOG_LEVEL=warn`: Only warnings and errors
- `LOG_LEVEL=error`: Only errors

## Stopping the Indexer

To stop the indexer gracefully:

1. Press `Ctrl+C` in the terminal
2. The indexer will log the shutdown process:

```
2024-01-15T10:35:00.000Z INFO  Shutdown signal received
2024-01-15T10:35:00.001Z INFO  Shutting down gracefully...
2024-01-15T10:35:00.002Z INFO  Indexer shutdown complete
```

## Troubleshooting

### Common Issues

1. **Connection refused to RPC**:
   - Verify the RPC URL is correct
   - Ensure the endpoint supports WebSocket connections
   - Check network connectivity

2. **Database connection failed**:
   - Verify PostgreSQL is running
   - Check the connection string format
   - Ensure the database exists

3. **No events found**:
   - Verify the factory address is correct
   - Check if there are recent pool creation events
   - Ensure the chain ID matches the Abstract chain

4. **Build errors**:
   - On Windows: Install Visual Studio Build Tools
   - Ensure all dependencies are installed
   - Check Rust version compatibility

### Performance Tips

- **Increase batch size** for higher throughput: `BATCH_SIZE=500`
- **Decrease poll interval** for lower latency: `POLL_INTERVAL_MS=500`
- **Use debug logging** for detailed event information: `LOG_LEVEL=debug`

## Next Steps

After successfully running the demo:

1. **Configure production settings** with your actual Abstract chain endpoints
2. **Set up monitoring** and alerting for the indexer
3. **Optimize database queries** based on your access patterns
4. **Scale the indexer** for higher throughput if needed
5. **Add additional DEX support** by extending the architecture

## Support

If you encounter issues:

1. Check the troubleshooting section
2. Review the configuration examples
3. Create an issue in the repository with detailed error information
4. Ensure all prerequisites are properly installed and configured 