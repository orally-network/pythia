use anyhow::Result;

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::{query, update};

use crate::{utils::validate_caller, Chain, PythiaError, CHAINS, U256};

#[update]
pub fn add_chain(chain_id: Nat, rpc: String, min_balance: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    let chain = Chain::new(&chain_id, &rpc, &min_balance).map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        if chains.contains_key(&chain.chain_id) {
            return Err(format!("{}", PythiaError::ChainAlreadyExists));
        };

        chains.insert(chain.chain_id, chain);

        Ok(())
    })
}

#[update]
pub fn remove_chain(chain_id: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        chains
            .remove(&U256::from(chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        Ok(())
    })
}

#[update]
pub fn update_chain_rpc(chain_id: Nat, rpc: String) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains
            .get_mut(&U256::from(chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        chain.rpc = rpc.parse().map_err(|e| format!("{}", e))?;

        Ok(())
    })
}

#[update]
pub fn update_chain_min_balance(chain_id: Nat, min_balance: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains
            .get_mut(&U256::from(chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        chain.min_balance = U256::from(min_balance);

        Ok(())
    })
}

#[update]
pub fn update_chain_native_price(chain_id: Nat, native_price: Nat) -> Result<(), String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains
            .get_mut(&U256::from(chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        chain.native_price = *native_price
            .0
            .to_u64_digits()
            .first()
            .expect("should have at least one digit");

        Ok(())
    })
}

#[query]
pub fn get_chain_rpc(chain_id: Nat) -> Result<String, String> {
    validate_caller().map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let chains = chains.borrow();
        let chain = chains
            .get(&U256::from(chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        Ok(chain.rpc.to_string())
    })
}
