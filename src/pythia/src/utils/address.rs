use std::str::FromStr;

use ic_web3::types::H160;

use anyhow::{Result, Context};

#[inline]
pub fn from_h160(h160: &H160) -> String {
    format!("0x{}", hex::encode(h160.as_bytes()))
}

#[inline]
pub fn to_h160(address: &str) -> Result<H160> {
    H160::from_str(address)
        .context("failed to convert address to H160")
}

#[inline]
pub fn normalize(address: &str) -> Result<String> {
    let h160 = to_h160(address)?;
    Ok(from_h160(&h160))
}
