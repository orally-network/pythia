use std::time::Duration;

use anyhow::{anyhow, Context, Result};

use ic_cdk::{api::management_canister::main::raw_rand, api::time};
use ic_cdk_timers::{clear_timer, TimerId};
use ic_utils::{logger::log_message, monitor::collect_metrics};
use ic_web3::{
    contract::{Contract, Options},
    ethabi::{Contract as EthabiContract, Token},
    ic::KeyInfo,
    transports::ICHttp,
    types::{TransactionCondition, H160, H256, U64},
    Web3,
};

use crate::{
    methods::get_exec_addr_from_pub,
    types::subs::MethodType,
    utils::{add_brackets, cast_to_param_type, check_balance, sybil::get_asset_data},
    Chain, PythiaError, Sub, CHAINS, KEY_NAME, SUBS,
};

const TIMEOUT: u64 = 60 * 60;
const MAX_RETRY_ATTEMPTS: u64 = 3;
const BITS_IN_BYTE: usize = 8;
const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

pub fn publish(sub_id: u64, owner: H160) {
    ic_cdk::spawn(_publish(sub_id, owner));
}

async fn _publish(sub_id: u64, owner: H160) {
    let sub = SUBS.with(|subs| {
        subs.borrow()
            .get(sub_id as usize)
            .expect("Sub should exist")
            .clone()
    });

    let chain = CHAINS.with(|chains| {
        chains
            .borrow()
            .get(&sub.chain_id)
            .expect("Chain should exist")
            .clone()
    });

    let exec_addr = get_exec_addr_from_pub(&owner)
        .await
        .expect("exec addr should be in cache");

    if !check_balance(&exec_addr, &chain)
        .await
        .expect("should check balance")
    {
        log_message(format!(
            "[{}] insufficient funds | exec_addr: {}, chain_id: {}",
            hex::encode(owner.as_bytes()),
            hex::encode(exec_addr.as_bytes()),
            sub.chain_id.0,
        ));
        return stop_sub(&sub);
    }

    if let Err(e) = notify(&sub, &owner, &exec_addr, &chain).await {
        ic_cdk::println!("[{}] Notify error: {}", owner, e);
        log_message(format!(
            "[ADDR: {}, CHAIN ID: {}] publishing, final err: {e:?}",
            hex::encode(owner.as_bytes()),
            chain.chain_id.0
        ))
    }
}

fn stop_sub(sub: &Sub) {
    let timer_id: TimerId = serde_json::from_str(&sub.timer_id).expect("should be valid timer id");
    SUBS.with(|subs| {
        let mut subs = subs.borrow_mut();

        let mut sub = subs.get_mut(sub.id as usize).expect("Sub should exist");

        sub.is_active = false;
    });

    clear_timer(timer_id)
}

async fn notify(sub: &Sub, pub_key: &H160, exec_addr: &H160, chain: &Chain) -> Result<()> {
    let w3 =
        Web3::new(ICHttp::new(chain.rpc.as_str(), None).context("failed to connect to a node")?);

    let abi = EthabiContract::load(add_brackets(&sub.method.abi).as_bytes())
        .expect("abi should be valid");
    let contract = Contract::new(w3.eth(), sub.contract_addr, abi);

    let input = get_input(&sub.method.method_type, sub.pair_id.clone()).await?;
    let key_info = KeyInfo {
        derivation_path: vec![pub_key.as_bytes().to_vec()],
        key_name: KEY_NAME.with(|key_name| key_name.borrow().clone()),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    };

    let nonce = w3.eth().transaction_count(*exec_addr, None).await
        .context("failed to get nonce")?;
    let gas_price = w3.eth().gas_price().await
        .context("failed to get gas price")?;
    let block_height = w3.eth().block_number().await
        .context("failed to get block height")?;

    let tx_otps = Options {
        gas: Some(sub.method.gas_limit.0),
        nonce: Some(nonce),
        gas_price: Some(gas_price),
        transaction_type: Some(U64::from(0)),
        condition: Some(TransactionCondition::Block(block_height.as_u64())),
        ..Default::default()
    };

    let signed_tx = contract.sign(
        &sub.method.name,
        input,
        tx_otps,
        hex::encode(exec_addr.as_bytes()),
        key_info,
        chain.chain_id.0.as_u64(),
    ).await?;

    for i in 1..=MAX_RETRY_ATTEMPTS {
        match w3.eth().send_raw_transaction(signed_tx.raw_transaction.clone()).await {
            Ok(_) => {
                log_message(format!(
                    "[EXEC ADDR: {}, CHAIN ID: {}, SUB TYPE: {:?}] published",
                    hex::encode(exec_addr.as_bytes()),
                    chain.chain_id.0,
                    sub.method.method_type
                ));
                return Ok(());
            }
            Err(err) => log_message(format!(
                "[EXEC ADDR: {}, CHAIN ID: {}] publishing: {}, err: {err:?}",
                hex::encode(exec_addr.as_bytes()),
                chain.chain_id.0,
                i
            )),
        }
    }

    collect_metrics();

    Ok(())
}

pub async fn get_input(method_type: &MethodType, pair_id: Option<String>) -> Result<Vec<Token>> {
    let input = match method_type {
        MethodType::Pair => get_sybil_input(&pair_id.expect("should be provided")).await?,
        MethodType::Random(abi_type) => vec![get_random_input(abi_type).await?],
        MethodType::Empty => vec![],
    };

    Ok(input)
}

async fn get_random_input(abi_type: &str) -> Result<Token> {
    let (mut raw_data,) = raw_rand().await.expect("random should be generated");

    let (insufficient_bytes_count, was_overflowed) = raw_data.len().overflowing_sub(BITS_IN_BYTE);

    if was_overflowed {
        raw_data.append(&mut vec![0; insufficient_bytes_count]);
    }

    let value = u64::from_be_bytes(
        raw_data[..BITS_IN_BYTE]
            .try_into()
            .expect("should be valid convertation"),
    );

    cast_to_param_type(value, abi_type).ok_or(anyhow!("invalid abi type"))
}

async fn get_sybil_input(pair_id: &str) -> Result<Vec<Token>> {
    let rate = get_asset_data(pair_id).await?;

    Ok(vec![
        Token::String(rate.symbol),
        Token::Uint(rate.rate.into()),
        Token::Uint(rate.decimals.into()),
        Token::Uint(rate.timestamp.into()),
    ])
}

pub async fn wait_until_confimation(tx_hash: &H256, w3: &Web3<ICHttp>) -> Result<()> {
    let start = Duration::from_nanos(time()).as_secs();
    let mut current_time = start;

    while (start - current_time) < TIMEOUT {
        let tx_receipt = w3
            .eth()
            .transaction_receipt(*tx_hash)
            .await
            .context("failed to get tx receipt")?;

        if let Some(tx_receipt) = tx_receipt {
            if let Some(status) = tx_receipt.status {
                if status.as_u64() == 0 {
                    return Err(anyhow!(PythiaError::TxFailed));
                }

                return Ok(());
            }
        }

        current_time = Duration::from_nanos(time()).as_secs();
    }

    Err(anyhow!(PythiaError::TxTimeout))
}
