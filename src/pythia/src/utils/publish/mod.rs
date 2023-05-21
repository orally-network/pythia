use anyhow::{anyhow, Context, Result};

use ic_cdk::{api::management_canister::main::raw_rand, api::time};
use ic_cdk_timers::{clear_timer, TimerId};
use ic_utils::logger::log_message;
use ic_web3::{
    contract::{Contract, Options},
    ethabi::{Contract as EthabiContract, Token},
    ic::KeyInfo,
    transports::ICHttp,
    types::{H160, H256},
    Web3,
};

use crate::{
    utils::{add_brackets, cast_to_param_type, check_balance, sybil::get_asset_data},
    Chain, PythiaError, Sub, User, CHAINS, KEY_NAME, USERS,
};

const TIMEOUT: u64 = 60 * 60;
const MAX_RETRY_ATTEMPTS: u64 = 3;
const BITS_IN_BYTE: usize = 8;

pub fn publish(sub_id: u64, owner: H160) {
    ic_cdk::spawn(_publish(sub_id, owner));
}

async fn _publish(sub_id: u64, owner: H160) {
    let user = USERS.with(|users| {
        users
            .borrow()
            .get(&owner)
            .expect("User should exist")
            .clone()
    });

    let sub = user
        .subs
        .iter()
        .find(|sub| sub.id == sub_id)
        .expect("Sub should exist");

    let chain = CHAINS.with(|chains| {
        chains
            .borrow()
            .get(&sub.chain_id)
            .expect("Chain should exist")
            .clone()
    });

    if check_balance(&user, &chain).await.is_err() {
        return stop_sub(sub, &user);
    }

    if let Err(e) = notify(sub, &user, &chain).await {
        ic_cdk::println!("[{}] Notify error: {}", owner, e);
        log_message(format!("[{}] {}", owner, e));
    }
}

fn stop_sub(sub: &Sub, user: &User) {
    log_message(format!(
        "[{}] insufficient funds | exec_addr: {}, chain_id: {}",
        user.pub_key, user.exec_addr, sub.chain_id.0,
    ));

    let timer_id: TimerId = serde_json::from_str(&sub.timer_id).expect("should be valid timer id");

    clear_timer(timer_id)
}

async fn notify(sub: &Sub, user: &User, chain: &Chain) -> Result<()> {
    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

    let abi = EthabiContract::load(add_brackets(&sub.method.abi).as_bytes())
        .expect("abi should be valid");

    let contract = Contract::new(w3.eth(), sub.contract_addr, abi);

    let input = get_input(sub).await?;

    let key_info = KeyInfo {
        derivation_path: vec![user.pub_key.as_bytes().to_vec()],
        key_name: KEY_NAME.with(|key_name| key_name.borrow().clone()),
    };

    for _ in 1..=MAX_RETRY_ATTEMPTS {
        let gas_price = w3
            .eth()
            .gas_price()
            .await
            .context("failed to get gas price")?;

        let nonce = w3
            .eth()
            .transaction_count(user.exec_addr, None)
            .await
            .context("failed to get nonce")?;

        let tx_otps = Options {
            gas: Some(sub.method.gas_limit.0),
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            transaction_type: None,
            ..Default::default()
        };

        let tx_hash = contract
            .signed_call(
                &sub.method.name,
                input.clone(),
                tx_otps,
                user.exec_addr.to_string(),
                key_info.clone(),
                chain.chain_id.0.as_u64(),
            )
            .await?;

        if let Err(err) = wait_until_confimation(&tx_hash, &w3).await {
            match err.root_cause().downcast_ref() {
                Some(PythiaError::TxTimeout) => break,
                _ => Err(err)?,
            }
        }
        
        break;
    }

    Ok(())
}

async fn get_input(sub: &Sub) -> Result<Token> {
    let raw_input = if sub.is_random {
        get_random_input().await
    } else if sub.pair_id.is_some() {
        get_sybil_input(sub).await?
    } else {
        0
    };

    Ok(cast_to_param_type(raw_input, &sub.method.param).expect("should be able to cast"))
}

async fn get_random_input() -> u64 {
    let (mut raw_data,) = raw_rand().await.expect("random should be generated");

    let (insufficient_bytes_count, was_overflowed) = raw_data.len().overflowing_sub(BITS_IN_BYTE);

    if was_overflowed {
        raw_data.append(&mut vec![0; insufficient_bytes_count]);
    }

    u64::from_be_bytes(
        raw_data[..BITS_IN_BYTE]
            .try_into()
            .expect("should be valid convertation"),
    )
}

async fn get_sybil_input(sub: &Sub) -> Result<u64> {
    let pair_id = sub
        .pair_id
        .clone()
        .ok_or(anyhow!("Pair id does not exists"))?;

    let rate = get_asset_data(&pair_id).await?;

    Ok(rate.rate)
}

async fn wait_until_confimation(tx_hash: &H256, w3: &Web3<ICHttp>) -> Result<()> {
    let start = time();
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

        current_time = time();
    }

    Err(anyhow!(PythiaError::TxTimeout))
}
