pub mod chains;
pub mod errors;
pub mod rate_data;
pub mod subs;
pub mod state;

use std::str::FromStr;

use candid::{CandidType, types::Type};
use ic_cdk::export::{
    candid::Nat,
    serde::{Deserialize, Serialize},
};
use num_bigint::BigUint;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Copy, Default, PartialOrd, Ord, Deserialize, Serialize, 
)]
pub struct U256(pub ic_web3::types::U256);

impl CandidType for U256 {
    fn _ty() -> Type {
        Type::Nat
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer {
        serializer.serialize_nat(&Nat::from(self.clone()))
    }
}

impl From<Nat> for U256 {
    fn from(nat: Nat) -> Self {
        U256(ic_web3::types::U256::from_big_endian(&nat.0.to_bytes_be()))
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        U256(ic_web3::types::U256::from(value))
    }
}

impl From<U256> for Nat {
    fn from(u256: U256) -> Self {
        Nat(BigUint::from_str(&u256.0.to_string()).expect("should be valid number"))
    }
}
