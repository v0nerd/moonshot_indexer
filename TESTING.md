# Testing Guide for Moonshot Indexer

This guide covers all the different ways to test the Moonshot Indexer project.

## Prerequisites

1. **Rust and Cargo** - Make sure you have Rust installed
2. **PostgreSQL** - For database testing
3. **Environment Variables** - Set up your `.env` file

## 1. Unit Tests

Unit tests test individual functions and modules in isolation.

### Running Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests with output (even for passing tests)
cargo test -- --nocapture

# Run a specific test
cargo test test_swap_event_creation

# Run tests in a specific module
cargo test tests::
```

### What Unit Tests Cover

- **Data Structure Validation**: Testing `SwapEvent` creation and validation
- **Environment Variable Loading**: Ensuring required env vars are present
- **Basic Logic**: Any business logic in your modules

## 2. Integration Tests

Integration tests test the full application flow, including external dependencies.

### Running Integration Tests

```bash
# Run integration tests
cargo test --test integration_tests

# Run with verbose output
cargo test --test integration_tests -- --nocapture
```

### What Integration Tests Cover

- **Database Connection**: Testing PostgreSQL connectivity
- **RPC Connection**: Testing WebSocket connection to blockchain node
- **End-to-End Flow**: Testing complete data flow

## 3. Manual Testing

### Setting Up Test Environment

1. **Copy the test environment file**:
   ```bash
   cp test.env.example .env.test
   ```

2. **Edit `.env.test`** with your test values:
   ```bash
   # Use a testnet RPC URL
   RPC_URL=wss://eth-sepolia.g.alchemy.com/v2/YOUR_TEST_API_KEY
   
   # Use a test database
   DATABASE_URL=postgresql://username:password@localhost:5432/moonshot_test
   ```

3. **Set up test database**:
   ```bash
   # Create test database
   createdb moonshot_test
   
   # Run migrations (if you have them)
   # cargo sqlx migrate run --database-url postgresql://username:password@localhost:5432/moonshot_test
   ```

### Running Manual Tests

```bash
# Test with test environment
RUST_ENV=test cargo run

# Test specific functionality
cargo run -- --test-mode
```

## 4. Mock Testing

For testing without external dependencies, you can create mocks:

### Example Mock Test

```rust
#[cfg(test)]
mod mock_tests {
    use super::*;
    
    // Mock provider for testing
    struct MockProvider;
    
    impl MockProvider {
        async fn get_block_number(&self) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(12345)
        }
    }
    
    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider;
        let block_number = provider.get_block_number().await.unwrap();
        assert_eq!(block_number, 12345);
    }
}
```

## 5. Property-Based Testing

For more comprehensive testing, consider using `proptest`:

```bash
# Add to Cargo.toml
[dependencies]
proptest = "1.0"
```

### Example Property Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_swap_event_properties(amount_in in 0.0..1000000.0f64) {
        let event = SwapEvent {
            tx_hash: "0x1234567890abcdef".to_string(),
            token_in: "0xTokenA".to_string(),
            token_out: "0xTokenB".to_string(),
            amount_in,
            amount_out: amount_in * 0.95, // 5% slippage
            timestamp: 1640995200,
        };
        
        prop_assert!(event.amount_in >= 0);
        prop_assert!(event.amount_out >= 0);
        prop_assert!(event.amount_out <= event.amount_in);
    }
}
```

## 6. Performance Testing

### Benchmarking

```bash
# Add to Cargo.toml
[dependencies]
criterion = "0.5"

[[bench]]
name = "swap_event_bench"
harness = false
```

### Example Benchmark

```rust
// benches/swap_event_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moonshot_indexer::SwapEvent;

fn swap_event_creation_benchmark(c: &mut Criterion) {
    c.bench_function("swap_event_creation", |b| {
        b.iter(|| {
            SwapEvent {
                tx_hash: black_box("0x1234567890abcdef".to_string()),
                token_in: black_box("0xTokenA".to_string()),
                token_out: black_box("0xTokenB".to_string()),
                amount_in: black_box(100.0),
                amount_out: black_box(95.0),
                timestamp: black_box(1640995200),
            }
        })
    });
}

criterion_group!(benches, swap_event_creation_benchmark);
criterion_main!(benches);
```

## 7. Continuous Integration Testing

### GitHub Actions Example

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run tests
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost:5432/test_db
        RPC_URL: wss://eth-sepolia.g.alchemy.com/v2/dummy
      run: cargo test
```

## 8. Testing Best Practices

### 1. Test Organization
- Keep unit tests close to the code they test
- Use integration tests for external dependencies
- Separate test data from test logic

### 2. Test Naming
- Use descriptive test names that explain what is being tested
- Follow the pattern: `test_[function_name]_[scenario]`

### 3. Test Data
- Use realistic test data
- Test edge cases and error conditions
- Avoid hardcoded values when possible

### 4. Async Testing
- Use `#[tokio::test]` for async tests
- Handle timeouts appropriately
- Mock external services when possible

### 5. Database Testing
- Use separate test databases
- Clean up test data after each test
- Use transactions to rollback changes

## 9. Common Test Commands

```bash
# Run all tests
cargo test

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin

# Run tests with specific features
cargo test --features test

# Run tests in release mode
cargo test --release

# Run tests with specific target
cargo test --target x86_64-unknown-linux-gnu

# Run tests with timeout
timeout 30s cargo test

# Run tests and generate HTML report
cargo test --no-run
cargo tarpaulin --out Html
```

## 10. Troubleshooting

### Common Issues

1. **Database Connection Fails**
   - Check if PostgreSQL is running
   - Verify DATABASE_URL format
   - Ensure database exists

2. **RPC Connection Fails**
   - Check if RPC_URL is correct
   - Verify network connectivity
   - Check API key validity

3. **Environment Variables Missing**
   - Ensure `.env` file exists
   - Check variable names match code
   - Verify file is in project root

4. **Test Timeouts**
   - Increase timeout values
   - Mock slow external services
   - Use faster test data

### Debugging Tests

```bash
# Run with debug output
RUST_LOG=debug cargo test

# Run single test with debug
RUST_LOG=debug cargo test test_name -- --nocapture

# Use println! in tests (will show with --nocapture)
cargo test -- --nocapture
``` 