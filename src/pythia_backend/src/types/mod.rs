pub mod errors;

use std::str::FromStr;

use anyhow::{Result, Context};
use url::Url;

use ic_cdk::export::candid::Nat;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct U256(primitive_types::U256);

impl From<&Nat> for U256 {
    fn from(nat: &Nat) -> Self {
        U256(primitive_types::U256::from_str(&nat.0.to_string()).unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct Chain {
    pub chain_id: U256,
    pub rpc: Url
}

impl Chain {
    pub fn new(chain_id: &Nat, rpc: &str) -> Result<Self> {
        let rpc = rpc.parse()
            .context("Failed to parse RPC URL")?;

        let chain_id = U256::from(chain_id);

        Ok(Self { chain_id, rpc })
    }
}
