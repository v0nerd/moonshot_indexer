use ethers::abi::{Abi, Token};
use ethers::types::{Address, U256};

// Moonshot Factory ABI - PoolCreated event
pub const MOONSHOT_FACTORY_ABI: &str = r#"[
    {
        "anonymous": false,
        "inputs": [
            {
                "indexed": true,
                "internalType": "address",
                "name": "token0",
                "type": "address"
            },
            {
                "indexed": true,
                "internalType": "address",
                "name": "token1",
                "type": "address"
            },
            {
                "indexed": false,
                "internalType": "uint24",
                "name": "fee",
                "type": "uint24"
            },
            {
                "indexed": false,
                "internalType": "int24",
                "name": "tickSpacing",
                "type": "int24"
            },
            {
                "indexed": true,
                "internalType": "address",
                "name": "pool",
                "type": "address"
            }
        ],
        "name": "PoolCreated",
        "type": "event"
    }
]"#;

// Moonshot Pool ABI - Swap event
pub const MOONSHOT_POOL_ABI: &str = r#"[
    {
        "anonymous": false,
        "inputs": [
            {
                "indexed": true,
                "internalType": "address",
                "name": "sender",
                "type": "address"
            },
            {
                "indexed": true,
                "internalType": "address",
                "name": "recipient",
                "type": "address"
            },
            {
                "indexed": false,
                "internalType": "int256",
                "name": "amount0",
                "type": "int256"
            },
            {
                "indexed": false,
                "internalType": "int256",
                "name": "amount1",
                "type": "int256"
            },
            {
                "indexed": false,
                "internalType": "uint160",
                "name": "sqrtPriceX96",
                "type": "uint160"
            },
            {
                "indexed": false,
                "internalType": "uint128",
                "name": "liquidity",
                "type": "uint128"
            },
            {
                "indexed": false,
                "internalType": "int24",
                "name": "tick",
                "type": "int24"
            }
        ],
        "name": "Swap",
        "type": "event"
    },
    {
        "inputs": [],
        "name": "token0",
        "outputs": [
            {
                "internalType": "address",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "token1",
        "outputs": [
            {
                "internalType": "address",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "fee",
        "outputs": [
            {
                "internalType": "uint24",
                "name": "",
                "type": "uint24"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "tickSpacing",
        "outputs": [
            {
                "internalType": "int24",
                "name": "",
                "type": "int24"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "slot0",
        "outputs": [
            {
                "internalType": "uint160",
                "name": "sqrtPriceX96",
                "type": "uint160"
            },
            {
                "internalType": "int24",
                "name": "tick",
                "type": "int24"
            },
            {
                "internalType": "uint16",
                "name": "observationIndex",
                "type": "uint16"
            },
            {
                "internalType": "uint16",
                "name": "observationCardinality",
                "type": "uint16"
            },
            {
                "internalType": "uint16",
                "name": "observationCardinalityNext",
                "type": "uint16"
            },
            {
                "internalType": "uint8",
                "name": "feeProtocol",
                "type": "uint8"
            },
            {
                "internalType": "bool",
                "name": "unlocked",
                "type": "bool"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "liquidity",
        "outputs": [
            {
                "internalType": "uint128",
                "name": "",
                "type": "uint128"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    }
]"#;

// ERC20 Token ABI for getting token metadata
pub const ERC20_ABI: &str = r#"[
    {
        "inputs": [],
        "name": "name",
        "outputs": [
            {
                "internalType": "string",
                "name": "",
                "type": "string"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "symbol",
        "outputs": [
            {
                "internalType": "string",
                "name": "",
                "type": "string"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "decimals",
        "outputs": [
            {
                "internalType": "uint8",
                "name": "",
                "type": "uint8"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "totalSupply",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    }
]"#;

pub fn get_factory_abi() -> Abi {
    serde_json::from_str(MOONSHOT_FACTORY_ABI).expect("Invalid factory ABI")
}

pub fn get_pool_abi() -> Abi {
    serde_json::from_str(MOONSHOT_POOL_ABI).expect("Invalid pool ABI")
}

pub fn get_erc20_abi() -> Abi {
    serde_json::from_str(ERC20_ABI).expect("Invalid ERC20 ABI")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_parsing() {
        // Test that ABIs can be parsed without errors
        let factory_abi = get_factory_abi();
        let pool_abi = get_pool_abi();
        let erc20_abi = get_erc20_abi();

        // Check that we have the expected events/functions
        assert!(factory_abi
            .events()
            .any(|event| event.name == "PoolCreated"));
        assert!(pool_abi.events().any(|event| event.name == "Swap"));
        assert!(erc20_abi
            .functions()
            .any(|function| function.name == "symbol"));
    }
}
