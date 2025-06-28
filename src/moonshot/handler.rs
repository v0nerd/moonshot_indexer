use anyhow::Result;
use ethers::abi::Abi;
use ethers::contract::Contract;
use ethers::providers::Provider;
use ethers::types::{Address, Log, U256};
use std::sync::Arc;

use super::abi::{get_erc20_abi, get_factory_abi, get_pool_abi};
use crate::types::{PoolData, SwapEvent};

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
        let event = self.factory_abi.event("PoolCreated")?;
        let raw_log = log.clone().into();
        let decoded = event.parse_log(raw_log)?;

        let token0: Address = decoded.params[0].value.clone().into().into_address()?;
        let token1: Address = decoded.params[1].value.clone().into().into_address()?;
        let fee: u32 = decoded.params[2].value.clone().into().into_uint()?.as_u32();
        let tick_spacing: i32 = decoded.params[3].value.clone().into().into_int()?.as_i32();
        let pool_address: Address = decoded.params[4].value.clone().into().into_address()?;

        let (token0_symbol, token0_decimals) = self.get_token_metadata(token0).await?;
        let (token1_symbol, token1_decimals) = self.get_token_metadata(token1).await?;

        Ok(PoolData {
            pool_address: format!("{:?}", pool_address),
            token0_address: format!("{:?}", token0),
            token1_address: format!("{:?}", token1),
            token0_symbol,
            token1_symbol,
            token0_decimals: Some(token0_decimals as i32),
            token1_decimals: Some(token1_decimals as i32),
            fee_tier: Some(fee as i32),
            tick_spacing: Some(tick_spacing),
            liquidity: Some(0),
            sqrt_price_x96: None,
            tick: None,
            chain_id,
            dex_name: "moonshot".to_string(),
        })
    }

    pub async fn handle_swap(&self, log: Log, chain_id: i64) -> Result<SwapEvent> {
        let event = self.pool_abi.event("Swap")?;
        let raw_log = log.clone().into();
        let decoded = event.parse_log(raw_log)?;

        let amount0: i128 = decoded.params[2].value.clone().into().into_int()?.as_i128();
        let amount1: i128 = decoded.params[3].value.clone().into().into_int()?.as_i128();
        let sqrt_price_x96: U256 = decoded.params[4].value.clone().into().into_uint()?;
        let liquidity: u128 = decoded.params[5]
            .value
            .clone()
            .into()
            .into_uint()?
            .as_u128();
        let tick: i32 = decoded.params[6].value.clone().into().into_int()?.as_i32();

        let (token_in, token_out, amount_in, amount_out) = if amount0 > 0 {
            ("token0", "token1", amount0 as i64, -amount1 as i64)
        } else {
            ("token1", "token0", amount1 as i64, -amount0 as i64)
        };

        Ok(SwapEvent::new(
            format!("{:?}", log.transaction_hash.unwrap_or_default()),
            format!("{:?}", log.address),
            token_in.to_string(),
            token_out.to_string(),
            amount_in,
            amount_out,
            log.block_number.unwrap_or_default().as_u64() as i64,
            log.block_number.unwrap_or_default().as_u64() as i64,
            log.log_index.unwrap_or_default().as_u64() as i32,
            chain_id,
        ))
    }

    async fn get_token_metadata(&self, token_address: Address) -> Result<(Option<String>, u8)> {
        let contract = Contract::new(token_address, self.erc20_abi.clone(), self.provider.clone());

        let symbol: String = match contract.method::<(), String>("symbol", ())?.call().await {
            Ok(s) => s,
            Err(_) => return Ok((None, 18)),
        };

        let decimals: u8 = match contract.method::<(), u8>("decimals", ())?.call().await {
            Ok(d) => d,
            Err(_) => 18,
        };

        Ok((Some(symbol), decimals))
    }

    pub async fn update_pool_state(
        &self,
        pool_address: Address,
        chain_id: i64,
    ) -> Result<PoolData> {
        let contract = Contract::new(pool_address, self.pool_abi.clone(), self.provider.clone());

        let token0: Address = contract.method("token0", ())?.call().await?;
        let token1: Address = contract.method("token1", ())?.call().await?;
        let fee: u32 = contract.method("fee", ())?.call().await?;
        let tick_spacing: i32 = contract.method("tickSpacing", ())?.call().await?;
        let liquidity: u128 = contract.method("liquidity", ())?.call().await?;
        let slot0: (U256, i32, u16, u16, u16, u8, bool) =
            contract.method("slot0", ())?.call().await?;
        let sqrt_price_x96 = slot0.0;
        let tick = slot0.1;

        let (token0_symbol, token0_decimals) = self.get_token_metadata(token0).await?;
        let (token1_symbol, token1_decimals) = self.get_token_metadata(token1).await?;

        Ok(PoolData {
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
        })
    }
}
