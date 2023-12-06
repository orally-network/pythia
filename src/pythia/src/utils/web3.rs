use std::str::FromStr;

use anyhow::{Context, Result};

use candid::{CandidType, Nat, Principal};

use eth_rpc::Source;
use futures::future::BoxFuture;
use ic_cdk::api::{
    call::call_with_payment128,
    management_canister::http_request::{TransformContext, TransformFunc},
};
use ic_web3_rs::{
    error::TransportError,
    helpers,
    ic::KeyInfo,
    transports::ic_http_client::{CallOptions, CallOptionsBuilder},
    types::{Transaction, TransactionId, TransactionParameters, TransactionReceipt, H256},
    RequestId, Transport, Web3,
};
use jsonrpc_core::{Call, Output, Request};
use serde::Deserialize;
use serde_json::Value;

use super::{address, canister, nat, time, web3};
use crate::{
    clone_with_state, log, retry_until_success,
    types::{chains::Chains, errors::PythiaError},
};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
pub const TRANSFER_GAS_LIMIT: u64 = 21_000;
const TX_SUCCESS_STATUS: u64 = 1;
const TX_WAIT_DELAY: u64 = 3;
const MAX_CYCLES: u128 = 6_000_000_000;
const MAX_RESPONSE_BYTES: u64 = 100000;

/// ICEthRpc deals with the JSON-RPC canister nametd "ic-eth-rpc" which is deployed on the IC.
#[derive(Clone, Debug)]
pub struct ICEthRpc {
    rpc_url: String,
    ic_ethr_rpc: Principal,
    max_response_bytes: u64,
}

impl ICEthRpc {
    /// Create new ICEthRpc instance
    pub fn new(url: &str, max_response_bytes: u64) -> Self {
        Self {
            rpc_url: url.to_string(),
            ic_ethr_rpc: clone_with_state!(ic_eth_rpc_canister),
            max_response_bytes,
        }
    }

    // we return constant id because ic_eth_rpc doesn't use it
    pub fn next_id(&self) -> RequestId {
        1
    }
}

#[derive(CandidType, Debug, Deserialize)]
pub enum EthRpcError {
    NoPermission,
    TooFewCycles { expected: u128, received: u128 },
    ServiceUrlParseError,
    ServiceHostNotAllowed(String),
    ProviderNotFound,
    HttpRequestError { code: u32, message: String },
}

async fn execute_canister_call(
    ic_eth_rpc: Principal,
    source: Source,
    json_rpc_payload: String,
    max_response_bytes: u64,
) -> Result<Value, ic_web3_rs::Error> {
    let (result,): (Result<String, EthRpcError>,) = call_with_payment128(
        ic_eth_rpc,
        "request",
        (source, json_rpc_payload, max_response_bytes),
        MAX_CYCLES,
    )
    .await
    .map_err(|(code, msg)| {
        ic_web3_rs::Error::Transport(TransportError::Message(format!("{:?}: {}", code, msg)))
    })?;

    let result = result.map_err(|err| {
        ic_web3_rs::Error::Transport(TransportError::Message(format!(
            "Error in ic_eth_rpc: {:?}",
            err
        )))
    })?;

    let output: Output = serde_json::from_str(&result).unwrap();

    match output {
        Output::Success(success) => {
            log!("ICEthRpc Response: {:#?}", result);
            Ok(success.result)
        }
        Output::Failure(failure) => {
            log!("ICEthRpc error: {:#?}", failure);
            Err(ic_web3_rs::Error::Transport(TransportError::Message(
                failure.error.message,
            )))
        }
    }
}

impl Transport for ICEthRpc {
    type Out = BoxFuture<'static, Result<Value, ic_web3_rs::Error>>;

    fn prepare(&self, method: &str, params: Vec<Value>) -> (RequestId, Call) {
        let id = self.next_id();
        let request = helpers::build_request(id, method, params);
        (id, request)
    }

    fn send(&self, _: RequestId, call: Call, _: CallOptions) -> Self::Out {
        let source: Source = Source::Url(self.rpc_url.to_string());
        let json_rpc_payload = serde_json::to_string(&Request::Single(call.clone())).unwrap();

        log!("ICEthRpc Request: {:#?}", json_rpc_payload);

        let ic_eth_rpc = self.ic_ethr_rpc;
        let max_response_bytes = self.max_response_bytes;

        Box::pin(async move {
            execute_canister_call(ic_eth_rpc, source, json_rpc_payload, max_response_bytes).await
        })
    }

    fn set_max_response_bytes(&mut self, v: u64) {
        self.max_response_bytes = v;
    }
}

pub fn instance(chain_id: &Nat) -> Result<Web3<ICEthRpc>> {
    Ok(Web3::new(ICEthRpc::new(
        &Chains::get(chain_id)?.rpc,
        MAX_RESPONSE_BYTES,
    )))
}

pub async fn get_tx(chain_id: &Nat, tx_hash: &str) -> Result<Transaction> {
    let tx_hash = H256::from_str(tx_hash)?;
    let w3 = instance(chain_id)?;

    let tx_receipt = retry_until_success!(w3
        .eth()
        .transaction_receipt(tx_hash, canister::transform_ctx_tx_with_logs()))?
    .context(PythiaError::TxDoesNotExist)?;

    match tx_receipt.status {
        Some(status) => {
            if status.as_u64() != 1 {
                return Err(PythiaError::TxHasFailed.into());
            }
        }
        None => return Err(PythiaError::TxNotExecuted.into()),
    }

    retry_until_success!(w3
        .eth()
        .transaction(TransactionId::from(tx_hash), canister::transform_ctx_tx()))?
    .context(PythiaError::TxDoesNotExist)
}

pub async fn gas_price(chain_id: &Nat) -> Result<Nat> {
    let w3 = instance(chain_id)?;

    let gas_price = nat::from_u256(&retry_until_success!(w3
        .eth()
        .gas_price(canister::transform_ctx()))?);

    Ok(gas_price)
}

#[inline(always)]
pub fn key_info() -> KeyInfo {
    KeyInfo {
        derivation_path: vec![vec![]],
        key_name: clone_with_state!(key_name),
        ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
    }
}

pub async fn transfer(chain_id: &Nat, to: &str, value: &Nat) -> Result<()> {
    let w3 = instance(chain_id)?;
    let from = canister::pma().await?;
    let from_h160 = address::to_h160(&from)?;
    let to = address::to_h160(to)?;

    let nonce = retry_until_success!(w3.eth().transaction_count(
        from_h160,
        None,
        canister::transform_ctx()
    ))?;
    let mut gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    gas_price = (gas_price / 10) * 12;

    let tx = TransactionParameters {
        gas: TRANSFER_GAS_LIMIT.into(),
        gas_price: Some(gas_price),
        to: Some(to),
        value: nat::to_u256(value),
        nonce: Some(nonce),
        ..Default::default()
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(tx, from, key_info(), nat::to_u64(chain_id))
        .await?;

    let tx_hash = retry_until_success!(w3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction.clone(), canister::transform_ctx()))?;
    web3::wait_for_success_confirmation(&w3, &tx_hash, 60)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;
    Ok(())
}

pub async fn transfer_all(chain_id: &Nat, to: &str) -> Result<()> {
    let w3 = instance(chain_id)?;
    let from = canister::pma().await?;
    let from_h160 = address::to_h160(&from)?;
    let to = address::to_h160(to)?;

    let nonce = retry_until_success!(w3.eth().transaction_count(
        from_h160,
        None,
        canister::transform_ctx()
    ))?;
    let mut gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;
    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    gas_price = (gas_price / 10) * 12;
    let mut value =
        retry_until_success!(w3.eth().balance(from_h160, None, canister::transform_ctx()))?;
    value -= gas_price * TRANSFER_GAS_LIMIT;

    let tx = TransactionParameters {
        gas: TRANSFER_GAS_LIMIT.into(),
        gas_price: Some(gas_price),
        to: Some(to),
        value,
        nonce: Some(nonce),
        ..Default::default()
    };

    let signed_tx = w3
        .accounts()
        .sign_transaction(tx, from, key_info(), nat::to_u64(chain_id))
        .await?;

    let tx_hash = retry_until_success!(w3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction.clone(), canister::transform_ctx()))?;
    web3::wait_for_success_confirmation(&w3, &tx_hash, 60)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;

    Ok(())
}

pub async fn wait_for_success_confirmation<T: Transport>(
    w3: &Web3<T>,
    tx_hash: &H256,
    timeout: u64,
) -> Result<TransactionReceipt> {
    let receipt = wait_for_confirmation(w3, tx_hash, timeout).await?;

    let tx_status = receipt.status.expect("tx should be confirmed").as_u64();

    if tx_status != TX_SUCCESS_STATUS {
        return Err(PythiaError::TxHasFailed.into());
    }

    Ok(receipt)
}

pub async fn wait_for_confirmation<T: Transport>(
    w3: &Web3<T>,
    tx_hash: &H256,
    timeout: u64,
) -> Result<TransactionReceipt> {
    let call_opts = CallOptionsBuilder::default()
        .transform(Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".into(),
            }),
            context: vec![],
        }))
        .cycles(None)
        .max_resp(None)
        .build()
        .expect("failed to build call options");

    let end_time = time::in_seconds() + timeout;
    while time::in_seconds() < end_time {
        time::wait(TX_WAIT_DELAY).await;

        let tx_receipt =
            retry_until_success!(w3.eth().transaction_receipt(*tx_hash, call_opts.clone()))
                .context(PythiaError::UnableToGetTxReceipt)?;

        if let Some(tx_receipt) = tx_receipt {
            if tx_receipt.status.is_some() {
                return Ok(tx_receipt);
            }
        }
    }

    Err(PythiaError::TxTimeout.into())
}
