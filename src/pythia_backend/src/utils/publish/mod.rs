use anyhow::{Context, Result};

use ic_cdk_timers::clear_timer;
use ic_cdk::export::Principal;
use ic_utils::logger::log_message;
use ic_web3::{
    ic::KeyInfo,
    contract::{Contract, Options},
    ethabi::Contract as EthabiContract,
    transports::ICHttp,
    Web3,
};

use crate::{
    utils::{add_brackets, cast_to_param_type, check_balance},
    Chain, Sub, User, CHAINS, USERS, KEY_NAME,
};

pub fn publish(sub_id: u64, owner: Principal) {
    ic_cdk::spawn(_publish(sub_id, owner));
}

async fn _publish(sub_id: u64, owner: Principal) {
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
        log_message(format!("[{}] {}", owner, e));
    }
}

fn stop_sub(sub: &Sub, user: &User) {
    log_message(format!(
        "[{}] insufficient funds | exec_addr: {}, chain_id: {}",
        ic_cdk::caller(),
        user.exec_addr,
        sub.chain_id.0,
    ));

    clear_timer(sub.timer_id)
}

async fn notify(sub: &Sub, user: &User, chain: &Chain) -> Result<()> {
    let w3 = Web3::new(
        ICHttp::new(chain.rpc.as_str(), None, None).context("failed to connect to a node")?,
    );

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

    let abi = EthabiContract::load(add_brackets(&sub.method.abi).as_bytes())
        .expect("abi should be valid");

    let contract = Contract::new(w3.eth(), sub.contract_addr, abi);

    let tx_otps = Options {
        gas: Some(sub.method.gas_limit.0),
        nonce: Some(nonce),
        gas_price: Some(gas_price),
        ..Default::default()
    };

    let input =
        cast_to_param_type(chain.native_price, &sub.method.param).expect("should be able to cast");

    let key_info = KeyInfo {
        derivation_path: vec![user.pub_key.as_bytes().to_vec()],
        key_name: KEY_NAME.with(|key_name| key_name.borrow().clone()),
    };

    let tx_hash = contract.signed_call(
        &sub.method.name,
        input,
        tx_otps,
        user.exec_addr.to_string(),
        key_info,
        chain.chain_id.0.as_u64(),
    )
    .await
    .context("tx failed to execute")?;

    ic_cdk::println!("tx_hash: {}", hex::encode(tx_hash.as_bytes()));

    Ok(())
}
