use std::collections::HashMap;

use anyhow::{Context, Result};

use candid::Nat;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

use crate::{log, STATE};

use super::{errors::PythiaError, logger::WITHDRAWER};

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub amount: Nat,
    pub receiver: String,
}

/// chain id => withdraw requests
#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct WithdrawRequests(pub HashMap<Nat, Vec<WithdrawRequest>>);

impl WithdrawRequests {
    pub fn add(chain_id: &Nat, receiver: &str, amount: &Nat) -> Result<()> {
        STATE.with(|state| {
            state
                .borrow_mut()
                .withdraw_requests
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .push(WithdrawRequest {
                    amount: amount.clone(),
                    receiver: receiver.to_string(),
                });

            log!(
                "[{WITHDRAWER}] Withdraw request added: chain_id = {}, amount = {}, receiver = {}",
                chain_id,
                amount,
                receiver
            );

            Ok(())
        })
    }

    pub fn erase(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            state
                .borrow_mut()
                .withdraw_requests
                .0
                .get_mut(chain_id)
                .context(PythiaError::ChainDoesNotExist)?
                .clear();

            log!(
                "[{WITHDRAWER}] Withdraw request removed: chain_id = {}",
                chain_id,
            );
            Ok(())
        })
    }

    pub fn init_new_chain(chain_id: &Nat) -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.withdraw_requests.0.contains_key(chain_id) {
                return Err(PythiaError::ChainAlreadyExists.into());
            }
            state.withdraw_requests.0.insert(chain_id.clone(), vec![]);
            log!("[{WITHDRAWER}] New chain added: {chain_id}");
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
            log!("[{WITHDRAWER}] Chain removed: {chain_id}");
            Ok(())
        })
    }
}
