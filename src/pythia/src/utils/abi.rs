use anyhow::{Context, Result};
use ic_cdk::api::management_canister::main::raw_rand;
use ic_web3::ethabi::{Function, Token};
use serde_json::json;

use crate::{
    types::methods::{Method, MethodType},
    utils::sybil,
    PythiaError,
};

const BITS_IN_BYTE: usize = 8;

pub fn resolve_abi(
    method_abi: String,
    pair_id: Option<String>,
    is_random: bool,
) -> Result<(String, MethodType)> {
    let raw_abi: Vec<&str> = method_abi.split_terminator(&['(', ')', ',']).collect();
    if let Some(pair_id) = pair_id {
        get_pair_abi(&raw_abi, &pair_id)
    } else if is_random {
        get_random_abi(&raw_abi)
    } else {
        get_empty_abi(&raw_abi)
    }
}

fn get_pair_abi(raw_abi: &[&str], pair_id: &str) -> Result<(String, MethodType)> {
    let func_name = raw_abi
        .first()
        .context(PythiaError::InvalidABIFunctionName)?
        .to_string();
    if raw_abi.len() != 5
        || raw_abi[1] != "string"
        || raw_abi[2] != " uint256"
        || raw_abi[3] != " uint256"
        || raw_abi[4] != " uint256"
    {
        return Err(PythiaError::InvalidABIParameters.into());
    }

    let data = json!({
        "inputs": [
            {
                "internalType": "string",
                "name": "pair_id",
                "type": "string",
            },
            {
                "internalType": "uint256",
                "name": "price",
                "type": "uint256",
            },
            {
                "internalType": "uint256",
                "name": "decimals",
                "type": "uint256",
            },
            {
                "internalType": "uint256",
                "name": "timestamp",
                "type": "uint256",
            }
        ],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Pair(pair_id.into())))
}

fn get_random_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    if raw_abi.len() != 2 {
        return Err(PythiaError::InvalidABIParametersNumber.into());
    }
    let func_name = raw_abi
        .first()
        .context(PythiaError::InvalidABIFunctionName)?
        .to_string();
    let param_type = raw_abi
        .get(1)
        .context(PythiaError::InvalidABIParameters)?
        .to_string();
    if !is_supported_func_param(&param_type) {
        return Err(PythiaError::InvalidABIParameterTypes.into());
    }

    let data = json!({
        "inputs": [
            {
                "internalType": param_type,
                "name": "template",
                "type": param_type,
            }
        ],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Random(param_type)))
}

fn get_empty_abi(raw_abi: &[&str]) -> Result<(String, MethodType)> {
    if raw_abi.len() != 2 && !raw_abi[1].is_empty() {
        return Err(PythiaError::InvalidABIParametersNumber.into());
    }

    let func_name = raw_abi
        .first()
        .context(PythiaError::InvalidABIFunctionName)?
        .to_string();

    let data = json!({
        "inputs": [],
        "name": func_name,
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    });

    Ok((data.to_string(), MethodType::Empty))
}

fn is_supported_func_param(func: &str) -> bool {
    matches!(func, f if f.starts_with("string") 
            || f.starts_with("bytes") 
            || f.starts_with("uint") 
            || f.starts_with("int"))
}

pub fn cast_to_param_type(value: u64, kind: &str) -> Option<Token> {
    if kind == "bytes" {
        return Some(Token::Bytes(value.to_le_bytes().to_vec()));
    }
    if kind.contains("bytes") {
        return Some(Token::FixedBytes(value.to_le_bytes().to_vec()));
    }
    if kind.contains("uint") {
        return Some(Token::Uint(value.into()));
    }
    if kind.contains("int") {
        return Some(Token::Int(value.into()));
    }
    if kind.contains("string") {
        return Some(Token::String(value.to_string()));
    }

    None
}

pub async fn get_call_data(method: &Method) -> Vec<u8> {
    let input = get_input(&method.method_type)
        .await
        .expect("should be valid input");

    serde_json::from_str::<Function>(&method.abi)
        .expect("should be valid abi")
        .encode_input(&input)
        .expect("should encode")
}

pub async fn get_input(method_type: &MethodType) -> Result<Vec<Token>> {
    let input = match method_type {
        MethodType::Pair(pair_id) => get_sybil_input(pair_id).await?,
        MethodType::Random(abi_type) => vec![get_random_input(abi_type).await?],
        MethodType::Empty => vec![],
    };

    Ok(input)
}

pub async fn get_random_input(abi_type: &str) -> Result<Token> {
    let (mut raw_data,) = raw_rand()
        .await
        .map_err(|_| PythiaError::UnableToGetRandom)?;

    let (insufficient_bytes_count, was_overflowed) = raw_data.len().overflowing_sub(BITS_IN_BYTE);

    if was_overflowed {
        raw_data.append(&mut vec![0; insufficient_bytes_count]);
    }

    let value = u64::from_be_bytes(
        raw_data[..BITS_IN_BYTE]
            .try_into()
            .expect("should be valid convertation"),
    );

    cast_to_param_type(value, abi_type).context(PythiaError::InvalidABIParameterTypes)
}

pub async fn get_sybil_input(pair_id: &str) -> Result<Vec<Token>> {
    let rate = sybil::get_asset_data(pair_id).await?;

    Ok(vec![
        Token::String(rate.symbol),
        Token::Uint(rate.rate.into()),
        Token::Uint(rate.decimals.into()),
        Token::Uint(rate.timestamp.into()),
    ])
}
