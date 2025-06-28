use sqlx::{PgPool, Row};
use anyhow::Result;
use crate::types::{PoolData, SwapEvent};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        // Create pools table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pools (
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
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create swaps table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS swaps (
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
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_pools_address ON pools(pool_address)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_pools_tokens ON pools(token0_address, token1_address)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_swaps_tx_hash ON swaps(tx_hash)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_swaps_pool ON swaps(pool_address)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_swaps_timestamp ON swaps(timestamp)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn upsert_pool(&self, pool: &PoolData) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pools (
                pool_address, token0_address, token1_address, token0_symbol, token1_symbol,
                token0_decimals, token1_decimals, fee_tier, tick_spacing, liquidity,
                sqrt_price_x96, tick, chain_id, dex_name, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, CURRENT_TIMESTAMP)
            ON CONFLICT (pool_address) DO UPDATE SET
                liquidity = EXCLUDED.liquidity,
                sqrt_price_x96 = EXCLUDED.sqrt_price_x96,
                tick = EXCLUDED.tick,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&pool.pool_address)
        .bind(&pool.token0_address)
        .bind(&pool.token1_address)
        .bind(&pool.token0_symbol)
        .bind(&pool.token1_symbol)
        .bind(pool.token0_decimals)
        .bind(pool.token1_decimals)
        .bind(pool.fee_tier)
        .bind(pool.tick_spacing)
        .bind(pool.liquidity)
        .bind(&pool.sqrt_price_x96)
        .bind(pool.tick)
        .bind(pool.chain_id)
        .bind(&pool.dex_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_swap(&self, swap: &SwapEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO swaps (
                tx_hash, pool_address, token_in, token_out, amount_in, amount_out,
                amount_in_usd, amount_out_usd, timestamp, block_number, log_index, chain_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (tx_hash, log_index, chain_id) DO NOTHING
            "#,
        )
        .bind(&swap.tx_hash)
        .bind(&swap.pool_address)
        .bind(&swap.token_in)
        .bind(&swap.token_out)
        .bind(swap.amount_in)
        .bind(swap.amount_out)
        .bind(swap.amount_in_usd)
        .bind(swap.amount_out_usd)
        .bind(swap.timestamp)
        .bind(swap.block_number)
        .bind(swap.log_index)
        .bind(swap.chain_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pool(&self, pool_address: &str) -> Result<Option<PoolData>> {
        let row = sqlx::query(
            "SELECT * FROM pools WHERE pool_address = $1"
        )
        .bind(pool_address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(PoolData {
                pool_address: row.get("pool_address"),
                token0_address: row.get("token0_address"),
                token1_address: row.get("token1_address"),
                token0_symbol: row.get("token0_symbol"),
                token1_symbol: row.get("token1_symbol"),
                token0_decimals: row.get("token0_decimals"),
                token1_decimals: row.get("token1_decimals"),
                fee_tier: row.get("fee_tier"),
                tick_spacing: row.get("tick_spacing"),
                liquidity: row.get("liquidity"),
                sqrt_price_x96: row.get("sqrt_price_x96"),
                tick: row.get("tick"),
                chain_id: row.get("chain_id"),
                dex_name: row.get("dex_name"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_pools_by_tokens(&self, token0: &str, token1: &str) -> Result<Vec<PoolData>> {
        let rows = sqlx::query(
            "SELECT * FROM pools WHERE (token0_address = $1 AND token1_address = $2) OR (token0_address = $2 AND token1_address = $1)"
        )
        .bind(token0)
        .bind(token1)
        .fetch_all(&self.pool)
        .await?;

        let pools = rows.into_iter().map(|row| PoolData {
            pool_address: row.get("pool_address"),
            token0_address: row.get("token0_address"),
            token1_address: row.get("token1_address"),
            token0_symbol: row.get("token0_symbol"),
            token1_symbol: row.get("token1_symbol"),
            token0_decimals: row.get("token0_decimals"),
            token1_decimals: row.get("token1_decimals"),
            fee_tier: row.get("fee_tier"),
            tick_spacing: row.get("tick_spacing"),
            liquidity: row.get("liquidity"),
            sqrt_price_x96: row.get("sqrt_price_x96"),
            tick: row.get("tick"),
            chain_id: row.get("chain_id"),
            dex_name: row.get("dex_name"),
        }).collect();

        Ok(pools)
    }

    pub async fn get_all_pool_addresses(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT pool_address FROM pools")
            .fetch_all(&self.pool)
            .await?;

        let addresses = rows.into_iter()
            .map(|row| row.get("pool_address"))
            .collect();

        Ok(addresses)
    }

    pub async fn get_stats(&self) -> Result<(u64, u64)> {
        let pool_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pools")
            .fetch_one(&self.pool)
            .await?;

        let swap_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM swaps")
            .fetch_one(&self.pool)
            .await?;

        Ok((pool_count as u64, swap_count as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PoolData, SwapEvent};

    #[tokio::test]
    async fn test_database_operations() {
        // This would require a test database setup
        // For now, just test that the struct can be created
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
            chain_id: 1,
            dex_name: "moonshot".to_string(),
        };

        assert_eq!(pool.pool_address, "0x1234567890123456789012345678901234567890");
        assert_eq!(pool.dex_name, "moonshot");
    }
}
