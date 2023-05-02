use anyhow::{Result, Context};
use url::Url;

use ic_cdk::export::candid::Nat;

use crate::types::U256;

#[derive(Clone, Debug)]
pub struct Chain {
    pub chain_id: U256,
    pub rpc: Url,
}

impl Chain {
    pub fn new(chain_id: &Nat, rpc: &str) -> Result<Self> {
        let rpc = rpc.parse()
            .context("Failed to parse RPC URL")?;

        let chain_id = U256::from(chain_id);

        Ok(Self { chain_id, rpc })
    }
}