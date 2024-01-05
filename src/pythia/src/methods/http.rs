use candid::CandidType;
use ic_cdk::query;
use serde::Deserialize;
use serde_bytes::ByteBuf;

use crate::utils::metrics::gather_metrics;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}

pub type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
}

#[query]
pub fn http_request(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    match parts[0] {
        "/metrics" => HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: ByteBuf::from(gather_metrics()),
        },
        _ => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(String::from("404 Not Found")),
        },
    }
}
