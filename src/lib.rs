mod encryption;
mod handlers;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::encryption::{Base64Encryptor, Encryptor};

#[derive(Clone)]
pub struct AppState {
    encryptor: Arc<dyn Encryptor>,
}

impl AppState {
    pub fn with_base64_encryptor() -> Self {
        Self {
            encryptor: Arc::new(Base64Encryptor),
        }
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/encrypt", post(handlers::encrypt))
        .route("/decrypt", post(handlers::decrypt))
        .with_state(state)
}
