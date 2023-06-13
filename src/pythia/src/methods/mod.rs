pub mod chains;
pub mod controllers;
pub mod users;
pub mod subscriptions;

use anyhow::Result;
use ic_cdk::{query, update};
use ic_utils::{
    api_type::{GetInformationRequest, GetInformationResponse, UpdateInformationRequest},
    get_information, update_information,
};

use crate::{STATE, types::state::State};

#[query(name = "getCanistergeekInformation")]
pub async fn get_canistergeek_information(
    request: GetInformationRequest,
) -> GetInformationResponse<'static> {
    get_information(request)
}

#[update(name = "updateCanistergeekInformation")]
pub async fn update_canistergeek_information(request: UpdateInformationRequest) {
    update_information(request);
}

#[update]
pub async fn get_pma() -> Result<String, String> {
    crate::utils::get_pma()
        .await
        .map_err(|e| format!("{e:?}"))
}

#[query]
pub fn get_state() -> State {
    let state = STATE.with(|state| state.borrow().clone());
    ic_cdk::println!("{state:?}");
    state
}