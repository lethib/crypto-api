pub mod encryption;
mod handlers;
pub mod signing;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{encryption::Encryptor, signing::Signer};

#[derive(Clone)]
pub struct AppState {
    encryptor: Arc<dyn Encryptor>,
    signer: Arc<dyn Signer>,
}

impl AppState {
    pub fn new(encryptor: impl Encryptor + 'static, signer: impl Signer + 'static) -> Self {
        Self {
            encryptor: Arc::new(encryptor),
            signer: Arc::new(signer),
        }
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/encrypt", post(handlers::encrypt))
        .route("/decrypt", post(handlers::decrypt))
        .route("/sign", post(handlers::sign))
        .route("/verify", post(handlers::verify))
        .with_state(state)
}
