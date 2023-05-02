pub mod subs;
pub mod users;
pub mod chains;
pub mod errors;

use std::str::FromStr;

use ic_cdk::export::candid::Nat;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default, PartialOrd, Ord)]
pub struct U256(pub ic_web3::types::U256);

impl From<&Nat> for U256 {
    fn from(nat: &Nat) -> Self {
        U256(ic_web3::types::U256::from_str(&nat.0.to_string()).unwrap())
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        U256(ic_web3::types::U256::from(value))
    }
}
