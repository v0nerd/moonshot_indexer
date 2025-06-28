# Moonshot Indexer

A high-performance blockchain indexer for Moonshot DEX pools on Abstract chain, built in Rust.

## Features

- **Real-time Pool Indexing**: Indexes Moonshot pool creation events
- **Swap Event Tracking**: Captures and stores all swap transactions
- **Unified Data Structure**: DEX-agnostic data format for easy querying
- **High Performance**: Sub-second indexing with PostgreSQL backend
- **Extensible Architecture**: Easy to add new DEXs and chains
- **Production Ready**: Error handling, logging, and monitoring

## Architecture

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

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Abstract chain RPC endpoint

### 1. Setup Environment

Create a `.env` file in the project root:

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

### 2. Setup Database

```bash
# Create database
createdb moonshot_indexer

# Or using Docker
docker run --name postgres-moonshot \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=moonshot_indexer \
  -p 5432:5432 \
  -d postgres:13
```

### 3. Build and Run

```bash
# Build the project
cargo build --release

# Run the indexer
cargo run --release
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RPC_URL` | Abstract chain WebSocket RPC URL | Required |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `CHAIN_ID` | Chain ID for Abstract | 8453 |
| `MOONSHOT_FACTORY_ADDRESS` | Moonshot factory contract address | Required |
| `BATCH_SIZE` | Number of blocks to process per batch | 100 |
| `POLL_INTERVAL_MS` | Polling interval in milliseconds | 1000 |
| `LOG_LEVEL` | Logging level (debug, info, warn, error) | info |

## Data Schema

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

## API Usage

### Query Pools

```sql
-- Get all pools for a token pair
SELECT * FROM pools 
WHERE (token0_address = '0x...' AND token1_address = '0x...') 
   OR (token0_address = '0x...' AND token1_address = '0x...');

-- Get pools with highest liquidity
SELECT * FROM pools 
WHERE liquidity > 0 
ORDER BY liquidity DESC 
LIMIT 10;
```

### Query Swaps

```sql
-- Get recent swaps for a pool
SELECT * FROM swaps 
WHERE pool_address = '0x...' 
ORDER BY timestamp DESC 
LIMIT 100;

-- Get swap volume by token
SELECT token_in, SUM(amount_in) as total_volume 
FROM swaps 
WHERE timestamp > extract(epoch from now() - interval '24 hours')
GROUP BY token_in 
ORDER BY total_volume DESC;
```

## Extending for New DEXs

The indexer is designed to be easily extensible. To add a new DEX:

1. **Create DEX Module**: Add a new module in `src/dex_name/`
2. **Define ABIs**: Create contract ABIs for the new DEX
3. **Implement Handler**: Create event parsing logic
4. **Update Indexer**: Add the new DEX to the main indexer

Example structure for a new DEX:

```rust
// src/uniswap/mod.rs
pub mod abi;
pub mod handler;

// src/uniswap/abi.rs
pub const UNISWAP_FACTORY_ABI: &str = "...";

// src/uniswap/handler.rs
pub struct UniswapHandler {
    // Implementation
}
```

## Performance Optimization

- **Batch Processing**: Process multiple blocks in batches
- **Database Indexes**: Optimized indexes for fast queries
- **Connection Pooling**: Efficient database connection management
- **Error Recovery**: Automatic retry on failures
- **Memory Management**: Efficient memory usage for large datasets

## Monitoring

The indexer provides basic monitoring through:

- Console logging with configurable levels
- Database statistics tracking
- Error reporting and recovery
- Block processing metrics

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_pool_creation
```

### Code Structure

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports
├── config.rs        # Configuration management
├── db.rs           # Database operations
├── indexer.rs      # Main indexing logic
├── types.rs        # Data structures
└── moonshot/       # Moonshot-specific logic
    ├── mod.rs
    ├── abi.rs      # Contract ABIs
    └── handler.rs  # Event handlers
```

## Deployment

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/moonshot_indexer /usr/local/bin/
CMD ["moonshot_indexer"]
```

### Systemd Service

```ini
[Unit]
Description=Moonshot Indexer
After=network.target

[Service]
Type=simple
User=indexer
WorkingDirectory=/opt/moonshot-indexer
ExecStart=/opt/moonshot-indexer/moonshot_indexer
Restart=always
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

For questions or issues:
- Create an issue on GitHub
- Check the documentation
- Review the test examples

---

**Note**: This is a demonstrative example for the first milestone. The actual Moonshot factory address and RPC endpoints need to be configured for the Abstract chain.
