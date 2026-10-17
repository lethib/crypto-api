use axum::{Json, extract::State};
use serde_json::{Map, Value, json};

use crate::{
    AppState,
    encryption::{decrypt_fields, encrypt_fields},
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
