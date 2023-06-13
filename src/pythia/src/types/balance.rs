use candid::Nat;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use anyhow::{Result, anyhow};

use crate::utils::multicall::GAS_PER_TRANSFER;

const ETH_TRANSFER_GAS_LIMIT: u64 = 21_000+GAS_PER_TRANSFER;

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct UserBalance {
    pub amount: Nat,
    pub nonces: Vec<Nat>,
}

impl UserBalance {
    pub fn get_value(&mut self, gas_price: &Nat) -> Result<Nat> {
        let gas = Nat::from(ETH_TRANSFER_GAS_LIMIT) * gas_price.clone();
        if self.amount < gas {
            return Err(anyhow!("not enough funds to pay for gas"));
        }

        let value = self.amount.clone() - gas;
        self.amount = Nat::from(0);

        Ok(value)
    }
}