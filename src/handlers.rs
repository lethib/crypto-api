use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::{
    AppState,
    encryption::{decrypt_fields, encrypt_fields},
    signing::{sign_value, verify_value},
};

type AtLeastOneDepthJSON = Map<String, Value>;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn encrypt(
    State(state): State<AppState>,
    Json(json): Json<AtLeastOneDepthJSON>,
) -> Json<AtLeastOneDepthJSON> {
    Json(encrypt_fields(state.encryptor.as_ref(), json))
}

pub async fn decrypt(
    State(state): State<AppState>,
    Json(json): Json<AtLeastOneDepthJSON>,
) -> Json<AtLeastOneDepthJSON> {
    Json(decrypt_fields(state.encryptor.as_ref(), json))
}

pub async fn sign(State(state): State<AppState>, Json(json): Json<Value>) -> Json<Value> {
    let signature = sign_value(state.signer.as_ref(), &json);
    Json(json!({ "signature": signature }))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    signature: String,
    data: Value,
}

pub async fn verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> StatusCode {
    if verify_value(state.signer.as_ref(), &request.data, &request.signature) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::BAD_REQUEST
    }
}
