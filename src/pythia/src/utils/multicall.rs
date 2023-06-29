use std::str::FromStr;

use anyhow::{Context, Result};

use candid::Nat;
use ic_web3_rs::{
    contract::{tokens::Tokenizable, Contract, Error, Options},
    ethabi::Token,
    types::{BlockId, Bytes, CallRequest, H160, U256},
    Transport, Web3,
};

use super::{address, canister, nat, web3};
use crate::{
    log, retry_until_success,
    types::{
        chains::{Chain, Chains},
        errors::PythiaError,
        logger::PUBLISHER,
    },
};

const MULTICALL_ABI: &[u8] = include_bytes!("../../assets/MulticallABI.json");
const MULTICALL_CONTRACT_ADDRESS: &str = "0xa27a3A7702Bc1010be95f73A2c64873d21D6D027";
const MULTICALL_CALL_FUNCTION: &str = "multicall";
const MULTICALL_TRANSFER_FUNCTION: &str = "multitransfer";
const BASE_GAS: u64 = 27_000;
pub const GAS_PER_TRANSFER: u64 = 7_900;
const GAS_FOR_OPS: u64 = 10_000;
const TX_TIMEOUT: u64 = 60 * 5;

#[derive(Debug, Clone, Default)]
pub struct Call {
    pub target: H160,
    pub call_data: Vec<u8>,
    pub gas_limit: U256,
}

impl Tokenizable for Call {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 3 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Address(target), Token::Bytes(call_data), Token::Uint(gas_limit)) =
                (tokens[0].clone(), tokens[1].clone(), tokens[2].clone())
            {
                return Ok(Self {
                    target,
                    call_data,
                    gas_limit,
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            Token::Address(self.target),
            Token::Bytes(self.call_data),
            Token::Uint(self.gas_limit),
        ])
    }
}

#[derive(Debug, Clone, Default)]
pub struct MulticallResult {
    pub success: bool,
    pub used_gas: U256,
    pub return_data: Vec<u8>,
}

impl Tokenizable for MulticallResult {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 3 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Bool(success), Token::Uint(used_gas), Token::Bytes(return_data)) =
                (tokens[0].clone(), tokens[1].clone(), tokens[2].clone())
            {
                return Ok(Self {
                    success,
                    used_gas,
                    return_data,
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            Token::Bool(self.success),
            Token::Bytes(self.return_data),
        ])
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transfer {
    pub target: H160,
    pub value: U256,
}

impl Tokenizable for Transfer {
    fn from_token(token: Token) -> std::result::Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 2 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Address(target), Token::Uint(value)) =
                (tokens[0].clone(), tokens[1].clone())
            {
                return Ok(Self { target, value });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![Token::Address(self.target), Token::Uint(self.value)])
    }
}

pub async fn multicall<T: Transport>(
    w3: &Web3<T>,
    chain_id: &Nat,
    calls: Vec<Call>,
    gas_price: U256,
) -> Result<Vec<MulticallResult>> {
    log!("[{PUBLISHER}] chain: {}, prepering multicall", chain_id);
    let mut calls = calls;
    let mut result: Vec<MulticallResult> = vec![];
    let chain = Chains::get(chain_id)?;
    let contract_addr = address::to_h160(MULTICALL_CONTRACT_ADDRESS)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .context(PythiaError::InvalidContractABI)?;
    let from = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price = (gas_price / 10) * 12;

    while !calls.is_empty() {
        let (current_calls_batch, _calls) = get_current_calls_batch(&calls, &chain);
        calls = _calls;

        let results = execute_multicall_batch(
            w3,
            &from,
            &gas_price,
            &contract,
            &current_calls_batch,
            chain_id,
        )
        .await?;

        result.append(
            &mut results
                .iter()
                .map(|token| {
                    MulticallResult::from_token(token.clone()).expect("failed to decode from token")
                })
                .collect::<Vec<MulticallResult>>(),
        );
    }

    Ok(result)
}

async fn execute_multicall_batch<T: Transport>(
    w3: &Web3<T>,
    from: &str,
    gas_price: &U256,
    contract: &Contract<T>,
    batch: &[Call],
    chain_id: &Nat,
) -> Result<Vec<Token>> {
    let options = Options {
        gas_price: Some(*gas_price),
        gas: Some(
            batch
                .iter()
                .fold(U256::from(BASE_GAS + GAS_FOR_OPS), |result, call| {
                    result + call.gas_limit
                }),
        ),
        nonce: Some(retry_until_success!(w3.eth().transaction_count(
            H160::from_str(from)?,
            None,
            canister::transform_ctx()
        ))?),
        ..Default::default()
    };

    let params: Vec<Token> = batch.iter().map(|c| c.clone().into_token()).collect();
    let signed_call = contract
        .sign(
            MULTICALL_CALL_FUNCTION,
            vec![params.clone()],
            options.clone(),
            from.to_string(),
            web3::key_info(),
            nat::to_u64(chain_id),
        )
        .await
        .context(PythiaError::UnableToSignContractCall)?;
    log!("[{PUBLISHER}] chain: {}, tx was signed", chain_id);
    let tx_hash = retry_until_success!(w3.eth().send_raw_transaction(
        signed_call.raw_transaction.clone(),
        canister::transform_ctx()
    ))
    .context(PythiaError::UnableToExecuteRawTx)?;

    log!("[{PUBLISHER}] chain: {}, tx was sent", chain_id);
    let tx_receipt = web3::wait_for_success_confirmation(w3, &tx_hash, TX_TIMEOUT)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;
    log!("[{PUBLISHER}] chain: {}, tx was executed", chain_id);
    let data = contract
        .abi()
        .function(MULTICALL_CALL_FUNCTION)
        .and_then(|f| f.encode_input(&[params.into_token()]))
        .context(PythiaError::UnableToFormCallData)?;
    let call_request = CallRequest {
        from: Some(tx_receipt.from),
        to: tx_receipt.to,
        data: Some(Bytes::from(data)),
        ..Default::default()
    };
    let block_number = BlockId::from(
        tx_receipt
            .block_number
            .expect("block number should be valid"),
    );
    let raw_result = retry_until_success!(w3.eth().call(
        call_request.clone(),
        Some(block_number),
        canister::transform_ctx()
    ))?;
    log!("[{PUBLISHER}] chain: {}, tx result was received", chain_id);
    let call_result: Vec<Token> = contract
        .abi()
        .function(MULTICALL_CALL_FUNCTION)
        .and_then(|f| f.decode_output(&raw_result.0))
        .context(PythiaError::UnableToDecodeOutputs)?;

    call_result
        .first()
        .context(PythiaError::InvalidMulticallResult)?
        .clone()
        .into_array()
        .context(PythiaError::InvalidMulticallResult)
}

fn get_current_calls_batch(calls: &[Call], chain: &Chain) -> (Vec<Call>, Vec<Call>) {
    let mut gas_counter = Nat::from(BASE_GAS + 1000);
    for (i, call) in calls.iter().enumerate() {
        gas_counter += nat::from_u256(&call.gas_limit);
        if gas_counter >= chain.block_gas_limit {
            return (calls[..i].to_vec(), calls[i..].to_vec());
        }
    }

    (calls.to_vec(), vec![])
}

pub async fn multitransfer<T: Transport>(
    w3: &Web3<T>,
    chain_id: &Nat,
    transfers: Vec<Transfer>,
) -> Result<()> {
    let contract_addr = address::to_h160(MULTICALL_CONTRACT_ADDRESS)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .context(PythiaError::InvalidContractABI)?;

    let from = canister::pma().await.context(PythiaError::UnableToGetPMA)?;
    let key_info = web3::key_info();

    let gas_price = retry_until_success!(w3.eth().gas_price(canister::transform_ctx()))?;
    let gas_limit = BASE_GAS + (GAS_PER_TRANSFER * transfers.len() as u64);
    let value = transfers.iter().fold(U256::from(0), |sum, t| sum + t.value);
    let nonce = retry_until_success!(w3.eth().transaction_count(
        H160::from_str(&from)?,
        None,
        canister::transform_ctx()
    ))?;

    let options = Options {
        gas_price: Some(gas_price),
        gas: Some(U256::from(gas_limit)),
        value: Some(value),
        nonce: Some(nonce),
        ..Default::default()
    };

    let params: Vec<Token> = transfers.iter().map(|c| c.clone().into_token()).collect();

    let signed_call = contract
        .sign(
            MULTICALL_TRANSFER_FUNCTION,
            vec![params.clone()],
            options,
            from,
            key_info,
            nat::to_u64(chain_id),
        )
        .await
        .context(PythiaError::UnableToSignContractCall)?;

    let tx_hash = retry_until_success!(w3.eth().send_raw_transaction(
        signed_call.raw_transaction.clone(),
        canister::transform_ctx()
    ))
    .context(PythiaError::UnableToExecuteRawTx)?;

    web3::wait_for_success_confirmation(w3, &tx_hash, TX_TIMEOUT)
        .await
        .context(PythiaError::WaitingForSuccessConfirmationFailed)?;

    Ok(())
}
