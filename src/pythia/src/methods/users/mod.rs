use std::str::FromStr;

use anyhow::{Result, anyhow, Context};

use ic_cdk::export::candid::Nat;
use ic_cdk_macros::update;
use ic_utils::logger::log_message;
use ic_web3::{
    ic::KeyInfo,
    transports::ICHttp,
    types::{TransactionParameters, H160},
    Web3,
};

use crate::{utils::rec_eth_addr, User, USERS, PythiaError, CHAINS, U256, KEY_NAME};

const ETH_TRANSFER_GAS_LIMIT: u64 = 21000;
const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

#[update]
pub async fn add_user(msg: String, sig: String) -> Result<String, String> {
    _add_user(msg, sig)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn _add_user(msg: String, sig: String) -> Result<String> {
    let pub_key = rec_eth_addr(&msg, &sig).await?;

    let user = USERS.with(|users_state| {
        users_state.borrow().get(&pub_key).map(|u| u.clone())
    });

    if user.is_some() {
        return Err(anyhow!("User already exists"));
    }

    let user = User::new(pub_key).await?;
    let exec_addr = hex::encode(user.exec_addr.as_bytes());

    USERS.with(|users_state| {
        users_state.borrow_mut().insert(pub_key, user);
    });

    log_message(format!(
        "[USER: {}] creation, exec_addr: {}", hex::encode(pub_key.as_bytes()), hex::encode(exec_addr.as_bytes())
    ));

    Ok(exec_addr)
}

#[update]
pub async fn withdraw(chain_id: Nat, msg: String, sig: String, receiver: String) -> Result<(), String> {
    _withdraw(chain_id, msg, sig, receiver)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn _withdraw(chain_id: Nat, msg: String, sig: String, receiver: String) -> Result<()> {
    let chain = CHAINS.with(|chains| {
        let chains = chains.borrow();
        chains
            .get(&U256::from(chain_id))
            .ok_or(PythiaError::ChainDoesNotExist)
            .map(|chain| chain.clone())
    })?;
    let pub_key = rec_eth_addr(&msg, &sig).await?;
    let user = USERS.with(|users_state| {
        users_state
            .borrow()
            .get(&pub_key)
            .map(|u| u.clone())
            .context("user does not exists")
    })?;
    let receiver = H160::from_str(&receiver)?;

    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None).context("failed to connect to a node")?,
    );

    let value = w3
        .eth()
        .balance(user.exec_addr, None)
        .await?;

    if value == 0.into() {
        return Ok(())
    }

    let nonce = w3
        .eth()
        .transaction_count(user.exec_addr, None)
        .await
        .context("failed to get nonce")?;

    let gas_price = w3
        .eth()
        .gas_price()
        .await
        .context("failed to get gas price")?;

    // 1.1 multiplication
    let gas_price = (gas_price / 10) * 11;

    let value = value - (gas_price * ETH_TRANSFER_GAS_LIMIT);

    let tx = TransactionParameters {
        to: Some(receiver),
        nonce: Some(nonce),
        value,
        gas_price: Some(gas_price),
        gas: ETH_TRANSFER_GAS_LIMIT.into(),
        ..Default::default()
    };

    let key_info = KeyInfo {
        derivation_path: vec![user.pub_key.as_bytes().to_vec()],
        key_name: KEY_NAME.with(|key_name| key_name.borrow().clone()),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(
            tx,
            user.exec_addr.to_string(),
            key_info,
            chain.chain_id.0.as_u64(),
        )
        .await?;

    w3.eth()
        .send_raw_transaction(signed_tx.raw_transaction)
        .await?;

    Ok(())
}
