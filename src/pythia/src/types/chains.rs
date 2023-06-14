use anyhow::{Context, Result};
use url::Url;

use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};
use ic_web3::types::H160;

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct Chain {
    pub chain_id: Nat,
    pub rpc: String,
    pub min_balance: Nat,
    pub treasurer: String,
    pub block_gas_limit: Nat,
}

impl Chain {
    pub fn new(
        chain_id: &Nat,
        rpc: &str,
        min_balance: &Nat,
        treasurer: &H160,
        block_gas_limit: &Nat,
    ) -> Result<Self> {
        let rpc: Url = rpc.parse().context("Failed to parse RPC URL")?;

        Ok(Self {
            chain_id: chain_id.clone(),
            rpc: rpc.to_string(),
            min_balance: min_balance.clone(),
            treasurer: hex::encode(treasurer.as_bytes()),
            block_gas_limit: block_gas_limit.clone(),
        })
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct CandidTypeChain {
    pub chain_id: Nat,
    pub rpc: String,
    pub min_balance: Nat,
    pub treasurer: String,
}
