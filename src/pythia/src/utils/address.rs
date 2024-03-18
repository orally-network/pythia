use std::str::FromStr;

use ic_web3_rs::types::H160;

use anyhow::{Context, Result};

use crate::types::errors::PythiaError;

#[inline]
pub fn from_h160(h160: &H160) -> String {
    format!("0x{}", hex::encode(h160.as_bytes()))
}

#[inline]
pub fn to_h160(address: &str) -> Result<H160> {
    H160::from_str(address).context(PythiaError::InvalidAddressFormat)
}

#[inline]
pub fn normalize(address: &str) -> Result<String> {
    let h160 = to_h160(address)?;
    Ok(from_h160(&h160))
}

pub fn eip55(address: String) -> Result<String> {
    let h160 = to_h160(&address)?;
    Ok(siwe::eip55(&h160.to_fixed_bytes()))
}
