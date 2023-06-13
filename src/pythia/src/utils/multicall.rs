use std::str::FromStr;

use anyhow::{Result, Context};

use candid::Nat;
use ic_dl_utils::retry_until_success;
use ic_web3::{types::{H160, U256, CallRequest, Bytes, BlockId}, contract::{Error, tokens::Tokenizable, Contract, Options}, ethabi::Token, Web3, Transport};

use super::{get_pma, nat_to_u64, get_key_info};

const MULTICALL_ABI: &[u8] = include_bytes!("../../assets/MulticallABI.json");
const MULTICALL_CONTRACT_ADDRESS: &str = "0x26df57f4577dcd7e1deea93299655b14df374b17";
const MULTICALL_CALL_FUNCTION: &str = "multicall";
const MULTICALL_TRANSFER_FUNCTION: &str = "multitransfer";
const BASE_GAS: u64 = 27_000;
pub const GAS_PER_TRANSFER: u64 = 7_900;
const TX_TIMEOUT: u64 = 60*5;

#[derive(Debug, Clone, Default)]
pub struct Call {
    pub target: H160,
    pub call_data: Vec<u8>,
    pub gas_limit: U256,
}

impl Tokenizable for Call {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 3 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (
                Token::Address(target),
                Token::Bytes(call_data),
                Token::Uint(gas_limit),
            ) = (tokens[0].clone(), tokens[1].clone(), tokens[2].clone()) {
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
        return Token::Tuple(vec![
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
        Self: Sized {
            if let Token::Tuple(tokens) = token {
                if tokens.len() != 3 {
                    return Err(Error::InvalidOutputType("invalid tokens number".into()));
                }
    
                if let (
                    Token::Bool(success),
                    Token::Uint(used_gas),
                    Token::Bytes(return_data),
                ) = (tokens[0].clone(), tokens[1].clone(), tokens[2].clone()) {
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
        return Token::Tuple(vec![
            Token::Bool(self.success),
            Token::Bytes(self.return_data),
        ])
    }
}

pub async fn multicall<T: Transport>(
    w3: &Web3<T>,
    chain_id: &Nat,
    calls: Vec<Call>,
    gas_price: U256,
) -> Result<Vec<MulticallResult>> {
    let contract_addr = H160::from_str(MULTICALL_CONTRACT_ADDRESS)
        .expect("should be valid contract address");
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .expect("should be valid init contract data");
    let from  = get_pma()
        .await
        .context("failed to get the PMA")?;
    let key_info = get_key_info();

    let options = Options {
        gas_price: Some(gas_price),
        gas: Some(U256::from(calls.iter().fold(U256::from(BASE_GAS+10000), |result, call| result + call.gas_limit))),
        nonce: Some(retry_until_success!(w3.eth().transaction_count(H160::from_str(&from)?, None))?),
        ..Default::default()
    };
    let params: Vec<Token> = calls.iter().map(|c| c.clone().into_token()).collect();
    let signed_call = contract.sign(
        MULTICALL_CALL_FUNCTION,
        vec![params.clone()],
        options,
        from,
        key_info,
        nat_to_u64(chain_id),
    )
    .await
    .context("failed to sign contract call")?;

    let tx_hash = retry_until_success!(w3.eth().send_raw_transaction(signed_call.raw_transaction.clone()))
        .context("failed to execute a raw tx")?;
    let tx_receipt = ic_dl_utils::evm::wait_for_success_confirmation(w3, &tx_hash, TX_TIMEOUT)
        .await
        .context("failed while waiting for a successful tx execution")?;

    let data = contract
        .abi()
        .function(MULTICALL_CALL_FUNCTION)
        .and_then(|f| f.encode_input(&vec![params.into_token()]))
        .context("failed to form data of a call")?;
    let call_request = CallRequest {
        from: Some(tx_receipt.from),
        to: tx_receipt.to,
        data: Some(Bytes::from(data)),
        ..Default::default()
    };
    let block_number = BlockId::from(tx_receipt.block_number.expect("block number should be valid"));
    let raw_result = retry_until_success!(w3.eth().call(call_request.clone(), Some(block_number)))?;
    let call_result: Vec<Token> = contract
        .abi()
        .function(MULTICALL_CALL_FUNCTION)
        .and_then(|f| f.decode_output(&raw_result.0))
        .context("failed to decode outputs")?;

    let results = call_result
        .first()
        .context("should be valid call result")?
        .clone()
        .into_array()
        .context("should be valid call result: array")?;

    results.iter().map(|token| {
        MulticallResult::from_token(token.clone()).context("failed to decode from token")
    }).collect()
}

#[derive(Debug, Clone, Default)]
pub struct Transfer {
    pub target: H160,
    pub value: U256,
}

impl Tokenizable for Transfer {
    fn from_token(token: Token) -> std::result::Result<Self, Error>
    where
        Self: Sized {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 2 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (
                Token::Address(target),
                Token::Uint(value),
            ) = (tokens[0].clone(), tokens[1].clone()) {
                return Ok(Self {
                    target,
                    value,
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        return Token::Tuple(vec![
            Token::Address(self.target),
            Token::Uint(self.value),
        ])
    }
}

pub async fn multitranfer<T: Transport>(w3: &Web3<T>, chain_id: &Nat, transfers: Vec<Transfer>) -> Result<()> {
    let contract_addr = H160::from_str(MULTICALL_CONTRACT_ADDRESS)
        .expect("should be valid contract address");
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .expect("should be valid init contract data");
    let from  = get_pma()
        .await
        .context("failed to get the PMA")?;
    let key_info = get_key_info();

    let gas_price = retry_until_success!(w3.eth().gas_price())?;

    let options = Options {
        gas_price: Some((gas_price/10)*12),
        gas: Some(U256::from(BASE_GAS + (GAS_PER_TRANSFER * transfers.len() as u64))),
        value: Some(transfers.iter().fold(U256::from(0), |sum, t| sum + t.value)),
        nonce: Some(retry_until_success!(w3.eth().transaction_count(H160::from_str(&from)?, None))?),
        ..Default::default()
    };
    let params: Vec<Token> = transfers.iter().map(|c| c.clone().into_token()).collect();
    let signed_call = contract.sign(
        MULTICALL_TRANSFER_FUNCTION,
        vec![params.clone()],
        options,
        from,
        key_info,
        nat_to_u64(&chain_id),
    )
    .await
    .context("failed to sign contract call")?;

    let tx_hash = retry_until_success!(w3.eth().send_raw_transaction(signed_call.raw_transaction.clone()))
        .context("failed to execute a raw tx")?;
    ic_cdk::println!("multitransfer tx_hash: {}", hex::encode(&tx_hash.as_bytes()));
    ic_dl_utils::evm::wait_for_success_confirmation(w3, &tx_hash, TX_TIMEOUT)
        .await
        .context("failed while waiting for a successful tx execution")?;
    Ok(())
}