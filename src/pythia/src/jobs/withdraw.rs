use std::str::FromStr;

use anyhow::{Context, Result};
use candid::Nat;
use ic_web3::types::H160;

use crate::{
    clone_with_state, log,
    types::{withdraw::{WithdrawRequest, WithdrawRequests}, errors::PythiaError},
    utils::{multicall, multicall::Transfer, nat, web3},
};

const MAX_TRANSFERS: usize = 100;

pub fn execute() {
    ic_cdk::spawn(withdraw())
}

pub async fn withdraw() {
    for (chain_id, reqs) in clone_with_state!(withdraw_requests).0 {
        if let Err(err) = send_funds(&chain_id, &reqs).await {
            log!("failed to send funds: {err:?}");
            continue;
        }

        WithdrawRequests::erase(&chain_id)
        .expect("should erase withdraw requests");
    }

    log!("withdraw job executed");
}

async fn send_funds(chain_id: &Nat, reqs: &[WithdrawRequest]) -> Result<()> {
    if reqs.is_empty() {
        return Ok(());
    }

    let mut transfers: Vec<Transfer> = reqs
        .iter()
        .map(|req| Transfer {
            target: H160::from_str(&req.receiver)
                .expect("should be valid address"),
            value: nat::to_u256(&req.amount),
        })
        .collect();

    while !transfers.is_empty() {
        multicall::multitranfer(
            &web3::instance(chain_id)?,
            chain_id,
            transfers.split_off((transfers.len() - 1) % MAX_TRANSFERS),
        )
        .await
        .context(PythiaError::UnableToTransferFunds)?;
    }

    Ok(())
}
