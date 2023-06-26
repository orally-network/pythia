use std::collections::HashMap;

use anyhow::{Context, Result};
use url::Url;

use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};

use super::errors::PythiaError;
use crate::STATE;

#[derive(Clone, Debug, Deserialize, Serialize, CandidType, Default)]
pub struct Chain {
    pub chain_id: Nat,
    pub rpc: String,
    pub min_balance: Nat,
    pub block_gas_limit: Nat,
    pub fee: Option<Nat>,
    pub symbol: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, CandidType)]
pub struct CreateChainRequest {
    pub chain_id: Nat,
    pub rpc: String,
    pub min_balance: Nat,
    pub block_gas_limit: Nat,
    pub fee: Nat,
    pub symbol: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, CandidType)]
pub struct ChainUpdator {
    pub rpc: Option<String>,
    pub min_balance: Option<Nat>,
    pub block_gas_limit: Option<Nat>,
    pub fee: Option<Nat>,
    pub symbol: Option<String>,
}

/// Chain id => Chain
#[derive(Clone, Debug, Default, Deserialize, Serialize, CandidType)]
pub struct Chains(pub HashMap<Nat, Chain>);

impl Chains {
    pub fn add(req: &CreateChainRequest) -> Result<()> {
        let rpc: Url = req.rpc.parse().context(PythiaError::InvalidChainRPC)?;

        STATE.with(|state| {
            state.borrow_mut().chains.0.insert(
                req.chain_id.clone(),
                Chain {
                    chain_id: req.chain_id.clone(),
                    rpc: rpc.to_string(),
                    min_balance: req.min_balance.clone(),
                    block_gas_limit: req.block_gas_limit.clone(),
                    fee: Some(req.fee.clone()),
                    symbol: Some(req.symbol.clone()),
                },
            );
        });
        Ok(())
    }

    pub fn remove(id: &Nat) -> Result<()> {
        STATE.with(|state| {
            state
                .borrow_mut()
                .chains
                .0
                .remove(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(())
        })
    }

    pub fn update(id: &Nat, updator: ChainUpdator) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let chain = state
                .chains
                .0
                .get_mut(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            if let Some(rpc) = updator.rpc {
                let rpc: Url = rpc.parse().context(PythiaError::InvalidChainRPC)?;
                chain.rpc = rpc.to_string();
            }

            if let Some(min_balance) = updator.min_balance {
                chain.min_balance = min_balance;
            }

            if let Some(block_gas_limit) = updator.block_gas_limit {
                chain.block_gas_limit = block_gas_limit;
            }

            if let Some(fee) = updator.fee {
                chain.fee = Some(fee);
            }

            if let Some(symbol) = updator.symbol {
                chain.symbol = Some(symbol);
            }

            Ok(())
        })
    }

    pub fn get(id: &Nat) -> Result<Chain> {
        STATE.with(|state| {
            state
                .borrow()
                .chains
                .0
                .get(id)
                .cloned()
                .context(PythiaError::ChainDoesNotExist)
        })
    }

    pub fn get_rpc(id: &Nat) -> Result<String> {
        STATE.with(|state| {
            let state = state.borrow();
            let chain = state
                .chains
                .0
                .get(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(chain.rpc.to_string())
        })
    }

    pub fn get_min_balance(id: &Nat) -> Result<Nat> {
        STATE.with(|state| {
            let state = state.borrow();
            let chain = state
                .chains
                .0
                .get(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(chain.min_balance.clone())
        })
    }

    pub fn get_block_gas_limit(id: &Nat) -> Result<Nat> {
        STATE.with(|state| {
            let state = state.borrow();
            let chain = state
                .chains
                .0
                .get(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(chain.block_gas_limit.clone())
        })
    }

    pub fn get_fee(id: &Nat) -> Result<Nat> {
        STATE.with(|state| {
            let state = state.borrow();
            let chain = state
                .chains
                .0
                .get(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(chain.fee.clone().unwrap())
        })
    }

    pub fn get_symbol(id: &Nat) -> Result<String> {
        STATE.with(|state| {
            let state = state.borrow();
            let chain = state
                .chains
                .0
                .get(id)
                .ok_or(PythiaError::ChainDoesNotExist)?;

            Ok(chain.symbol.clone().unwrap())
        })
    }

    pub fn is_exists(id: &Nat) -> bool {
        STATE.with(|state| state.borrow().chains.0.contains_key(id))
    }

    pub fn get_all() -> Vec<Chain> {
        STATE.with(|state| state.borrow().chains.0.values().cloned().collect())
    }
}
