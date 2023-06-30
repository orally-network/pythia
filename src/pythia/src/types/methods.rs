use ic_cdk::export::{
    candid::{CandidType, Nat},
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Default)]
pub enum MethodType {
    Pair(String),
    Random(String),
    #[default]
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct Method {
    pub name: String,
    pub abi: String,
    pub gas_limit: Nat,
    pub chain_id: Nat,
    pub method_type: MethodType,
}
