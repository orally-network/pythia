use anyhow::{Context, Result};
use url::Url;

use ic_cdk::export::candid::Nat;
use ic_web3::types::H160;

use crate::types::U256;

#[derive(Clone, Debug)]
pub struct Chain {
    pub chain_id: U256,
    pub rpc: Url,
    pub min_balance: U256,
    pub treasurer: H160,
}

impl Chain {
    pub fn new(chain_id: &Nat, rpc: &str, min_balance: &Nat, treasurer: &H160) -> Result<Self> {
        let rpc = rpc.parse().context("Failed to parse RPC URL")?;

        let chain_id = U256::from(chain_id.clone());

        let min_balance = U256::from(min_balance.clone());

        Ok(Self {
            chain_id,
            rpc,
            min_balance,
            treasurer: *treasurer,
        })
    }
}
