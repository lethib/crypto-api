// Each file in tests/ is its own crate and uses only part of these helpers,
// so rustc would flag the rest as dead code.
#![allow(dead_code)]

use axum::{
    Router,
    body::Body,
    http::{Request, Response, header},
};
use crypto_api::{AppState, encryption::Base64Encryptor, signing::hmac_sha256::HmacSha256Signer};
use http_body_util::BodyExt;
use serde_json::Value;

const TEST_HMAC_SECRET: &str = "test-hmac-secret-of-at-least-32-bytes";

pub fn app() -> Router {
    let app_state = AppState::new(
        Base64Encryptor,
        HmacSha256Signer::new(TEST_HMAC_SECRET.to_owned()),
    );

    crypto_api::app(app_state)
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
