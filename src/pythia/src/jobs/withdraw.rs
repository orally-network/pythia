use std::str::FromStr;

use anyhow::{Result, Context};
use candid::Nat;
use ic_utils::logger::log_message;
use ic_web3::{Web3, transports::ICHttp, types::H160};

use crate::{STATE, types::withdraw::WithdrawRequest, clone_with_state, utils::{{multicall::Transfer, nat_to_u256}, multicall}};

const MAX_TRANSFERS: usize = 100;

pub fn execute() {
    ic_cdk::spawn(withdraw())
}

pub async fn withdraw() {
    for (chain_id, reqs) in clone_with_state!(withdraw_requests) {
        if let Err(err) = send_funds(&chain_id, &reqs).await {
            log_message(format!("failed to send funds: {:?}", err));
            ic_cdk::println!("failed to send funds: {:?}", err);
            continue;
        }

        remove_requests(&chain_id);
    }

    ic_cdk::println!("withdraw job executed");
    log_message("withdraw job executed".into());
}

async fn send_funds(chain_id: &Nat, reqs: &[WithdrawRequest]) -> Result<()> {
    if reqs.len() == 0 {
        return Ok(());
    }

    let w3 = Web3::new(ICHttp::new(&get_rpc(chain_id), None)?);

    let mut transfers: Vec<Transfer> = reqs
        .iter()
        .map(|req| {
            Transfer {
                target: H160::from_str(&req.receiver).expect("should be valid address"),
                value: nat_to_u256(&req.amount),
            }
        })
        .collect();

    while !transfers.is_empty() {
        multicall::multitranfer(&w3, chain_id, transfers.split_off((transfers.len()-1) % MAX_TRANSFERS))
            .await
            .context("failed to transfer funds")?;
    }

    Ok(())
}

fn remove_requests(chain_id: &Nat) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .withdraw_requests
            .remove(chain_id);
    })
}

fn get_rpc(chain_id: &Nat) -> String {
    STATE.with(|state| {
        state
            .borrow()
            .chains
            .get(chain_id)
            .expect("chain should exist")
            .rpc
            .clone()
    })
}