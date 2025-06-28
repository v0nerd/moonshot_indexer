# Moonshot Indexer for Abstract Chain

A production-grade blockchain indexer built in Rust that ingests Ethereum node data via WebSocket, maps DeFi protocol events from Moonshot pools on the Abstract chain, and stores data in PostgreSQL.

## Features

- **Real-time Event Processing**: Listens for `PoolCreated` and `Swap` events from Moonshot factory and pools
- **Multi-chain Support**: Designed for Abstract chain (Chain ID: 8453) with extensible architecture
- **PostgreSQL Storage**: Efficient storage with proper indexing for fast queries
- **Production Ready**: Error handling, logging, graceful shutdown, and monitoring
- **DEX Agnostic**: Modular design allows easy extension to other DEX protocols

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Abstract      │    │   Moonshot      │    │   PostgreSQL    │
│   Chain RPC     │───▶│   Indexer       │───▶│   Database      │
│   (WebSocket)   │    │   (Rust)        │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Components

- **Config Module**: Environment-based configuration management
- **Database Module**: PostgreSQL schema and operations
- **Indexer Module**: Main event processing loop
- **Moonshot Handler**: Protocol-specific event parsing
- **Types Module**: Data structures for pools and swaps

## Quick Start

### Prerequisites

1. **Rust Toolchain**: Install Rust 1.70+ with Cargo
2. **PostgreSQL**: Install and configure PostgreSQL database
3. **Visual Studio Build Tools** (Windows): Required for native dependencies
4. **Abstract Chain RPC**: WebSocket endpoint for the Abstract chain

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd Moonshot_indexer
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Configure environment**:
   ```bash
   cp test.env.example .env
   # Edit .env with your actual values
   ```

4. **Set up database**:
   ```bash
   # Create PostgreSQL database
   createdb moonshot_indexer
   ```

5. **Run the indexer**:
   ```bash
   cargo run
   ```

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RPC_URL` | Abstract chain WebSocket RPC URL | - | Yes |
| `DATABASE_URL` | PostgreSQL connection string | - | Yes |
| `CHAIN_ID` | Chain ID (Abstract = 8453) | 8453 | No |
| `MOONSHOT_FACTORY_ADDRESS` | Moonshot factory contract address | - | Yes |
| `BATCH_SIZE` | Number of blocks to process per batch | 100 | No |
| `POLL_INTERVAL_MS` | Polling interval in milliseconds | 1000 | No |
| `LOG_LEVEL` | Logging level (debug, info, warn, error) | info | No |

### Example Configuration

```env
# Abstract Chain RPC URL
RPC_URL=wss://abstract-chain-rpc.example.com

# PostgreSQL Database
DATABASE_URL=postgresql://username:password@localhost:5432/moonshot_indexer

# Abstract Chain Configuration
CHAIN_ID=8453
MOONSHOT_FACTORY_ADDRESS=00xB784bBd5CCe24b510d06377f6b0af3D33B73585a

# Indexer Settings
BATCH_SIZE=100
POLL_INTERVAL_MS=1000
LOG_LEVEL=info
```

## Database Schema

### Pools Table

Stores Moonshot pool information:

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

Stores swap event data:

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

## Event Processing

### Pool Creation Events

The indexer listens for `PoolCreated` events from the Moonshot factory:

```solidity
event PoolCreated(
    address indexed token0,
    address indexed token1,
    uint24 indexed fee,
    int24 tickSpacing,
    address pool
);
```

### Swap Events

The indexer processes `Swap` events from all known pools:

```solidity
event Swap(
    address indexed sender,
    address indexed recipient,
    int256 amount0,
    int256 amount1,
    uint160 sqrtPriceX96,
    uint128 liquidity,
    int24 tick
);
```

## Development

### Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # Configuration management
├── db.rs           # Database operations
├── indexer.rs      # Main indexing logic
├── types.rs        # Data structures
└── moonshot/
    ├── mod.rs      # Moonshot module
    ├── handler.rs  # Event handlers
    └── abi.rs      # Contract ABIs
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_config_loading
```

### Building for Production

```bash
# Release build
cargo build --release

# Run release binary
./target/release/moonshot_indexer
```

## Monitoring and Logging

The indexer provides comprehensive logging with different levels:

- **INFO**: General operation status, pool creation, statistics
- **DEBUG**: Detailed event processing information
- **WARN**: Non-critical issues (e.g., failed pool state updates)
- **ERROR**: Critical errors requiring attention

### Example Log Output

```
2024-01-15T10:30:00.000Z INFO  🚀 Starting Moonshot Indexer on Abstract Chain
2024-01-15T10:30:00.001Z INFO  Configuration loaded successfully
2024-01-15T10:30:00.002Z INFO  Connected to RPC: wss://abstract-chain-rpc.example.com
2024-01-15T10:30:00.003Z INFO  Connected to database
2024-01-15T10:30:00.004Z INFO  Database schema initialized
2024-01-15T10:30:00.005Z INFO  Starting from block: 12345678
2024-01-15T10:30:01.000Z INFO  New pool created: 0x1234... (tokens: USDC <-> WETH)
2024-01-15T10:30:02.000Z DEBUG Swap event: token0 -> token1 (amount: 1000000)
```

## Troubleshooting

### Common Issues

1. **Build Errors on Windows**:
   - Install Visual Studio Build Tools
   - Ensure C++ build tools are included

2. **Database Connection Issues**:
   - Verify PostgreSQL is running
   - Check connection string format
   - Ensure database exists

3. **RPC Connection Issues**:
   - Verify WebSocket URL format
   - Check network connectivity
   - Ensure RPC endpoint supports WebSocket

4. **Event Processing Errors**:
   - Verify factory address is correct
   - Check contract ABI compatibility
   - Review chain ID configuration

### Performance Tuning

- **Batch Size**: Increase for higher throughput, decrease for lower latency
- **Poll Interval**: Adjust based on network conditions and event frequency
- **Database Indexes**: Optimize queries based on your access patterns

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions and support:
- Create an issue in the repository
- Check the troubleshooting section
- Review the configuration examples
