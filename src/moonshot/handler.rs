use anyhow::Result;
use ethers::abi::{Abi, Token};
use ethers::contract::Contract;
use ethers::providers::Provider;
use ethers::types::{Address, Log, U256};
use std::sync::Arc;

use crate::types::{PoolData, SwapEvent};
use super::abi::{get_factory_abi, get_pool_abi, get_erc20_abi};

pub struct MoonshotHandler {
    factory_abi: Abi,
    pool_abi: Abi,
    erc20_abi: Abi,
    provider: Arc<Provider<ethers::providers::Ws>>,
}

impl MoonshotHandler {
    pub fn new(provider: Arc<Provider<ethers::providers::Ws>>) -> Self {
        Self {
            factory_abi: get_factory_abi(),
            pool_abi: get_pool_abi(),
            erc20_abi: get_erc20_abi(),
            provider,
        }
    }

    pub async fn handle_pool_created(&self, log: Log, chain_id: i64) -> Result<PoolData> {
        // Parse the PoolCreated event
        let event = self.factory_abi.event("PoolCreated")?;
        let decoded = event.parse_log(log.clone())?;

        let token0: Address = decoded.params[0].value.clone().into_abi().into_address()?;
        let token1: Address = decoded.params[1].value.clone().into_abi().into_address()?;
        let fee: u32 = decoded.params[2].value.clone().into_abi().into_uint()?.as_u32();
        let tick_spacing: i32 = decoded.params[3].value.clone().into_abi().into_int()?.as_i32();
        let pool_address: Address = decoded.params[4].value.clone().into_abi().into_address()?;

        // Get token metadata
        let (token0_symbol, token0_decimals) = self.get_token_metadata(token0).await?;
        let (token1_symbol, token1_decimals) = self.get_token_metadata(token1).await?;

        let pool_data = PoolData {
            pool_address: format!("{:?}", pool_address),
            token0_address: format!("{:?}", token0),
            token1_address: format!("{:?}", token1),
            token0_symbol,
            token1_symbol,
            token0_decimals: Some(token0_decimals as i32),
            token1_decimals: Some(token1_decimals as i32),
            fee_tier: Some(fee as i32),
            tick_spacing: Some(tick_spacing),
            liquidity: Some(0), // Will be updated when liquidity is added
            sqrt_price_x96: None, // Will be updated when first swap occurs
            tick: None, // Will be updated when first swap occurs
            chain_id,
            dex_name: "moonshot".to_string(),
        };

        Ok(pool_data)
    }

    pub async fn handle_swap(&self, log: Log, chain_id: i64) -> Result<SwapEvent> {
        // Parse the Swap event
        let event = self.pool_abi.event("Swap")?;
        let decoded = event.parse_log(log.clone())?;

        let sender: Address = decoded.params[0].value.clone().into_abi().into_address()?;
        let recipient: Address = decoded.params[1].value.clone().into_abi().into_address()?;
        let amount0: i128 = decoded.params[2].value.clone().into_abi().into_int()?.as_i128();
        let amount1: i128 = decoded.params[3].value.clone().into_abi().into_int()?.as_i128();
        let sqrt_price_x96: U256 = decoded.params[4].value.clone().into_abi().into_uint()?;
        let liquidity: u128 = decoded.params[5].value.clone().into_abi().into_uint()?.as_u128();
        let tick: i32 = decoded.params[6].value.clone().into_abi().into_int()?.as_i32();

        // Determine which token is being swapped in/out
        let (token_in, token_out, amount_in, amount_out) = if amount0 > 0 {
            // amount0 is positive, so token0 is being swapped in
            ("token0", "token1", amount0 as i64, -(amount1 as i64))
        } else {
            // amount1 is positive, so token1 is being swapped in
            ("token1", "token0", amount1 as i64, -(amount0 as i64))
        };

        let swap_event = SwapEvent::new(
            format!("{:?}", log.transaction_hash.unwrap()),
            format!("{:?}", log.address),
            token_in.to_string(),
            token_out.to_string(),
            amount_in,
            amount_out,
            log.block_number.unwrap().as_u64() as i64, // Using block number as timestamp for now
            log.block_number.unwrap().as_u64() as i64,
            log.log_index.unwrap().as_u64() as i32,
            chain_id,
        );

        Ok(swap_event)
    }

    async fn get_token_metadata(&self, token_address: Address) -> Result<(Option<String>, u8)> {
        let contract = Contract::new(token_address, self.erc20_abi.clone(), self.provider.clone());

        // Get symbol
        let symbol: String = match contract.method("symbol", ())?.call().await {
            Ok(s) => s,
            Err(_) => return Ok((None, 18)), // Default to 18 decimals if symbol call fails
        };

        // Get decimals
        let decimals: u8 = match contract.method("decimals", ())?.call().await {
            Ok(d) => d,
            Err(_) => 18, // Default to 18 decimals
        };

        Ok((Some(symbol), decimals))
    }

    pub async fn update_pool_state(&self, pool_address: Address, chain_id: i64) -> Result<PoolData> {
        let contract = Contract::new(pool_address, self.pool_abi.clone(), self.provider.clone());

        // Get pool parameters
        let token0: Address = contract.method("token0", ())?.call().await?;
        let token1: Address = contract.method("token1", ())?.call().await?;
        let fee: u32 = contract.method("fee", ())?.call().await?;
        let tick_spacing: i32 = contract.method("tickSpacing", ())?.call().await?;
        let liquidity: u128 = contract.method("liquidity", ())?.call().await?;

        // Get current price and tick
        let slot0: (U256, i32, u16, u16, u16, u8, bool) = contract.method("slot0", ())?.call().await?;
        let sqrt_price_x96 = slot0.0;
        let tick = slot0.1;

        // Get token metadata
        let (token0_symbol, token0_decimals) = self.get_token_metadata(token0).await?;
        let (token1_symbol, token1_decimals) = self.get_token_metadata(token1).await?;

        let pool_data = PoolData {
            pool_address: format!("{:?}", pool_address),
            token0_address: format!("{:?}", token0),
            token1_address: format!("{:?}", token1),
            token0_symbol,
            token1_symbol,
            token0_decimals: Some(token0_decimals as i32),
            token1_decimals: Some(token1_decimals as i32),
            fee_tier: Some(fee as i32),
            tick_spacing: Some(tick_spacing),
            liquidity: Some(liquidity as i64),
            sqrt_price_x96: Some(format!("{:?}", sqrt_price_x96)),
            tick: Some(tick),
            chain_id,
            dex_name: "moonshot".to_string(),
        };

        Ok(pool_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{H160, H256, U256};

    #[test]
    fn test_pool_data_creation() {
        let pool_data = PoolData::new(
            "0x1234567890123456789012345678901234567890".to_string(),
            "0xTokenA".to_string(),
            "0xTokenB".to_string(),
            1,
            "moonshot".to_string(),
        );

        assert_eq!(pool_data.pool_address, "0x1234567890123456789012345678901234567890");
        assert_eq!(pool_data.token0_address, "0xTokenA");
        assert_eq!(pool_data.token1_address, "0xTokenB");
        assert_eq!(pool_data.chain_id, 1);
        assert_eq!(pool_data.dex_name, "moonshot");
    }

    #[test]
    fn test_swap_event_creation() {
        let swap_event = SwapEvent::new(
            "0x1234567890abcdef".to_string(),
            "0xPoolAddress".to_string(),
            "token0".to_string(),
            "token1".to_string(),
            1000,
            950,
            1640995200,
            12345,
            0,
            1,
        );

        assert_eq!(swap_event.tx_hash, "0x1234567890abcdef");
        assert_eq!(swap_event.pool_address, "0xPoolAddress");
        assert_eq!(swap_event.token_in, "token0");
        assert_eq!(swap_event.token_out, "token1");
        assert_eq!(swap_event.amount_in, 1000);
        assert_eq!(swap_event.amount_out, 950);
        assert_eq!(swap_event.chain_id, 1);
    }
}
