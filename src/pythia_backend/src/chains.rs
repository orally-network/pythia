use anyhow::Result;

use ic_cdk_macros::{update, query};
use ic_cdk::export::candid::Nat;

use crate::{
    CHAINS,
    PythiaError,
    U256,
    Chain,
    utils::validate_caller,
};

#[update]
pub fn add_chain(chain_id: Nat, rpc: String) -> Result<(), String> {
    validate_caller()
        .map_err(|e| format!("{}", e))?;

    let chain = Chain::new(&chain_id, &rpc)
        .map_err(|e| format!("{}", e))?;
    
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
    validate_caller()
        .map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        chains.remove(&U256::from(&chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        Ok(())
    })
}

#[update]
pub fn update_rpc(chain_id: Nat, rpc: String) -> Result<(), String> {
    validate_caller()
        .map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let mut chains = chains.borrow_mut();
        let chain = chains.get_mut(&U256::from(&chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        chain.rpc = rpc.parse()
            .map_err(|e| format!("{}", e))?;

        Ok(())
    })
}

#[query]
pub fn get_chain_rpc(chain_id: Nat) -> Result<String, String> {
    validate_caller()
        .map_err(|e| format!("{}", e))?;

    CHAINS.with(|chains| {
        let chains = chains.borrow();
        let chain = chains.get(&U256::from(&chain_id))
            .ok_or_else(|| format!("{}", PythiaError::ChainDoesNotExist))?;

        Ok(chain.rpc.to_string())
    })
}