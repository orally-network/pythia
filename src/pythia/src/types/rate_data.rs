use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct RateDataLight {
    pub symbol: String,
    pub rate: u64,
    pub timestamp: u64,
    pub decimals: u32,
}
