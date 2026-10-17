mod encryption;
mod handlers;

use axum::{Router, routing::get};
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
        .with_state(state)
}
