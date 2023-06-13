use std::str::FromStr;

use anyhow::{Context, Result, anyhow, Error};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::update;
use ic_dl_utils::retry_until_success;
use ic_utils::logger::log_message;
use ic_web3::{
    transports::ICHttp,
    types::{H160, H256, TransactionId, Transaction},
    Web3,
};

use crate::{utils::{rec_eth_addr, u256_to_nat, get_gas_price, get_pma}, STATE, types::{balance::UserBalance, withdraw::WithdrawRequest}, clone_with_state, jobs::{publisher, withdraw}};

#[update]
pub async fn withdraw(
    chain_id: Nat,
    msg: String,
    sig: String,
    receiver: String,
) -> Result<(), String> {
    _withdraw(chain_id, msg, sig, receiver)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn _withdraw(chain_id: Nat, msg: String, sig: String, receiver: String) -> Result<()> {
    let pub_key = hex::encode(rec_eth_addr(&msg, &sig).await?.as_bytes());

    let gas_price = get_gas_price(&chain_id).await?;

    STATE.with(|state|{
        let mut state = state.borrow_mut();

        let amount = state
            .balances
            .get_mut(&chain_id)
            .context("chain does not exist")?
            .get_mut(&pub_key)
            .context("user does not exist")?
            .get_value(&gas_price)?;

        state
            .subscriptions
            .get_mut(&chain_id)
            .context("chain does not exist")?
            .iter_mut()
            .for_each(|sub| {
                if sub.owner == pub_key {
                    sub.status.is_active = false;
                }
            });
        
        state
            .withdraw_requests
            .get_mut(&chain_id)
            .context("chain does not exist")?
            .push(WithdrawRequest {
                receiver,
                amount,
            });
        
        Ok::<(), Error>(())
    })?;

    if !clone_with_state!(is_timer_active) {
        withdraw::execute();
    }

    Ok(())
}

#[update]
pub async fn deposit(tx_hash: String, chain_id: Nat, msg: String, sig: String) -> Result<(), String> {
    _deposit(tx_hash, chain_id, msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn _deposit(tx_hash: String, chain_id: Nat, msg: String, sig: String) -> Result<()> {
    let pub_key = rec_eth_addr(&msg, &sig)
        .await?;
    let tx = get_tx(&tx_hash, &chain_id)
        .await?;

    if tx.to.context("to should be in tx")? != H160::from_str(&get_pma().await?)? {
        return Err(anyhow!("tx is not sent to the PMA"));
    }

    if is_used_nonce(&chain_id, &pub_key, &u256_to_nat(tx.nonce))? {
        return Err(anyhow!("nonce has already been used for previous deposit"));
    }

    let deposit_amount = get_deposit_amount_without_fee(&u256_to_nat(tx.value));
    if deposit_amount == 0 {
        return Ok(())
    }

    add_deposit_amount_to_balance(&deposit_amount, &chain_id, &pub_key);

    if !clone_with_state!(is_timer_active) {
        publisher::execute();
    }

    let owner = hex::encode(pub_key.as_bytes());
    ic_cdk::println!("[{owner}] deposited amount {deposit_amount}");
    log_message(format!("[{owner}] deposited {deposit_amount}"));

    Ok(())
}

async fn get_tx(tx_hash: &str, chain_id: &Nat) -> Result<Transaction> {
    let chain = STATE.with(|state| {
        state
            .borrow()
            .chains
            .get(&chain_id)
            .cloned()
            .context("chain does not exist")
    })?;
    let tx_hash = H256::from_str(&tx_hash)?;

    let w3 = Web3::new(ICHttp::new(&chain.rpc, None)?);
    let tx_receipt = retry_until_success!(
        w3
            .eth()
            .transaction_receipt(tx_hash)
    )?.context("tx does not exist")?;

    if let Some(status) = tx_receipt.status {
        if status.as_u64() != 1 {
            return Err(anyhow!("tx has failed"));
        }
    } else {
        return Err(anyhow!("tx is not executed yet"));
    }

    let tx = retry_until_success!(
        w3
            .eth()
            .transaction(TransactionId::from(tx_hash))
    )?.context("tx does not exist");

    tx
}

fn is_used_nonce(chain_id: &Nat, pub_key: &H160, nonce: &Nat) -> Result<bool> {
    STATE.with(|state| {
        let is_used = state
            .borrow_mut()
            .balances
            .get_mut(chain_id)
            .context("chain does not exist")?
            .entry(hex::encode(pub_key.as_bytes()))
            .or_insert(UserBalance::default())
            .nonces
            .contains(nonce);

        Ok(is_used)
    })
}

fn get_deposit_amount_without_fee(value: &Nat) -> Nat {
    value.clone() - clone_with_state!(tx_fee)
}

fn add_deposit_amount_to_balance(deposit_amount: &Nat, chain_id: &Nat, pub_key: &H160) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .balances
            .get_mut(chain_id)
            .expect("balances should have the provided chain id")
            .get_mut(&hex::encode(pub_key.as_bytes()))
            .expect("public key should exists in balances")
            .amount += deposit_amount.clone()
    })
}
