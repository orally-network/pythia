use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum AssetData {
    DefaultPriceFeed {
        symbol: String,
        rate: u64,
        decimals: u64,
        timestamp: u64,
    },
    CustomPriceFeed {
        symbol: String,
        rate: u64,
        decimals: Option<u64>,
        timestamp: u64,
    },
    CustomNumber {
        id: String,
        value: u64,
        decimals: u64,
    },
    CustomString {
        id: String,
        value: String,
    },
}

impl Default for AssetData {
    fn default() -> Self {
        AssetData::DefaultPriceFeed {
            symbol: "".to_string(),
            rate: 0,
            decimals: 0,
            timestamp: 0,
        }
    }
}

#[derive(Clone, Default, Debug, CandidType, Serialize, Deserialize)]
pub struct AssetDataResult {
    pub data: AssetData,
    pub signature: Option<String>,
}
