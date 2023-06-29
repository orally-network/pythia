use std::str::FromStr;

use anyhow::{Context, Result};
use candid::Nat;
use ic_web3_rs::types::H160;

use crate::{
    clone_with_state, log,
    types::{
        errors::PythiaError,
        logger::WITHDRAWER,
        withdraw::{WithdrawRequest, WithdrawRequests},
    },
    utils::{multicall, multicall::Transfer, nat, web3},
};

const MAX_TRANSFERS: usize = 100;

pub fn execute() {
    ic_cdk::spawn(withdraw())
}

pub async fn withdraw() {
    log!("[{WITHDRAWER}] withdraw job started");
    for (chain_id, reqs) in clone_with_state!(withdraw_requests).0 {
        if let Err(err) = send_funds(&chain_id, &reqs).await {
            log!("failed to send funds: {err:?}");
            continue;
        }

        WithdrawRequests::erase(&chain_id).expect("should erase withdraw requests");
    }

    log!("[{WITHDRAWER}] withdraw job executed");
}

async fn send_funds(chain_id: &Nat, reqs: &[WithdrawRequest]) -> Result<()> {
    if reqs.is_empty() {
        return Ok(());
    }

    let transfers: Vec<Transfer> = reqs
        .iter()
        .map(|req| Transfer {
            target: H160::from_str(&req.receiver).expect("should be valid address"),
            value: nat::to_u256(&req.amount),
        })
        .collect();
    
    for transfers_chunk in transfers.chunks(MAX_TRANSFERS) {
        multicall::multitransfer(
            &web3::instance(chain_id)?,
            chain_id,
            transfers_chunk.to_vec(),
        )
        .await
        .context(PythiaError::UnableToTransferFunds)?;
    }

    Ok(())
}
