use anyhow::{Context, Result};
use url::Url;

use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};
use ic_web3::types::H160;

use crate::types::U256;

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct Chain {
    pub chain_id: U256,
    pub rpc: String,
    pub min_balance: U256,
    pub treasurer: String,
}

impl Chain {
    pub fn new(chain_id: &Nat, rpc: &str, min_balance: &Nat, treasurer: &H160) -> Result<Self> {
        let rpc: Url = rpc.parse().context("Failed to parse RPC URL")?;
        let chain_id = U256::from(chain_id.clone());
        let min_balance = U256::from(min_balance.clone());

        Ok(Self {
            chain_id,
            rpc: rpc.to_string(),
            min_balance,
            treasurer: hex::encode(treasurer.as_bytes()),
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
