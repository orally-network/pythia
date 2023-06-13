use candid::Nat;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub amount: Nat,
    pub receiver: String,
}
