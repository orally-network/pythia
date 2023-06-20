use std::collections::HashMap;

use candid::Nat;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use anyhow::{anyhow, Result, Context};

use crate::{utils::{multicall::GAS_PER_TRANSFER, address}, STATE};

use super::{errors::PythiaError, chains::Chains};

const ETH_TRANSFER_GAS_LIMIT: u64 = 21_000 + GAS_PER_TRANSFER;

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct UserBalance {
    pub amount: Nat,
    pub nonces: Vec<Nat>,
}

/// chain id => user's public key => PUB (Pythia User Balance)
#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct Balances(pub HashMap<Nat, HashMap<String, UserBalance>>);

impl Balances {
    pub fn get_value_for_witndraw(chain_id: &Nat, address: &str, gas_price: &Nat) -> Result<Nat> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let mut balance = state
                .balances
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .get_mut(&address::normalize(address)?)
                .context(PythiaError::BalanceDoesNotExist)?;
            
            let gas = Nat::from(ETH_TRANSFER_GAS_LIMIT) * gas_price.clone();
            if balance.amount < gas {
                return Err(anyhow!("not enough funds to pay for gas"));
            }
            let value = balance.amount.clone() - gas;
            balance.amount = Nat::from(0);
            Ok(value)
        })
    }
    
    pub fn create(chain_id: &Nat, address: &str) -> Result<()> {
        let address = address::normalize(address)?;
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let balances = state
                .balances
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?;
            if balances.contains_key(&address) {
                return Err(PythiaError::BalanceAlreadyExists.into());
            }
            balances.insert(address, UserBalance::default());
            Ok(())
        })
    }
    
    pub fn is_exists(chain_id: &Nat, address: &str) -> Result<bool> {
        STATE.with(|state| {
            Ok(state
                .borrow()
                .balances
                .0
                .get(chain_id)
                .context(PythiaError::UnableToAddNewBalance)?
                .contains_key(address))
        })
    }
    
    pub fn save_nonce(chain_id: &Nat, address: &str, nonce: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let balance = state
                .balances
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .get_mut(address)
                .context(PythiaError::BalanceDoesNotExist)?;
            if balance.nonces.contains(nonce) {
                return Err(PythiaError::NonceAlreadyExists.into());
            }
            balance.nonces.push(nonce.clone());
            Ok(())
        })
    }
    
    pub fn add_amount(chain_id: &Nat, address: &str, amount: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let balance = state
                .balances
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .get_mut(address)
                .context(PythiaError::BalanceDoesNotExist)?;
            balance.amount += amount.clone();
            Ok(())
        })
    }

    pub fn get(chain_id: &Nat, address: &str) -> Result<Nat> {
        STATE.with(|state| {
            Ok(state
                .borrow()
                .balances
                .0
                .get(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .get(address)
                .context(PythiaError::BalanceDoesNotExist)?
                .amount
                .clone())
        })
    }

    pub fn init_new_chain(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.balances.0.contains_key(chain_id) {
                return Err(PythiaError::ChainAlreadyExists.into());
            }
            state.balances.0.insert(chain_id.clone(), HashMap::new());
            Ok(())
        })
    }

    pub fn deinit_chain(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state
                .balances
                .0
                .remove(chain_id)
                .context(PythiaError::ChainDoesNotExist)?;
            Ok(())
        })
    }

    pub fn is_sufficient(chain_id: &Nat, address: &str) -> Result<bool> {
        let balance = STATE.with(|state| {
            state
                .borrow()
                .balances
                .0
                .get(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .get(address)
                .context(PythiaError::BalanceDoesNotExist)
                .map(|balance| balance.amount.clone())
        })?;

        Ok(balance >= Chains::get_min_balance(chain_id)?)
    }
}
