pub mod chains;
pub mod controllers;
pub mod subs;
pub mod users;

use ic_cdk::query;
use ic_utils::{
    api_type::{GetInformationRequest, GetInformationResponse},
    get_information,
};

use crate::utils::validate_caller;

#[query]
pub async fn get_canistergeek_information(
    request: GetInformationRequest,
) -> GetInformationResponse<'static> {
    if let Err(err) = validate_caller() {
        ic_cdk::trap(&err.to_string())
    };

    get_information(request)
}
