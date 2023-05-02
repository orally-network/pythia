use std::{
    str::FromStr,
    time::Duration
};

use anyhow::{Result, Context};

use ic_web3::types::H160;
use ic_cdk::export::Principal;
use ic_cdk_timers::{TimerId, set_timer_interval};

use crate::U256;

#[derive(Clone, Debug)]
pub struct Sub {
    pub chain_id: U256,
    pub contract_addr: H160,
    pub method_abi: Vec<u8>,
    pub frequency: u64,
    pub principal: Principal,
    pub timer_id: TimerId,
}

impl Sub {
    pub fn new(
        chain_id: &U256,
        contract_addr: &str,
        method_abi: &[u8],
        frequency: &u64,
    ) -> Result<Self> {
        let contract_addr = H160::from_str(contract_addr)
            .context("failed to parse contract address")?;

        let timer_id = set_timer_interval(Duration::from_secs(frequency.clone()), || {});

        Ok(Self {
            chain_id: chain_id.clone(),
            contract_addr,
            method_abi: method_abi.to_vec(),
            frequency: frequency.clone(),
            principal: ic_cdk::caller(),
            timer_id,
        })
    }
}
