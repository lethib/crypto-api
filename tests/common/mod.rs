// Each file in tests/ is its own crate and uses only part of these helpers,
// so rustc would flag the rest as dead code.
#![allow(dead_code)]

use axum::{
    Router,
    body::Body,
    http::{Request, Response, header},
};
use crypto_api::AppState;
use http_body_util::BodyExt;
use serde_json::Value;

pub fn app() -> Router {
    crypto_api::app(AppState::with_base64_encryptor())
}

pub fn post_json(uri: &str, body: &str) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

pub trait ResponseBodyToJson {
    async fn body_to_json(self) -> Value;
}

impl ResponseBodyToJson for Response<Body> {
    async fn body_to_json(self) -> Value {
        let body = self.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }
}
